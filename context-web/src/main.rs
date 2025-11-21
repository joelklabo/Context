use anyhow::Result;
use axum::{routing::get, Router};
use context_telemetry::{context_span, init_tracing, LogContext};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

async fn health() -> &'static str {
    let span = tracing::info_span!("web.healthz");
    let _guard = span.enter();
    tracing::info!("Healthz served");
    "OK"
}

async fn agent_doc() -> String {
    let span = tracing::info_span!("web.agent-doc");
    let _guard = span.enter();
    tracing::info!("Agent doc served");
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
    let server_span = tracing::info_span!(
        "web.server",
        scenario_id = log_context.scenario_id,
        project = log_context.project,
        command = log_context.command
    );
    let _server_guard = server_span.enter();

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::tempdir;

    #[tokio::test]
    async fn logs_include_spans_for_handlers() {
        let temp = tempdir().unwrap();
        std::env::set_var("CONTEXT_LOG_DIR", temp.path());
        std::env::set_var("CONTEXT_SCENARIO", "web-test");
        let guard = context_telemetry::init_tracing("context-web", &["context_web"]).unwrap();

        health().await;
        agent_doc().await;

        drop(guard);
        std::env::remove_var("CONTEXT_LOG_DIR");
        std::env::remove_var("CONTEXT_SCENARIO");

        let log_path = temp.path().join("context-web.jsonl");
        let contents = std::fs::read_to_string(log_path).unwrap();

        let mut saw_healthz = false;
        let mut saw_agent_doc = false;
        for line in contents.lines() {
            let json: Value = serde_json::from_str(line).unwrap();
            if let Some(spans) = json["spans"].as_array() {
                if spans.iter().any(|span| span["name"] == "web.healthz") {
                    saw_healthz = true;
                }
                if spans.iter().any(|span| span["name"] == "web.agent-doc") {
                    saw_agent_doc = true;
                }
            }
        }

        assert!(saw_healthz, "expected web.healthz span");
        assert!(saw_agent_doc, "expected web.agent-doc span");
    }
}
