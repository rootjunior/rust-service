use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<AppState> {
    Router::new()
}
