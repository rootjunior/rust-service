use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub secret_token: String,
    pub server_address: String,
    pub workers_count: usize,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        let secret_token =
            std::env::var("SECRET_TOKEN").expect("SECRET_TOKEN must be set");
        let server_address = format!(
            "{}:{}",
            std::env::var("HOST").expect("HOST must be set"),
            std::env::var("PORT").expect("PORT must be set")
        );
        let workers_count = std::env::var("WORKERS_COUNT")
            .expect("WORKERS_COUNT must be set")
            .parse()
            .expect("WORKERS_COUNT must be a number");

        Self { secret_token, server_address, workers_count }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
}
// Реализуем FromRef вручную (производный макрос может не работать)
impl FromRef<Arc<AppState>> for AppState {
    fn from_ref(state: &Arc<AppState>) -> Self {
        (**state).clone()
    }
}
