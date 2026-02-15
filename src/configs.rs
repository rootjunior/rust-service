use std::env::var;
use tracing::Level;
use tracing::level_filters::LevelFilter;

#[derive(Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub secret_token: String,
    pub server_address: String,
    #[allow(dead_code)]
    _workers_count: usize,
    _log_level: String,
    pub log_file_name_prefix: String,
    pub log_dir_path: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv::from_filename(".env").expect("Failed to read .env file");
        let secret_token =
            var("SECRET_TOKEN").expect("SECRET_TOKEN must be set");
        let server_address = format!(
            "{}:{}",
            var("HOST").expect("HOST must be set"),
            var("PORT").expect("PORT must be set")
        );
        let _workers_count = var("WORKERS_COUNT")
            .expect("WORKERS_COUNT must be set")
            .parse()
            .expect("WORKERS_COUNT must be a number");
        let _log_level = var("LOG_LEVEL").expect("LOG_LEVEL must be set");
        let log_file_name_prefix = var("LOG_FILE_NAME_PREFIX")
            .expect("LOG_FILE_NAME_PREFIX must be set");
        let log_dir_path =
            var("LOG_DIR_PATH").expect("LOG_DIR_PATH must be set");

        Self {
            secret_token,
            server_address,
            _log_level,
            _workers_count,
            log_file_name_prefix,
            log_dir_path,
        }
    }
    pub fn log_level(&self) -> LevelFilter {
        self._log_level
            .parse::<Level>()
            .map(LevelFilter::from)
            .unwrap_or(LevelFilter::INFO)
    }

    pub fn workers_count(&self) -> usize {
        self._workers_count
    }
}
