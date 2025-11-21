use std::env;

use anyhow::Result;
use clap::{Parser, Subcommand};
use context_telemetry::{context_span, init_tracing, LogContext};

/// context – CLI entrypoint (skeleton)
#[derive(Parser)]
#[command(name = "context", version, about = "Context CLI (skeleton)", long_about = None)]
struct Cli {
    /// Optional project override (otherwise inferred from repo/config)
    #[arg(long, global = true)]
    project: Option<String>,

    /// Output JSON where applicable (for agents)
    #[arg(long, global = true)]
    json: bool,

    /// Optional scenario identifier for correlating logs
    #[arg(long, global = true)]
    scenario: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print agent usage documentation.
    AgentDoc {
        /// Output format (currently only 'markdown')
        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Initialize context configuration (stub)
    Init,

    /// Store or update a document (stub)
    Put {
        /// Optional key for the document
        #[arg(long)]
        key: Option<String>,

        /// Read body from stdin
        #[arg(long, default_value_t = true)]
        stdin: bool,
    },

    /// Retrieve a document (stub)
    Get {
        #[arg(long)]
        key: Option<String>,

        #[arg(long)]
        id: Option<String>,

        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Dump document content for agents (stub)
    Cat {
        #[arg(long)]
        key: Option<String>,

        #[arg(long)]
        id: Option<String>,
    },

    /// Search documents (stub)
    Find {
        /// Search query text
        query: String,

        /// Optionally limit results
        #[arg(long)]
        limit: Option<usize>,

        /// Search across all projects
        #[arg(long)]
        all_projects: bool,
    },

    /// List documents (stub)
    Ls {},

    /// Soft-delete a document (stub)
    Rm {
        #[arg(long)]
        key: Option<String>,

        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        force: bool,
    },

    /// Garbage-collect tombstones, vacuum DB (stub)
    Gc {
        #[arg(long)]
        dry_run: bool,
    },

    /// Run user-facing web UI (stub wrapper)
    Web {
        #[arg(long, default_value_t = 8077)]
        port: u16,
    },

    /// Run dev web UI (stub wrapper)
    WebDev {
        #[arg(long, default_value_t = 8078)]
        port: u16,
    },

    /// Create a debug bundle (stub)
    DebugBundle {
        #[arg(long)]
        scenario: Option<String>,

        #[arg(long)]
        out: Option<String>,
    },

    /// Emit agent configs for Codex / Claude / Copilot (stub)
    AgentConfig {
        #[arg(long, default_value = "all")]
        target: String,
    },
}

fn main() -> Result<()> {
    let _telemetry = init_tracing("context-cli", &["context_cli", "context_core"])?;
    let cli = Cli::parse();
    let command_name = command_name(&cli.command).to_string();
    let project_label = cli.project.clone().unwrap_or_else(|| "default".to_string());
    let scenario = cli
        .scenario
        .clone()
        .or_else(|| env::var("CONTEXT_SCENARIO").ok());

    let log_context = LogContext {
        scenario_id: scenario.as_deref(),
        project: Some(project_label.as_str()),
        command: Some(command_name.as_str()),
    };

    let span = context_span(log_context);
    let _span_guard = span.enter();
    tracing::info!(
        scenario_id = log_context.scenario_id,
        project = log_context.project,
        command = log_context.command,
        "Command start"
    );

    match cli.command {
        Commands::AgentDoc { format } => match format.as_str() {
            "markdown" | "md" => {
                let md = context_agent::agent_doc_markdown();
                print!("{md}");
            }
            other => {
                eprintln!("Unsupported format: {other}. Try --format markdown");
                std::process::exit(2);
            }
        },
        Commands::Init => {
            println!("context init (stub): configuration will be set up here.");
        }
        Commands::Put { key, stdin } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?stdin,
                "Put command invoked (stub)"
            );
            eprintln!("TODO: implement `context put`");
        }
        Commands::Get { key, id, format } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                ?format,
                "Get command invoked (stub)"
            );
            eprintln!("TODO: implement `context get`");
        }
        Commands::Cat { key, id } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                "Cat command invoked (stub)"
            );
            eprintln!("TODO: implement `context cat`");
        }
        Commands::Find {
            query,
            limit,
            all_projects,
        } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                %query,
                ?limit,
                ?all_projects,
                "Find command invoked (stub)"
            );
            eprintln!("TODO: implement `context find`");
        }
        Commands::Ls {} => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                "Ls command invoked (stub)"
            );
            eprintln!("TODO: implement `context ls`");
        }
        Commands::Rm { key, id, force } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                ?force,
                "Rm command invoked (stub)"
            );
            eprintln!("TODO: implement `context rm`");
        }
        Commands::Gc { dry_run } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?dry_run,
                "Gc command invoked (stub)"
            );
            eprintln!("TODO: implement `context gc`");
        }
        Commands::Web { port } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?port,
                "Web command invoked (stub)"
            );
            eprintln!("TODO: implement `context web` wrapper");
        }
        Commands::WebDev { port } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?port,
                "WebDev command invoked (stub)"
            );
            eprintln!("TODO: implement `context web-dev` wrapper");
        }
        Commands::DebugBundle { scenario, out } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?scenario,
                ?out,
                "DebugBundle command invoked (stub)"
            );
            eprintln!("TODO: implement `context debug-bundle`");
        }
        Commands::AgentConfig { target } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                %target,
                "AgentConfig command invoked (stub)"
            );
            eprintln!("TODO: implement `context agent-config`");
        }
    }

    Ok(())
}

fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::AgentDoc { .. } => "agent-doc",
        Commands::Init => "init",
        Commands::Put { .. } => "put",
        Commands::Get { .. } => "get",
        Commands::Cat { .. } => "cat",
        Commands::Find { .. } => "find",
        Commands::Ls {} => "ls",
        Commands::Rm { .. } => "rm",
        Commands::Gc { .. } => "gc",
        Commands::Web { .. } => "web",
        Commands::WebDev { .. } => "web-dev",
        Commands::DebugBundle { .. } => "debug-bundle",
        Commands::AgentConfig { .. } => "agent-config",
    }
}
