use crate::core::handlers::hello::{
    Command, CommandHandler, Query, QueryHandler,
};
use crate::mediator::errors::MediatorError;
use futures_util::future::BoxFuture;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

// Типы замыканий

type CommandFn = Arc<
    dyn Fn(Box<dyn Any + Send>) -> BoxFuture<'static, Result<(), MediatorError>>
        + Send
        + Sync,
>;

type QueryFn = Arc<
    dyn Fn(
            Box<dyn Any + Send>,
        ) -> BoxFuture<
            'static,
            Result<Box<dyn Any + Send + Sync>, MediatorError>,
        > + Send
        + Sync,
>;

pub struct Mediator {
    commands: Mutex<HashMap<TypeId, CommandFn>>,
    queries: Mutex<HashMap<TypeId, QueryFn>>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(HashMap::new()),
            queries: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register_command<C, H>(&self, handler: H)
    where
        C: Command + 'static,
        H: CommandHandler<C> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);

        let f: CommandFn = Arc::new(move |cmd: Box<dyn Any + Send>| {
            let handler = handler.clone();
            Box::pin(async move {
                let cmd = cmd.downcast::<C>().map_err(|_| {
                    MediatorError::CommandTypeMismatch(
                        std::any::type_name::<C>().to_string(),
                    )
                })?;
                handler.execute(*cmd).await;
                Ok(())
            })
        });

        self.commands.lock().await.insert(TypeId::of::<C>(), f);
    }

    pub async fn register_query<Q, R, H>(&self, handler: H)
    where
        Q: Query + 'static,
        R: Send + Sync + 'static,
        H: QueryHandler<Q, R> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);

        let f: QueryFn = Arc::new(move |q: Box<dyn Any + Send>| {
            let handler = handler.clone();
            Box::pin(async move {
                let q = q.downcast::<Q>().map_err(|_| {
                    MediatorError::QueryTypeMismatch(
                        std::any::type_name::<Q>().to_string(),
                    )
                })?;
                let result = handler.execute(*q).await;
                Ok(Box::new(result) as Box<dyn Any + Send + Sync>)
            })
        });

        self.queries.lock().await.insert(TypeId::of::<Q>(), f);
    }

    pub async fn send<C: Command + 'static>(
        &self,
        command: C,
    ) -> Result<(), MediatorError> {
        let f = self.get_command::<C>().await?;
        f(Box::new(command)).await
    }

    pub async fn query<Q: Query + 'static, R: Send + Sync + 'static>(
        &self,
        query: Q,
    ) -> Result<R, MediatorError> {
        let f = self.get_query::<Q>().await?;

        f(Box::new(query)).await?.downcast::<R>().map(|b| *b).map_err(|_| {
            let msg = format!(
                "Query result type mismatch for {}",
                std::any::type_name::<Q>()
            );
            error!("{msg}");
            MediatorError::QueryResultMismatch(msg)
        })
    }

    fn get_handler<T: Clone>(
        map: &HashMap<TypeId, T>,
        type_id: TypeId,
        type_name: &str,
        not_found: impl FnOnce(String) -> MediatorError,
    ) -> Result<T, MediatorError> {
        map.get(&type_id).cloned().ok_or_else(|| {
            let msg = format!("No handler for {type_name}");
            error!("{msg}");
            not_found(msg)
        })
    }

    async fn get_command<C: Command + 'static>(
        &self,
    ) -> Result<CommandFn, MediatorError> {
        Self::get_handler(
            &*self.commands.lock().await,
            TypeId::of::<C>(),
            std::any::type_name::<C>(),
            MediatorError::CommandNotFound,
        )
    }

    async fn get_query<Q: Query + 'static>(
        &self,
    ) -> Result<QueryFn, MediatorError> {
        Self::get_handler(
            &*self.queries.lock().await,
            TypeId::of::<Q>(),
            std::any::type_name::<Q>(),
            MediatorError::QueryNotFound,
        )
    }
}
