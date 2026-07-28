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
    appDatabase._migrate();
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
        ..execute('DELETE FROM reading_sessions')
        ..execute('DELETE FROM bookmarks')
        ..execute('DELETE FROM document_preferences')
        ..execute('DELETE FROM documents')
        ..execute('DELETE FROM app_metadata');
    });
  }

  void close() => database.close();
}
