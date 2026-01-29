mod client;
mod configs;
mod extractors;
mod handlers;
mod logger;
mod models;
mod routing;
mod swagger;

use crate::client::PostClient;
use crate::configs::{AppState, Config};
use crate::logger::Tracing;
use crate::routing::router;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::{join, signal, spawn};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
async fn setup() -> Arc<AppState> {
    dotenv::from_filename(".env").expect("Failed to read .env file");
    Tracing::init().await.expect("Failed init tracing");
    Arc::new(AppState { cfg: Config::load() })
}
async fn run_axum(shutdown: CancellationToken) -> std::io::Result<()> {
    let state = setup().await;

    let openapi = swagger::ApiDoc::openapi();
    let app = router()
        .with_state(state.clone())
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", openapi.clone()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );
    let listener =
        tokio::net::TcpListener::bind(&state.cfg.server_address).await?;
    info!("Server started successfully");
    info!(
        "Swagger UI available at: http://{}/swagger-ui/",
        &state.cfg.server_address
    );
    info!("Starting server on http://{}", &state.cfg.server_address);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            tokio::time::sleep(Duration::from_secs(5)).await;

            info!("SIGINT received, shutting down...");
            shutdown.cancel();
        })
        .await?;
    Ok(())
}

pub async fn start_cron_scheduler(
    shutdown: CancellationToken,
) -> Result<(), JobSchedulerError> {
    let mut scheduler = JobScheduler::new().await?;

    // 🔹 Простой cron
    scheduler
        .add(Job::new("1/10 * * * * *", |_uuid, _l| {
            info!("⏰ I run every 10 seconds");
        })?)
        .await?;

    // 🔹 Async cron
    scheduler
        .add(Job::new_async("1/1 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                info!("⏰ I run async every 7 seconds");
                let client = PostClient {
                    url: "https://jsonplaceholder.typicode.com/posts"
                        .to_string(),
                };
                let result = async {
                    let posts = client.get_posts().await?;
                    for post in posts {
                        println!("📬 Post {} {}", post.id, post.body);
                    }
                    Ok::<(), reqwest::Error>(())
                }
                .await;

                if let Err(err) = result {
                    error!("❌ Cron job error: {}", err);
                }
            })
        })?)
        .await?;

    // 🔹 English cron
    scheduler
        .add(Job::new_async("every 4 seconds", |_uuid, _l| {
            Box::pin(async move {
                info!("⏰ I run every 4 seconds");
            })
        })?)
        .await?;

    // 🔹 One-shot
    scheduler
        .add(Job::new_one_shot(Duration::from_secs(18), |_uuid, _l| {
            info!("🔥 I only run once");
        })?)
        .await?;

    // 🔹 Repeated job
    let jj = Job::new_repeated(Duration::from_secs(8), |_uuid, _l| {
        info!("🔁 I run repeatedly every 8 seconds");
    })?;
    scheduler.add(jj).await?;

    scheduler.start().await?;
    info!("✅ Cron scheduler started");

    shutdown.cancelled().await;

    info!("🛑 Cron scheduler shutting down");
    scheduler.shutdown().await?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shutdown = CancellationToken::new();

    // ---------------- GAME LOOP (sync blocking func)
    let game_shutdown = shutdown.clone();
    spawn_blocking(move || {
        while !game_shutdown.is_cancelled() {
            info!("🎮 Работает игровой цикл...");
            thread::sleep(Duration::from_millis(50));
        }
        info!("🛑 Game task stopped gracefully");
    });

    // ---------------- ASYNC LOOP (async non-blocking func)
    let some_shutdown = shutdown.clone();
    spawn(async move {
        while !some_shutdown.is_cancelled() {
            info!("⚙ Async loop...");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        info!("🛑 Async task stopped gracefully");
    });

    // ---------------- CRON JOBS (async non-blocking periodic func)
    let cron_shutdown = shutdown.clone();
    let cron_handle = spawn(async move {
        if let Err(e) = start_cron_scheduler(cron_shutdown).await {
            error!("Cron error: {:?}", e);
        }
    });

    // ---------------- AXUM (async non-blocking func)
    let axum_shutdown = shutdown.clone();
    let axum_handle = spawn(async move {
        if let Err(e) = run_axum(axum_shutdown).await {
            error!("Axum error: {:?}", e);
        }
    });

    // ---------------- CTRL+C
    signal::ctrl_c().await?;
    info!("🧨 Ctrl+C received");

    shutdown.cancel();

    let _results = join!(cron_handle, axum_handle);

    info!("✅ Application stopped cleanly");
    Ok(())
}
