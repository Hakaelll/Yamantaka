-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE annotations ADD COLUMN review_state TEXT NOT NULL DEFAULT 'pending' CHECK(review_state IN ('pending', 'accepted', 'resolved'));

ALTER TABLE workspaces ADD COLUMN query_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE workspaces ADD COLUMN grouping TEXT NOT NULL DEFAULT 'none';

ALTER TABLE workspaces ADD COLUMN sort_mode TEXT NOT NULL DEFAULT 'lastOpened';

ALTER TABLE workspaces ADD COLUMN density TEXT NOT NULL DEFAULT 'comfortable';

ALTER TABLE workspaces ADD COLUMN visible_columns_json TEXT NOT NULL DEFAULT '["document","status","progress"]';

ALTER TABLE workspaces ADD COLUMN focus_mode INTEGER NOT NULL DEFAULT 0 CHECK(focus_mode IN (0, 1));

ALTER TABLE workspace_tabs ADD COLUMN queue_status TEXT NOT NULL DEFAULT 'pending' CHECK(queue_status IN ('pending', 'inReview', 'blocked', 'reviewed', 'archived'));

ALTER TABLE workspace_tabs ADD COLUMN blocking_reason TEXT;

ALTER TABLE workspace_tabs ADD COLUMN next_action TEXT;

CREATE TABLE IF NOT EXISTS document_relations (
  id TEXT NOT NULL PRIMARY KEY,
  source_document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  target_document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  relation_type TEXT NOT NULL CHECK(relation_type IN (
    'related', 'replaces', 'sourceOf', 'attachmentOf'
  )),
  source_anchor_kind TEXT,
  source_anchor_id TEXT,
  target_anchor_kind TEXT,
  target_anchor_id TEXT,
  created_at TEXT NOT NULL,
  CHECK(source_document_id <> target_document_id),
  UNIQUE(source_document_id, target_document_id, relation_type)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_document_relations_source ON document_relations(source_document_id, relation_type);

CREATE INDEX IF NOT EXISTS idx_document_relations_target ON document_relations(target_document_id, relation_type);

CREATE TABLE IF NOT EXISTS document_revision_groups (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 120),
  current_document_id TEXT
    REFERENCES documents(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS document_revision_members (
  group_id TEXT NOT NULL REFERENCES document_revision_groups(id)
    ON DELETE CASCADE,
  document_id TEXT NOT NULL REFERENCES documents(id)
    ON DELETE CASCADE,
  added_at TEXT NOT NULL,
  PRIMARY KEY(group_id, document_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_revision_members_document ON document_revision_members(document_id);

CREATE TABLE IF NOT EXISTS automation_rules (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 120),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  trigger_json TEXT NOT NULL,
  actions_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS operation_journal (
  id TEXT NOT NULL PRIMARY KEY,
  operation_kind TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN (
    'planned', 'running', 'completed', 'cancelled',
    'interrupted', 'rolledBack', 'failed'
  )),
  summary_json TEXT NOT NULL DEFAULT '{}',
  total_items INTEGER NOT NULL DEFAULT 0 CHECK(total_items >= 0),
  completed_items INTEGER NOT NULL DEFAULT 0
    CHECK(completed_items >= 0),
  undo_json TEXT,
  error_code TEXT,
  started_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_operation_journal_status ON operation_journal(status, updated_at DESC);

PRAGMA user_version = 10;
