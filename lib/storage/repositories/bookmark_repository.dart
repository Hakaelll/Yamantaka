import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/bookmarks/domain/bookmark.dart';
import '../database/app_database.dart';

final class BookmarkRepository {
  const BookmarkRepository(this._database);

  final AppDatabase _database;

  List<Bookmark> forDocument(String documentId) {
    final rows = _database.database.select(
      'SELECT * FROM bookmarks WHERE document_id = ? ORDER BY page_index',
      [documentId],
    );
    return rows
        .map(
          (row) => Bookmark(
            id: row['id'] as String,
            documentId: row['document_id'] as String,
            pageIndex: row['page_index'] as int,
            label: row['label'] as String?,
            createdAt: DateTime.parse(row['created_at'] as String),
            updatedAt: DateTime.parse(row['updated_at'] as String),
          ),
        )
        .toList(growable: false);
  }

  Bookmark? atPage(String documentId, int pageIndex) {
    final rows = _database.database.select(
      'SELECT * FROM bookmarks WHERE document_id = ? AND page_index = ?',
      [documentId, pageIndex],
    );
    if (rows.isEmpty) return null;
    final row = rows.first;
    return Bookmark(
      id: row['id'] as String,
      documentId: documentId,
      pageIndex: pageIndex,
      label: row['label'] as String?,
      createdAt: DateTime.parse(row['created_at'] as String),
      updatedAt: DateTime.parse(row['updated_at'] as String),
    );
  }

  Bookmark add(String documentId, int pageIndex, {String? label}) {
    final now = DateTime.now().toUtc();
    final id = sha256
        .convert(
          utf8.encode('$documentId:$pageIndex:${now.microsecondsSinceEpoch}'),
        )
        .toString();
    final bookmark = Bookmark(
      id: id,
      documentId: documentId,
      pageIndex: pageIndex,
      label: label?.trim().isEmpty ?? true ? null : label!.trim(),
      createdAt: now,
      updatedAt: now,
    );
    _database.database.execute(
      '''
      INSERT INTO bookmarks (
        id, document_id, page_index, label, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?)
      ON CONFLICT(document_id, page_index) DO UPDATE SET
        label = excluded.label,
        updated_at = excluded.updated_at
      ''',
      [
        bookmark.id,
        bookmark.documentId,
        bookmark.pageIndex,
        bookmark.label,
        now.toIso8601String(),
        now.toIso8601String(),
      ],
    );
    return atPage(documentId, pageIndex)!;
  }

  void remove(String id) {
    _database.database.execute('DELETE FROM bookmarks WHERE id = ?', [id]);
  }
}
