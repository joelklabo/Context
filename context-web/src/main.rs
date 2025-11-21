use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

async fn health() -> &'static str {
    "OK"
}

async fn agent_doc() -> String {
    context_agent::agent_doc_markdown().to_string()
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::from_default_env().add_directive("context_web=info".parse().unwrap());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/agent-doc", get(agent_doc));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8077));
    tracing::info!("Starting context-web on http://{addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
