// ignore_for_file: prefer_initializing_formals

import 'dart:math' as math;

import 'package:crypto/crypto.dart';

import '../../../storage/database/app_database.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/duplicate_resolution.dart';
import '../domain/library_document.dart';

typedef DuplicatePreBackup = Future<String> Function();
typedef DuplicateBackupRestore = Future<String> Function(String path);

enum DuplicateResolutionAction { merge, keepBoth }

final class DuplicateMergeResult {
  const DuplicateMergeResult({
    required this.action,
    required this.keptDocumentId,
    required this.incomingDocumentId,
    required this.preBackupPath,
    required this.conflictFields,
    required this.historyId,
  });

  final DuplicateResolutionAction action;
  final String keptDocumentId;
  final String incomingDocumentId;
  final String? preBackupPath;
  final List<String> conflictFields;
  final String? historyId;

  bool get changedLibrary => action == DuplicateResolutionAction.merge;
}

final class DuplicateMergeRequiresConfirmation implements Exception {
  const DuplicateMergeRequiresConfirmation(this.kind);

  final DuplicateMatchKind kind;

  @override
  String toString() => 'Duplicate merge requires confirmation: ${kind.name}.';
}

final class DuplicateMergeImpact {
  const DuplicateMergeImpact({
    required this.localRecords,
    required this.conflictFields,
  });

  final int localRecords;
  final List<String> conflictFields;
}

/// Merges only local records. The source PDF files are never opened for write,
/// moved, renamed, or deleted by this service.
final class DuplicateResolutionService {
  const DuplicateResolutionService({
    required AppDatabase database,
    required DocumentRepository documents,
    required DuplicatePreBackup createPreBackup,
    DuplicateBackupRestore? restoreBackup,
  }) : _database = database,
       _documents = documents,
       _createPreBackup = createPreBackup,
       _restoreBackup = restoreBackup;

  final AppDatabase _database;
  final DocumentRepository _documents;
  final DuplicatePreBackup _createPreBackup;
  final DuplicateBackupRestore? _restoreBackup;

  DuplicateEvidence classify(String existingId, String incomingId) {
    final existing = _required(existingId);
    final incoming = _required(incomingId);
    return DuplicateClassifier.classify(existing, incoming);
  }

  List<DuplicateCandidate> candidates({int limit = 80, int scanLimit = 5000}) {
    final documents = _documents
        .scan(limit: scanLimit.clamp(2, 100000))
        .toList(growable: false);
    final results = <DuplicateCandidate>[];
    final seenPairs = <String>{};
    void addPair(LibraryDocument left, LibraryDocument right) {
      if (results.length >= limit.clamp(1, 200)) return;
      final pair = [left.id, right.id]..sort();
      if (!seenPairs.add(pair.join(':'))) return;
      final evidence = DuplicateClassifier.classify(left, right);
      if (evidence.kind == DuplicateMatchKind.none) return;
      results.add(
        DuplicateCandidate(existing: left, incoming: right, evidence: evidence),
      );
    }

    final exactBuckets = <String, List<LibraryDocument>>{};
    final probableBuckets = <String, List<LibraryDocument>>{};
    for (final document in documents) {
      exactBuckets.putIfAbsent(document.fingerprint, () => []).add(document);
      final probableKey =
          '${document.fileName.toLowerCase()}:${document.fileSize}';
      probableBuckets.putIfAbsent(probableKey, () => []).add(document);
    }
    for (final bucket in exactBuckets.values.where(
      (items) => items.length > 1,
    )) {
      for (var index = 1; index < bucket.length; index++) {
        addPair(bucket.first, bucket[index]);
      }
    }
    for (final bucket in probableBuckets.values.where(
      (items) => items.length > 1,
    )) {
      for (var index = 1; index < bucket.length; index++) {
        addPair(bucket.first, bucket[index]);
      }
    }
    return List.unmodifiable(results);
  }

  DuplicateMergeImpact previewImpact(String existingId, String incomingId) {
    final existing = _required(existingId);
    final incoming = _required(incomingId);
    final conflicts = MetadataMergePolicy.merge(existing, incoming).conflicts;
    final db = _database.database;
    int count(String table, String predicate, List<Object?> parameters) =>
        db
                .select(
                  'SELECT COUNT(*) FROM $table WHERE $predicate',
                  parameters,
                )
                .first
                .columnAt(0)
            as int;
    final records =
        1 +
        count('bookmarks', 'document_id = ?', [incomingId]) +
        count('reading_sessions', 'document_id = ?', [incomingId]) +
        count('annotations', 'document_id = ?', [incomingId]) +
        count('document_tags', 'document_id = ?', [incomingId]) +
        count('collection_documents', 'document_id = ?', [incomingId]) +
        count('workspace_tabs', 'document_id = ?', [incomingId]) +
        count(
          'document_relations',
          'source_document_id = ? OR target_document_id = ?',
          [incomingId, incomingId],
        ) +
        count('document_revision_members', 'document_id = ?', [incomingId]);
    return DuplicateMergeImpact(
      localRecords: records,
      conflictFields: List.unmodifiable(conflicts),
    );
  }

