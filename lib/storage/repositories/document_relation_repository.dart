import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/relations/domain/document_relation.dart';
import '../database/app_database.dart';

final class DocumentRelationRepository {
  const DocumentRelationRepository(this._database);

  final AppDatabase _database;

  DocumentRelation create({
    required String sourceDocumentId,
    required String targetDocumentId,
    required DocumentRelationType type,
    DocumentAnchor? sourceAnchor,
    DocumentAnchor? targetAnchor,
  }) {
    if (sourceDocumentId == targetDocumentId) {
      throw ArgumentError('A document cannot relate to itself.');
    }
    final now = DateTime.now().toUtc();
    final id = sha256
        .convert(
          utf8.encode(
            '$sourceDocumentId:$targetDocumentId:${type.name}:'
            '${now.microsecondsSinceEpoch}',
          ),
        )
        .toString();
    _database.database.execute(
      '''
      INSERT INTO document_relations(
        id, source_document_id, target_document_id, relation_type,
        source_anchor_kind, source_anchor_id, target_anchor_kind,
        target_anchor_id, created_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      ''',
      [
        id,
        sourceDocumentId,
        targetDocumentId,
        type.name,
        sourceAnchor?.kind.name,
        sourceAnchor?.id,
        targetAnchor?.kind.name,
        targetAnchor?.id,
        now.toIso8601String(),
      ],
    );
    return DocumentRelation(
      id: id,
      sourceDocumentId: sourceDocumentId,
      targetDocumentId: targetDocumentId,
      type: type,
      sourceAnchor: sourceAnchor,
      targetAnchor: targetAnchor,
      createdAt: now,
    );
  }

  List<DocumentRelationView> forDocument(String documentId, {int limit = 80}) {
    final rows = _database.database.select(
      '''
      SELECT r.*,
        CASE WHEN r.source_document_id = ?
          THEN COALESCE(td.title_override, td.title, td.file_name)
          ELSE COALESCE(sd.title_override, sd.title, sd.file_name)
        END AS other_title
      FROM document_relations r
      JOIN documents sd ON sd.id = r.source_document_id
      JOIN documents td ON td.id = r.target_document_id
      WHERE r.source_document_id = ? OR r.target_document_id = ?
      ORDER BY r.created_at DESC, r.id
      LIMIT ?
      ''',
      [documentId, documentId, documentId, limit.clamp(1, 80)],
    );
    return rows
        .map((row) {
          final sourceId = row['source_document_id'] as String;
          final targetId = row['target_document_id'] as String;
          final backlink = targetId == documentId;
          final relation = DocumentRelation(
            id: row['id'] as String,
            sourceDocumentId: sourceId,
            targetDocumentId: targetId,
            type: _enumByName(
              DocumentRelationType.values,
              row['relation_type'] as String,
              DocumentRelationType.related,
            ),
            sourceAnchor: _anchor(
              row['source_anchor_kind'] as String?,
              row['source_anchor_id'] as String?,
            ),
            targetAnchor: _anchor(
              row['target_anchor_kind'] as String?,
              row['target_anchor_id'] as String?,
            ),
            createdAt: DateTime.parse(row['created_at'] as String),
          );
          return DocumentRelationView(
            relation: relation,
            otherDocumentId: backlink ? sourceId : targetId,
            otherDocumentTitle: row['other_title'] as String,
            isBacklink: backlink,
          );
        })
        .toList(growable: false);
  }

  void remove(String id) {
    _database.database.execute('DELETE FROM document_relations WHERE id = ?', [
      id,
    ]);
  }

