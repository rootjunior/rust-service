use crate::core::results::hello::GetHelloResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[async_trait]
pub trait Query: Send + Sync {}

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct HelloQuery {
    pub name: String,
}

#[async_trait]
impl Query for HelloQuery {}

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct ByQuery {
    pub name: String,
}

#[async_trait]
impl Query for ByQuery {}

#[async_trait]
pub trait Command: Send + Sync {}

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct HelloCommand {
    pub name: String,
}

#[async_trait]
impl Command for HelloCommand {}

#[async_trait]
pub trait CommandUseCase<C: Command>: Send + Sync {
    async fn execute(&self, command: C);
}

#[async_trait]
pub trait QueryUseCase<Q: Query, R: Send + Sync>: Send + Sync {
    async fn execute(&self, query: Q) -> R;
}

pub struct CreateHelloUseCase;

#[async_trait]
impl CommandUseCase<HelloCommand> for CreateHelloUseCase {
    async fn execute(&self, command: HelloCommand) {
        println!("Hello from HelloUseCase: {}", command.name);
    }
}

pub struct HelloRepository;

impl HelloRepository {
    pub async fn get_by_id(&self, id: i32) -> String {
        "hello world".to_string()
    }
}

pub struct GetHelloUseCase {
    hello_repo: HelloRepository,
}

#[async_trait]
impl QueryUseCase<HelloQuery, GetHelloResult> for GetHelloUseCase {
    async fn execute(&self, query: HelloQuery) -> GetHelloResult {
        let name = self.hello_repo.get_by_id(13).await;
        GetHelloResult { name }
    }
}

impl GetHelloUseCase {
    pub fn new(hello_repo: HelloRepository) -> Self {
        Self { hello_repo }
    }
}