  Future<DuplicateMergeResult> resolve({
    required String existingId,
    required String incomingId,
    required DuplicateResolutionAction action,
    bool confirmed = false,
    MetadataConflictChoice conflict = MetadataConflictChoice.nonEmptyExisting,
    Map<String, MetadataConflictChoice>? conflictChoices,
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
        historyId: null,
      );
    }
    if (evidence.requiresConfirmation && !confirmed) {
      throw DuplicateMergeRequiresConfirmation(evidence.kind);
    }

    // The backup is deliberately completed before BEGIN IMMEDIATE. If it
    // fails, the database remains untouched and the user can retry safely.
    final preBackupPath = await _createPreBackup();
    final mergedMetadata = conflictChoices == null
        ? MetadataMergePolicy.merge(existing, incoming, conflict: conflict)
        : MetadataMergePolicy.mergePerField(
            existing,
            incoming,
            conflictChoices,
          );
    final database = _database.database;
    final historyId = sha256
        .convert(
          '$existingId:$incomingId:${DateTime.now().microsecondsSinceEpoch}'
              .codeUnits,
        )
        .toString();
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
      _mergeRelations(existingId, incomingId);
      _mergeRevisions(existingId, incomingId);
      _mergeCover(existingId, incomingId);
      _dropSearchEntries(existingId, incomingId);
      database.execute('DELETE FROM documents WHERE id = ?', [incomingId]);
      final now = _date(DateTime.now());
      database.execute(
        '''
        INSERT INTO duplicate_merge_history(
          id, kept_document_id, incoming_document_id, backup_path, status,
          created_at, updated_at
        ) VALUES (?, ?, ?, ?, 'completed', ?, ?)
        ''',
        [historyId, existingId, incomingId, preBackupPath, now, now],
      );
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
      historyId: historyId,
    );
  }

  Future<bool> undoMerge(String historyId) async {
    final restore = _restoreBackup;
    if (restore == null) return false;
    final rows = _database.database.select(
      'SELECT * FROM duplicate_merge_history '
      "WHERE id = ? AND status = 'completed'",
      [historyId],
    );
    if (rows.isEmpty) return false;
    final row = rows.first;
    final keptId = row['kept_document_id'] as String;
    final incomingId = row['incoming_document_id'] as String;
    final backupPath = row['backup_path'] as String;
    final createdAt = row['created_at'] as String;
    try {
      await restore(backupPath);
      final now = _date(DateTime.now());
      _database.database.execute(
        '''
        INSERT OR REPLACE INTO duplicate_merge_history(
          id, kept_document_id, incoming_document_id, backup_path, status,
          created_at, updated_at
        ) VALUES (?, ?, ?, ?, 'rolledBack', ?, ?)
        ''',
        [historyId, keptId, incomingId, backupPath, createdAt, now],
      );
      return true;
    } on Object {
      _database.database.execute(
        "UPDATE duplicate_merge_history SET status = 'failed', updated_at = ? "
        'WHERE id = ?',
        [_date(DateTime.now()), historyId],
      );
      rethrow;
    }
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
      '''
      UPDATE annotations SET
        origin_document_id = COALESCE(origin_document_id, document_id),
        origin_fingerprint = COALESCE(
          origin_fingerprint,
          document_fingerprint
        ),
        document_id = ?
      WHERE document_id = ?
      ''',
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
        zoom_level, page_layout, reading_mode, scroll_offset, queue_status,
        blocking_reason, next_action
      )
      SELECT id, workspace_id, ?, sort_order, is_pinned, page_index,
        zoom_level, page_layout, reading_mode, scroll_offset, queue_status,
        blocking_reason, next_action
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

  void _mergeRelations(String existingId, String incomingId) {
    final db = _database.database;
    final incomingRelations = db.select(
      'SELECT * FROM document_relations '
      'WHERE source_document_id = ? OR target_document_id = ?',
      [incomingId, incomingId],
    );
    for (final relation in incomingRelations) {
      db.execute('DELETE FROM document_relations WHERE id = ?', [
        relation['id'],
      ]);
      final source = relation['source_document_id'] == incomingId
          ? existingId
          : relation['source_document_id'] as String;
      final target = relation['target_document_id'] == incomingId
          ? existingId
          : relation['target_document_id'] as String;
      // A direct edge between both duplicate records collapses to a self edge;
      // an already-existing equivalent edge is intentionally de-duplicated.
      if (source == target) continue;
      db.execute(
        '''
        INSERT OR IGNORE INTO document_relations(
          id, source_document_id, target_document_id, relation_type,
          source_anchor_kind, source_anchor_id, target_anchor_kind,
          target_anchor_id, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''',
        [
          relation['id'],
          source,
          target,
          relation['relation_type'],
          relation['source_anchor_kind'],
          relation['source_anchor_id'],
          relation['target_anchor_kind'],
          relation['target_anchor_id'],
          relation['created_at'],
        ],
      );
    }
  }

  void _mergeRevisions(String existingId, String incomingId) {
    final db = _database.database;
    db.execute(
      '''
      INSERT OR IGNORE INTO document_revision_members(
        group_id, document_id, added_at
      )
      SELECT group_id, ?, added_at FROM document_revision_members
      WHERE document_id = ?
      ''',
      [existingId, incomingId],
    );
    db.execute('DELETE FROM document_revision_members WHERE document_id = ?', [
      incomingId,
    ]);
    db.execute(
      'UPDATE document_revision_groups SET current_document_id = ?, '
      'updated_at = ? WHERE current_document_id = ?',
      [existingId, _date(DateTime.now()), incomingId],
    );
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
