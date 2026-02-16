use crate::api::router::router;
use crate::api::swagger;
use crate::state::AppState;
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
        let openapi = swagger::ApiDoc::openapi();
        let app = router()
            .with_state(state.clone())
            .merge(
                SwaggerUi::new("/docs")
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
