use crate::core::use_cases::hello::{
    Command, CommandUseCase, Query, QueryUseCase,
};
use crate::mediator::errors::MediatorError;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

type H = Arc<dyn Any + Send + Sync>;
type B = Box<dyn Any + Send + Sync>;
type F = Pin<Box<dyn Future<Output = B> + Send>>;

#[derive(Debug, Clone)]
struct ErasedCommand {
    handler: H,
    executor: fn(H, B) -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl ErasedCommand {
    fn new<C, U>(handler: U) -> Self
    where
        C: Send + Sync + 'static + Clone + Command,
        U: CommandUseCase<C> + Send + Sync + 'static,
    {
        Self {
            handler: Arc::new(handler) as H,
            executor: |h: H, cmd: B| {
                Box::pin(async move {
                    h.downcast::<U>()
                        .expect("Handler type mismatch")
                        .execute(
                            *cmd.downcast::<C>()
                                .expect("Command type mismatch"),
                        )
                        .await;
                }) as Pin<Box<dyn Future<Output = ()> + Send>>
            },
        }
    }

    async fn execute(&self, command: B) {
        (self.executor)(self.handler.clone(), command).await;
    }
}

#[derive(Debug, Clone)]
struct ErasedQuery {
    handler: H,
    executor: fn(H, B) -> F,
}

impl ErasedQuery {
    fn new<Q, R, U>(handler: U) -> Self
    where
        Q: Send + Sync + 'static + Clone + Query,
        R: Send + Sync + 'static,
        U: QueryUseCase<Q, R> + Send + Sync + 'static,
    {
        Self {
            handler: Arc::new(handler) as H,
            executor: |h: H, q: B| {
                Box::pin(async move {
                    let res = h
                        .downcast::<U>()
                        .expect("Handler type mismatch")
                        .execute(
                            *q.downcast::<Q>().expect("Query type mismatch"),
                        )
                        .await;
                    Box::new(res) as B
                }) as F
            },
        }
    }

    async fn execute(&self, query: B) -> B {
        (self.executor)(self.handler.clone(), query).await
    }
}

pub struct Mediator {
    command_handlers: Mutex<HashMap<TypeId, ErasedCommand>>,
    query_handlers: Mutex<HashMap<TypeId, ErasedQuery>>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            command_handlers: Mutex::new(HashMap::new()),
            query_handlers: Mutex::new(HashMap::new()),
        }
    }

    // REGISTER
    pub async fn register_command<C, U>(&self, use_case: U)
    where
        C: Send + Sync + 'static + Clone + Command,
        U: CommandUseCase<C> + Send + Sync + 'static,
    {
        self.command_handlers
            .lock()
            .await
            .insert(TypeId::of::<C>(), ErasedCommand::new::<C, U>(use_case));
    }

    pub async fn register_query<Q, R, U>(&self, use_case: U)
    where
        Q: Send + Sync + 'static + Clone + Query,
        R: Send + Sync + 'static,
        U: QueryUseCase<Q, R> + Send + Sync + 'static,
    {
        self.query_handlers
            .lock()
            .await
            .insert(TypeId::of::<Q>(), ErasedQuery::new::<Q, R, U>(use_case));
    }

    // SEND
    pub async fn send<C>(&self, command: C) -> Result<(), MediatorError>
    where
        C: Send + Sync + 'static + Clone,
    {
        let type_id = TypeId::of::<C>();
        let handler = self
            .command_handlers
            .lock()
            .await
            .get(&type_id)
            .cloned()
            .ok_or_else(|| {
                let msg = format!(
                    "No command use_case registered for {}",
                    std::any::type_name::<C>()
                );
                error!("{}", msg);
                MediatorError::CommandNotFound(msg)
            })?;

        handler.execute(Box::new(command)).await;
        Ok(())
    }

    // QUERY
    pub async fn query<Q, R>(&self, query: Q) -> Result<R, MediatorError>
    where
        Q: Send + Sync + 'static + Clone,
        R: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<Q>();
        let handler = self
            .query_handlers
            .lock()
            .await
            .get(&type_id)
            .cloned()
            .ok_or_else(|| {
            let msg = format!(
                "No query use_case registered for {}",
                std::any::type_name::<Q>()
            );
            error!("{}", msg);
            MediatorError::QueryNotFound(msg)
        })?;

        let res = handler.execute(Box::new(query)).await;

        res.downcast::<R>().map(|boxed| *boxed).map_err(|_| {
            let msg = format!(
                "Query result type mismatch for {}",
                std::any::type_name::<Q>()
            );
            error!("{}", msg);
            MediatorError::QueryResultMismatch(msg)
        })
    }
}
