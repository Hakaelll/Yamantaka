import 'package:sqlite3/common.dart' show Row;

import '../../features/library/domain/library_document.dart';
import '../../features/library/domain/document_cover.dart';
import '../../features/library/domain/document_enrichment.dart';
import '../../features/library/domain/file_validation.dart';
import '../../features/library/domain/library_query.dart';
import '../../features/organization/domain/organization_models.dart';
import '../../features/reader/domain/reader_models.dart';
import '../database/app_database.dart';

final class DocumentRepository {
  const DocumentRepository(this._database);

  final AppDatabase _database;

  int get documentCount =>
      _database.database
              .select('SELECT COUNT(*) FROM documents')
              .first
              .columnAt(0)
          as int;

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

  List<LibraryDocument> needingEnrichment(int analyzerVersion) {
    final rows = _database.database.select(
      '$_documentSelect WHERE d.is_missing = 0 AND '
      '(e.document_id IS NULL OR e.fingerprint <> d.fingerprint OR '
      'e.analyzer_version < ? OR e.status IN (?, ?)) '
      'ORDER BY d.last_opened_at DESC',
      [
        analyzerVersion,
        EnrichmentStatus.pending.name,
        EnrichmentStatus.analyzing.name,
      ],
    );
    return rows.map(_fromRow).toList(growable: false);
  }

  List<LibraryFacet> authorFacets([LibraryQuery query = const LibraryQuery()]) {
    final base = query.copyWith(authors: const {}).normalized();
    final where = _whereClause(base);
    final rows = _database.database.select('''
      SELECT trim(coalesce(d.author_override, d.author, '')) AS value,
        COUNT(*) AS item_count
      FROM documents d
      LEFT JOIN document_notes n ON n.document_id = d.id
      WHERE ${where.sql}
      GROUP BY lower(trim(coalesce(d.author_override, d.author, '')))
      HAVING value <> ''
      ORDER BY value COLLATE NOCASE
      ''', where.parameters);
    return rows
        .map(
          (row) => LibraryFacet(
            value: row['value'] as String,
            count: row['item_count'] as int,
          ),
        )
        .toList(growable: false);
  }

  Map<LibraryTopic, int> topicFacets([
    LibraryQuery query = const LibraryQuery(),
  ]) {
    final base = query.copyWith(topics: const {}).normalized();
    final where = _whereClause(base);
    final rows = _database.database.select('''
      SELECT dt.topic_key, COUNT(DISTINCT dt.document_id) AS item_count
      FROM document_topics dt
      JOIN documents d ON d.id = dt.document_id
      LEFT JOIN document_notes n ON n.document_id = d.id
      WHERE ${where.sql}
      GROUP BY dt.topic_key
      ''', where.parameters);
    final facets = <LibraryTopic, int>{};
    for (final row in rows) {
      final topic = _topicByName(row['topic_key'] as String);
      if (topic != null) facets[topic] = row['item_count'] as int;
    }
    return facets;
  }

  Map<LibraryQueryReadingState, int> readingStateCounts([
    LibraryQuery query = const LibraryQuery(),
  ]) {
    final base = query
        .copyWith(readingState: LibraryQueryReadingState.all)
        .normalized();
    final where = _whereClause(base);
    final row = _database.database.select('''
      SELECT COUNT(*) AS all_count,
        SUM(CASE WHEN d.reading_progress > 0 AND d.reading_progress < 1
          THEN 1 ELSE 0 END) AS progress_count,
        SUM(CASE WHEN d.reading_progress <= 0 THEN 1 ELSE 0 END)
          AS not_started_count,
        SUM(CASE WHEN d.reading_progress >= 1 THEN 1 ELSE 0 END)
          AS completed_count
      FROM documents d
      LEFT JOIN document_notes n ON n.document_id = d.id
      WHERE ${where.sql}
      ''', where.parameters).first;
    int count(String key) => (row[key] as int?) ?? 0;
    return {
      LibraryQueryReadingState.all: count('all_count'),
      LibraryQueryReadingState.inProgress: count('progress_count'),
      LibraryQueryReadingState.notStarted: count('not_started_count'),
      LibraryQueryReadingState.completed: count('completed_count'),
    };
  }