  DocumentRevisionGroup createRevisionGroup({
    required String name,
    required Iterable<String> documentIds,
    String? currentDocumentId,
  }) {
    final cleanName = name.trim();
    final members = documentIds.toSet().take(80).toList(growable: false);
    if (cleanName.isEmpty || members.length < 2) {
      throw ArgumentError('A revision group needs a name and two documents.');
    }
    if (currentDocumentId != null && !members.contains(currentDocumentId)) {
      throw ArgumentError.value(currentDocumentId, 'currentDocumentId');
    }
    final now = DateTime.now().toUtc();
    final id = sha256
        .convert(
          utf8.encode('revision:$cleanName:${now.microsecondsSinceEpoch}'),
        )
        .toString();
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      db.execute(
        'INSERT INTO document_revision_groups('
        'id, name, current_document_id, created_at, updated_at) '
        'VALUES (?, ?, ?, ?, ?)',
        [
          id,
          cleanName,
          currentDocumentId,
          now.toIso8601String(),
          now.toIso8601String(),
        ],
      );
      for (final documentId in members) {
        db.execute(
          'INSERT INTO document_revision_members('
          'group_id, document_id, added_at) VALUES (?, ?, ?)',
          [id, documentId, now.toIso8601String()],
        );
      }
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
    return byRevisionGroupId(id)!;
  }

  DocumentRevisionGroup? byRevisionGroupId(String id) {
    final rows = _database.database.select(
      'SELECT * FROM document_revision_groups WHERE id = ?',
      [id],
    );
    if (rows.isEmpty) return null;
    final row = rows.first;
    final memberRows = _database.database.select(
      'SELECT document_id FROM document_revision_members '
      'WHERE group_id = ? ORDER BY added_at, document_id',
      [id],
    );
    return DocumentRevisionGroup(
      id: id,
      name: row['name'] as String,
      documentIds: memberRows
          .map((item) => item['document_id'] as String)
          .toList(growable: false),
      currentDocumentId: row['current_document_id'] as String?,
      createdAt: DateTime.parse(row['created_at'] as String),
      updatedAt: DateTime.parse(row['updated_at'] as String),
    );
  }

  void setCurrentRevision(String groupId, String documentId) {
    final member = _database.database.select(
      'SELECT 1 FROM document_revision_members '
      'WHERE group_id = ? AND document_id = ? LIMIT 1',
      [groupId, documentId],
    );
    if (member.isEmpty) throw ArgumentError.value(documentId, 'documentId');
    _database.database.execute(
      'UPDATE document_revision_groups SET current_document_id = ?, '
      'updated_at = ? WHERE id = ?',
      [documentId, DateTime.now().toUtc().toIso8601String(), groupId],
    );
  }

  List<DocumentRevisionGroup> revisionGroupsForDocument(
    String documentId, {
    int limit = 80,
  }) {
    final groupRows = _database.database.select(
      'SELECT g.* FROM document_revision_groups g '
      'JOIN document_revision_members own ON own.group_id = g.id '
      'WHERE own.document_id = ? ORDER BY g.updated_at DESC LIMIT ?',
      [documentId, limit.clamp(1, 80)],
    );
    if (groupRows.isEmpty) return const [];
    final ids = groupRows.map((row) => row['id'] as String).toList();
    final placeholders = List.filled(ids.length, '?').join(',');
    final memberRows = _database.database.select(
      'SELECT group_id, document_id FROM document_revision_members '
      'WHERE group_id IN ($placeholders) ORDER BY added_at, document_id',
      ids,
    );
    final members = <String, List<String>>{};
    for (final row in memberRows) {
      (members[row['group_id'] as String] ??= <String>[]).add(
        row['document_id'] as String,
      );
    }
    return groupRows
        .map(
          (row) => DocumentRevisionGroup(
            id: row['id'] as String,
            name: row['name'] as String,
            documentIds: List.unmodifiable(
              members[row['id'] as String] ?? const [],
            ),
            currentDocumentId: row['current_document_id'] as String?,
            createdAt: DateTime.parse(row['created_at'] as String),
            updatedAt: DateTime.parse(row['updated_at'] as String),
          ),
        )
        .toList(growable: false);
  }

  static DocumentAnchor? _anchor(String? kind, String? id) {
    if (kind == null || id == null) return null;
    final resolved = DocumentAnchorKind.values
        .where((item) => item.name == kind)
        .firstOrNull;
    return resolved == null ? null : DocumentAnchor(kind: resolved, id: id);
  }

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;
}
