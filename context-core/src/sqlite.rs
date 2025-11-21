use std::cmp::Ordering;

use anyhow::bail;
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
            other => bail!("unknown source type: {other}"),
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
            "SELECT * FROM documents \
             WHERE project_id = ? \
               AND key = ? \
               AND deleted_at IS NULL \
               AND (ttl_seconds IS NULL OR strftime('%s','now') < strftime('%s', created_at) + ttl_seconds) \
             LIMIT 1",
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
            "SELECT d.*, bm25(documents_fts) AS bm25_score FROM documents_fts \
             JOIN documents d ON d.id = documents_fts.document_id \
             WHERE documents_fts MATCH ? AND (? IS NULL OR documents_fts.project_id = ?) AND d.deleted_at IS NULL \
               AND (d.ttl_seconds IS NULL OR strftime('%s','now') < strftime('%s', d.created_at) + d.ttl_seconds) \
             ORDER BY bm25_score ASC \
             LIMIT ?",
        )
        .bind(&query.text)
        .bind(&project)
        .bind(&project)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let terms: Vec<String> = query
            .text
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .collect();
        let now = Utc::now();

        let mut hits = Vec::with_capacity(rows.len());
        for row in rows {
            let bm25_score: f32 = row.try_get("bm25_score")?;
            let doc = Self::deserialize_row(row)?;
            let text_score = -bm25_score;
            let recency_score = recency_score(&doc, now);
            let tag_score = tag_match_bonus(&doc.tags, &terms);
            let total_score = text_score + recency_score + tag_score;
            hits.push(SearchHit {
                document: doc,
                score: total_score,
            });
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    b.document
                        .updated_at
                        .partial_cmp(&a.document.updated_at)
                        .unwrap_or(Ordering::Equal)
                })
        });

        if let Some(max) = query.limit {
            hits.truncate(max);
        }

        Ok(hits)
    }
}

fn parse_datetime(raw: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(raw)?.with_timezone(&Utc))
}

fn recency_score(doc: &Document, now: DateTime<Utc>) -> f32 {
    let age_secs = (now - doc.updated_at).num_seconds().max(0) as f32;
    1.0 / (1.0 + age_secs / 3600.0)
}

fn tag_match_bonus(tags: &[String], terms: &[String]) -> f32 {
    let mut matches = 0;
    for tag in tags {
        let tag_lower = tag.to_lowercase();
        if terms.contains(&tag_lower) {
            matches += 1;
        }
    }

    matches as f32 * 0.5
}
