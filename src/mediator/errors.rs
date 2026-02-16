use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediatorError {
    #[error("No command handler registered for type {0}")]
    CommandNotFound(String),

    #[error("No query handler registered for type {0}")]
    QueryNotFound(String),

    #[error("Query result type mismatch for type {0}")]
    QueryResultMismatch(String),
}
