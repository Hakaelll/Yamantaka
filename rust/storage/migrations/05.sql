-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.
ALTER TABLE documents ADD COLUMN source_title TEXT;

ALTER TABLE documents ADD COLUMN source_author TEXT;

ALTER TABLE documents ADD COLUMN title_override TEXT;

ALTER TABLE documents ADD COLUMN author_override TEXT;

UPDATE documents SET source_title = title WHERE source_title IS NULL;

UPDATE documents SET source_author = author WHERE source_author IS NULL;

PRAGMA user_version = 5;
