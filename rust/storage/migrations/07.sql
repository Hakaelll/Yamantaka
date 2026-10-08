-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE document_preferences ADD COLUMN cover_policy TEXT NOT NULL DEFAULT 'noCover';

ALTER TABLE document_preferences ADD COLUMN initial_side TEXT NOT NULL DEFAULT 'left';

PRAGMA user_version = 7;
