import 'dart:collection';

import '../../app/app_config.dart';
import '../../features/library/domain/library_document.dart';
import '../../features/search/domain/global_search_models.dart';
import '../database/app_database.dart';

final class GlobalSearchRepository {
  GlobalSearchRepository(this._database);

  final AppDatabase _database;
  final LinkedHashMap<String, _SearchCacheEntry> _recentSearches =
      LinkedHashMap();
  static final Map<String, GlobalSearchKind> _kindsByName = GlobalSearchKind
      .values
      .asNameMap();
  static const int _cacheLifetimeMicroseconds = 1000000;
  static const int _maxCachedSearches = 8;

  List<GlobalSearchResult> search(
    String query, {
    int limit = AppConfig.maxSearchResults,
    int offset = 0,
    Set<GlobalSearchKind> kinds = const {},
  }) {
    final expression = _ftsExpression(query);
    if (expression.isEmpty) return const [];
    final selectedKinds = kinds.toList(growable: false);
    selectedKinds.sort((left, right) => left.index.compareTo(right.index));
    final boundedLimit = limit.clamp(1, 200);
    final boundedOffset = offset.clamp(0, 10000);
    final cacheKey =
        '$expression|$boundedLimit|$boundedOffset|'
        '${selectedKinds.map((kind) => kind.name).join(',')}';
    final now = DateTime.now().microsecondsSinceEpoch;
    final cached = _recentSearches.remove(cacheKey);
    if (cached != null &&
        now - cached.createdAtMicroseconds <= _cacheLifetimeMicroseconds) {
      _recentSearches[cacheKey] = cached;
      return cached.results;
    }
    final kindClause = selectedKinds.isEmpty
        ? ''
        : 'AND library_search.kind IN '
              '(${List.filled(selectedKinds.length, '?').join(',')})';
    final rows = _database.database.select(
      '''
      SELECT library_search.document_id, library_search.page_index,
        library_search.kind, library_search.title,
        snippet(library_search, 4, '‹', '›', ' … ', 18) AS excerpt,
        bm25(library_search, 4.0, 1.0, 0.5) AS rank,
        d.is_missing, d.is_modified
      FROM library_search
      JOIN documents d ON d.id = library_search.document_id
      WHERE library_search MATCH ?
        $kindClause
      ORDER BY rank, d.last_opened_at DESC,
        library_search.document_id, library_search.page_index,
        library_search.kind
      LIMIT ? OFFSET ?
      ''',
      [
        expression,
        ...selectedKinds.map((kind) => kind.name),
        boundedLimit,
        boundedOffset,
      ],
    );
    final results = List<GlobalSearchResult>.unmodifiable(
      rows.map(
        (row) => GlobalSearchResult(
          documentId: row['document_id'] as String,
          pageIndex: (row['page_index'] as int) == -1
              ? null
              : row['page_index'] as int,
          kind: _kindsByName[row['kind']] ?? GlobalSearchKind.metadata,
          title: row['title'] as String? ?? 'Documento',
          snippet: row['excerpt'] as String? ?? '',
          rank: (row['rank'] as num).toDouble(),
          isMissing: (row['is_missing'] as int? ?? 0) == 1,
          isStale: (row['is_modified'] as int? ?? 0) == 1,
        ),
      ),
    );
    _recentSearches[cacheKey] = _SearchCacheEntry(
      createdAtMicroseconds: now,
      results: results,
    );
    while (_recentSearches.length > _maxCachedSearches) {
      _recentSearches.remove(_recentSearches.keys.first);
    }
    return results;
  }

