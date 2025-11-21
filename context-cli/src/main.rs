use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

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

fn init_tracing() {
    let filter = EnvFilter::from_default_env()
        .add_directive("context_cli=info".parse().unwrap())
        .add_directive("context_core=info".parse().unwrap());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

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
            tracing::info!(?key, ?stdin, "Put command invoked (stub)");
            eprintln!("TODO: implement `context put`");
        }
        Commands::Get { key, id, format } => {
            tracing::info!(?key, ?id, ?format, "Get command invoked (stub)");
            eprintln!("TODO: implement `context get`");
        }
        Commands::Cat { key, id } => {
            tracing::info!(?key, ?id, "Cat command invoked (stub)");
            eprintln!("TODO: implement `context cat`");
        }
        Commands::Find {
            query,
            limit,
            all_projects,
        } => {
            tracing::info!(%query, ?limit, ?all_projects, "Find command invoked (stub)");
            eprintln!("TODO: implement `context find`");
        }
        Commands::Ls {} => {
            tracing::info!("Ls command invoked (stub)");
            eprintln!("TODO: implement `context ls`");
        }
        Commands::Rm { key, id, force } => {
            tracing::info!(?key, ?id, ?force, "Rm command invoked (stub)");
            eprintln!("TODO: implement `context rm`");
        }
        Commands::Gc { dry_run } => {
            tracing::info!(?dry_run, "Gc command invoked (stub)");
            eprintln!("TODO: implement `context gc`");
        }
        Commands::Web { port } => {
            tracing::info!(?port, "Web command invoked (stub)");
            eprintln!("TODO: implement `context web` wrapper");
        }
        Commands::WebDev { port } => {
            tracing::info!(?port, "WebDev command invoked (stub)");
            eprintln!("TODO: implement `context web-dev` wrapper");
        }
        Commands::DebugBundle { scenario, out } => {
            tracing::info!(?scenario, ?out, "DebugBundle command invoked (stub)");
            eprintln!("TODO: implement `context debug-bundle`");
        }
        Commands::AgentConfig { target } => {
            tracing::info!(%target, "AgentConfig command invoked (stub)");
            eprintln!("TODO: implement `context agent-config`");
        }
    }

    Ok(())
}
