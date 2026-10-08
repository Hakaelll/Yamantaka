-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
CREATE TABLE annotations (
  id TEXT NOT NULL PRIMARY KEY,
  document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  page_index INTEGER NOT NULL CHECK(page_index >= 0),
  type TEXT NOT NULL,
  color TEXT NOT NULL,
  note TEXT,
  selected_text TEXT,
  context_before TEXT,
  context_after TEXT,
  rects_json TEXT NOT NULL DEFAULT '[]',
  page_rotation INTEGER NOT NULL DEFAULT 0,
  document_fingerprint TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'attached',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_annotations_document_page ON annotations(document_id, page_index);

CREATE INDEX idx_annotations_document_updated ON annotations(document_id, updated_at DESC);

CREATE TABLE tags (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  normalized_name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE annotation_tags (
  annotation_id TEXT NOT NULL
    REFERENCES annotations(id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY(annotation_id, tag_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE document_tags (
  document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY(document_id, tag_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE collections (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  normalized_name TEXT NOT NULL UNIQUE,
  icon TEXT NOT NULL DEFAULT 'folder',
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_pinned INTEGER NOT NULL DEFAULT 0
    CHECK(is_pinned IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE collection_documents (
  collection_id TEXT NOT NULL
    REFERENCES collections(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  added_at TEXT NOT NULL,
  PRIMARY KEY(collection_id, document_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE workspaces (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0
    CHECK(is_default IN (0, 1)),
  active_tab_id TEXT,
  panel_section TEXT,
  panel_width REAL NOT NULL DEFAULT 320,
  split_json TEXT,
  collection_id TEXT REFERENCES collections(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE workspace_tabs (
  id TEXT NOT NULL PRIMARY KEY,
  workspace_id TEXT NOT NULL
    REFERENCES workspaces(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL
    REFERENCES documents(id) ON DELETE CASCADE,
  sort_order INTEGER NOT NULL,
  is_pinned INTEGER NOT NULL DEFAULT 0
    CHECK(is_pinned IN (0, 1)),
  page_index INTEGER NOT NULL DEFAULT 0 CHECK(page_index >= 0),
  zoom_level REAL NOT NULL DEFAULT 1 CHECK(zoom_level > 0),
  page_layout TEXT NOT NULL DEFAULT 'single',
  reading_mode TEXT NOT NULL DEFAULT 'document',
  scroll_offset REAL NOT NULL DEFAULT 0,
  UNIQUE(workspace_id, document_id)
) STRICT;

CREATE INDEX idx_workspace_tabs_order ON workspace_tabs(workspace_id, sort_order);

CREATE TABLE reading_profiles (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  is_builtin INTEGER NOT NULL DEFAULT 0
    CHECK(is_builtin IN (0, 1)),
  settings_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE VIRTUAL TABLE library_search USING fts5(
  document_id UNINDEXED,
  page_index UNINDEXED,
  kind UNINDEXED,
  title,
  body,
  tags,
  tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TABLE search_index_state (
  document_id TEXT NOT NULL PRIMARY KEY
    REFERENCES documents(id) ON DELETE CASCADE,
  fingerprint TEXT NOT NULL,
  status TEXT NOT NULL,
  indexed_pages INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL,
  error_code TEXT
) STRICT;

PRAGMA user_version = 2;
