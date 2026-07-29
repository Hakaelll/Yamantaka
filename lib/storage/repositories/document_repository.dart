import 'dart:io';

import 'package:sqlite3/common.dart' show Row;

import '../../features/library/domain/library_document.dart';
import '../../features/reader/domain/reader_models.dart';
import '../database/app_database.dart';

final class DocumentRepository {
  const DocumentRepository(this._database);

  final AppDatabase _database;

  List<LibraryDocument> recent({int limit = 40}) {
    final rows = _database.database.select(
      'SELECT * FROM documents ORDER BY last_opened_at DESC LIMIT ?',
      [limit],
    );
    return rows.map(_fromRow).toList(growable: false);
  }

  LibraryDocument? byId(String id) {
    final rows = _database.database.select(
      'SELECT * FROM documents WHERE id = ? LIMIT 1',
      [id],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  LibraryDocument? byNormalizedPath(String normalizedPath) {
    final rows = _database.database.select(
      'SELECT * FROM documents WHERE normalized_path = ? LIMIT 1',
      [normalizedPath],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  LibraryDocument? byFingerprint(String fingerprint) {
    final rows = _database.database.select(
      'SELECT * FROM documents WHERE fingerprint = ? '
      'ORDER BY last_opened_at DESC LIMIT 1',
      [fingerprint],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  void save(LibraryDocument document) {
    _database.database.execute(
      '''
      INSERT INTO documents (
        id, path, normalized_path, file_name, title, author, page_count,
        file_size, file_modified_at, added_at, last_opened_at, last_page_index,
        reading_progress, preferred_view_mode, preferred_theme, is_missing,
        fingerprint, thumbnail_path
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(id) DO UPDATE SET
        path = excluded.path,
        normalized_path = excluded.normalized_path,
        file_name = excluded.file_name,
        title = excluded.title,
        author = excluded.author,
        page_count = excluded.page_count,
        file_size = excluded.file_size,
        file_modified_at = excluded.file_modified_at,
        last_opened_at = excluded.last_opened_at,
        last_page_index = excluded.last_page_index,
        reading_progress = excluded.reading_progress,
        preferred_view_mode = excluded.preferred_view_mode,
        preferred_theme = excluded.preferred_theme,
        is_missing = excluded.is_missing,
        fingerprint = excluded.fingerprint,
        thumbnail_path = excluded.thumbnail_path
      ''',
      [
        document.id,
        document.path,
        document.normalizedPath,
        document.fileName,
        document.title,
        document.author,
        document.pageCount,
        document.fileSize,
        _date(document.fileModifiedAt),
        _date(document.addedAt),
        _date(document.lastOpenedAt),
        document.lastPageIndex,
        document.readingProgress,
        document.preferredViewMode.name,
        document.preferredTheme.name,
        document.isMissing ? 1 : 0,
        document.fingerprint,
        document.thumbnailPath,
      ],
    );
  }

  void updateProgress({
    required String documentId,
    required int pageIndex,
    required int pageCount,
    required ReaderMode viewMode,
    required ReaderTheme theme,
  }) {
    _database.database.execute(
      '''
      UPDATE documents SET
        last_page_index = ?,
        reading_progress = ?,
        preferred_view_mode = ?,
        preferred_theme = ?,
        last_opened_at = ?
      WHERE id = ?
      ''',
      [
        pageIndex,
        ReadingProgress.calculate(pageIndex: pageIndex, pageCount: pageCount),
        viewMode.name,
        theme.name,
        _date(DateTime.now()),
        documentId,
      ],
    );
  }

  void updateThumbnail(String documentId, String thumbnailPath) {
    _database.database.execute(
      'UPDATE documents SET thumbnail_path = ? WHERE id = ?',
      [thumbnailPath, documentId],
    );
  }

  void clearThumbnailPath(String thumbnailPath) {
    _database.database.execute(
      'UPDATE documents SET thumbnail_path = NULL WHERE thumbnail_path = ?',
      [thumbnailPath],
    );
  }

  void clearThumbnailPaths() {
    _database.database.execute('UPDATE documents SET thumbnail_path = NULL');
  }

  int refreshMissingState() {
    var changed = 0;
    final rows = _database.database.select(
      'SELECT id, path, is_missing FROM documents',
    );
    for (final row in rows) {
      final missing = !File(row['path'] as String).existsSync();
      final wasMissing = (row['is_missing'] as int) == 1;
      if (missing != wasMissing) {
        _database.database.execute(
          'UPDATE documents SET is_missing = ? WHERE id = ?',
          [missing ? 1 : 0, row['id']],
        );
        changed++;
      }
    }
    return changed;
  }

  void deleteHistory() {
    _database.database.execute('DELETE FROM documents');
  }

  void resetProgress(String documentId) {
    _database.database.execute(
      '''
      UPDATE documents
      SET last_page_index = 0, reading_progress = 0
      WHERE id = ?
      ''',
      [documentId],
    );
  }

  void remove(String documentId) {
    _database.database.execute('DELETE FROM documents WHERE id = ?', [
      documentId,
    ]);
  }

  LibraryDocument _fromRow(Row row) {
    final path = row['path'] as String;
    final storedSize = row['file_size'] as int;
    final storedModifiedAt = DateTime.parse(row['file_modified_at'] as String);
    var hasFileChanged = false;
    try {
      final stat = File(path).statSync();
      hasFileChanged =
          stat.type != FileSystemEntityType.notFound &&
          (stat.size != storedSize ||
              stat.modified.toUtc() != storedModifiedAt.toUtc());
    } on FileSystemException {
      // Missing and inaccessible files are represented by is_missing instead.
    }
    return LibraryDocument(
      id: row['id'] as String,
      path: path,
      normalizedPath: row['normalized_path'] as String,
      fileName: row['file_name'] as String,
      title: row['title'] as String?,
      author: row['author'] as String?,
      pageCount: row['page_count'] as int,
      fileSize: storedSize,
      fileModifiedAt: storedModifiedAt,
      addedAt: DateTime.parse(row['added_at'] as String),
      lastOpenedAt: DateTime.parse(row['last_opened_at'] as String),
      lastPageIndex: row['last_page_index'] as int,
      readingProgress: row['reading_progress'] as double,
      preferredViewMode: _enumByName(
        ReaderMode.values,
        row['preferred_view_mode'] as String,
        ReaderMode.document,
      ),
      preferredTheme: _enumByName(
        ReaderTheme.values,
        row['preferred_theme'] as String,
        ReaderTheme.ivory,
      ),
      isMissing: (row['is_missing'] as int) == 1,
      hasFileChanged: hasFileChanged,
      fingerprint: row['fingerprint'] as String,
      thumbnailPath: row['thumbnail_path'] as String?,
    );
  }

  static String _date(DateTime value) => value.toUtc().toIso8601String();

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;
}
