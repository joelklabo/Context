use chrono::{DateTime, Utc};
use sqlx::{migrate::Migrator, sqlite::SqliteRow, Row, SqlitePool};

use crate::{
    Document, DocumentId, Key, ProjectId, Result, SearchHit, SearchQuery, SourceType, Storage,
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Run database migrations for the SQLite backend.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        run_migrations(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn deserialize_row(row: SqliteRow) -> Result<Document> {
        let tags_json: String = row.try_get("tags")?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;

        let created_at: String = row.try_get("created_at")?;
        let updated_at: String = row.try_get("updated_at")?;
        let deleted_at: Option<String> = row.try_get("deleted_at")?;

        let source_raw: String = row.try_get("source")?;
        let source = match source_raw.as_str() {
            "Agent" => SourceType::Agent,
            "User" => SourceType::User,
            "Import" => SourceType::Import,
            "System" => SourceType::System,
            other => {
                return Err(format!("unknown source type: {other}").into());
            }
        };

        Ok(Document {
            id: DocumentId(row.try_get("id")?),
            project: row.try_get::<String, _>("project_id")?,
            key: row.try_get::<Option<Key>, _>("key")?,
            namespace: row.try_get("namespace")?,
            title: row.try_get("title")?,
            tags,
            body_markdown: row.try_get("body_markdown")?,
            created_at: parse_datetime(&created_at)?,
            updated_at: parse_datetime(&updated_at)?,
            source,
            version: row.try_get::<i64, _>("version")? as u64,
            ttl_seconds: row.try_get("ttl_seconds")?,
            deleted_at: match deleted_at {
                Some(ts) => Some(parse_datetime(&ts)?),
                None => None,
            },
        })
    }
}

#[async_trait::async_trait]
impl Storage for SqliteStorage {
    async fn put(&self, doc: Document) -> Result<Document> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT OR IGNORE INTO projects (id) VALUES (?)")
            .bind(&doc.project)
            .execute(&mut *tx)
            .await?;

        let tags = serde_json::to_string(&doc.tags)?;

        sqlx::query(
            "INSERT INTO documents (id, project_id, key, namespace, title, tags, body_markdown, created_at, updated_at, source, version, ttl_seconds, deleted_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
                 project_id=excluded.project_id, \
                 key=excluded.key, \
                 namespace=excluded.namespace, \
                 title=excluded.title, \
                 tags=excluded.tags, \
                 body_markdown=excluded.body_markdown, \
                 created_at=excluded.created_at, \
                 updated_at=excluded.updated_at, \
                 source=excluded.source, \
                 version=excluded.version, \
                 ttl_seconds=excluded.ttl_seconds, \
                 deleted_at=excluded.deleted_at",
        )
        .bind(&doc.id.0)
        .bind(&doc.project)
        .bind(&doc.key)
        .bind(&doc.namespace)
        .bind(&doc.title)
        .bind(&tags)
        .bind(&doc.body_markdown)
        .bind(doc.created_at.to_rfc3339())
        .bind(doc.updated_at.to_rfc3339())
        .bind(format!("{:?}", doc.source))
        .bind(doc.version as i64)
        .bind(doc.ttl_seconds)
        .bind(doc.deleted_at.map(|t| t.to_rfc3339()))
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO document_versions (document_id, version, title, tags, body_markdown, namespace, key, source, ttl_seconds, deleted_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&doc.id.0)
        .bind(doc.version as i64)
        .bind(&doc.title)
        .bind(&tags)
        .bind(&doc.body_markdown)
        .bind(&doc.namespace)
        .bind(&doc.key)
        .bind(format!("{:?}", doc.source))
        .bind(doc.ttl_seconds)
        .bind(doc.deleted_at.map(|t| t.to_rfc3339()))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(doc)
    }

    async fn get_by_key(&self, project: &ProjectId, key: &str) -> Result<Option<Document>> {
        let row = sqlx::query(
            "SELECT * FROM documents WHERE project_id = ? AND key = ? AND deleted_at IS NULL LIMIT 1",
        )
        .bind(project)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Self::deserialize_row(row)?)),
            None => Ok(None),
        }
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        let project = query.project.clone();
        let limit: i64 = query.limit.map(|l| l as i64).unwrap_or(-1);

        let rows = sqlx::query(
            "SELECT d.* FROM documents_fts \
             JOIN documents d ON d.id = documents_fts.document_id \
             WHERE documents_fts MATCH ? AND (? IS NULL OR documents_fts.project_id = ?) AND d.deleted_at IS NULL \
             ORDER BY d.updated_at DESC \
             LIMIT ?",
        )
        .bind(&query.text)
        .bind(&project)
        .bind(&project)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut hits = Vec::with_capacity(rows.len());
        for row in rows {
            let doc = Self::deserialize_row(row)?;
            hits.push(SearchHit {
                document: doc,
                score: 0.0,
            });
        }

        Ok(hits)
    }
}

fn parse_datetime(raw: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(raw)?.with_timezone(&Utc))
}
