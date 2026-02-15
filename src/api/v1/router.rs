use super::handlers::{hello, me};
use crate::state::AppState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/hello", get(hello)).route("/me", get(me))
}
