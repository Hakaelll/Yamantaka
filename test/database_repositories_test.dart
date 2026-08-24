import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:sqlite3/sqlite3.dart';
import 'package:yamantaka/app/app_config.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/application/duplicate_resolution_service.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/document_enrichment.dart';
import 'package:yamantaka/features/library/domain/import_queue.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/import_queue_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

void main() {
  late AppDatabase database;
  late DocumentRepository documents;

  setUp(() {
    database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
    documents = DocumentRepository(database);
  });

  tearDown(() => database.close());

  test('migration creates the complete current schema', () {
    final version = database.database
        .select('PRAGMA user_version')
        .first
        .columnAt(0);
    final tables = database.database
        .select(
          "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .map((row) => row['name'])
        .toSet();

    expect(version, AppConfig.databaseVersion);
    expect(
      tables,
      containsAll({
        'documents',
        'document_preferences',
        'bookmarks',
        'reading_sessions',
        'app_metadata',
        'annotations',
        'collections',
        'workspaces',
        'library_search',
        'document_covers',
        'document_enrichment',
        'document_topics',
      }),
    );
  });

  test('author, topic and reading filters use complete-library counts', () {
    final first = _document().copyWith(
      id: 'first',
      normalizedPath: 'first.pdf',
      path: 'first.pdf',
      fileName: 'first.pdf',
      author: 'Octavia Butler',
      readingProgress: 0.4,
      fingerprint: 'first',
    );
    final second = _document().copyWith(
      id: 'second',
      normalizedPath: 'second.pdf',
      path: 'second.pdf',
      fileName: 'second.pdf',
      author: 'Octavia Butler',
      readingProgress: 1,
      fingerprint: 'second',
    );
    final third = _document().copyWith(
      id: 'third',
      normalizedPath: 'third.pdf',
      path: 'third.pdf',
      fileName: 'third.pdf',
      author: null,
      readingProgress: 0,
      fingerprint: 'third',
    );
    for (final document in [first, second, third]) {
      documents.save(document);
    }
    documents.setManualTopics(first.id, {
      LibraryTopic.literature,
      LibraryTopic.society,
    }, analyzerVersion: 1);
    documents.setManualTopics(second.id, {
      LibraryTopic.literature,
    }, analyzerVersion: 1);

    final query = LibraryQuery(
      limit: 1,
      authors: const {'octavia butler'},
      topics: const {LibraryTopic.literature, LibraryTopic.politics},
    );
    final page = documents.page(query);
    final counts = documents.readingStateCounts(query);

    expect(page.documents, hasLength(1));
    expect(page.total, 2);
    expect(page.hasMore, isTrue);
    expect(counts[LibraryQueryReadingState.all], 2);
    expect(counts[LibraryQueryReadingState.inProgress], 1);
    expect(counts[LibraryQueryReadingState.completed], 1);
    expect(
      documents
          .page(
            query.copyWith(readingState: LibraryQueryReadingState.completed),
          )
          .total,
      1,
    );
    expect(
      documents.authorFacets(),
      contains(
        isA<LibraryFacet>()
            .having((facet) => facet.value, 'value', 'Octavia Butler')
            .having((facet) => facet.count, 'count', 2),
      ),
    );
    expect(documents.topicFacets()[LibraryTopic.literature], 2);
  });

  test('migration 7 to 8 creates enrichment tables transactionally', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-schema-7-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}state.sqlite3';
    final seeded = AppDatabase.open(
      path,
      AppLogger(minimumLevel: LogLevel.error),
    );
    seeded.close();
    final raw = sqlite3.open(path)
      ..execute('DROP TABLE document_topics')
      ..execute('DROP TABLE document_enrichment')
      ..execute('PRAGMA user_version = 7');
    raw.close();

    final migrated = AppDatabase.open(
      path,
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(migrated.close);
    final tables = migrated.database
        .select("SELECT name FROM sqlite_master WHERE type = 'table'")
        .map((row) => row['name'])
        .toSet();

    expect(
      migrated.database.select('PRAGMA user_version').first.columnAt(0),
      AppConfig.databaseVersion,
    );
    expect(tables, containsAll({'document_enrichment', 'document_topics'}));
    expect(migrated.integrityCheck(), 'ok');
    expect(
      directory.listSync().where(
        (entry) => entry.path.contains('pre-migration-v7'),
      ),
      isNotEmpty,
    );
  });

  test('recent documents are updated without duplicates', () {
    final original = _document();
    documents.save(original);
    documents.save(
      original.copyWith(
        lastOpenedAt: original.lastOpenedAt.add(const Duration(hours: 1)),
        lastPageIndex: 4,
        readingProgress: 0.4,
      ),
    );

    final recent = documents.recent();
    expect(recent, hasLength(1));
    expect(recent.single.lastPageIndex, 4);
    expect(recent.single.readingProgress, 0.4);
  });

  test(
    'duplicate merge keeps local relations and never touches PDF paths',
    () async {
      final existing = _document().copyWith(
        path: r'C:\Books\existing.pdf',
        normalizedPath: r'c:\books\existing.pdf',
        fileName: 'existing.pdf',
        title: 'Local title',
        fingerprint: 'same-fingerprint',
        isFavorite: true,
        libraryNote: 'Existing note',
      );
      final incoming = _document().copyWith(
        id: 'doc-2',
        path: r'D:\Books\incoming.pdf',
        normalizedPath: r'd:\books\incoming.pdf',
        fileName: 'incoming.pdf',
        title: 'Incoming title',
        fingerprint: 'same-fingerprint',
        lastPageIndex: 8,
        readingProgress: 0.8,
        libraryNote: 'Incoming note',
      );
      documents.save(existing);
      documents.save(incoming);
      BookmarkRepository(database).add(incoming.id, 8, label: 'Keep me');

      String? backupPath;
      final service = DuplicateResolutionService(
        database: database,
        documents: documents,
        createPreBackup: () async {
          backupPath = 'pre-merge.json';
          return backupPath!;
        },
      );
      final result = await service.resolve(
        existingId: existing.id,
        incomingId: incoming.id,
        action: DuplicateResolutionAction.merge,
      );

      expect(result.preBackupPath, 'pre-merge.json');
      expect(backupPath, isNotNull);
      expect(documents.byId(incoming.id), isNull);
      expect(documents.byId(existing.id)?.path, existing.path);
      expect(documents.byId(existing.id)?.readingProgress, 0.8);
      expect(documents.byId(existing.id)?.libraryNote, 'Existing note');
      expect(
        BookmarkRepository(database).forDocument(existing.id),
        hasLength(1),
      );
    },
  );

  test('library notes round-trip, can be cleared, and are searchable', () {
    final original = _document().copyWith(libraryNote: 'Project reading note');
    documents.save(original);

    expect(documents.byId(original.id)?.libraryNote, 'Project reading note');
    expect(
      documents.page(const LibraryQuery(text: 'reading note')).documents,
      hasLength(1),
    );

    documents.updateLibraryNote(original.id, '   ');
    expect(documents.byId(original.id)?.libraryNote, isNull);
  });

  test(
    'confirmed import queue survives a restart and reopens running work',
    () {
      final queueRepository = ImportQueueRepository(database);
      final queue = ImportQueue(maxPending: 4)
        ..enqueue(
          const ImportQueueItem(
            id: 'queued',
            path: r'C:\Books\queued.pdf',
            normalizedPath: r'c:\books\queued.pdf',
          ),
        );
      queue.claimNext();
      queueRepository.save(queue);

      final restored = queueRepository.load();
      expect(restored.items.single.id, 'queued');
      expect(restored.items.single.state, ImportWorkState.pending);
    },
  );

  test(
    'document identity follows a moved file and invalidates changed covers',
    () {
      final service = DocumentService(documents);
      final original = service.register(
        file: InspectedFile(
          path: r'C:\Books\book.pdf',
          normalizedPath: r'c:\books\book.pdf',
          fileName: 'book.pdf',
          fileSize: 1024,
          modifiedAt: DateTime.utc(2026, 7, 28),
          fingerprint: 'fingerprint-a',
        ),
        pageCount: 10,
      );
      documents.updateThumbnail(original.id, r'C:\cache\cover.png');

      final changed = service.register(
        file: InspectedFile(
          path: r'C:\Books\book.pdf',
          normalizedPath: r'c:\books\book.pdf',
          fileName: 'book.pdf',
          fileSize: 2048,
          modifiedAt: DateTime.utc(2026, 7, 29),
          fingerprint: 'fingerprint-b',
        ),
        pageCount: 12,
      );
      expect(changed.id, original.id);
      expect(changed.thumbnailPath, isNull);

      final moved = service.register(
        file: InspectedFile(
          path: r'D:\Moved\book.pdf',
          normalizedPath: r'd:\moved\book.pdf',
          fileName: 'book.pdf',
          fileSize: 2048,
          modifiedAt: DateTime.utc(2026, 7, 29),
          fingerprint: 'fingerprint-b',
        ),
        pageCount: 12,
      );
      expect(moved.id, original.id);
      expect(moved.path, r'D:\Moved\book.pdf');
      expect(documents.recent(), hasLength(1));
    },
  );

  test('document preferences survive a repository round trip', () {
    documents.save(_document());
    final repository = DocumentPreferencesRepository(database);
    final expected = DocumentPreferences(
      documentId: 'doc-1',
      viewMode: ReaderMode.book,
      readingTheme: ReaderTheme.sepia,
      zoomMode: ZoomMode.fitWidth,
      zoomLevel: 1.3,
      pageLayout: PageLayoutMode.facing,
      pageRotation: 90,
      horizontalMargin: 44,
      verticalMargin: 36,
      controlsAutoHide: false,
      scrollOffset: 0.25,
    );

    repository.save(expected);
    final actual = repository.load('doc-1');

    expect(repository.exists('doc-1'), isTrue);
    expect(actual.viewMode, ReaderMode.book);
    expect(actual.readingTheme, ReaderTheme.sepia);
    expect(actual.pageLayout, PageLayoutMode.facing);
    expect(actual.pageRotation, 90);
    expect(actual.horizontalMargin, 44);
  });

  test('bookmark creation, replacement and deletion are persisted', () {
    documents.save(_document());
    final repository = BookmarkRepository(database);

    final first = repository.add('doc-1', 3, label: 'Capítulo');
    repository.add('doc-1', 3, label: 'Capítulo revisado');
    expect(repository.forDocument('doc-1'), hasLength(1));
    expect(repository.atPage('doc-1', 3)?.label, 'Capítulo revisado');

    repository.remove(first.id);
    expect(repository.forDocument('doc-1'), isEmpty);
  });

  test('reading session stores bounded active time and final page', () {
    documents.save(_document());
    final repository = ReadingSessionRepository(database);
    final session = repository.start('doc-1', 1);
    repository.finish(id: session.id, endPage: 8, activeSeconds: 42);

    final row = database.database.select(
      'SELECT * FROM reading_sessions WHERE id = ?',
      [session.id],
    ).single;
    expect(row['end_page'], 8);
    expect(row['active_seconds'], 42);
    expect(row['ended_at'], isNotNull);
  });

  test('reading history is bounded, ordered and individually clearable', () {
    documents.save(_document());
    final repository = ReadingSessionRepository(database);
    for (var index = 0; index < 30; index++) {
      final session = repository.start('doc-1', index % 5);
      repository.finish(
        id: session.id,
        endPage: (index % 5) + 1,
        activeSeconds: index,
      );
    }
    repository.trimToLimit(25);
    final history = repository.history(limit: 100);
    expect(history, hasLength(25));
    expect(history.first.documentTitle, 'book.pdf');
    expect(
      history.first.session.startedAt.isAfter(history.last.session.startedAt),
      isTrue,
    );

    repository.delete(history.first.session.id);
    expect(repository.history(limit: 100), hasLength(24));
    repository.clear();
    expect(repository.history(), isEmpty);
  });
}

LibraryDocument _document() {
  final now = DateTime.utc(2026, 7, 28, 18);
  return LibraryDocument(
    id: 'doc-1',
    path: r'C:\Books\book.pdf',
    normalizedPath: r'c:\books\book.pdf',
    fileName: 'book.pdf',
    pageCount: 11,
    fileSize: 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint',
  );
}
