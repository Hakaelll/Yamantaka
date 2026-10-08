-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE documents ADD COLUMN is_modified INTEGER NOT NULL DEFAULT 0 CHECK(is_modified IN (0, 1));

ALTER TABLE documents ADD COLUMN last_validated_at TEXT;

CREATE INDEX IF NOT EXISTS idx_documents_validation ON documents(last_validated_at, id);

CREATE INDEX IF NOT EXISTS idx_documents_opened_stable ON documents(last_opened_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_documents_favorite_opened_stable ON documents(is_favorite, last_opened_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_documents_added_stable ON documents(added_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_documents_progress_stable ON documents(reading_progress DESC, id ASC);

PRAGMA user_version = 9;
