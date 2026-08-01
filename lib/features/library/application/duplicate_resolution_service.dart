// ignore_for_file: prefer_initializing_formals

import 'dart:math' as math;

import '../../../storage/database/app_database.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/duplicate_resolution.dart';
import '../domain/library_document.dart';

typedef DuplicatePreBackup = Future<String> Function();

enum DuplicateResolutionAction { merge, keepBoth }

final class DuplicateMergeResult {
  const DuplicateMergeResult({
    required this.action,
    required this.keptDocumentId,
    required this.incomingDocumentId,
    required this.preBackupPath,
    required this.conflictFields,
  });

  final DuplicateResolutionAction action;
  final String keptDocumentId;
  final String incomingDocumentId;
  final String? preBackupPath;
  final List<String> conflictFields;

  bool get changedLibrary => action == DuplicateResolutionAction.merge;
}

final class DuplicateMergeRequiresConfirmation implements Exception {
  const DuplicateMergeRequiresConfirmation(this.kind);

  final DuplicateMatchKind kind;

  @override
  String toString() => 'Duplicate merge requires confirmation: ${kind.name}.';
}

/// Merges only local records. The source PDF files are never opened for write,
/// moved, renamed, or deleted by this service.
final class DuplicateResolutionService {
  const DuplicateResolutionService({
    required AppDatabase database,
    required DocumentRepository documents,
    required DuplicatePreBackup createPreBackup,
  }) : _database = database,
       _documents = documents,
       _createPreBackup = createPreBackup;

  final AppDatabase _database;
  final DocumentRepository _documents;
  final DuplicatePreBackup _createPreBackup;

  DuplicateEvidence classify(String existingId, String incomingId) {
    final existing = _required(existingId);
    final incoming = _required(incomingId);
    return DuplicateClassifier.classify(existing, incoming);
  }

  Future<DuplicateMergeResult> resolve({
    required String existingId,
    required String incomingId,
    required DuplicateResolutionAction action,
    bool confirmed = false,
    MetadataConflictChoice conflict = MetadataConflictChoice.nonEmptyExisting,
  }) async {
    if (existingId == incomingId) {
      throw ArgumentError.value(
        incomingId,
        'incomingId',
        'Documents must differ.',
      );
    }
    final existing = _required(existingId);
    final incoming = _required(incomingId);
    final evidence = DuplicateClassifier.classify(existing, incoming);
    if (evidence.kind == DuplicateMatchKind.none) {
      throw StateError('Documents are not classified as duplicates.');
    }
    if (action == DuplicateResolutionAction.keepBoth) {
      return DuplicateMergeResult(
        action: action,
        keptDocumentId: existingId,
        incomingDocumentId: incomingId,
        preBackupPath: null,
        conflictFields: const [],
      );
    }
    if (evidence.requiresConfirmation && !confirmed) {
      throw DuplicateMergeRequiresConfirmation(evidence.kind);
    }

    // The backup is deliberately completed before BEGIN IMMEDIATE. If it
    // fails, the database remains untouched and the user can retry safely.
    final preBackupPath = await _createPreBackup();
    final mergedMetadata = MetadataMergePolicy.merge(
      existing,
      incoming,
      conflict: conflict,
    );
    final database = _database.database;
    database.execute('BEGIN IMMEDIATE');
    try {
      _mergeDocumentRow(existing, incoming, mergedMetadata);
      _mergeBookmarks(existingId, incomingId);
      _mergePreferences(existingId, incomingId);
      _mergeSessions(existingId, incomingId);
      _mergeAnnotations(existingId, incomingId);
      _mergeTags(existingId, incomingId);
      _mergeCollections(existingId, incomingId);
      _mergeWorkspaceTabs(existingId, incomingId);
      _mergeCover(existingId, incomingId);
      _dropSearchEntries(existingId, incomingId);
      database.execute('DELETE FROM documents WHERE id = ?', [incomingId]);
      database.execute('COMMIT');
    } catch (_) {
      database.execute('ROLLBACK');
      rethrow;
    }
    return DuplicateMergeResult(
      action: action,
      keptDocumentId: existingId,
      incomingDocumentId: incomingId,
      preBackupPath: preBackupPath,
      conflictFields: List.unmodifiable(mergedMetadata.conflicts),
    );
  }

