-- Projects table captures the logical namespace for documents.
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Documents stores the latest version of each document.
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    key TEXT,
    namespace TEXT,
    title TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    body_markdown TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    source TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    ttl_seconds INTEGER,
    deleted_at TEXT,
    CONSTRAINT fk_documents_project FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CONSTRAINT documents_source_valid CHECK (source IN ('Agent', 'User', 'Import', 'System'))
);

-- Unique user key per project (multiple NULL keys OK).
CREATE UNIQUE INDEX idx_documents_project_key ON documents(project_id, key) WHERE key IS NOT NULL;
CREATE INDEX idx_documents_project ON documents(project_id);
CREATE INDEX idx_documents_project_updated ON documents(project_id, updated_at);

-- Version history retains every revision for auditing and future features.
CREATE TABLE document_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    title TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    body_markdown TEXT NOT NULL,
    namespace TEXT,
    key TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ttl_seconds INTEGER,
    deleted_at TEXT,
    CONSTRAINT fk_document_versions_document FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    CONSTRAINT document_versions_source_valid CHECK (source IN ('Agent', 'User', 'Import', 'System')),
    CONSTRAINT version_unique UNIQUE (document_id, version)
);

CREATE INDEX idx_document_versions_document ON document_versions(document_id);

-- Full-text search index on the latest document content.
CREATE VIRTUAL TABLE documents_fts USING fts5(
    document_id UNINDEXED,
    project_id UNINDEXED,
    title,
    body,
    tags,
    namespace
);

CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, document_id, project_id, title, body, tags, namespace)
    VALUES (
        new.rowid,
        new.id,
        new.project_id,
        coalesce(new.title, ''),
        new.body_markdown,
        coalesce((SELECT group_concat(value, ' ') FROM json_each(new.tags)), ''),
        coalesce(new.namespace, '')
    );
END;

CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
    DELETE FROM documents_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
    DELETE FROM documents_fts WHERE rowid = old.rowid;
    INSERT INTO documents_fts(rowid, document_id, project_id, title, body, tags, namespace)
    VALUES (
        new.rowid,
        new.id,
        new.project_id,
        coalesce(new.title, ''),
        new.body_markdown,
        coalesce((SELECT group_concat(value, ' ') FROM json_each(new.tags)), ''),
        coalesce(new.namespace, '')
    );
END;
