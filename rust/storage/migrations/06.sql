-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
CREATE TABLE document_notes (
  document_id TEXT NOT NULL PRIMARY KEY
    REFERENCES documents(id) ON DELETE CASCADE,
  note TEXT NOT NULL CHECK(length(note) <= 4000),
  updated_at TEXT NOT NULL
) STRICT;

PRAGMA user_version = 6;