  LibraryDocument _required(String id) =>
      _documents.byId(id) ??
      (throw StateError('Document is unavailable for duplicate resolution.'));

  void _mergeDocumentRow(
    LibraryDocument existing,
    LibraryDocument incoming,
    MetadataMergeResult metadata,
  ) {
    final titleBase = existing.sourceTitle ?? existing.title;
    final authorBase = existing.sourceAuthor ?? existing.author;
    final titleOverride = metadata.title == titleBase ? null : metadata.title;
    final authorOverride = metadata.author == authorBase
        ? null
        : metadata.author;
    final incomingIsMoreAdvanced =
        incoming.readingProgress > existing.readingProgress;
    _database.database.execute(
      '''
      UPDATE documents SET
        title_override = ?,
        author_override = ?,
        is_favorite = CASE WHEN is_favorite = 1 OR ? = 1 THEN 1 ELSE 0 END,
        last_page_index = ?,
        reading_progress = ?,
        last_opened_at = CASE
          WHEN last_opened_at >= ? THEN last_opened_at
          ELSE ?
        END
      WHERE id = ?
      ''',
      [
        titleOverride,
        authorOverride,
        incoming.isFavorite ? 1 : 0,
        incomingIsMoreAdvanced
            ? incoming.lastPageIndex
            : existing.lastPageIndex,
        math.max(existing.readingProgress, incoming.readingProgress),
        _date(incoming.lastOpenedAt),
        _date(incoming.lastOpenedAt),
        existing.id,
      ],
    );
    final note = metadata.libraryNote?.trim();
    if (note == null || note.isEmpty) {
      _database.database.execute(
        'DELETE FROM document_notes WHERE document_id = ?',
        [existing.id],
      );
    } else {
      if (note.length > 4000) {
        throw ArgumentError.value(
          metadata.libraryNote,
          'libraryNote',
          'Maximum length is 4000.',
        );
      }
      _database.database.execute(
        '''
        INSERT INTO document_notes(document_id, note, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(document_id) DO UPDATE SET
          note = excluded.note,
          updated_at = excluded.updated_at
        ''',
        [existing.id, note, _date(DateTime.now())],
      );
    }
  }

  void _mergeBookmarks(String existingId, String incomingId) {
    final db = _database.database;
    db.execute(
      '''
      UPDATE bookmarks
      SET label = COALESCE(
            label,
            (SELECT incoming.label FROM bookmarks AS incoming
             WHERE incoming.document_id = ?
               AND incoming.page_index = bookmarks.page_index)
          ),
          updated_at = MAX(
            updated_at,
            (SELECT incoming.updated_at FROM bookmarks AS incoming
             WHERE incoming.document_id = ?
               AND incoming.page_index = bookmarks.page_index)
          )
      WHERE document_id = ?
        AND EXISTS (
          SELECT 1 FROM bookmarks AS incoming
          WHERE incoming.document_id = ?
            AND incoming.page_index = bookmarks.page_index
        )
      ''',
      [incomingId, incomingId, existingId, incomingId],
    );
    db.execute(
      '''
      DELETE FROM bookmarks
      WHERE document_id = ?
        AND page_index IN (
          SELECT page_index FROM bookmarks WHERE document_id = ?
        )
      ''',
      [incomingId, existingId],
    );
    db.execute('UPDATE bookmarks SET document_id = ? WHERE document_id = ?', [
      existingId,
      incomingId,
    ]);
  }

  void _mergePreferences(String existingId, String incomingId) {
    final db = _database.database;
    db.execute(
      '''
      INSERT OR IGNORE INTO document_preferences(
        document_id, view_mode, reading_theme, zoom_mode, zoom_level,
        page_layout, page_rotation, horizontal_margin, vertical_margin,
        controls_auto_hide, scroll_offset, updated_at
      )
      SELECT ?, view_mode, reading_theme, zoom_mode, zoom_level,
        page_layout, page_rotation, horizontal_margin, vertical_margin,
        controls_auto_hide, scroll_offset, updated_at
      FROM document_preferences WHERE document_id = ?
      ''',
      [existingId, incomingId],
    );
    db.execute('DELETE FROM document_preferences WHERE document_id = ?', [
      incomingId,
    ]);
  }

