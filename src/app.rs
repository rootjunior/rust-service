use crate::api::server::ProjectHTTPServer;
use crate::configs::Config;
use crate::cron::ProjectCron;
use crate::state::AppState;
use std::thread;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::{signal, spawn};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct App {
    pub cfg: Config,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        App { cfg }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::setup(self.cfg.clone()).await;
        let shutdown = CancellationToken::new();

        let server_shutdown = shutdown.clone();
        let game_shutdown = shutdown.clone();
        let some_shutdown = shutdown.clone();
        let cron_shutdown = shutdown.clone();

        // ---------------- GAME LOOP (sync blocking func)
        let game_loop_handle = spawn_blocking(move || {
            while !game_shutdown.is_cancelled() {
                info!("🎮 Работает игровой цикл...");
                thread::sleep(Duration::from_millis(50));
            }
            info!("🛑 Game task stopped gracefully");
        });

        // ---------------- ASYNC LOOP
        let some_loop_handle = spawn(async move {
            while !some_shutdown.is_cancelled() {
                info!("⚙ Async loop...");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            info!("🛑 Async task stopped gracefully");
        });

        // ---------------- CRON JOBS
        let cron_handle = spawn(async move {
            if let Err(e) = ProjectCron::setup(cron_shutdown).await {
                error!("Cron error: {:?}", e);
            }
        });

        // ----------------HTTP SERVER
        let server_handle = spawn(async move {
            if let Err(e) =
                ProjectHTTPServer::run(server_shutdown, &state).await
            {
                error!("Axum error: {:?}", e);
            }
        });

        // ---------------- HANDLE CTRL+C
        signal::ctrl_c().await?;
        info!("✅ Ctrl+C received");
        shutdown.cancel();

        tokio::try_join!(
            server_handle,
            game_loop_handle,
            some_loop_handle,
            cron_handle
        )?;
        info!("✅ Application stopped cleanly");

        Ok(())
    }
}
