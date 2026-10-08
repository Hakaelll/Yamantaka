-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
CREATE INDEX IF NOT EXISTS idx_documents_missing_opened ON documents(is_missing, last_opened_at DESC);

CREATE INDEX IF NOT EXISTS idx_documents_favorite_opened ON documents(is_favorite, last_opened_at DESC);

CREATE INDEX IF NOT EXISTS idx_documents_title_nocase ON documents(title COLLATE NOCASE);

CREATE INDEX IF NOT EXISTS idx_documents_modified ON documents(file_modified_at DESC);

CREATE INDEX IF NOT EXISTS idx_search_state_status ON search_index_state(status, updated_at DESC);

PRAGMA user_version = 4;