  void _mergeSessions(String existingId, String incomingId) {
    _database.database.execute(
      'UPDATE reading_sessions SET document_id = ? WHERE document_id = ?',
      [existingId, incomingId],
    );
  }

  void _mergeAnnotations(String existingId, String incomingId) {
    _database.database.execute(
      'UPDATE annotations SET document_id = ? WHERE document_id = ?',
      [existingId, incomingId],
    );
  }

  void _mergeTags(String existingId, String incomingId) {
    final db = _database.database;
    db.execute(
      '''
      INSERT OR IGNORE INTO document_tags(document_id, tag_id)
      SELECT ?, tag_id FROM document_tags WHERE document_id = ?
      ''',
      [existingId, incomingId],
    );
    db.execute('DELETE FROM document_tags WHERE document_id = ?', [incomingId]);
  }

  void _mergeCollections(String existingId, String incomingId) {
    final db = _database.database;
    db.execute(
      '''
      INSERT OR IGNORE INTO collection_documents(
        collection_id, document_id, added_at
      )
      SELECT collection_id, ?, added_at
      FROM collection_documents WHERE document_id = ?
      ''',
      [existingId, incomingId],
    );
    db.execute('DELETE FROM collection_documents WHERE document_id = ?', [
      incomingId,
    ]);
  }

  void _mergeWorkspaceTabs(String existingId, String incomingId) {
    final db = _database.database;
    final incomingTabs = db.select(
      'SELECT id, workspace_id FROM workspace_tabs WHERE document_id = ?',
      [incomingId],
    );
    db.execute(
      '''
      INSERT OR IGNORE INTO workspace_tabs(
        id, workspace_id, document_id, sort_order, is_pinned, page_index,
        zoom_level, page_layout, reading_mode, scroll_offset
      )
      SELECT id, workspace_id, ?, sort_order, is_pinned, page_index,
        zoom_level, page_layout, reading_mode, scroll_offset
      FROM workspace_tabs WHERE document_id = ?
      ''',
      [existingId, incomingId],
    );
    for (final tab in incomingTabs) {
      db.execute(
        '''
        UPDATE workspaces
        SET active_tab_id = COALESCE(
          (SELECT id FROM workspace_tabs
           WHERE workspace_id = ? AND document_id = ? LIMIT 1),
          active_tab_id
        )
        WHERE id = ? AND active_tab_id = ?
        ''',
        [tab['workspace_id'], existingId, tab['workspace_id'], tab['id']],
      );
    }
    db.execute('DELETE FROM workspace_tabs WHERE document_id = ?', [
      incomingId,
    ]);
  }

  void _mergeCover(String existingId, String incomingId) {
    final db = _database.database;
    final existing = db.select(
      'SELECT selection_mode FROM document_covers WHERE document_id = ?',
      [existingId],
    );
    final incoming = db.select(
      'SELECT selection_mode FROM document_covers WHERE document_id = ?',
      [incomingId],
    );
    if (incoming.isEmpty) return;
    if (existing.isEmpty ||
        (existing.first['selection_mode'] as String) == 'automatic' &&
            (incoming.first['selection_mode'] as String) == 'manual') {
      if (existing.isNotEmpty) {
        db.execute('DELETE FROM document_covers WHERE document_id = ?', [
          existingId,
        ]);
      }
      db.execute(
        'UPDATE document_covers SET document_id = ? WHERE document_id = ?',
        [existingId, incomingId],
      );
      final thumbnail = db.select(
        'SELECT small_thumbnail_path FROM document_covers WHERE document_id = ?',
        [existingId],
      ).first['small_thumbnail_path'];
      db.execute('UPDATE documents SET thumbnail_path = ? WHERE id = ?', [
        thumbnail,
        existingId,
      ]);
    } else {
      db.execute('DELETE FROM document_covers WHERE document_id = ?', [
        incomingId,
      ]);
    }
  }

  void _dropSearchEntries(String existingId, String incomingId) {
    final db = _database.database;
    db.execute('DELETE FROM library_search WHERE document_id IN (?, ?)', [
      existingId,
      incomingId,
    ]);
    db.execute('DELETE FROM search_index_state WHERE document_id IN (?, ?)', [
      existingId,
      incomingId,
    ]);
  }

  static String _date(DateTime value) => value.toUtc().toIso8601String();
}
