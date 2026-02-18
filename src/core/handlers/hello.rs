use crate::core::results::hello::GetHelloResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[async_trait]
pub trait Query: Send + Sync {}

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct HelloQuery {
    pub name: String,
}

#[async_trait]
impl Query for HelloQuery {}

#[async_trait]
pub trait Command: Send + Sync {}

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct HelloCommand {
    pub name: String,
}

#[async_trait]
impl Command for HelloCommand {}

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn execute(&self, command: C);
}

#[async_trait]
pub trait QueryHandler<Q: Query, R: Send + Sync>: Send + Sync {
    async fn execute(&self, query: Q) -> R;
}

pub struct CreateHelloHandler;

#[async_trait]
impl CommandHandler<HelloCommand> for CreateHelloHandler {
    async fn execute(&self, command: HelloCommand) {
        println!("Hello from HelloHandler: {}", command.name);
    }
}

pub struct HelloRepository;

impl HelloRepository {
    pub async fn get_by_id(&self, id: i32) -> String {
        "hello world".to_string()
    }
}

pub struct GetHelloHandler {
    hello_repo: HelloRepository,
}

#[async_trait]
impl QueryHandler<HelloQuery, GetHelloResult> for GetHelloHandler {
    async fn execute(&self, query: HelloQuery) -> GetHelloResult {
        let name = self.hello_repo.get_by_id(13).await;
        GetHelloResult { name }
    }
}

impl GetHelloHandler {
    pub fn new(hello_repo: HelloRepository) -> Self {
        Self { hello_repo }
    }
}
