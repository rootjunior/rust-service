use crate::api::server::ProjectHTTPServer;
use crate::configs::Config;
use crate::cron::ProjectCron;
use crate::state::AppState;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, bb8};
use diesel_async::{AsyncMigrationHarness, AsyncPgConnection};
use diesel_migrations::{
    EmbeddedMigrations, MigrationHarness, embed_migrations,
};
use std::thread;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::{signal, spawn};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
pub type Pool = bb8::Pool<AsyncPgConnection>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct App {
    pub cfg: Config,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        App { cfg }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_pool: Pool = bb8::Pool::builder()
            .build(AsyncDieselConnectionManager::<AsyncPgConnection>::new(
                &self.cfg.db_url,
            ))
            .await
            .expect("Failed to create pool");

        let state = AppState::setup(self.cfg.clone(), db_pool.clone()).await;

        //  Применение  миграций
        let mut harness =
            AsyncMigrationHarness::new(db_pool.get_owned().await.expect("Occurred due to an error establishing a connection to the database"));
        harness
            .run_pending_migrations(MIGRATIONS)
            .expect("An error occurred applying migrations");

        let shutdown = CancellationToken::new();
        let server_shutdown = shutdown.clone();
        let game_shutdown = shutdown.clone();
        let some_shutdown = shutdown.clone();
        let cron_shutdown = shutdown.clone();

        // ---------------- RUN GAME LOOP (sync blocking func)
        let game_loop_handle = spawn_blocking(move || {
            while !game_shutdown.is_cancelled() {
                info!("🎮 Работает игровой цикл...");
                thread::sleep(Duration::from_millis(50));
            }
            info!("🛑 Game task stopped gracefully");
        });

        // ---------------- RUN ASYNC LOOP
        let some_loop_handle = spawn(async move {
            while !some_shutdown.is_cancelled() {
                info!("⚙ Async loop...");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            info!("🛑 Async task stopped gracefully");
        });

        // ---------------- RUN CRON JOBS
        let cron_handle = spawn(async move {
            if let Err(e) = ProjectCron::setup(cron_shutdown).await {
                error!("Cron error: {:?}", e);
            }
        });

        // ---------------- RUN HTTP SERVER
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
