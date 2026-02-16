use crate::app::Pool;
use crate::configs::Config;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub db_pool: Pool,
}
impl FromRef<Arc<AppState>> for AppState {
    fn from_ref(state: &Arc<AppState>) -> Self {
        (**state).clone()
    }
}
impl AppState {
    pub async fn setup(cfg: Config, db_pool: Pool) -> Arc<Self> {
        Arc::new(AppState { cfg, db_pool })
    }
}
