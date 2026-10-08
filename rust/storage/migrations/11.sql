-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE documents ADD COLUMN last_failure_code TEXT;

ALTER TABLE documents ADD COLUMN restore_blocked INTEGER NOT NULL DEFAULT 0 CHECK(restore_blocked IN (0, 1));

ALTER TABLE annotations ADD COLUMN origin_document_id TEXT;

ALTER TABLE annotations ADD COLUMN origin_fingerprint TEXT;

ALTER TABLE operation_journal ADD COLUMN phase TEXT NOT NULL DEFAULT 'planned';

ALTER TABLE operation_journal ADD COLUMN interruption_reason TEXT;

ALTER TABLE operation_journal ADD COLUMN resume_capability TEXT NOT NULL DEFAULT 'none' CHECK(resume_capability IN ('none', 'retry', 'resume', 'rollback'));

ALTER TABLE operation_journal ADD COLUMN compatibility_hash TEXT;

CREATE TABLE IF NOT EXISTS duplicate_merge_history (
  id TEXT NOT NULL PRIMARY KEY,
  kept_document_id TEXT NOT NULL,
  incoming_document_id TEXT NOT NULL,
  backup_path TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN (
    'completed', 'rolledBack', 'failed'
  )),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_duplicate_merge_history_status ON duplicate_merge_history(status, updated_at DESC);

PRAGMA user_version = 11;
