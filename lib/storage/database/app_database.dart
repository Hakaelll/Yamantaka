import 'package:sqlite3/sqlite3.dart';

import '../../app/app_config.dart';
import '../../core/logging/app_logger.dart';

final class AppDatabase {
  AppDatabase._(this.database, this.logger);

  final Database database;
  final AppLogger logger;

  static AppDatabase open(String path, AppLogger logger) {
    final database = sqlite3.open(path);
    final appDatabase = AppDatabase._(database, logger);
    appDatabase._configure();
    appDatabase._backupBeforeMajorMigration(path);
    appDatabase._migrate();
    if (appDatabase.integrityCheck() != 'ok') {
      database.close();
      throw StateError('Database integrity check failed after migration.');
    }
    return appDatabase;
  }

  static AppDatabase inMemory(AppLogger logger) {
    final appDatabase = AppDatabase._(sqlite3.openInMemory(), logger);
    appDatabase._configure();
    appDatabase._migrate();
    return appDatabase;
  }

  void _configure() {
    database
      ..execute('PRAGMA foreign_keys = ON')
      ..execute('PRAGMA journal_mode = WAL')
      ..execute('PRAGMA synchronous = NORMAL')
      ..execute('PRAGMA busy_timeout = 3000');
  }

  void _backupBeforeMajorMigration(String path) {
    final currentVersion =
        database.select('PRAGMA user_version').first.columnAt(0) as int;
    if (currentVersion == 0 || currentVersion >= AppConfig.databaseVersion) {
      return;
    }
    final timestamp = DateTime.now().toUtc().millisecondsSinceEpoch;
    final backupPath =
        '$path.pre-migration-v$currentVersion-$timestamp.sqlite3';
    final escaped = backupPath.replaceAll("'", "''");
    database.execute("VACUUM INTO '$escaped'");
    logger.info('Created a local pre-migration database backup.');
  }

