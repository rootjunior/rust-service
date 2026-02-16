use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Deserialize, Serialize)]
pub struct GetHelloResult {
    pub name: String,
}
