use super::v1::router::router as v1_router;
use super::v2::router::router as v2_router;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().nest("/api/v1", v1_router()).nest("/api/v2", v2_router())
}