  void _migrate() {
    final currentVersion =
        database.select('PRAGMA user_version').first.columnAt(0) as int;
    if (currentVersion > AppConfig.databaseVersion) {
      throw StateError(
        'Database version $currentVersion is newer than supported '
        '${AppConfig.databaseVersion}.',
      );
    }
    if (currentVersion == 0) {
      _transaction(() {
        database
          ..execute('''
            CREATE TABLE documents (
              id TEXT NOT NULL PRIMARY KEY,
              path TEXT NOT NULL,
              normalized_path TEXT NOT NULL UNIQUE,
              file_name TEXT NOT NULL,
              title TEXT,
              author TEXT,
              page_count INTEGER NOT NULL CHECK(page_count >= 0),
              file_size INTEGER NOT NULL CHECK(file_size >= 0),
              file_modified_at TEXT NOT NULL,
              added_at TEXT NOT NULL,
              last_opened_at TEXT NOT NULL,
              last_page_index INTEGER NOT NULL DEFAULT 0 CHECK(last_page_index >= 0),
              reading_progress REAL NOT NULL DEFAULT 0 CHECK(reading_progress BETWEEN 0 AND 1),
              preferred_view_mode TEXT NOT NULL DEFAULT 'document',
              preferred_theme TEXT NOT NULL DEFAULT 'ivory',
              is_missing INTEGER NOT NULL DEFAULT 0 CHECK(is_missing IN (0, 1)),
              fingerprint TEXT NOT NULL,
              thumbnail_path TEXT
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_documents_last_opened '
            'ON documents(last_opened_at DESC)',
          )
          ..execute(
            'CREATE INDEX idx_documents_fingerprint '
            'ON documents(fingerprint)',
          )
          ..execute('''
            CREATE TABLE document_preferences (
              document_id TEXT NOT NULL PRIMARY KEY
                REFERENCES documents(id) ON DELETE CASCADE,
              view_mode TEXT NOT NULL,
              reading_theme TEXT NOT NULL,
              zoom_mode TEXT NOT NULL,
              zoom_level REAL NOT NULL CHECK(zoom_level > 0),
              page_layout TEXT NOT NULL,
              page_rotation INTEGER NOT NULL DEFAULT 0,
              horizontal_margin REAL NOT NULL DEFAULT 32,
              vertical_margin REAL NOT NULL DEFAULT 24,
              controls_auto_hide INTEGER NOT NULL DEFAULT 1
                CHECK(controls_auto_hide IN (0, 1)),
              scroll_offset REAL NOT NULL DEFAULT 0,
              updated_at TEXT NOT NULL
            ) STRICT
          ''')
          ..execute('''
            CREATE TABLE bookmarks (
              id TEXT NOT NULL PRIMARY KEY,
              document_id TEXT NOT NULL
                REFERENCES documents(id) ON DELETE CASCADE,
              page_index INTEGER NOT NULL CHECK(page_index >= 0),
              label TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(document_id, page_index)
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_bookmarks_document_page '
            'ON bookmarks(document_id, page_index)',
          )
          ..execute('''
            CREATE TABLE reading_sessions (
              id TEXT NOT NULL PRIMARY KEY,
              document_id TEXT NOT NULL
                REFERENCES documents(id) ON DELETE CASCADE,
              started_at TEXT NOT NULL,
              ended_at TEXT,
              start_page INTEGER NOT NULL CHECK(start_page >= 0),
              end_page INTEGER NOT NULL CHECK(end_page >= 0),
              active_seconds INTEGER NOT NULL DEFAULT 0
                CHECK(active_seconds >= 0)
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_sessions_document_started '
            'ON reading_sessions(document_id, started_at DESC)',
          )
          ..execute('''
            CREATE TABLE app_metadata (
              key TEXT NOT NULL PRIMARY KEY,
              value TEXT NOT NULL,
              updated_at TEXT NOT NULL
            ) STRICT
          ''')
          ..execute('PRAGMA user_version = 1');
      });
      logger.info('Created database schema version 1.');
    }
    final versionAfterBootstrap =
        database.select('PRAGMA user_version').first.columnAt(0) as int;
    if (versionAfterBootstrap < 2) {
      _transaction(() {
        database
          ..execute('''
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
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_annotations_document_page '
            'ON annotations(document_id, page_index)',
          )
          ..execute(
            'CREATE INDEX idx_annotations_document_updated '
            'ON annotations(document_id, updated_at DESC)',
          )
          ..execute('''
            CREATE TABLE tags (
              id TEXT NOT NULL PRIMARY KEY,
              name TEXT NOT NULL,
              normalized_name TEXT NOT NULL UNIQUE,
              color TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            ) STRICT
          ''')
          ..execute('''
            CREATE TABLE annotation_tags (
              annotation_id TEXT NOT NULL
                REFERENCES annotations(id) ON DELETE CASCADE,
              tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
              PRIMARY KEY(annotation_id, tag_id)
            ) STRICT, WITHOUT ROWID
          ''')
          ..execute('''
            CREATE TABLE document_tags (
              document_id TEXT NOT NULL
                REFERENCES documents(id) ON DELETE CASCADE,
              tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
              PRIMARY KEY(document_id, tag_id)
            ) STRICT, WITHOUT ROWID
          ''')
          ..execute('''
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
            ) STRICT
          ''')
          ..execute('''
            CREATE TABLE collection_documents (
              collection_id TEXT NOT NULL
                REFERENCES collections(id) ON DELETE CASCADE,
              document_id TEXT NOT NULL
                REFERENCES documents(id) ON DELETE CASCADE,
              added_at TEXT NOT NULL,
              PRIMARY KEY(collection_id, document_id)
            ) STRICT, WITHOUT ROWID
          ''')
          ..execute('''
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
            ) STRICT
          ''')
          ..execute('''
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
              page_layout TEXT NOT NULL DEFAULT 'continuous',
              reading_mode TEXT NOT NULL DEFAULT 'document',
              scroll_offset REAL NOT NULL DEFAULT 0,
              UNIQUE(workspace_id, document_id)
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_workspace_tabs_order '
            'ON workspace_tabs(workspace_id, sort_order)',
          )
          ..execute('''
            CREATE TABLE reading_profiles (
              id TEXT NOT NULL PRIMARY KEY,
              name TEXT NOT NULL,
              is_builtin INTEGER NOT NULL DEFAULT 0
                CHECK(is_builtin IN (0, 1)),
              settings_json TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            ) STRICT
          ''')
          ..execute('''
            CREATE VIRTUAL TABLE library_search USING fts5(
              document_id UNINDEXED,
              page_index UNINDEXED,
              kind UNINDEXED,
              title,
              body,
              tags,
              tokenize = 'unicode61 remove_diacritics 2'
            )
          ''')
          ..execute('''
            CREATE TABLE search_index_state (
              document_id TEXT NOT NULL PRIMARY KEY
                REFERENCES documents(id) ON DELETE CASCADE,
              fingerprint TEXT NOT NULL,
              status TEXT NOT NULL,
              indexed_pages INTEGER NOT NULL DEFAULT 0,
              updated_at TEXT NOT NULL,
              error_code TEXT
            ) STRICT
          ''')
          ..execute('PRAGMA user_version = 2');
      });
      logger.info('Migrated database schema to version 2.');
    }
    final versionAfterCapabilities =
        database.select('PRAGMA user_version').first.columnAt(0) as int;
    if (versionAfterCapabilities < 3) {
      _transaction(() {
        database
          ..execute(
            'ALTER TABLE documents ADD COLUMN is_favorite INTEGER NOT NULL '
            'DEFAULT 0 CHECK(is_favorite IN (0, 1))',
          )
          ..execute('''
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
            ) STRICT
          ''')
          ..execute(
            'CREATE INDEX idx_document_covers_generated '
            'ON document_covers(generated_at)',
          )
          ..execute('PRAGMA user_version = 3');
      });
      logger.info('Migrated database schema to version 3.');
    }
  }

  void _transaction(void Function() operation) {
    database.execute('BEGIN IMMEDIATE');
    try {
      operation();
      database.execute('COMMIT');
    } catch (_) {
      database.execute('ROLLBACK');
      rethrow;
    }
  }

  void clearUserData() {
    _transaction(() {
      database
        ..execute('DELETE FROM library_search')
        ..execute('DELETE FROM search_index_state')
        ..execute('DELETE FROM workspace_tabs')
        ..execute('DELETE FROM workspaces')
        ..execute('DELETE FROM collection_documents')
        ..execute('DELETE FROM collections')
        ..execute('DELETE FROM document_tags')
        ..execute('DELETE FROM annotation_tags')
        ..execute('DELETE FROM tags')
        ..execute('DELETE FROM annotations')
        ..execute('DELETE FROM reading_profiles')
        ..execute('DELETE FROM reading_sessions')
        ..execute('DELETE FROM bookmarks')
        ..execute('DELETE FROM document_preferences')
        ..execute('DELETE FROM documents')
        ..execute('DELETE FROM app_metadata');
    });
  }

  void clearReadingHistory() {
    _transaction(() {
      database
        ..execute('DELETE FROM reading_sessions')
        ..execute('DELETE FROM bookmarks')
        ..execute('DELETE FROM document_preferences')
        ..execute(
          'UPDATE documents SET last_page_index = 0, reading_progress = 0',
        );
    });
  }

  String integrityCheck() =>
      database.select('PRAGMA integrity_check').first.columnAt(0) as String;

  void optimize() {
    database
      ..execute('PRAGMA optimize')
      ..execute('ANALYZE');
  }

  int tableCount(String table) {
    const allowed = {
      'documents',
      'annotations',
      'collections',
      'tags',
      'workspaces',
      'search_index_state',
    };
    if (!allowed.contains(table)) {
      throw ArgumentError.value(table, 'table', 'Unsupported diagnostic table');
    }
    return database.select('SELECT COUNT(*) FROM $table').first.columnAt(0)
        as int;
  }

  bool recordStartup() {
    final clean = _metadataValue('last_exit_clean') != '0';
    final previousFailures =
        int.tryParse(_metadataValue('startup_failure_count') ?? '') ?? 0;
    final failures = clean ? 0 : previousFailures + 1;
    _setMetadata('last_exit_clean', '0');
    _setMetadata('startup_failure_count', '$failures');
    return failures >= 3;
  }

  void markCleanShutdown() {
    _setMetadata('last_exit_clean', '1');
    _setMetadata('startup_failure_count', '0');
  }

  String? _metadataValue(String key) {
    final rows = database.select(
      'SELECT value FROM app_metadata WHERE key = ?',
      [key],
    );
    return rows.isEmpty ? null : rows.first['value'] as String;
  }

  void _setMetadata(String key, String value) {
    database.execute(
      '''
      INSERT INTO app_metadata(key, value, updated_at) VALUES (?, ?, ?)
      ON CONFLICT(key) DO UPDATE SET
        value = excluded.value,
        updated_at = excluded.updated_at
      ''',
      [key, value, DateTime.now().toUtc().toIso8601String()],
    );
  }

  void close() => database.close();
}
