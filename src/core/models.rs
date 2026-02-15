use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResult {
    #[schema(example = 42)]
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = 42)]
    pub user_id: i32,
}
impl IntoResponse for UserResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
#[derive(Debug)]
pub struct AuthenticatedUser(pub AuthResult);
