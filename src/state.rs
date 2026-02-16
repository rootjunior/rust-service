use crate::configs::Config;
use crate::mediator::mediator::Mediator;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub mediator: Arc<Mediator>,
}
impl FromRef<Arc<AppState>> for AppState {
    fn from_ref(state: &Arc<AppState>) -> Self {
        (**state).clone()
    }
}
impl AppState {
    pub async fn setup(cfg: Config, mediator: Arc<Mediator>) -> Arc<Self> {
        Arc::new(AppState { cfg, mediator })
    }
}