  LibraryPage page([LibraryQuery query = const LibraryQuery()]) {
    final normalized = query.normalized();
    final where = _whereClause(normalized);
    final rows = _database.database.select(
      '$_documentSelect WHERE ${where.sql} '
      'ORDER BY ${_sortColumn(normalized.sort)} LIMIT ? OFFSET ?',
      [...where.parameters, normalized.limit, normalized.offset],
    );
    final total =
        _database.database
                .select(
                  'SELECT COUNT(*) FROM documents d '
                  'LEFT JOIN document_notes n ON n.document_id = d.id '
                  'WHERE ${where.sql}',
                  where.parameters,
                )
                .first
                .columnAt(0)
            as int;
    final documents = rows.map(_fromRow).toList(growable: false);
    return LibraryPage(
      documents: documents,
      offset: normalized.offset,
      total: total,
      hasMore: normalized.offset + documents.length < total,
    );
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
        fingerprint, thumbnail_path, is_favorite, source_title, source_author,
        title_override, author_override
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        is_favorite = excluded.is_favorite,
        source_title = excluded.source_title,
        source_author = excluded.source_author,
        title_override = excluded.title_override,
        author_override = excluded.author_override
      ''',
      [
        document.id,
        document.path,
        document.normalizedPath,
        document.fileName,
        document.sourceTitle ?? document.title,
        document.sourceAuthor ?? document.author,
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
        document.sourceTitle ?? document.title,
        document.sourceAuthor ?? document.author,
        document.titleOverride,
        document.authorOverride,
      ],
    );
    final note = _cleanNote(document.libraryNote);
    if (note == null) {
      _database.database.execute(
        'DELETE FROM document_notes WHERE document_id = ?',
        [document.id],
      );
    } else {
      _database.database.execute(
        '''
        INSERT INTO document_notes(document_id, note, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(document_id) DO UPDATE SET
          note = excluded.note,
          updated_at = excluded.updated_at
        ''',
        [document.id, note, _date(DateTime.now())],
      );
    }
  }

  LibraryDocument? updateMetadataOverrides(
    String documentId, {
    Object? title = _keepMetadata,
    Object? author = _keepMetadata,
  }) => updateLocalMetadata(documentId, title: title, author: author);

  /// Updates all local metadata fields in one transaction so a failed note
  /// validation cannot leave title/author and note out of sync.
  LibraryDocument? updateLocalMetadata(
    String documentId, {
    Object? title = _keepMetadata,
    Object? author = _keepMetadata,
    Object? note = _keepMetadata,
  }) {
    final current = byId(documentId);
    if (current == null) return null;
    final nextTitle = identical(title, _keepMetadata)
        ? current.titleOverride
        : _cleanMetadata(title as String?);
    final nextAuthor = identical(author, _keepMetadata)
        ? current.authorOverride
        : _cleanMetadata(author as String?);
    final nextNote = identical(note, _keepMetadata)
        ? current.libraryNote
        : _cleanNote(note as String?);
    final database = _database.database;
    database.execute('BEGIN IMMEDIATE');
    try {
      database.execute(
        'UPDATE documents SET title_override = ?, author_override = ? '
        'WHERE id = ?',
        [nextTitle, nextAuthor, documentId],
      );
      _writeLibraryNote(database, documentId, nextNote);
      database.execute('COMMIT');
    } catch (_) {
      database.execute('ROLLBACK');
      rethrow;
    }
    return byId(documentId);
  }

  LibraryDocument? clearMetadataOverrides(String documentId) =>
      updateMetadataOverrides(documentId, title: null, author: null);

  LibraryDocument? updateLibraryNote(String documentId, String? note) {
    return updateLocalMetadata(documentId, note: note);
  }

  static void _writeLibraryNote(
    dynamic database,
    String documentId,
    String? note,
  ) {
    if (note == null) {
      database.execute('DELETE FROM document_notes WHERE document_id = ?', [
        documentId,
      ]);
      return;
    }
    database.execute(
      '''
      INSERT INTO document_notes(document_id, note, updated_at)
      VALUES (?, ?, ?)
      ON CONFLICT(document_id) DO UPDATE SET
        note = excluded.note,
        updated_at = excluded.updated_at
      ''',
      [documentId, note, _date(DateTime.now())],
    );
  }

  void beginEnrichment(LibraryDocument document, int analyzerVersion) {
    _database.database.execute(
      '''
      INSERT INTO document_enrichment(
        document_id, fingerprint, analyzer_version, status, topics_manual
      ) VALUES (?, ?, ?, ?, ?)
      ON CONFLICT(document_id) DO UPDATE SET
        fingerprint = excluded.fingerprint,
        analyzer_version = excluded.analyzer_version,
        status = excluded.status,
        error_code = NULL
      ''',
      [
        document.id,
        document.fingerprint,
        analyzerVersion,
        EnrichmentStatus.analyzing.name,
        document.enrichment?.topicsManual == true ? 1 : 0,
      ],
    );
  }

  void saveEnrichment({
    required LibraryDocument document,
    required int analyzerVersion,
    required EnrichmentStatus status,
    String? detectedTitle,
    String? detectedAuthor,
    String? suggestedTitle,
    String? suggestedAuthor,
    double? titleConfidence,
    double? authorConfidence,
    List<DocumentTopicAssignment> automaticTopics = const [],
    String? errorCode,
  }) {
    final db = _database.database;
    final now = _date(DateTime.now());
    db.execute('BEGIN IMMEDIATE');
    try {
      if (detectedTitle != null || detectedAuthor != null) {
        db.execute(
          '''
          UPDATE documents SET
            title = coalesce(?, title),
            source_title = coalesce(?, source_title),
            author = coalesce(?, author),
            source_author = coalesce(?, source_author)
          WHERE id = ?
          ''',
          [
            _cleanMetadata(detectedTitle),
            _cleanMetadata(detectedTitle),
            _cleanMetadata(detectedAuthor),
            _cleanMetadata(detectedAuthor),
            document.id,
          ],
        );
      }
      final topicsManual =
          db.select(
            'SELECT topics_manual FROM document_enrichment '
            'WHERE document_id = ?',
            [document.id],
          ).firstOrNull?['topics_manual'] ==
          1;
      if (!topicsManual) {
        db.execute(
          "DELETE FROM document_topics WHERE document_id = ? AND origin = 'automatic'",
          [document.id],
        );
        for (final assignment in automaticTopics.take(3)) {
          db.execute(
            '''
            INSERT INTO document_topics(
              document_id, topic_key, confidence, origin, analyzer_version,
              fingerprint, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(document_id, topic_key) DO NOTHING
            ''',
            [
              document.id,
              assignment.topic.name,
              assignment.confidence.clamp(0, 1),
              EnrichmentOrigin.automatic.name,
              analyzerVersion,
              document.fingerprint,
              now,
            ],
          );
        }
      }
      db.execute(
        '''
        INSERT INTO document_enrichment(
          document_id, fingerprint, analyzer_version, status,
          suggested_title, suggested_author, title_confidence,
          author_confidence, topics_manual, analyzed_at, error_code
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(document_id) DO UPDATE SET
          fingerprint = excluded.fingerprint,
          analyzer_version = excluded.analyzer_version,
          status = excluded.status,
          suggested_title = excluded.suggested_title,
          suggested_author = excluded.suggested_author,
          title_confidence = excluded.title_confidence,
          author_confidence = excluded.author_confidence,
          analyzed_at = excluded.analyzed_at,
          error_code = excluded.error_code
        ''',
        [
          document.id,
          document.fingerprint,
          analyzerVersion,
          status.name,
          _cleanMetadata(suggestedTitle),
          _cleanMetadata(suggestedAuthor),
          titleConfidence,
          authorConfidence,
          topicsManual ? 1 : 0,
          now,
          errorCode,
        ],
      );
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
  }

  LibraryDocument? acceptEnrichmentSuggestions(String documentId) {
    final current = byId(documentId);
    final enrichment = current?.enrichment;
    if (current == null || enrichment == null) return current;
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      db.execute(
        '''
        UPDATE documents SET
          title = coalesce(?, title), source_title = coalesce(?, source_title),
          author = coalesce(?, author), source_author = coalesce(?, source_author)
        WHERE id = ?
        ''',
        [
          enrichment.suggestedTitle,
          enrichment.suggestedTitle,
          enrichment.suggestedAuthor,
          enrichment.suggestedAuthor,
          documentId,
        ],
      );
      db.execute(
        '''
        UPDATE document_enrichment SET status = ?, suggested_title = NULL,
          suggested_author = NULL, analyzed_at = ? WHERE document_id = ?
        ''',
        [EnrichmentStatus.completed.name, _date(DateTime.now()), documentId],
      );
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
    return byId(documentId);
  }

  LibraryDocument? dismissEnrichmentSuggestions(String documentId) {
    _database.database.execute(
      '''
      UPDATE document_enrichment SET status = ?, suggested_title = NULL,
        suggested_author = NULL, analyzed_at = ? WHERE document_id = ?
      ''',
      [EnrichmentStatus.completed.name, _date(DateTime.now()), documentId],
    );
    return byId(documentId);
  }

  LibraryDocument? setManualTopics(
    String documentId,
    Set<LibraryTopic> topics, {
    required int analyzerVersion,
  }) {
    final document = byId(documentId);
    if (document == null) return null;
    final db = _database.database;
    final now = _date(DateTime.now());
    db.execute('BEGIN IMMEDIATE');
    try {
      db.execute('DELETE FROM document_topics WHERE document_id = ?', [
        documentId,
      ]);
      for (final topic in topics) {
        db.execute(
          '''
          INSERT INTO document_topics(
            document_id, topic_key, confidence, origin, analyzer_version,
            fingerprint, updated_at
          ) VALUES (?, ?, 1, ?, ?, ?, ?)
          ''',
          [
            documentId,
            topic.name,
            EnrichmentOrigin.manual.name,
            analyzerVersion,
            document.fingerprint,
            now,
          ],
        );
      }
      db.execute(
        '''
        INSERT INTO document_enrichment(
          document_id, fingerprint, analyzer_version, status, topics_manual,
          analyzed_at
        ) VALUES (?, ?, ?, ?, 1, ?)
        ON CONFLICT(document_id) DO UPDATE SET topics_manual = 1
        ''',
        [
          documentId,
          document.fingerprint,
          analyzerVersion,
          EnrichmentStatus.completed.name,
          now,
        ],
      );
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
    return byId(documentId);
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

  void setFavorite(String documentId, bool value) {
    _database.database.execute(
      'UPDATE documents SET is_favorite = ? WHERE id = ?',
      [value ? 1 : 0, documentId],
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

  int get fileValidationCount => documentCount;

  List<FileValidationCandidate> fileValidationBatch({
    String? afterId,
    int limit = 24,
  }) {
    final boundedLimit = limit.clamp(1, 100);
    final rows = _database.database.select(
      'SELECT id, path, file_size, file_modified_at, is_missing, is_modified '
      'FROM documents ${afterId == null ? '' : 'WHERE id > ? '} '
      'ORDER BY id LIMIT ?',
      [?afterId, boundedLimit],
    );
    return rows
        .map(
          (row) => FileValidationCandidate(
            id: row['id'] as String,
            path: row['path'] as String,
            fileSize: row['file_size'] as int,
            fileModifiedAt: DateTime.parse(row['file_modified_at'] as String),
            wasMissing: (row['is_missing'] as int) == 1,
            wasModified: (row['is_modified'] as int) == 1,
          ),
        )
        .toList(growable: false);
  }

  /// Persists a small validation batch and returns only documents whose
  /// user-visible availability state actually changed.
  Set<String> applyFileValidationBatch(
    List<FileValidationResult> results, {
    DateTime? validatedAt,
  }) {
    if (results.isEmpty) return const {};
    final stamp = _date(validatedAt ?? DateTime.now());
    final changed = <String>{};
    _database.writeTransaction(() {
      for (final result in results) {
        if (result.status == FileValidationStatus.failed) continue;
        final missing = result.status == FileValidationStatus.missing;
        final modified = result.status == FileValidationStatus.modified;
        _database.database.execute(
          'UPDATE documents SET is_missing = ?, is_modified = ?, '
          'last_validated_at = ? WHERE id = ? AND '
          '(is_missing <> ? OR is_modified <> ?)',
          [
            missing ? 1 : 0,
            modified ? 1 : 0,
            stamp,
            result.id,
            missing ? 1 : 0,
            modified ? 1 : 0,
          ],
        );
        final changedRows =
            _database.database.select('SELECT changes()').first.columnAt(0)
                as int;
        if (changedRows > 0) {
          changed.add(result.id);
        } else {
          _database.database.execute(
            'UPDATE documents SET last_validated_at = ? WHERE id = ?',
            [stamp, result.id],
          );
        }
      }
    });
    return Set.unmodifiable(changed);
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

  void markFinished(String documentId) {
    _database.database.execute(
      '''
      UPDATE documents
      SET last_page_index = CASE WHEN page_count > 0 THEN page_count - 1 ELSE 0 END,
          reading_progress = 1,
          last_opened_at = ?
      WHERE id = ?
      ''',
      [_date(DateTime.now()), documentId],
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
    final topics = <DocumentTopicAssignment>[];
    final topicData = row['topic_data'] as String?;
    if (topicData != null && topicData.isNotEmpty) {
      for (final encoded in topicData.split(';;')) {
        final parts = encoded.split('|');
        if (parts.length != 3) continue;
        final topic = _topicByName(parts[0]);
        final confidence = double.tryParse(parts[1]);
        final origin = _enumByName(
          EnrichmentOrigin.values,
          parts[2],
          EnrichmentOrigin.automatic,
        );
        if (topic != null && confidence != null) {
          topics.add(
            DocumentTopicAssignment(
              topic: topic,
              confidence: confidence,
              origin: origin,
            ),
          );
        }
      }
    }
    final enrichmentDocumentId = row['enrichment_document_id'] as String?;
    final enrichment = enrichmentDocumentId == null
        ? null
        : DocumentEnrichment(
            documentId: enrichmentDocumentId,
            fingerprint: row['enrichment_fingerprint'] as String,
            analyzerVersion: row['enrichment_analyzer_version'] as int,
            status: _enumByName(
              EnrichmentStatus.values,
              row['enrichment_status'] as String,
              EnrichmentStatus.pending,
            ),
            suggestedTitle: row['suggested_title'] as String?,
            suggestedAuthor: row['suggested_author'] as String?,
            titleConfidence: row['title_confidence'] as double?,
            authorConfidence: row['author_confidence'] as double?,
            topicsManual: (row['topics_manual'] as int) == 1,
            analyzedAt: row['enrichment_analyzed_at'] == null
                ? null
                : DateTime.parse(row['enrichment_analyzed_at'] as String),
            errorCode: row['enrichment_error_code'] as String?,
          );
    return LibraryDocument(
      id: row['id'] as String,
      path: path,
      normalizedPath: row['normalized_path'] as String,
      fileName: row['file_name'] as String,
      title: (row['title_override'] as String?) ?? row['title'] as String?,
      author: (row['author_override'] as String?) ?? row['author'] as String?,
      sourceTitle: row['source_title'] as String? ?? row['title'] as String?,
      sourceAuthor: row['source_author'] as String? ?? row['author'] as String?,
      titleOverride: row['title_override'] as String?,
      authorOverride: row['author_override'] as String?,
      libraryNote: row['library_note'] as String?,
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
      hasFileChanged: (row['is_modified'] as int) == 1,
      fingerprint: row['fingerprint'] as String,
      thumbnailPath:
          cover?.smallThumbnailPath ?? row['thumbnail_path'] as String?,
      coverMetadata: cover,
      isFavorite: (row['is_favorite'] as int) == 1,
      topics: List.unmodifiable(topics),
      enrichment: enrichment,
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
      , n.note AS library_note
      , e.document_id AS enrichment_document_id
      , e.fingerprint AS enrichment_fingerprint
      , e.analyzer_version AS enrichment_analyzer_version
      , e.status AS enrichment_status
      , e.suggested_title AS suggested_title
      , e.suggested_author AS suggested_author
      , e.title_confidence AS title_confidence
      , e.author_confidence AS author_confidence
      , e.topics_manual AS topics_manual
      , e.analyzed_at AS enrichment_analyzed_at
      , e.error_code AS enrichment_error_code
      , (
        SELECT group_concat(
          dt.topic_key || '|' || dt.confidence || '|' || dt.origin,
          ';;'
        )
        FROM document_topics dt WHERE dt.document_id = d.id
      ) AS topic_data
    FROM documents d
    LEFT JOIN document_covers c ON c.document_id = d.id
    LEFT JOIN document_notes n ON n.document_id = d.id
    LEFT JOIN document_enrichment e ON e.document_id = d.id
  ''';

  static String _date(DateTime value) => value.toUtc().toIso8601String();

  ({String sql, List<Object?> parameters}) _whereClause(LibraryQuery query) {
    final conditions = <String>['1 = 1'];
    final parameters = <Object?>[];
    if (!query.includeMissing) conditions.add('d.is_missing = 0');
    if (query.favoritesOnly) conditions.add('d.is_favorite = 1');
    switch (query.readingState) {
      case LibraryQueryReadingState.all:
        break;
      case LibraryQueryReadingState.inProgress:
        conditions.add('d.reading_progress > 0 AND d.reading_progress < 1');
      case LibraryQueryReadingState.notStarted:
        conditions.add('d.reading_progress <= 0');
      case LibraryQueryReadingState.completed:
        conditions.add('d.reading_progress >= 1');
    }
    if (query.authors.isNotEmpty) {
      final placeholders = List.filled(query.authors.length, '?').join(', ');
      conditions.add(
        'lower(trim(coalesce(d.author_override, d.author, \'\'))) '
        'IN ($placeholders)',
      );
      parameters.addAll(query.authors);
    }
    if (query.topics.isNotEmpty) {
      final placeholders = List.filled(query.topics.length, '?').join(', ');
      conditions.add(
        'EXISTS (SELECT 1 FROM document_topics filter_topics '
        'WHERE filter_topics.document_id = d.id '
        'AND filter_topics.topic_key IN ($placeholders))',
      );
      parameters.addAll(query.topics.map((topic) => topic.name));
    }
    if (query.collectionId case final collectionId?) {
      conditions.add(
        'EXISTS (SELECT 1 FROM collection_documents scoped_collection '
        'WHERE scoped_collection.document_id = d.id '
        'AND scoped_collection.collection_id = ?)',
      );
      parameters.add(collectionId);
    }
    switch (query.smartCollection) {
      case SmartCollection.inProgress:
        conditions.add('d.reading_progress > 0 AND d.reading_progress < 0.995');
      case SmartCollection.notStarted:
        conditions.add('d.reading_progress <= 0');
      case SmartCollection.finished:
        conditions.add('d.reading_progress >= 0.995');
      case SmartCollection.annotated:
        conditions.add(
          'EXISTS (SELECT 1 FROM annotations scoped_annotations '
          'WHERE scoped_annotations.document_id = d.id)',
        );
      case SmartCollection.bookmarked:
        conditions.add(
          'EXISTS (SELECT 1 FROM bookmarks scoped_bookmarks '
          'WHERE scoped_bookmarks.document_id = d.id)',
        );
      case SmartCollection.missing:
        conditions.add('d.is_missing = 1');
      case SmartCollection.recentlyOpened:
        conditions.add("d.last_opened_at >= datetime('now', '-14 days')");
      case SmartCollection.recentlyAdded:
        conditions.add("d.added_at >= datetime('now', '-14 days')");
      case null:
        break;
    }
    final text = query.text.toLowerCase();
    if (text.isNotEmpty) {
      conditions.add(
        '(lower(d.file_name) LIKE ? OR '
        'lower(coalesce(d.title_override, d.title, \'\')) LIKE ? OR '
        'lower(coalesce(d.author_override, d.author, \'\')) LIKE ? OR '
        'lower(coalesce(n.note, \'\')) LIKE ? OR '
        'EXISTS (SELECT 1 FROM document_topics text_topics '
        'WHERE text_topics.document_id = d.id '
        'AND lower(text_topics.topic_key) LIKE ?))',
      );
      final pattern = '%$text%';
      parameters.addAll([pattern, pattern, pattern, pattern, pattern]);
    }
    return (sql: conditions.join(' AND '), parameters: parameters);
  }

  static String _sortColumn(LibraryQuerySort sort) => switch (sort) {
    LibraryQuerySort.lastOpened => 'd.last_opened_at DESC, d.id ASC',
    LibraryQuerySort.added => 'd.added_at DESC, d.id ASC',
    LibraryQuerySort.title =>
      'lower(coalesce(d.title_override, d.title, d.file_name)) ASC, d.id ASC',
    LibraryQuerySort.progress => 'd.reading_progress DESC, d.id ASC',
  };

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;

  static LibraryTopic? _topicByName(String name) =>
      LibraryTopic.values.where((topic) => topic.name == name).firstOrNull;

  static String? _cleanMetadata(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    if (clean.length > 512) {
      throw ArgumentError.value(value, 'metadata', 'Maximum length is 512.');
    }
    return clean;
  }

  static String? _cleanNote(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    if (clean.length > 4000) {
      throw ArgumentError.value(value, 'note', 'Maximum length is 4000.');
    }
    return clean;
  }
}

const Object _keepMetadata = Object();
