use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use context_core::{
    sync::{self, SyncConfig},
    Document, DocumentId, SourceType,
};
use context_telemetry::{context_span, init_tracing, LogContext};
use serde::{Deserialize, Serialize};
use tracing::Span;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

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

    /// Synchronize the local database with a filesystem remote
    Sync {
        #[command(subcommand)]
        action: SyncCommands,
    },

    /// Create a debug bundle
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

    /// Manage default project selection
    Project {
        #[command(subcommand)]
        action: ProjectCommands,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Show the current project in use
    Current,
    /// Set the default project for this workspace
    Set {
        /// Project identifier to set as default
        project: String,
    },
    /// List known projects
    List,
}

#[derive(Subcommand)]
enum SyncCommands {
    /// Show sync status between local and remote
    Status {
        /// Override remote path (defaults to CONTEXT_SYNC_REMOTE or $CONTEXT_HOME/sync-remote)
        #[arg(long)]
        remote: Option<PathBuf>,
    },
    /// Push local database to remote
    Push {
        /// Override remote path (defaults to CONTEXT_SYNC_REMOTE or $CONTEXT_HOME/sync-remote)
        #[arg(long)]
        remote: Option<PathBuf>,

        /// Overwrite remote even if diverged
        #[arg(long)]
        force: bool,
    },
    /// Pull remote database into local
    Pull {
        /// Override remote path (defaults to CONTEXT_SYNC_REMOTE or $CONTEXT_HOME/sync-remote)
        #[arg(long)]
        remote: Option<PathBuf>,

        /// Overwrite local even if diverged
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let _telemetry = init_tracing("context-cli", &["context_cli", "context_core"])?;
    let Cli {
        project,
        json,
        scenario,
        command,
    } = Cli::parse();

    let command_name = command_name(&command).to_string();
    let project_label = resolve_project(project.clone())?;
    let scenario = scenario.or_else(|| env::var("CONTEXT_SCENARIO").ok());

    let log_context = LogContext {
        scenario_id: scenario.as_deref(),
        project: Some(project_label.as_str()),
        command: Some(command_name.as_str()),
    };

    let span = context_span(log_context);
    let _span_guard = span.enter();
    let command_span = command_span(log_context, &command);
    let _command_guard = command_span.enter();
    let resolved_project = Some(project_label.clone());
    tracing::info!(
        scenario_id = log_context.scenario_id,
        project = log_context.project,
        command = log_context.command,
        "Command start"
    );

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
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?file,
                tags = ?tags,
                "Put command invoked"
            );
            handle_put(resolved_project.clone(), json, key, file, tags)?;
        }
        Commands::Get { key, id, format } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                ?format,
                "Get command invoked"
            );
            handle_get(resolved_project.clone(), json, key, id, format)?;
        }
        Commands::Cat { key, id } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                "Cat command invoked"
            );
            handle_cat(resolved_project.clone(), json, key, id)?;
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
                "Find command invoked"
            );
            handle_find(resolved_project.clone(), json, query, limit, all_projects)?;
        }
        Commands::Ls {} => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                "Ls command invoked"
            );
            handle_ls(resolved_project.clone(), json)?;
        }
        Commands::Rm { key, id, force } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?key,
                ?id,
                ?force,
                "Rm command invoked"
            );
            handle_rm(resolved_project.clone(), json, key, id, force)?;
        }
        Commands::Gc { dry_run } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?dry_run,
                "Gc command invoked"
            );
            handle_gc(resolved_project.clone(), json, dry_run)?;
        }
        Commands::Web { port } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?port,
                "Web command invoked"
            );
            handle_web(json, port)?;
        }
        Commands::WebDev { port } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?port,
                "WebDev command invoked"
            );
            handle_web_dev(json, port)?;
        }
        Commands::Sync { action } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                "Sync command invoked"
            );
            match action {
                SyncCommands::Status { remote } => handle_sync_status(json, remote)?,
                SyncCommands::Push { remote, force } => handle_sync_push(json, remote, force)?,
                SyncCommands::Pull { remote, force } => handle_sync_pull(json, remote, force)?,
            }
        }
        Commands::DebugBundle { scenario, out } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                ?scenario,
                ?out,
                "DebugBundle command invoked"
            );
            let scenario_value = scenario.or_else(|| log_context.scenario_id.map(str::to_string));
            let bundle_path = create_debug_bundle(scenario_value.clone(), out)?;
            if json {
                let payload = serde_json::json!({
                    "status": "ok",
                    "path": bundle_path,
                    "scenario": scenario_value,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("{}", bundle_path.display());
            }
        }
        Commands::AgentConfig { target } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                %target,
                "AgentConfig command invoked"
            );
            eprintln!("TODO: implement `context agent-config`");
        }
        Commands::Project { action } => {
            tracing::info!(
                scenario_id = log_context.scenario_id,
                project = log_context.project,
                command = log_context.command,
                "Project command invoked"
            );
            match action {
                ProjectCommands::Current => handle_project_current(json, project)?,
                ProjectCommands::Set {
                    project: new_project,
                } => handle_project_set(json, new_project)?,
                ProjectCommands::List => handle_project_list(json)?,
            }
        }
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

