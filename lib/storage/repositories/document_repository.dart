import 'dart:io';

import 'package:sqlite3/common.dart' show Row;

import '../../features/library/domain/library_document.dart';
import '../../features/library/domain/document_cover.dart';
import '../../features/reader/domain/reader_models.dart';
import '../database/app_database.dart';

final class DocumentRepository {
  const DocumentRepository(this._database);

  final AppDatabase _database;

  List<LibraryDocument> recent({int limit = 40}) {
    final rows = _database.database.select(
      '$_documentSelect ORDER BY d.last_opened_at DESC LIMIT ?',
      [limit],
    );
    return rows.map(_fromRow).toList(growable: false);
  }

  List<LibraryDocument> all() {
    final rows = _database.database.select(
      '$_documentSelect ORDER BY d.last_opened_at DESC',
    );
    return rows.map(_fromRow).toList(growable: false);
  }

  LibraryDocument? byId(String id) {
    final rows = _database.database.select(
      '$_documentSelect WHERE d.id = ? LIMIT 1',
      [id],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  LibraryDocument? byNormalizedPath(String normalizedPath) {
    final rows = _database.database.select(
      '$_documentSelect WHERE d.normalized_path = ? LIMIT 1',
      [normalizedPath],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  LibraryDocument? byFingerprint(String fingerprint) {
    final rows = _database.database.select(
      '$_documentSelect WHERE d.fingerprint = ? '
      'ORDER BY d.last_opened_at DESC LIMIT 1',
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
        fingerprint, thumbnail_path, is_favorite
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        thumbnail_path = excluded.thumbnail_path,
        is_favorite = excluded.is_favorite
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
        document.isFavorite ? 1 : 0,
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

  void saveCover(DocumentCoverMetadata metadata) {
    _database.database.execute(
      '''
      INSERT INTO document_covers (
        document_id, source_page_index, small_thumbnail_path,
        large_thumbnail_path, generated_at, source_modified_at, confidence,
        selection_mode, algorithm_version, crop_mode, page_aspect_ratio
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(document_id) DO UPDATE SET
        source_page_index = excluded.source_page_index,
        small_thumbnail_path = excluded.small_thumbnail_path,
        large_thumbnail_path = excluded.large_thumbnail_path,
        generated_at = excluded.generated_at,
        source_modified_at = excluded.source_modified_at,
        confidence = excluded.confidence,
        selection_mode = excluded.selection_mode,
        algorithm_version = excluded.algorithm_version,
        crop_mode = excluded.crop_mode,
        page_aspect_ratio = excluded.page_aspect_ratio
      ''',
      [
        metadata.documentId,
        metadata.sourcePageIndex,
        metadata.smallThumbnailPath,
        metadata.largeThumbnailPath,
        _date(metadata.generatedAt),
        _date(metadata.sourceModifiedAt),
        metadata.confidence,
        metadata.selectionMode.name,
        metadata.algorithmVersion,
        metadata.cropMode.name,
        metadata.pageAspectRatio,
      ],
    );
    updateThumbnail(metadata.documentId, metadata.smallThumbnailPath);
  }

  void invalidateCover(String documentId) {
    _database.database.execute(
      'DELETE FROM document_covers WHERE document_id = ?',
      [documentId],
    );
    _database.database.execute(
      'UPDATE documents SET thumbnail_path = NULL WHERE id = ?',
      [documentId],
    );
  }

  void toggleFavorite(String documentId) {
    _database.database.execute(
      '''
      UPDATE documents
      SET is_favorite = CASE is_favorite WHEN 1 THEN 0 ELSE 1 END
      WHERE id = ?
      ''',
      [documentId],
    );
  }

  void clearThumbnailPath(String thumbnailPath) {
    _database.database.execute(
      '''
      DELETE FROM document_covers
      WHERE small_thumbnail_path = ? OR large_thumbnail_path = ?
      ''',
      [thumbnailPath, thumbnailPath],
    );
    _database.database.execute(
      'UPDATE documents SET thumbnail_path = NULL WHERE thumbnail_path = ?',
      [thumbnailPath],
    );
  }

  void clearThumbnailPaths() {
    _database.database.execute('DELETE FROM document_covers');
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
    final coverDocumentId = row['cover_document_id'] as String?;
    final cover = coverDocumentId == null
        ? null
        : DocumentCoverMetadata(
            documentId: coverDocumentId,
            sourcePageIndex: row['cover_source_page_index'] as int,
            smallThumbnailPath: row['cover_small_path'] as String,
            largeThumbnailPath: row['cover_large_path'] as String,
            generatedAt: DateTime.parse(row['cover_generated_at'] as String),
            sourceModifiedAt: DateTime.parse(
              row['cover_source_modified_at'] as String,
            ),
            confidence: row['cover_confidence'] as double,
            selectionMode: _enumByName(
              CoverSelectionMode.values,
              row['cover_selection_mode'] as String,
              CoverSelectionMode.automatic,
            ),
            algorithmVersion: row['cover_algorithm_version'] as int,
            cropMode: _enumByName(
              CoverCropMode.values,
              row['cover_crop_mode'] as String,
              CoverCropMode.contain,
            ),
            pageAspectRatio: row['cover_page_aspect_ratio'] as double,
          );
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
      thumbnailPath:
          cover?.smallThumbnailPath ?? row['thumbnail_path'] as String?,
      coverMetadata: cover,
      isFavorite: (row['is_favorite'] as int) == 1,
    );
  }

  static const String _documentSelect = '''
    SELECT d.*,
      c.document_id AS cover_document_id,
      c.source_page_index AS cover_source_page_index,
      c.small_thumbnail_path AS cover_small_path,
      c.large_thumbnail_path AS cover_large_path,
      c.generated_at AS cover_generated_at,
      c.source_modified_at AS cover_source_modified_at,
      c.confidence AS cover_confidence,
      c.selection_mode AS cover_selection_mode,
      c.algorithm_version AS cover_algorithm_version,
      c.crop_mode AS cover_crop_mode
      , c.page_aspect_ratio AS cover_page_aspect_ratio
    FROM documents d
    LEFT JOIN document_covers c ON c.document_id = d.id
  ''';

  static String _date(DateTime value) => value.toUtc().toIso8601String();

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;
}
