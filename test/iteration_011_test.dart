import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/performance/local_performance_monitor.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/domain/reading_session.dart';
import 'package:yamantaka/features/search/application/global_search_controller.dart';
import 'package:yamantaka/features/search/domain/global_search_models.dart';
import 'package:yamantaka/features/workspaces/application/document_tabs_controller.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';
import 'package:yamantaka/storage/repositories/organization_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

void main() {
  group('0.11 reading continuity', () {
    test(
      'rich sessions, recently closed documents and local goals round trip',
      () {
        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final documents = DocumentRepository(database);
        final document = _document('one');
        documents.save(document);
        final repository = ReadingSessionRepository(database);
        final session = repository.start(document.id, 3);
        repository.finish(
          id: session.id,
          endPage: 8,
          activeSeconds: 42,
          context: const ReaderSessionContext(
            zoomLevel: 1.35,
            pageLayout: 'continuous',
            readingMode: 'document',
            readingTheme: 'sepia',
            scrollOffset: 0.42,
            panelSection: 'annotations',
          ),
        );

        final restored = repository.history().single.session;
        expect(restored.endPage, 8);
        expect(restored.context?.zoomLevel, 1.35);
        expect(restored.context?.panelSection, 'annotations');

        final closed = RecentlyClosedDocument(
          documentId: document.id,
          path: document.path,
          title: document.effectiveTitle,
          pageIndex: 8,
          zoomLevel: 1.35,
          pageLayout: 'continuous',
          readingMode: 'document',
          readingTheme: 'sepia',
          scrollOffset: 0.42,
          panelSection: 'annotations',
          closedAt: DateTime.utc(2026, 8, 27),
          isModified: true,
        );
        repository
          ..rememberClosed(closed)
          ..rememberClosed(closed);
        expect(repository.recentlyClosed(), hasLength(1));
        expect(repository.recentlyClosed().single.isModified, isTrue);

        repository.setGoal(
          document.id,
          const ReadingGoal(pages: 25, minutes: 30),
        );
        expect(repository.goalFor(document.id)?.pages, 25);
        expect(repository.goalFor(document.id)?.minutes, 30);
        repository.setGoal(document.id, const ReadingGoal());
        expect(repository.goalFor(document.id), isNull);
      },
    );

    test('at most three document tabs can be pinned', () {
      final tabs = DocumentTabsController(maxTabs: 6);
      addTearDown(tabs.dispose);
      final opened = [for (var i = 0; i < 4; i++) tabs.open(_document('$i'))];
      for (final tab in opened) {
        tabs.togglePinned(tab.id);
      }

      expect(tabs.tabs.where((tab) => tab.isPinned), hasLength(3));
      expect(tabs.tabs.last.isPinned, isFalse);
    });
  });

  group('0.11 scalable library and index', () {
    test('organization memberships load in two bounded set queries', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final first = _document('one');
      final second = _document('two');
      documents
        ..save(first)
        ..save(second);
      final organization = OrganizationRepository(database);
      final collection = organization.createCollection('Estudio');
      organization
        ..addDocument(collection.id, first.id)
        ..addDocument(collection.id, second.id);
      final tag = database.database.select('SELECT id FROM tags LIMIT 1');
      if (tag.isEmpty) {
        database.database.execute(
          "INSERT INTO tags(id, name, normalized_name, color, created_at, updated_at) "
          "VALUES ('t', 'Clave', 'clave', 'blue', ?, ?)",
          [
            DateTime.utc(2026, 8, 27).toIso8601String(),
            DateTime.utc(2026, 8, 27).toIso8601String(),
          ],
        );
      }
      organization.assignTagToDocument('t', second.id);

      expect(organization.collectionMemberships()[collection.id], {
        first.id,
        second.id,
      });
      expect(organization.tagMemberships()['t'], {second.id});
    });

    test('safe indexing cursor resumes without discarding completed rows', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final document = _document('resume');
      documents.save(document);
      final repository = GlobalSearchRepository(database);
      repository
        ..beginDocument(document)
        ..addPage(document: document, pageIndex: 5, text: 'needle retained')
        ..updateCursor(document.id, 2);

      expect(repository.resumableCursor(document), 2);
      expect(repository.indexedPageIndexes(document.id), contains(5));
      repository.beginDocument(document, resume: true);
      expect(repository.search('needle').single.pageIndex, 5);

      final changed = document.copyWith(fingerprint: 'changed');
      repository.beginDocument(changed, resume: true);
      expect(repository.resumableCursor(changed), 0);
      expect(repository.search('needle'), isEmpty);
    });

    test(
      'search metrics distinguish cold, warm, filtered and cancelled work',
      () async {
        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final documents = DocumentRepository(database);
        final document = _document('metrics');
        documents.save(document);
        final repository = GlobalSearchRepository(database)
          ..beginDocument(document)
          ..addPage(document: document, pageIndex: 0, text: 'local metric')
          ..finishDocument(document.id, hasExtractableText: true);
        final metrics = LocalPerformanceMonitor();
        final controller = GlobalLibrarySearchController(repository, metrics);
        addTearDown(controller.dispose);

        await controller.search('local');
        await controller.search('metric');
        controller.toggleKind(GlobalSearchKind.pageText);
        await Future<void>.delayed(const Duration(milliseconds: 220));
        final pending = controller.search('cancel me');
        await Future<void>.delayed(const Duration(milliseconds: 10));
        controller.cancel();
        await pending;

        final durations =
            metrics.toJson()['durations']! as Map<String, Object?>;
        expect(durations, contains('global_search_cold'));
        expect(durations, contains('global_search_warm'));
        expect(durations, contains('global_search_filtered'));
        expect(durations, contains('global_search_cancelled'));
      },
    );

    test(
      'overlapping local queries only publish the newest generation',
      () async {
        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final document = _document('query-generation');
        DocumentRepository(database).save(document);
        final repository = GlobalSearchRepository(database)
          ..beginDocument(document)
          ..addPage(document: document, pageIndex: 0, text: 'primera segunda')
          ..finishDocument(document.id, hasExtractableText: true);
        final controller = GlobalLibrarySearchController(repository);
        addTearDown(controller.dispose);

        final first = controller.search('primera');
        final second = controller.search('segunda');
        await Future.wait([first, second]);

        expect(controller.query, 'segunda');
        expect(controller.results, isNotEmpty);
        expect(
          controller.results.every(
            (result) => result.snippet.contains('segunda'),
          ),
          isTrue,
        );
        expect(controller.state, GlobalSearchState.results);
      },
    );
  });
}

LibraryDocument _document(String id) {
  final now = DateTime.utc(2026, 8, 27);
  return LibraryDocument(
    id: id,
    path: 'C:\\library\\$id.pdf',
    normalizedPath: 'c:\\library\\$id.pdf',
    fileName: '$id.pdf',
    title: 'Documento $id',
    pageCount: 10,
    fileSize: 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint-$id',
    preferredViewMode: ReaderMode.document,
    preferredTheme: ReaderTheme.ivory,
  );
}