  void beginDocument(LibraryDocument document, {bool resume = false}) {
    _invalidateSearchCache();
    final db = _database.database;
    if (resume && resumableCursor(document) != null) {
      refreshMetadataEntry(document);
      return;
    }
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
          document.effectiveTitle,
          [
            document.effectiveAuthor,
            document.fileName,
            document.libraryNote,
          ].whereType<String>().join(' '),
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

  /// Updates metadata rows immediately after a local title, author or note
  /// edit without forcing a full PDF re-index. Page text remains untouched.
  void refreshMetadataEntry(LibraryDocument document) {
    _invalidateSearchCache();
    final db = _database.database;
    final body = [
      document.effectiveAuthor,
      document.fileName,
      document.libraryNote,
    ].whereType<String>().join(' ');
    db
      ..execute('UPDATE library_search SET title = ? WHERE document_id = ?', [
        document.effectiveTitle,
        document.id,
      ])
      ..execute(
        'DELETE FROM library_search WHERE document_id = ? AND page_index = -1 '
        'AND kind = ?',
        [document.id, GlobalSearchKind.metadata.name],
      )
      ..execute(
        '''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        ) VALUES (?, -1, ?, ?, ?, ?)
        ''',
        [
          document.id,
          GlobalSearchKind.metadata.name,
          document.effectiveTitle,
          body,
          _documentTags(document.id),
        ],
      );
  }

  void addPage({
    required LibraryDocument document,
    required int pageIndex,
    required String text,
  }) {
    _invalidateSearchCache();
    _database.database.execute(
      '''
        INSERT OR REPLACE INTO library_search(
          document_id, page_index, kind, title, body, tags
        ) VALUES (?, ?, ?, ?, ?, ?)
        ''',
      [
        document.id,
        pageIndex,
        GlobalSearchKind.pageText.name,
        document.effectiveTitle,
        text,
        _documentTags(document.id),
      ],
    );
  }

  int? resumableCursor(LibraryDocument document) {
    final rows = _database.database.select(
      'SELECT fingerprint, status, indexed_pages FROM search_index_state '
      'WHERE document_id = ?',
      [document.id],
    );
    if (rows.isEmpty ||
        rows.first['fingerprint'] != document.fingerprint ||
        rows.first['status'] != IndexDocumentStatus.indexing.name) {
      return null;
    }
    return (rows.first['indexed_pages'] as int? ?? 0).clamp(0, 10000000);
  }

  Set<int> indexedPageIndexes(String documentId) => _database.database
      .select(
        'SELECT page_index FROM library_search WHERE document_id = ? '
        'AND kind = ? AND page_index >= 0',
        [documentId, GlobalSearchKind.pageText.name],
      )
      .map((row) => row['page_index'] as int)
      .toSet();

  void updateCursor(String documentId, int indexedPages) {
    _database.database.execute(
      'UPDATE search_index_state SET indexed_pages = ?, updated_at = ? '
      'WHERE document_id = ? AND status = ?',
      [
        indexedPages.clamp(0, 10000000),
        DateTime.now().toUtc().toIso8601String(),
        documentId,
        IndexDocumentStatus.indexing.name,
      ],
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
    _invalidateSearchCache();
    final db = _database.database;
    db.execute('DELETE FROM library_search WHERE kind = ?', [
      GlobalSearchKind.annotation.name,
    ]);
    db.execute('''
      INSERT INTO library_search(
        document_id, page_index, kind, title, body, tags
      )
      SELECT a.document_id, a.page_index, '${GlobalSearchKind.annotation.name}',
        COALESCE(d.title_override, d.title, d.file_name),
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
    _invalidateSearchCache();
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
        COALESCE(d.title_override, d.title, d.file_name), c.name, ''
        FROM collection_documents cd
        JOIN collections c ON c.id = cd.collection_id
        JOIN documents d ON d.id = cd.document_id
      ''')
      ..execute('''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        )
        SELECT dt.document_id, -1, '${GlobalSearchKind.tag.name}',
        COALESCE(d.title_override, d.title, d.file_name), '', t.name
        FROM document_tags dt
        JOIN tags t ON t.id = dt.tag_id
        JOIN documents d ON d.id = dt.document_id
      ''');
  }

  void refreshBookmarkEntries() {
    _invalidateSearchCache();
    final db = _database.database;
    db
      ..execute('DELETE FROM library_search WHERE kind = ?', [
        GlobalSearchKind.bookmark.name,
      ])
      ..execute('''
        INSERT INTO library_search(
          document_id, page_index, kind, title, body, tags
        )
        SELECT b.document_id, b.page_index,
          '${GlobalSearchKind.bookmark.name}',
          COALESCE(d.title_override, d.title, d.file_name),
          COALESCE(b.label, '') || ' página ' || CAST(b.page_index + 1 AS TEXT),
          ''
        FROM bookmarks b
        JOIN documents d ON d.id = b.document_id
      ''');
  }

  void clear() {
    _invalidateSearchCache();
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
    if (rows.first['fingerprint'] != document.fingerprint ||
        rows.first['status'] == IndexDocumentStatus.failed.name ||
        rows.first['status'] == IndexDocumentStatus.indexing.name) {
      return true;
    }
    final metadataRows = _database.database.select(
      '''
      SELECT title, body FROM library_search
      WHERE document_id = ? AND page_index = -1 AND kind = ?
      LIMIT 1
      ''',
      [document.id, GlobalSearchKind.metadata.name],
    );
    if (metadataRows.isEmpty) return true;
    final expectedBody = [
      document.effectiveAuthor,
      document.fileName,
      document.libraryNote,
    ].whereType<String>().join(' ');
    return metadataRows.first['title'] != document.effectiveTitle ||
        metadataRows.first['body'] != expectedBody;
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

  void _invalidateSearchCache() => _recentSearches.clear();
}

final class _SearchCacheEntry {
  const _SearchCacheEntry({
    required this.createdAtMicroseconds,
    required this.results,
  });

  final int createdAtMicroseconds;
  final List<GlobalSearchResult> results;
}
