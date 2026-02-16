use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediatorError {
    #[error("No command handler registered for type {0}")]
    CommandNotFound(String),

    #[error("No query handler registered for type {0}")]
    QueryNotFound(String),

    #[error("Handler type mismatch for {0}")]
    HandlerTypeMismatch(String),

    #[error("Command type mismatch for {0}")]
    CommandTypeMismatch(String),

    #[error("Query type mismatch for {0}")]
    QueryTypeMismatch(String),

    #[error("Query result type mismatch for {0}")]
    QueryResultMismatch(String),
}
