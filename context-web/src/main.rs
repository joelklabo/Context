use anyhow::Result;
use axum::{routing::get, Router};
use context_telemetry::{context_span, init_tracing, LogContext};
use std::{env, net::SocketAddr};
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
    let scenario = env::var("CONTEXT_SCENARIO").ok();
    let project = env::var("CONTEXT_PROJECT").ok();
    let log_context = LogContext {
        scenario_id: scenario.as_deref(),
        project: project.as_deref(),
        command: Some("web"),
    };
    let span = context_span(log_context);
    let _span_guard = span.enter();

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/agent-doc", get(agent_doc));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8077));
    tracing::info!(
        scenario_id = log_context.scenario_id,
        project = log_context.project,
        command = log_context.command,
        "Starting context-web on http://{addr}"
    );
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
