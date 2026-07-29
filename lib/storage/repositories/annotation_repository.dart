import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:sqlite3/common.dart' show Row;

import '../../app/app_config.dart';
import '../../features/annotations/domain/annotation.dart';
import '../database/app_database.dart';

final class AnnotationRepository {
  const AnnotationRepository(this._database);

  final AppDatabase _database;

  List<PdfAnnotation> forDocument(
    String documentId, {
    AnnotationType? type,
    AnnotationColor? color,
    String? query,
    String? tag,
    bool newestFirst = false,
  }) {
    final clauses = <String>['a.document_id = ?'];
    final arguments = <Object?>[documentId];
    if (type != null) {
      clauses.add('a.type = ?');
      arguments.add(type.name);
    }
    if (color != null) {
      clauses.add('a.color = ?');
      arguments.add(color.name);
    }
    final normalizedQuery = query?.trim().toLowerCase();
    if (normalizedQuery != null && normalizedQuery.isNotEmpty) {
      clauses.add(
        "(LOWER(COALESCE(a.note, '')) LIKE ? OR "
        "LOWER(COALESCE(a.selected_text, '')) LIKE ?)",
      );
      arguments
        ..add('%$normalizedQuery%')
        ..add('%$normalizedQuery%');
    }
    if (tag != null && tag.trim().isNotEmpty) {
      clauses.add('''
        EXISTS (
          SELECT 1 FROM annotation_tags at
          JOIN tags t ON t.id = at.tag_id
          WHERE at.annotation_id = a.id AND t.normalized_name = ?
        )
      ''');
      arguments.add(_normalizeTag(tag));
    }
    final rows = _database.database.select('''
      SELECT a.* FROM annotations a
      WHERE ${clauses.join(' AND ')}
      ORDER BY ${newestFirst ? 'a.updated_at DESC, a.page_index' : 'a.page_index, a.created_at'}
      ''', arguments);
    return rows.map(_fromRow).toList(growable: false);
  }

  PdfAnnotation? byId(String id) {
    final rows = _database.database.select(
      'SELECT * FROM annotations WHERE id = ?',
      [id],
    );
    return rows.isEmpty ? null : _fromRow(rows.first);
  }

  PdfAnnotation create({
    required String documentId,
    required int pageIndex,
    required AnnotationType type,
    required AnnotationColor color,
    required String documentFingerprint,
    String? note,
    String? selectedText,
    String? contextBefore,
    String? contextAfter,
    List<NormalizedRect> rects = const [],
    int pageRotation = 0,
    List<String> tags = const [],
  }) {
    if (pageIndex < 0) throw ArgumentError.value(pageIndex, 'pageIndex');
    _validateText(note);
    if (rects.any((rect) => !rect.isMeaningful)) {
      throw ArgumentError.value(rects, 'rects', 'Rectangles must have area.');
    }
    final now = DateTime.now().toUtc();
    final id = sha256
        .convert(
          utf8.encode(
            '$documentId:$pageIndex:${now.microsecondsSinceEpoch}:'
            '${type.name}',
          ),
        )
        .toString();
    final annotation = PdfAnnotation(
      id: id,
      documentId: documentId,
      pageIndex: pageIndex,
      type: type,
      color: color,
      note: _clean(note),
      selectedText: _clean(selectedText),
      contextBefore: _clean(contextBefore),
      contextAfter: _clean(contextAfter),
      rects: List.unmodifiable(rects),
      pageRotation: pageRotation % 360,
      documentFingerprint: documentFingerprint,
      tags: _cleanTags(tags),
      createdAt: now,
      updatedAt: now,
    );
    save(annotation);
    return byId(id)!;
  }

