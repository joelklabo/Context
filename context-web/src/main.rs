use anyhow::Result;
use axum::{routing::get, Router};
use context_telemetry::init_tracing;
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn health() -> &'static str {
    "OK"
}

async fn agent_doc() -> String {
    context_agent::agent_doc_markdown().to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = init_tracing("context-web", &["context_web"])?;

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/agent-doc", get(agent_doc));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8077));
    tracing::info!("Starting context-web on http://{addr}");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
