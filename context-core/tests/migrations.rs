use std::str::FromStr;

use chrono::Utc;
use context_core::{sqlite::run_migrations, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

async fn test_pool() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    Ok(pool)
}

#[tokio::test]
async fn migrations_create_expected_tables() -> Result<()> {
    let pool = test_pool().await?;
    run_migrations(&pool).await?;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master \
         WHERE type = 'table' AND name IN ('projects', 'documents', 'document_versions', 'documents_fts') \
         ORDER BY name",
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(
        tables,
        vec![
            "document_versions".to_string(),
            "documents".to_string(),
            "documents_fts".to_string(),
            "projects".to_string()
        ]
    );

    Ok(())
}

#[tokio::test]
async fn documents_enforce_unique_key_per_project() -> Result<()> {
    let pool = test_pool().await?;
    run_migrations(&pool).await?;

    sqlx::query("INSERT INTO projects (id) VALUES (?)")
        .bind("demo")
        .execute(&pool)
        .await?;

    let tags = serde_json::to_string(&["alpha"])?;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO documents (id, project_id, key, namespace, title, tags, body_markdown, created_at, updated_at, source, version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("doc-1")
    .bind("demo")
    .bind("install")
    .bind("notes")
    .bind("Install Guide")
    .bind(&tags)
    .bind("Rust setup instructions")
    .bind(&now)
    .bind(&now)
    .bind("User")
    .bind(1_i64)
    .execute(&pool)
    .await?;

    let duplicate = sqlx::query(
        "INSERT INTO documents (id, project_id, key, namespace, title, tags, body_markdown, created_at, updated_at, source, version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("doc-2")
    .bind("demo")
    .bind("install")
    .bind("notes")
    .bind("Install Guide Copy")
    .bind(&tags)
    .bind("Rust setup instructions")
    .bind(&now)
    .bind(&now)
    .bind("User")
    .bind(1_i64)
    .execute(&pool)
    .await;

    assert!(
        duplicate.is_err(),
        "second document with same key in project should fail"
    );

    Ok(())
}

#[tokio::test]
async fn full_text_index_tracks_document_changes() -> Result<()> {
    let pool = test_pool().await?;
    run_migrations(&pool).await?;

    sqlx::query("INSERT INTO projects (id) VALUES (?)")
        .bind("demo")
        .execute(&pool)
        .await?;

    let now = Utc::now().to_rfc3339();
    let tags = serde_json::to_string(&["rust", "search"])?;

    sqlx::query(
        "INSERT INTO documents (id, project_id, key, namespace, title, tags, body_markdown, created_at, updated_at, source, version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("doc-fts")
    .bind("demo")
    .bind("search-doc")
    .bind("notes")
    .bind("Search Notes")
    .bind(&tags)
    .bind("rust search works")
    .bind(&now)
    .bind(&now)
    .bind("System")
    .bind(1_i64)
    .execute(&pool)
    .await?;

    let hits = search_ids(&pool, "rust", "demo").await?;
    assert_eq!(hits, vec!["doc-fts".to_string()]);

    let later = Utc::now().to_rfc3339();
    let new_tags = serde_json::to_string(&["sqlite", "fts"])?;

    sqlx::query(
        "UPDATE documents \
         SET body_markdown = ?, tags = ?, updated_at = ?, version = version + 1 \
         WHERE id = ?",
    )
    .bind("sqlite fts is great")
    .bind(&new_tags)
    .bind(&later)
    .bind("doc-fts")
    .execute(&pool)
    .await?;

    assert!(
        search_ids(&pool, "rust", "demo").await?.is_empty(),
        "old content should no longer be indexed"
    );
    assert_eq!(
        search_ids(&pool, "sqlite", "demo").await?,
        vec!["doc-fts".to_string()]
    );

    sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind("doc-fts")
        .execute(&pool)
        .await?;

    assert!(
        search_ids(&pool, "sqlite", "demo").await?.is_empty(),
        "deleting document removes from FTS index"
    );

    Ok(())
}

async fn search_ids(pool: &SqlitePool, term: &str, project: &str) -> Result<Vec<String>> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT document_id FROM documents_fts WHERE documents_fts MATCH ? AND project_id = ? ORDER BY rowid",
    )
    .bind(term)
    .bind(project)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
