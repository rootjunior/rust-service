use crate::core::handlers::hello::{
    Command, CommandHandler, Query, QueryHandler,
};
use crate::mediator::errors::MediatorError;
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;

type H = Arc<dyn Any + Send + Sync>;
type B = Box<dyn Any + Send + Sync>;
type F = Pin<Box<dyn Future<Output = B> + Send>>;

type MediatorResult<T> = Result<T, MediatorError>;

#[derive(Debug, Clone)]
pub struct ErasedCommand {
    handler: H,
    executor:
        fn(H, B) -> Pin<Box<dyn Future<Output = MediatorResult<()>> + Send>>,
}

impl ErasedCommand {
    pub fn new<C, U>(handler: U) -> Self
    where
        C: Send + Sync + 'static + Clone + Command,
        U: CommandHandler<C> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler) as H;
        let executor = |h: H, cmd: B| {
            Box::pin(async move {
                let h = h.downcast::<U>().map_err(|_| {
                    MediatorError::HandlerTypeMismatch(
                        std::any::type_name::<U>().to_string(),
                    )
                })?;

                let cmd = cmd.downcast::<C>().map_err(|_| {
                    MediatorError::CommandTypeMismatch(
                        std::any::type_name::<C>().to_string(),
                    )
                })?;

                h.execute(*cmd).await;
                Ok(())
            })
                as Pin<Box<dyn Future<Output = MediatorResult<()>> + Send>>
        };
        Self { handler, executor }
    }

    pub async fn execute(&self, command: B) -> MediatorResult<()> {
        (self.executor)(self.handler.clone(), command).await
    }
}

#[derive(Debug, Clone)]
pub struct ErasedQuery {
    handler: H,
    executor:
        fn(H, B) -> Pin<Box<dyn Future<Output = MediatorResult<B>> + Send>>,
}

impl ErasedQuery {
    pub fn new<Q, R, U>(handler: U) -> Self
    where
        Q: Send + Sync + 'static + Clone + Query,
        R: Send + Sync + 'static,
        U: QueryHandler<Q, R> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler) as H;
        let executor = |h: H, q: B| {
            Box::pin(async move {
                let h = h.downcast::<U>().map_err(|_| {
                    MediatorError::HandlerTypeMismatch(
                        std::any::type_name::<U>().to_string(),
                    )
                })?;

                let q = q.downcast::<Q>().map_err(|_| {
                    MediatorError::QueryTypeMismatch(
                        std::any::type_name::<Q>().to_string(),
                    )
                })?;

                let res = h.execute(*q).await;
                Ok(Box::new(res) as B)
            })
                as Pin<Box<dyn Future<Output = MediatorResult<B>> + Send>>
        };
        Self { handler, executor }
    }

    pub async fn execute(&self, query: B) -> MediatorResult<B> {
        (self.executor)(self.handler.clone(), query).await
    }
}
