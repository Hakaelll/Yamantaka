-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE documents ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0 CHECK(is_favorite IN (0, 1));

CREATE TABLE document_covers (
  document_id TEXT NOT NULL PRIMARY KEY
    REFERENCES documents(id) ON DELETE CASCADE,
  source_page_index INTEGER NOT NULL CHECK(source_page_index >= 0),
  small_thumbnail_path TEXT NOT NULL,
  large_thumbnail_path TEXT NOT NULL,
  generated_at TEXT NOT NULL,
  source_modified_at TEXT NOT NULL,
  confidence REAL NOT NULL CHECK(confidence BETWEEN 0 AND 1),
  selection_mode TEXT NOT NULL
    CHECK(selection_mode IN ('automatic', 'manual')),
  algorithm_version INTEGER NOT NULL CHECK(algorithm_version > 0),
  crop_mode TEXT NOT NULL
    CHECK(crop_mode IN ('contain')),
  page_aspect_ratio REAL NOT NULL
    CHECK(page_aspect_ratio BETWEEN 0.1 AND 10)
) STRICT;

CREATE INDEX idx_document_covers_generated ON document_covers(generated_at);

PRAGMA user_version = 3;
