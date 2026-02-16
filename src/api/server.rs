use crate::api::router::router;
use crate::api::swagger;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::routing::get;
use axum::{response::IntoResponse, serve};
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct ProjectHTTPServer;

impl ProjectHTTPServer {
    pub async fn run(
        shutdown: CancellationToken,
        state: &Arc<AppState>,
    ) -> std::io::Result<()> {
        let app = router()
            .route("/ws", get(ws_handler))
            .with_state(state.clone())
            .merge(SwaggerUi::new("/docs").url(
                "/api-doc/openapi.json",
                swagger::ApiDoc::openapi().clone(),
            ))
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

        serve(listener, app)
            .with_graceful_shutdown(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");
                tokio::time::sleep(Duration::from_secs(5)).await;

                info!("SIGINT received, shutting down...");
                shutdown.cancel();
            })
            .await?;
        Ok(())
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("Client connected");

    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                socket
                    .send(Message::Text(format!("Echo: {}", text).into()))
                    .await
                    .unwrap();
            }
            Message::Close(_) => {
                println!("Client disconnected");
                return;
            }
            _ => {}
        }
    }
}