  void save(PdfAnnotation annotation) {
    _validateText(annotation.note);
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      db.execute(
        '''
        INSERT INTO annotations (
          id, document_id, page_index, type, color, note, selected_text,
          context_before, context_after, rects_json, page_rotation,
          document_fingerprint, status, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          type = excluded.type,
          color = excluded.color,
          note = excluded.note,
          selected_text = excluded.selected_text,
          context_before = excluded.context_before,
          context_after = excluded.context_after,
          rects_json = excluded.rects_json,
          page_rotation = excluded.page_rotation,
          document_fingerprint = excluded.document_fingerprint,
          status = excluded.status,
          updated_at = excluded.updated_at
        ''',
        [
          annotation.id,
          annotation.documentId,
          annotation.pageIndex,
          annotation.type.name,
          annotation.color.name,
          _clean(annotation.note),
          _clean(annotation.selectedText),
          _clean(annotation.contextBefore),
          _clean(annotation.contextAfter),
          jsonEncode(annotation.rects.map((rect) => rect.toJson()).toList()),
          annotation.pageRotation,
          annotation.documentFingerprint,
          annotation.status.name,
          annotation.createdAt.toUtc().toIso8601String(),
          annotation.updatedAt.toUtc().toIso8601String(),
        ],
      );
      db.execute('DELETE FROM annotation_tags WHERE annotation_id = ?', [
        annotation.id,
      ]);
      for (final tag in _cleanTags(annotation.tags)) {
        final tagId = _ensureTag(tag, db);
        db.execute(
          'INSERT OR IGNORE INTO annotation_tags(annotation_id, tag_id) '
          'VALUES (?, ?)',
          [annotation.id, tagId],
        );
      }
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
  }

  void remove(String id) {
    _database.database.execute('DELETE FROM annotations WHERE id = ?', [id]);
  }

  void markDocumentChanged(String documentId, String currentFingerprint) {
    _database.database.execute(
      '''
      UPDATE annotations
      SET status = CASE
        WHEN document_fingerprint = ? THEN 'attached'
        ELSE 'reviewRequired'
      END
      WHERE document_id = ?
      ''',
      [currentFingerprint, documentId],
    );
  }

  PdfAnnotation _fromRow(Row row) {
    final id = row['id'] as String;
    final tagRows = _database.database.select(
      '''
      SELECT t.name FROM tags t
      JOIN annotation_tags at ON at.tag_id = t.id
      WHERE at.annotation_id = ?
      ORDER BY t.name COLLATE NOCASE
      ''',
      [id],
    );
    final rawRects = jsonDecode(row['rects_json'] as String) as List<Object?>;
    return PdfAnnotation(
      id: id,
      documentId: row['document_id'] as String,
      pageIndex: row['page_index'] as int,
      type: _enumByName(
        AnnotationType.values,
        row['type'] as String,
        AnnotationType.pageComment,
      ),
      color: _enumByName(
        AnnotationColor.values,
        row['color'] as String,
        AnnotationColor.yellow,
      ),
      note: row['note'] as String?,
      selectedText: row['selected_text'] as String?,
      contextBefore: row['context_before'] as String?,
      contextAfter: row['context_after'] as String?,
      rects: rawRects
          .map(
            (item) =>
                NormalizedRect.fromJson((item as Map).cast<String, Object?>()),
          )
          .toList(growable: false),
      pageRotation: row['page_rotation'] as int,
      documentFingerprint: row['document_fingerprint'] as String,
      status: _enumByName(
        AnnotationAttachmentStatus.values,
        row['status'] as String,
        AnnotationAttachmentStatus.reviewRequired,
      ),
      tags: tagRows
          .map((tagRow) => tagRow['name'] as String)
          .toList(growable: false),
      createdAt: DateTime.parse(row['created_at'] as String),
      updatedAt: DateTime.parse(row['updated_at'] as String),
    );
  }

  String _ensureTag(String name, dynamic db) {
    final normalized = _normalizeTag(name);
    final existing = db.select(
      'SELECT id FROM tags WHERE normalized_name = ?',
      [normalized],
    );
    if (existing.isNotEmpty) return existing.first['id'] as String;
    final now = DateTime.now().toUtc().toIso8601String();
    final id = sha256.convert(utf8.encode('tag:$normalized')).toString();
    db.execute(
      'INSERT INTO tags(id, name, normalized_name, color, created_at, updated_at) '
      'VALUES (?, ?, ?, ?, ?, ?)',
      [id, name.trim(), normalized, AnnotationColor.blue.name, now, now],
    );
    return id;
  }

  static String _normalizeTag(String value) => value.trim().toLowerCase();

  static List<String> _cleanTags(List<String> values) => values
      .map((value) => value.trim())
      .where((value) => value.isNotEmpty)
      .toSet()
      .take(12)
      .toList(growable: false);

  static String? _clean(String? value) {
    final clean = value?.trim();
    return clean == null || clean.isEmpty ? null : clean;
  }

  static void _validateText(String? value) {
    if (value != null && value.length > AppConfig.maxAnnotationNoteLength) {
      throw ArgumentError.value(value.length, 'note', 'Note is too long.');
    }
  }

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;
}
