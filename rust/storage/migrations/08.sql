-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
CREATE TABLE document_enrichment (
  document_id TEXT NOT NULL PRIMARY KEY
    REFERENCES documents(id) ON DELETE CASCADE,
  fingerprint TEXT NOT NULL,
  analyzer_version INTEGER NOT NULL CHECK(analyzer_version > 0),
  status TEXT NOT NULL CHECK(status IN (
    'pending', 'analyzing', 'completed', 'review', 'noText', 'failed'
  )),
  suggested_title TEXT,
  suggested_author TEXT,
  title_confidence REAL CHECK(title_confidence BETWEEN 0 AND 1),
  author_confidence REAL CHECK(author_confidence BETWEEN 0 AND 1),
  topics_manual INTEGER NOT NULL DEFAULT 0
    CHECK(topics_manual IN (0, 1)),
  analyzed_at TEXT,
  error_code TEXT
) STRICT;

CREATE TABLE document_topics (
  document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  topic_key TEXT NOT NULL,
  confidence REAL NOT NULL CHECK(confidence BETWEEN 0 AND 1),
  origin TEXT NOT NULL CHECK(origin IN ('automatic', 'manual')),
  analyzer_version INTEGER NOT NULL CHECK(analyzer_version > 0),
  fingerprint TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(document_id, topic_key)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_document_enrichment_status ON document_enrichment(status, analyzed_at);

CREATE INDEX idx_document_topics_topic ON document_topics(topic_key, document_id);

PRAGMA user_version = 8;
