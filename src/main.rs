mod configs;

mod api;
mod app;
mod core;
mod cron;
mod infra;
mod logger;
mod state;

use crate::app::App;
use crate::configs::Config;
use crate::logger::Tracing;
use tokio::runtime::{Builder, Handle};
use tracing::info;

// #[tokio::main(flavor = "multi_thread", worker_threads = 1)]
fn main() {
    let cfg = Config::load();
    let app = App::new(cfg.clone());

    let tokio_runtime = Builder::new_multi_thread()
        .worker_threads(cfg.workers_count)
        .enable_all()
        .build()
        .expect("Error building tokio runtime");

    tokio_runtime.block_on(async {
        Tracing::setup(&cfg).await.expect("Failed to init tracing");
        info!(
            "✅ Starting Tokio runtime with {} workers",
            Handle::current().metrics().num_workers()
        );
        app.run().await.expect("Error running main");
    });
}
