use crate::configs::Config;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*};

#[derive(Clone)]
pub struct Tracing {}

impl Tracing {
    pub async fn setup(cfg: &Config) -> std::io::Result<()> {
        tokio::fs::create_dir_all(&cfg.log_dir_path).await?;

        let log_level_filter = cfg.get_log_level_filter();
        let file_subscriber = fmt::layer()
            .with_writer(rolling::daily(
                &cfg.log_dir_path,
                &cfg.log_file_name_prefix,
            ))
            .with_ansi(false)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_filter(log_level_filter);

        let stdout_subscriber = fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_filter(log_level_filter);

        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(stdout_subscriber)
            .try_init()
            .expect("Failed to initialize logger");

        Ok(())
    }
}
