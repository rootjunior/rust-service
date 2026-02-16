use crate::core::handlers::hello::{
    Command, CommandHandler, Query, QueryHandler,
};
use crate::mediator::errors::MediatorError;
use std::any::TypeId;
use std::collections::HashMap;

use crate::mediator::erase::{ErasedCommand, ErasedQuery};
use tokio::sync::Mutex;
use tracing::error;

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
        U: CommandHandler<C> + Send + Sync + 'static,
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
        U: QueryHandler<Q, R> + Send + Sync + 'static,
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
        let handler = self
            .command_handlers
            .lock()
            .await
            .get(&TypeId::of::<C>())
            .cloned()
            .ok_or_else(|| {
                let msg = format!(
                    "No command handler registered for {}",
                    std::any::type_name::<C>()
                );
                error!("{}", msg);
                MediatorError::CommandNotFound(msg)
            })?;

        handler.execute(Box::new(command)).await?;
        Ok(())
    }

    // QUERY
    pub async fn query<Q, R>(&self, query: Q) -> Result<R, MediatorError>
    where
        Q: Send + Sync + 'static + Clone,
        R: Send + Sync + 'static,
    {
        let handler = self
            .query_handlers
            .lock()
            .await
            .get(&TypeId::of::<Q>())
            .cloned()
            .ok_or_else(|| {
                let msg = format!(
                    "No query handler registered for {}",
                    std::any::type_name::<Q>()
                );
                error!("{}", msg);
                MediatorError::QueryNotFound(msg)
            })?;

        handler
            .execute(Box::new(query))
            .await?
            .downcast::<R>()
            .map(|boxed| *boxed)
            .map_err(|_| {
                let msg = format!(
                    "Query result type mismatch for {}",
                    std::any::type_name::<Q>()
                );
                error!("{}", msg);
                MediatorError::QueryResultMismatch(msg)
            })
    }
}
