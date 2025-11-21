use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type ProjectId = String;
pub type Key = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Agent,
    User,
    Import,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub project: ProjectId,
    pub key: Option<Key>,
    pub namespace: Option<String>,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub body_markdown: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: SourceType,
    pub version: u64,
    pub ttl_seconds: Option<i64>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct SearchQuery {
    pub project: Option<ProjectId>,
    pub text: String,
    pub limit: Option<usize>,
}

#[derive(Debug)]
pub struct SearchHit {
    pub document: Document,
    pub score: f32,
}

pub type Result<T> = anyhow::Result<T>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, doc: Document) -> Result<Document>;
    async fn get_by_key(&self, project: &ProjectId, key: &str) -> Result<Option<Document>>;
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>>;
}

pub mod sqlite;
