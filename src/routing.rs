use crate::configs::AppState;
use crate::handlers::{hello, me};
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().nest("/v1", v1_router())
}

fn v1_router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(hello)).route("/me", get(me))
}