fn handle_web_dev(json_output: bool, port: u16) -> Result<()> {
    let host = "127.0.0.1";
    let addr = format!("http://{host}:{port}");

    if json_output {
        let payload = serde_json::json!({
            "status": "starting",
            "host": host,
            "port": port,
            "url": addr,
            "mode": "dev",
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("Starting context web-dev server on {addr} (wrapper).");
    Ok(())
}

fn handle_sync_status(json_output: bool, remote: Option<PathBuf>) -> Result<()> {
    let cfg = sync_config(remote)?;
    let status = sync::status(&cfg)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    let hash = status
        .local
        .as_ref()
        .map(|m| m.db_hash.as_str())
        .unwrap_or("<none>");

    match status.state {
        sync::SyncState::InSync => println!("Local and remote are in sync (hash {}).", hash),
        sync::SyncState::Ahead => println!("Local is ahead of remote (push recommended)."),
        sync::SyncState::Behind => println!("Remote is ahead of local (pull recommended)."),
        sync::SyncState::Diverged => {
            println!("Local and remote have diverged; resolve with --force push/pull.")
        }
        sync::SyncState::Unknown => println!("No sync metadata yet; try push to initialize."),
    }

    Ok(())
}

fn handle_sync_push(json_output: bool, remote: Option<PathBuf>, force: bool) -> Result<()> {
    let cfg = sync_config(remote)?;
    let result = sync::push(&cfg, force)
        .with_context(|| format!("Failed to push to {}", cfg.remote.display()))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!(
        "Pushed db.sqlite to {} (gen {}, bytes {}, hash {}).",
        cfg.remote.display(),
        result.generation,
        result.db_bytes,
        result.db_hash
    );
    Ok(())
}

fn handle_sync_pull(json_output: bool, remote: Option<PathBuf>, force: bool) -> Result<()> {
    let cfg = sync_config(remote)?;
    let result = match sync::pull(&cfg, force) {
        Ok(res) => res,
        Err(err) => {
            let mut msg = err.to_string();
            if !force && !msg.to_lowercase().contains("force") {
                msg.push_str("; rerun with --force to overwrite");
            }
            return Err(anyhow!(
                "Failed to pull from {}: {}",
                cfg.remote.display(),
                msg
            ));
        }
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!(
        "Pulled db.sqlite from {} (gen {}, bytes {}, hash {}).",
        cfg.remote.display(),
        result.generation,
        result.db_bytes,
        result.db_hash
    );
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
    if json_output {
        let payload = serde_json::json!({
            "status": "ok",
            "project": project,
            "dry_run": dry_run,
            "deleted": 0,
            "vacuumed": !dry_run,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("Garbage collection complete for project {project}");
    if dry_run {
        println!("dry-run (no changes made)");
    } else {
        println!("vacuumed");
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectConfig {
    current: Option<String>,
    known: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            current: None,
            known: vec!["default".to_string()],
        }
    }
}

impl ProjectConfig {
    fn ensure_known(&mut self, project: &str) {
        if project.trim().is_empty() {
            return;
        }
        if !self.known.contains(&project.to_string()) {
            self.known.push(project.to_string());
        }
    }
}

fn resolve_project(project_arg: Option<String>) -> Result<String> {
    if let Some(explicit) = project_arg {
        return Ok(explicit);
    }

    if let Ok(env_project) = env::var("CONTEXT_PROJECT") {
        if !env_project.trim().is_empty() {
            return Ok(env_project);
        }
    }

    let config = load_project_config()?;
    Ok(config.current.unwrap_or_else(|| "default".to_string()))
}

fn handle_project_current(json_output: bool, project_arg: Option<String>) -> Result<()> {
    let project = resolve_project(project_arg)?;
    if json_output {
        let payload = serde_json::json!({ "project": project });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Current project: {project}");
    }
    Ok(())
}

fn handle_project_set(json_output: bool, project: String) -> Result<()> {
    let project = project.trim().to_string();
    if project.is_empty() {
        bail!("Project name cannot be empty.");
    }

    let mut config = load_project_config()?;
    config.current = Some(project.clone());
    config.ensure_known("default");
    config.ensure_known(&project);
    save_project_config(&config)?;

    if json_output {
        let payload = serde_json::json!({
            "status": "ok",
            "project": project,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Set current project to {project}");
    }

    Ok(())
}

fn handle_project_list(json_output: bool) -> Result<()> {
    let mut config = load_project_config()?;
    let current = config.current.clone();
    if let Some(curr) = current.as_deref() {
        config.ensure_known(curr);
    }
    config.ensure_known("default");
    config.known.sort();
    config.known.dedup();

    if json_output {
        println!("{}", serde_json::to_string_pretty(&config.known)?);
        return Ok(());
    }

    println!("Projects:");
    for project in &config.known {
        println!("- {project}");
    }
    Ok(())
}

fn load_project_config() -> Result<ProjectConfig> {
    let path = project_config_path()?;
    if !path.exists() {
        return Ok(ProjectConfig::default());
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read project config at {}", path.display()))?;
    let mut config: ProjectConfig = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse project config at {}", path.display()))?;

    config.ensure_known("default");
    Ok(config)
}

fn save_project_config(config: &ProjectConfig) -> Result<()> {
    let path = project_config_path()?;
    let serialized = serde_json::to_string_pretty(config)?;
    fs::write(&path, serialized)
        .with_context(|| format!("Failed to write project config to {}", path.display()))?;
    Ok(())
}

fn project_config_path() -> Result<PathBuf> {
    let dir = context_home()?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.json"))
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
    let mut stdin = io::stdin();
    stdin
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.trim().is_empty() {
        bail!("No input provided. Use --file or pipe content to stdin.");
    }

    Ok(buffer)
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
        Commands::Sync { .. } => "sync",
        Commands::DebugBundle { .. } => "debug-bundle",
        Commands::AgentConfig { .. } => "agent-config",
        Commands::Project { .. } => "project",
    }
}

fn command_span(log_context: LogContext<'_>, command: &Commands) -> Span {
    match command {
        Commands::AgentDoc { .. } => tracing::info_span!(
            "cli.agent-doc",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Init => tracing::info_span!(
            "cli.init",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Put { .. } => tracing::info_span!(
            "cli.put",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Get { .. } => tracing::info_span!(
            "cli.get",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Cat { .. } => tracing::info_span!(
            "cli.cat",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Find { .. } => tracing::info_span!(
            "cli.find",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Ls {} => tracing::info_span!(
            "cli.ls",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Rm { .. } => tracing::info_span!(
            "cli.rm",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Gc { .. } => tracing::info_span!(
            "cli.gc",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Sync { .. } => tracing::info_span!(
            "cli.sync",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Web { .. } => tracing::info_span!(
            "cli.web",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::WebDev { .. } => tracing::info_span!(
            "cli.web-dev",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::DebugBundle { .. } => tracing::info_span!(
            "cli.debug-bundle",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::AgentConfig { .. } => tracing::info_span!(
            "cli.agent-config",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
        Commands::Project { .. } => tracing::info_span!(
            "cli.project",
            scenario_id = log_context.scenario_id,
            project = log_context.project,
            command = log_context.command
        ),
    }
}

fn resolve_log_dir() -> Result<PathBuf> {
    let log_dir = match env::var("CONTEXT_LOG_DIR") {
        Ok(dir) if Path::new(&dir).is_absolute() => PathBuf::from(dir),
        Ok(dir) => env::current_dir()?.join(dir),
        Err(_) => context_home()?.join("logs"),
    };

    fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

fn create_debug_bundle(scenario: Option<String>, out: Option<String>) -> Result<PathBuf> {
    let log_dir = resolve_log_dir()?;
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let bundle_path = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("debug-bundle-{timestamp}.zip")));

    let file = fs::File::create(&bundle_path)?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let meta = serde_json::json!({
        "scenario_id": scenario,
        "created_at": timestamp,
        "log_dir": log_dir,
    });
    writer.start_file("meta.json", options)?;
    writer.write_all(meta.to_string().as_bytes())?;

    if log_dir.exists() {
        for entry in WalkDir::new(&log_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let rel = entry.path().strip_prefix(&log_dir).unwrap();
            let zip_path = Path::new("logs").join(rel);
            writer.start_file(zip_path.to_string_lossy(), options)?;
            let data = fs::read(entry.path())?;
            writer.write_all(&data)?;
        }
    }

    writer.finish()?;
    Ok(bundle_path)
}

fn sync_config(remote_override: Option<PathBuf>) -> Result<SyncConfig> {
    let home = context_home()?;
    fs::create_dir_all(&home)?;
    let remote = resolve_remote(remote_override, &home)?;
    Ok(SyncConfig {
        local_db: home.join("db.sqlite"),
        local_meta: home.join("sync-meta.json"),
        remote,
    })
}

fn resolve_remote(remote_override: Option<PathBuf>, home: &Path) -> Result<PathBuf> {
    if let Some(remote) = remote_override {
        return absolutize(remote);
    }

    if let Ok(env_remote) = env::var("CONTEXT_SYNC_REMOTE") {
        return absolutize(PathBuf::from(env_remote));
    }

    Ok(home.join("sync-remote"))
}

fn absolutize(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = env::current_dir()?;
    Ok(cwd.join(path))
}

fn context_home() -> Result<PathBuf> {
    if let Ok(home) = env::var("CONTEXT_HOME") {
        let path = PathBuf::from(home);
        return Ok(if path.is_absolute() {
            path
        } else {
            env::current_dir()?.join(path)
        });
    }

    if let Some(home) = dirs::home_dir() {
        return Ok(home.join(".context"));
    }

    Ok(env::current_dir()?.join(".context"))
}
