import '../../app/app_config.dart';
import '../../features/library/domain/library_document.dart';
import '../../features/search/domain/global_search_models.dart';
import '../database/app_database.dart';

final class GlobalSearchRepository {
  const GlobalSearchRepository(this._database);

  final AppDatabase _database;

  List<GlobalSearchResult> search(
    String query, {
    int limit = AppConfig.maxSearchResults,
    int offset = 0,
  }) {
    final expression = _ftsExpression(query);
    if (expression.isEmpty) return const [];
    final rows = _database.database.select(
      '''
      SELECT document_id, page_index, kind, title,
        snippet(library_search, 4, '‹', '›', ' … ', 18) AS excerpt,
        bm25(library_search, 4.0, 1.0, 0.5) AS rank
      FROM library_search
      WHERE library_search MATCH ?
      ORDER BY rank
      LIMIT ? OFFSET ?
      ''',
      [expression, limit.clamp(1, 200), offset.clamp(0, 10000)],
    );
    return rows
        .map(
          (row) => GlobalSearchResult(
            documentId: row['document_id'] as String,
            pageIndex: (row['page_index'] as int) == -1
                ? null
                : row['page_index'] as int,
            kind:
                GlobalSearchKind.values
                    .where((value) => value.name == row['kind'])
                    .firstOrNull ??
                GlobalSearchKind.metadata,
            title: row['title'] as String? ?? 'Documento',
            snippet: row['excerpt'] as String? ?? '',
            rank: (row['rank'] as num).toDouble(),
          ),
        )
        .toList(growable: false);
  }

  void beginDocument(LibraryDocument document) {
    final db = _database.database;
    db
      ..execute('DELETE FROM library_search WHERE document_id = ?', [
        document.id,
      ])
      ..execute(
        '''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        ) VALUES (?, -1, ?, ?, ?, ?)
        ''',
        [
          document.id,
          GlobalSearchKind.metadata.name,
          document.title ?? document.fileName,
          [document.author, document.fileName].whereType<String>().join(' '),
          _documentTags(document.id),
        ],
      )
      ..execute(
        '''
        INSERT INTO search_index_state(
          document_id, fingerprint, status, indexed_pages, updated_at, error_code
        ) VALUES (?, ?, ?, 0, ?, NULL)
        ON CONFLICT(document_id) DO UPDATE SET
          fingerprint = excluded.fingerprint,
          status = excluded.status,
          indexed_pages = 0,
          updated_at = excluded.updated_at,
          error_code = NULL
        ''',
        [
          document.id,
          document.fingerprint,
          IndexDocumentStatus.indexing.name,
          DateTime.now().toUtc().toIso8601String(),
        ],
      );
  }

  void addPage({
    required LibraryDocument document,
    required int pageIndex,
    required String text,
  }) {
    _database.database
      ..execute(
        '''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        ) VALUES (?, ?, ?, ?, ?, ?)
        ''',
        [
          document.id,
          pageIndex,
          GlobalSearchKind.pageText.name,
          document.title ?? document.fileName,
          text,
          _documentTags(document.id),
        ],
      )
      ..execute(
        'UPDATE search_index_state SET indexed_pages = ?, updated_at = ? '
        'WHERE document_id = ?',
        [pageIndex + 1, DateTime.now().toUtc().toIso8601String(), document.id],
      );
  }

  void finishDocument(String documentId, {required bool hasExtractableText}) {
    _database.database.execute(
      'UPDATE search_index_state SET status = ?, updated_at = ? '
      'WHERE document_id = ?',
      [
        hasExtractableText
            ? IndexDocumentStatus.indexed.name
            : IndexDocumentStatus.notIndexable.name,
        DateTime.now().toUtc().toIso8601String(),
        documentId,
      ],
    );
  }

  void failDocument(String documentId, String errorCode) {
    _database.database.execute(
      'UPDATE search_index_state SET status = ?, error_code = ?, '
      'updated_at = ? WHERE document_id = ?',
      [
        IndexDocumentStatus.failed.name,
        errorCode,
        DateTime.now().toUtc().toIso8601String(),
        documentId,
      ],
    );
  }

  void refreshAnnotationEntries() {
    final db = _database.database;
    db.execute('DELETE FROM library_search WHERE kind = ?', [
      GlobalSearchKind.annotation.name,
    ]);
    db.execute('''
      INSERT INTO library_search(
        document_id, page_index, kind, title, body, tags
      )
      SELECT a.document_id, a.page_index, '${GlobalSearchKind.annotation.name}',
        COALESCE(d.title, d.file_name),
        COALESCE(a.selected_text, '') || ' ' || COALESCE(a.note, ''),
        COALESCE((
          SELECT group_concat(t.name, ' ')
          FROM annotation_tags at
          JOIN tags t ON t.id = at.tag_id
          WHERE at.annotation_id = a.id
        ), '')
      FROM annotations a
      JOIN documents d ON d.id = a.document_id
    ''');
  }

  void refreshOrganizationEntries() {
    final db = _database.database;
    db
      ..execute('DELETE FROM library_search WHERE kind IN (?, ?)', [
        GlobalSearchKind.collection.name,
        GlobalSearchKind.tag.name,
      ])
      ..execute('''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        )
        SELECT cd.document_id, -1, '${GlobalSearchKind.collection.name}',
          COALESCE(d.title, d.file_name), c.name, ''
        FROM collection_documents cd
        JOIN collections c ON c.id = cd.collection_id
        JOIN documents d ON d.id = cd.document_id
      ''')
      ..execute('''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        )
        SELECT dt.document_id, -1, '${GlobalSearchKind.tag.name}',
          COALESCE(d.title, d.file_name), '', t.name
        FROM document_tags dt
        JOIN tags t ON t.id = dt.tag_id
        JOIN documents d ON d.id = dt.document_id
      ''');
  }

  void clear() {
    _database.database
      ..execute('DELETE FROM library_search')
      ..execute('DELETE FROM search_index_state');
  }

  bool needsIndex(LibraryDocument document) {
    final rows = _database.database.select(
      'SELECT fingerprint, status FROM search_index_state WHERE document_id = ?',
      [document.id],
    );
    if (rows.isEmpty) return true;
    return rows.first['fingerprint'] != document.fingerprint ||
        rows.first['status'] == IndexDocumentStatus.failed.name ||
        rows.first['status'] == IndexDocumentStatus.indexing.name;
  }

  int get indexedDocumentCount =>
      _database.database
              .select(
                'SELECT COUNT(*) FROM search_index_state WHERE status = ?',
                [IndexDocumentStatus.indexed.name],
              )
              .first
              .columnAt(0)
          as int;

  String _documentTags(String documentId) => _database.database
      .select(
        '''
        SELECT t.name FROM tags t
        JOIN document_tags dt ON dt.tag_id = t.id
        WHERE dt.document_id = ?
        ''',
        [documentId],
      )
      .map((row) => row['name'] as String)
      .join(' ');

  static String _ftsExpression(String query) => query
      .trim()
      .split(RegExp(r'\s+'))
      .map((term) => term.replaceAll('"', ''))
      .where((term) => term.isNotEmpty)
      .take(8)
      .map((term) => '"$term"*')
      .join(' AND ');
}
