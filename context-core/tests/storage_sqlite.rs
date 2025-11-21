use std::str::FromStr;

use chrono::{TimeZone, Utc};
use context_core::{
    sqlite::SqliteStorage, Document, DocumentId, Key, ProjectId, SearchQuery, SourceType, Storage,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn test_pool() -> TestResult<SqlitePool> {
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

async fn test_storage() -> TestResult<SqliteStorage> {
    let pool = test_pool().await?;
    let storage = SqliteStorage::new(pool).await?;
    Ok(storage)
}

fn sample_document(id: &str, project: &str, key: &str, body: &str) -> Document {
    let now = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    Document {
        id: DocumentId(id.to_string()),
        project: ProjectId::from(project),
        key: Some(Key::from(key)),
        namespace: Some("notes".to_string()),
        title: Some("Sample".to_string()),
        tags: vec!["rust".to_string()],
        body_markdown: body.to_string(),
        created_at: now,
        updated_at: now,
        source: SourceType::User,
        version: 1,
        ttl_seconds: None,
        deleted_at: None,
    }
}

#[tokio::test]
async fn put_and_get_by_key_roundtrip() -> TestResult<()> {
    let storage = test_storage().await?;
    let doc = sample_document("doc-1", "demo", "intro", "hello world");

    storage.put(doc.clone()).await?;

    let fetched = storage
        .get_by_key(&doc.project, doc.key.as_ref().unwrap())
        .await?;

    let fetched = fetched.expect("document exists");
    assert_eq!(fetched.id.0, doc.id.0);
    assert_eq!(fetched.project, doc.project);
    assert_eq!(fetched.key, doc.key);
    assert_eq!(fetched.body_markdown, doc.body_markdown);
    assert_eq!(fetched.version, doc.version);

    Ok(())
}

#[tokio::test]
async fn put_overwrites_existing_document_by_id() -> TestResult<()> {
    let storage = test_storage().await?;
    let mut doc = sample_document("doc-1", "demo", "intro", "v1");

    storage.put(doc.clone()).await?;

    doc.body_markdown = "v2 body".to_string();
    doc.version = 2;
    doc.updated_at += chrono::Duration::minutes(5);

    storage.put(doc.clone()).await?;

    let fetched = storage
        .get_by_key(&doc.project, doc.key.as_ref().unwrap())
        .await?
        .expect("document exists");

    assert_eq!(fetched.body_markdown, "v2 body");
    assert_eq!(fetched.version, 2);

    // document_versions table should record both revisions.
    let versions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM document_versions WHERE document_id = ?")
            .bind(&doc.id.0)
            .fetch_one(storage.pool())
            .await?;

    assert_eq!(versions, 2);

    Ok(())
}

#[tokio::test]
async fn search_returns_matches_in_project() -> TestResult<()> {
    let storage = test_storage().await?;

    let rust_doc = sample_document("doc-rust", "proj-a", "rust", "rust search works");
    let mut python_doc = sample_document("doc-py", "proj-a", "py", "python tips");
    python_doc.tags = vec!["python".to_string()];
    let mut other_project_doc =
        sample_document("doc-other", "proj-b", "rust", "rust in another project");
    other_project_doc.tags = vec!["rust".to_string()];

    storage.put(rust_doc.clone()).await?;
    storage.put(python_doc).await?;
    storage.put(other_project_doc).await?;

    let hits = storage
        .search(SearchQuery {
            project: Some(rust_doc.project.clone()),
            text: "rust".to_string(),
            limit: None,
        })
        .await?;

    let ids: Vec<_> = hits.into_iter().map(|h| h.document.id.0).collect();
    assert_eq!(ids, vec!["doc-rust".to_string()]);

    Ok(())
}
