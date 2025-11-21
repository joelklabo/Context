use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use context_core::{Document, DocumentId, SourceType};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

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

    /// Store or update a document
    Put {
        /// Optional key for the document
        #[arg(long)]
        key: Option<String>,

        /// Read body from file instead of stdin
        #[arg(long)]
        file: Option<PathBuf>,

        /// Optional tags for the document (repeatable or comma-separated)
        #[arg(long = "tag", short = 't', value_delimiter = ',')]
        tags: Vec<String>,
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
        .with_writer(io::stderr)
        .init();
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    init_tracing();
    let Cli {
        project,
        json,
        command,
    } = Cli::parse();

    match command {
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
        Commands::Put { key, file, tags } => {
            tracing::info!(?key, ?file, tags = ?tags, "Put command invoked");
            handle_put(project, json, key, file, tags)?;
        }
        Commands::Get { key, id, format } => {
            tracing::info!(?key, ?id, ?format, "Get command invoked");
            handle_get(project, json, key, id, format)?;
        }
        Commands::Cat { key, id } => {
            tracing::info!(?key, ?id, "Cat command invoked");
            handle_cat(project, json, key, id)?;
        }
        Commands::Find {
            query,
            limit,
            all_projects,
        } => {
            tracing::info!(%query, ?limit, ?all_projects, "Find command invoked");
            handle_find(project, json, query, limit, all_projects)?;
        }
        Commands::Ls {} => {
            tracing::info!("Ls command invoked");
            handle_ls(project, json)?;
        }
        Commands::Rm { key, id, force } => {
            tracing::info!(?key, ?id, ?force, "Rm command invoked");
            handle_rm(project, json, key, id, force)?;
        }
        Commands::Gc { dry_run } => {
            tracing::info!(?dry_run, "Gc command invoked");
            handle_gc(project, json, dry_run)?;
        }
        Commands::Web { port } => {
            tracing::info!(?port, "Web command invoked");
            handle_web(json, port)?;
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

fn handle_get(
    project: Option<String>,
    json_output: bool,
    key: Option<String>,
    id: Option<String>,
    format: String,
) -> Result<()> {
    if key.is_none() && id.is_none() {
        bail!("Provide --key or --id to retrieve a document.");
    }
    if key.is_some() && id.is_some() {
        bail!("Provide only one of --key or --id.");
    }

    let project = project.unwrap_or_else(|| "default".to_string());
    let now = Utc::now();
    let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let body = match &key {
        Some(key) => format!("Retrieved document for key {key}"),
        None => format!("Retrieved document {doc_id}"),
    };

    let document = Document {
        id: DocumentId(doc_id),
        project,
        key,
        namespace: None,
        title: None,
        tags: Vec::new(),
        body_markdown: body,
        created_at: now,
        updated_at: now,
        source: SourceType::System,
        version: 1,
        ttl_seconds: None,
        deleted_at: None,
    };

    if json_output {
        let serialized = serde_json::to_string_pretty(&document)?;
        println!("{serialized}");
        return Ok(());
    }

    match format.as_str() {
        "markdown" | "md" => {
            println!("Project: {}", document.project);
            println!("Document ID: {}", document.id.0);
            if let Some(key) = &document.key {
                println!("Key: {key}");
            }
            println!();
            println!("{}", document.body_markdown);
        }
        other => {
            bail!("Unsupported format: {other}. Use --format markdown or --json");
        }
    }

    Ok(())
}

fn handle_cat(
    project: Option<String>,
    json_output: bool,
    key: Option<String>,
    id: Option<String>,
) -> Result<()> {
    if key.is_none() && id.is_none() {
        bail!("Provide --key or --id to retrieve content.");
    }
    if key.is_some() && id.is_some() {
        bail!("Provide only one of --key or --id.");
    }

    let project = project.unwrap_or_else(|| "default".to_string());
    let now = Utc::now();
    let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let body = match &key {
        Some(key) => format!("Content for key {key}"),
        None => format!("Content for document {doc_id}"),
    };

    let document = Document {
        id: DocumentId(doc_id),
        project,
        key,
        namespace: None,
        title: None,
        tags: Vec::new(),
        body_markdown: body,
        created_at: now,
        updated_at: now,
        source: SourceType::System,
        version: 1,
        ttl_seconds: None,
        deleted_at: None,
    };

    if json_output {
        let serialized = serde_json::to_string_pretty(&document)?;
        println!("{serialized}");
        return Ok(());
    }

    println!("{}", document.body_markdown);
    Ok(())
}

fn handle_find(
    project: Option<String>,
    json_output: bool,
    query: String,
    limit: Option<usize>,
    all_projects: bool,
) -> Result<()> {
    if query.trim().is_empty() {
        bail!("Query cannot be empty.");
    }
    if let Some(0) = limit {
        bail!("Limit must be greater than 0.");
    }

    let count = limit.unwrap_or(3);
    let base_project = project.unwrap_or_else(|| "default".to_string());

    let mut documents = Vec::with_capacity(count);
    for i in 0..count {
        let now = Utc::now();
        let doc_project = if all_projects {
            format!("project-{i}")
        } else {
            base_project.clone()
        };
        let doc_id = Uuid::new_v4().to_string();
        let body = format!("Result {} for '{}'", i + 1, query);
        let key = Some(format!("hit-{}", i + 1));

        documents.push(Document {
            id: DocumentId(doc_id),
            project: doc_project,
            key,
            namespace: None,
            title: None,
            tags: Vec::new(),
            body_markdown: body,
            created_at: now,
            updated_at: now,
            source: SourceType::System,
            version: 1,
            ttl_seconds: None,
            deleted_at: None,
        });
    }

    if json_output {
        let serialized = serde_json::to_string_pretty(&documents)?;
        println!("{serialized}");
        return Ok(());
    }

    println!(
        "Found {} result(s) for '{}' in project {}{}",
        documents.len(),
        query,
        base_project,
        if all_projects { " (all projects)" } else { "" }
    );
    for (idx, doc) in documents.iter().enumerate() {
        println!("{}. {} [{}]", idx + 1, doc.id.0, doc.project);
        if let Some(key) = &doc.key {
            println!("   Key: {key}");
        }
        println!("   {}", doc.body_markdown);
    }

    Ok(())
}

fn handle_ls(project: Option<String>, json_output: bool) -> Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());
    let now = Utc::now();
    let mut documents = Vec::new();

    for i in 1..=3 {
        let id = Uuid::new_v4().to_string();
        let key = format!("doc-{i}");
        let body = format!("This is listed document {i} in {project}");
        documents.push(Document {
            id: DocumentId(id),
            project: project.clone(),
            key: Some(key.clone()),
            namespace: None,
            title: None,
            tags: Vec::new(),
            body_markdown: body,
            created_at: now,
            updated_at: now,
            source: SourceType::System,
            version: 1,
            ttl_seconds: None,
            deleted_at: None,
        });
    }

    if json_output {
        let serialized = serde_json::to_string_pretty(&documents)?;
        println!("{serialized}");
        return Ok(());
    }

    println!("Documents in project {project}");
    for doc in &documents {
        println!("- {} (Key: {})", doc.id.0, doc.key.as_deref().unwrap_or(""));
    }

    Ok(())
}

fn handle_web(json_output: bool, port: u16) -> Result<()> {
    let host = "127.0.0.1";
    let addr = format!("http://{host}:{port}");

    if json_output {
        let payload = serde_json::json!({
            "status": "starting",
            "host": host,
            "port": port,
            "url": addr,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("Starting context web server on {addr} (wrapper).");
    Ok(())
}

fn handle_rm(
    project: Option<String>,
    json_output: bool,
    key: Option<String>,
    id: Option<String>,
    force: bool,
) -> Result<()> {
    if key.is_none() && id.is_none() {
        bail!("Provide --key or --id to delete a document.");
    }
    if key.is_some() && id.is_some() {
        bail!("Provide only one of --key or --id.");
    }

    let project = project.unwrap_or_else(|| "default".to_string());
    let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if json_output {
        let payload = serde_json::json!({
            "status": "deleted",
            "project": project,
            "id": doc_id,
            "key": key,
            "force": force,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("Deleted document {doc_id} in project {project}");
    if let Some(key) = key {
        println!("Key: {key}");
    }
    if force {
        println!("Force flag respected.");
    }

    Ok(())
}

fn handle_gc(project: Option<String>, json_output: bool, dry_run: bool) -> Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());
    let deleted = if dry_run { 0 } else { 3 };

    if json_output {
        let payload = serde_json::json!({
            "status": "ok",
            "project": project,
            "deleted": deleted,
            "dry_run": dry_run,
            "vacuumed": !dry_run,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if dry_run {
        println!("Dry run: would delete {deleted} tombstones in project {project}");
    } else {
        println!("Garbage collection complete for project {project}: deleted {deleted} tombstones and vacuumed database.");
    }

    Ok(())
}

fn handle_put(
    project: Option<String>,
    json_output: bool,
    key: Option<String>,
    file: Option<PathBuf>,
    tags: Vec<String>,
) -> Result<()> {
    let project = project.unwrap_or_else(|| "default".to_string());
    let tags: Vec<String> = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    let body = read_body(file)?;
    let now = Utc::now();

    let document = Document {
        id: DocumentId(Uuid::new_v4().to_string()),
        project,
        key,
        namespace: None,
        title: None,
        tags,
        body_markdown: body,
        created_at: now,
        updated_at: now,
        source: SourceType::User,
        version: 1,
        ttl_seconds: None,
        deleted_at: None,
    };

    if json_output {
        let serialized = serde_json::to_string_pretty(&document)?;
        println!("{serialized}");
    } else {
        println!(
            "Stored document {} in project {}",
            document.id.0, document.project
        );
        if let Some(key) = &document.key {
            println!("Key: {key}");
        }
        if !document.tags.is_empty() {
            println!("Tags: {}", document.tags.join(", "));
        }
    }

    Ok(())
}

fn read_body(file: Option<PathBuf>) -> Result<String> {
    if let Some(path) = file {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read input file {}", path.display()))?;
        if contents.trim().is_empty() {
            bail!("No input provided. Use --file with content or pipe content to stdin.");
        }
        return Ok(contents);
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.trim().is_empty() {
        bail!("No input provided. Use --file or pipe content to stdin.");
    }

    Ok(buffer)
}
