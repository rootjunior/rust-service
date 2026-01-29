use tracing_appender::rolling;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, prelude::*};

#[derive(Clone)]
pub struct Tracing {}

impl Tracing {
    pub async fn init() -> std::io::Result<()> {
        // Путь к файлу логов
        tokio::fs::create_dir_all("logs").await?;

        // Настраиваем подписчика для файла
        let file_subscriber = fmt::layer()
            .with_writer(rolling::daily("logs", "app.log"))
            .with_ansi(false)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_filter(LevelFilter::INFO);

        // Настраиваем подписчика для консоли
        let stdout_subscriber = fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_filter(LevelFilter::TRACE);

        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(stdout_subscriber)
            .try_init()
            .expect("Failed to initialize logger");

        Ok(())
    }
}
