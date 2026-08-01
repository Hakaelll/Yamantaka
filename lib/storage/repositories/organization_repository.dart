import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/organization/domain/organization_models.dart';
import '../database/app_database.dart';

final class OrganizationRepository {
  const OrganizationRepository(this._database);

  final AppDatabase _database;

  List<DocumentCollection> collections() {
    final rows = _database.database.select('''
      SELECT c.*, COUNT(cd.document_id) AS document_count
      FROM collections c
      LEFT JOIN collection_documents cd ON cd.collection_id = c.id
      GROUP BY c.id
      ORDER BY c.is_pinned DESC, c.sort_order, c.name COLLATE NOCASE
    ''');
    return rows
        .map(
          (row) => DocumentCollection(
            id: row['id'] as String,
            name: row['name'] as String,
            icon: row['icon'] as String,
            sortOrder: row['sort_order'] as int,
            isPinned: (row['is_pinned'] as int) == 1,
            documentCount: row['document_count'] as int,
            createdAt: DateTime.parse(row['created_at'] as String),
            updatedAt: DateTime.parse(row['updated_at'] as String),
          ),
        )
        .toList(growable: false);
  }

  DocumentCollection createCollection(String name, {String icon = 'folder'}) {
    final cleanName = _validatedName(name);
    const allowedIcons = {'folder', 'study', 'reference', 'archive', 'star'};
    final cleanIcon = allowedIcons.contains(icon) ? icon : 'folder';
    final now = DateTime.now().toUtc();
    final normalized = cleanName.toLowerCase();
    final id = sha256
        .convert(
          utf8.encode('collection:$normalized:${now.microsecondsSinceEpoch}'),
        )
        .toString();
    final nextOrder =
        (_database.database
                .select(
                  'SELECT COALESCE(MAX(sort_order), -1) + 1 FROM collections',
                )
                .first
                .columnAt(0)
            as int);
    _database.database.execute(
      '''
      INSERT INTO collections(
        id, name, normalized_name, icon, sort_order, is_pinned,
        created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, 0, ?, ?)
      ''',
      [
        id,
        cleanName,
        normalized,
        cleanIcon,
        nextOrder,
        now.toIso8601String(),
        now.toIso8601String(),
      ],
    );
    return collections().firstWhere((collection) => collection.id == id);
  }

  DocumentCollection ensureCollection(String name, {String icon = 'folder'}) {
    final normalized = name.trim().toLowerCase();
    for (final collection in collections()) {
      if (collection.name.trim().toLowerCase() == normalized) return collection;
    }
    return createCollection(name, icon: icon);
  }

  void renameCollection(String id, String name) {
    final cleanName = _validatedName(name);
    _database.database.execute(
      'UPDATE collections SET name = ?, normalized_name = ?, updated_at = ? '
      'WHERE id = ?',
      [
        cleanName,
        cleanName.toLowerCase(),
        DateTime.now().toUtc().toIso8601String(),
        id,
      ],
    );
  }

  void deleteCollection(String id) {
    _database.database.execute('DELETE FROM collections WHERE id = ?', [id]);
  }

  void setPinned(String id, bool pinned) {
    _database.database.execute(
      'UPDATE collections SET is_pinned = ?, updated_at = ? WHERE id = ?',
      [pinned ? 1 : 0, DateTime.now().toUtc().toIso8601String(), id],
    );
  }

  void reorder(List<String> ids) {
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      for (var index = 0; index < ids.length; index++) {
        db.execute('UPDATE collections SET sort_order = ? WHERE id = ?', [
          index,
          ids[index],
        ]);
      }
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
  }

  void addDocument(String collectionId, String documentId) {
    _database.database.execute(
      '''
      INSERT OR IGNORE INTO collection_documents(
        collection_id, document_id, added_at
      ) VALUES (?, ?, ?)
      ''',
      [collectionId, documentId, DateTime.now().toUtc().toIso8601String()],
    );
  }

  void removeDocument(String collectionId, String documentId) {
    _database.database.execute(
      'DELETE FROM collection_documents '
      'WHERE collection_id = ? AND document_id = ?',
      [collectionId, documentId],
    );
  }

  Set<String> documentIds(String collectionId) => _database.database
      .select(
        'SELECT document_id FROM collection_documents WHERE collection_id = ?',
        [collectionId],
      )
      .map((row) => row['document_id'] as String)
      .toSet();

  Set<String> smartDocumentIds(SmartCollection smart) {
    final sql = switch (smart) {
      SmartCollection.inProgress =>
        'SELECT id FROM documents WHERE reading_progress > 0 '
            'AND reading_progress < 0.995',
      SmartCollection.notStarted =>
        'SELECT id FROM documents WHERE reading_progress <= 0',
      SmartCollection.finished =>
        'SELECT id FROM documents WHERE reading_progress >= 0.995',
      SmartCollection.annotated =>
        'SELECT DISTINCT document_id AS id FROM annotations',
      SmartCollection.bookmarked =>
        'SELECT DISTINCT document_id AS id FROM bookmarks',
      SmartCollection.missing =>
        'SELECT id FROM documents WHERE is_missing = 1',
      SmartCollection.recentlyOpened =>
        "SELECT id FROM documents WHERE last_opened_at >= datetime('now', '-14 days')",
      SmartCollection.recentlyAdded =>
        "SELECT id FROM documents WHERE added_at >= datetime('now', '-14 days')",
    };
    return _database.database
        .select(sql)
        .map((row) => row['id'] as String)
        .toSet();
  }

  List<DocumentTag> tags() {
    final rows = _database.database.select('''
      SELECT t.*, COUNT(DISTINCT dt.document_id) +
        COUNT(DISTINCT at.annotation_id) AS usage_count
      FROM tags t
      LEFT JOIN document_tags dt ON dt.tag_id = t.id
      LEFT JOIN annotation_tags at ON at.tag_id = t.id
      GROUP BY t.id
      ORDER BY t.name COLLATE NOCASE
    ''');
    return rows
        .map(
          (row) => DocumentTag(
            id: row['id'] as String,
            name: row['name'] as String,
            color: row['color'] as String,
            usageCount: row['usage_count'] as int,
          ),
        )
        .toList(growable: false);
  }

  void assignTagToDocument(String tagId, String documentId) {
    _database.database.execute(
      'INSERT OR IGNORE INTO document_tags(document_id, tag_id) VALUES (?, ?)',
      [documentId, tagId],
    );
  }

  void mergeTags(String sourceId, String targetId) {
    if (sourceId == targetId) return;
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      db
        ..execute(
          'INSERT OR IGNORE INTO document_tags(document_id, tag_id) '
          'SELECT document_id, ? FROM document_tags WHERE tag_id = ?',
          [targetId, sourceId],
        )
        ..execute(
          'INSERT OR IGNORE INTO annotation_tags(annotation_id, tag_id) '
          'SELECT annotation_id, ? FROM annotation_tags WHERE tag_id = ?',
          [targetId, sourceId],
        )
        ..execute('DELETE FROM tags WHERE id = ?', [sourceId])
        ..execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
  }

  void deleteTag(String id) {
    _database.database.execute('DELETE FROM tags WHERE id = ?', [id]);
  }

  static String _validatedName(String value) {
    final clean = value.trim();
    if (clean.isEmpty || clean.length > 60) {
      throw ArgumentError.value(value, 'name', 'Use between 1 and 60 chars.');
    }
    return clean;
  }
}
