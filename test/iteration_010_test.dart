import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/domain/page_destination.dart';
import 'package:yamantaka/features/search/application/global_search_controller.dart';
import 'package:yamantaka/features/search/domain/global_search_models.dart';
import 'package:yamantaka/features/workspaces/application/document_tabs_controller.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';

void main() {
  group('0.10 page destination', () {
    test('accepts page, percentage and localized boundaries', () {
      expect(PageDestination.parse('24', pageCount: 80).page, 24);
      expect(PageDestination.parse('50%', pageCount: 81).page, 41);
      expect(PageDestination.parse('inicio', pageCount: 80).page, 1);
      expect(PageDestination.parse('last', pageCount: 80).page, 80);
    });

    test('keeps invalid input in the editor with an explicit reason', () {
      expect(
        PageDestination.parse('81', pageCount: 80).error,
        PageDestinationError.outOfRange,
      );
      expect(
        PageDestination.parse('chapter', pageCount: 80).error,
        PageDestinationError.invalidFormat,
      );
    });
  });

  test('tab switch order wraps in both directions without changing state', () {
    final tabs = DocumentTabsController(maxTabs: 4, maxWarmTabs: 1);
    final first = tabs.open(_document(id: 'a', title: 'A'));
    final second = tabs.open(_document(id: 'b', title: 'B'));

    expect(tabs.relativeTab(reverse: false)?.id, first.id);
    expect(tabs.relativeTab(reverse: true)?.id, first.id);
    expect(tabs.activeTab?.id, second.id);
    expect(tabs.activeTab?.resourceState.name, 'active');
    expect(tabs.tabs.first.resourceState.name, 'warm');
  });

  test('a forced duplicate tab preserves independent reader presentation', () {
    final tabs = DocumentTabsController(maxTabs: 4, maxWarmTabs: 1);
    final document = _document(id: 'shared', title: 'Shared document');
    final first = tabs.open(document);
    tabs
      ..updateActiveNavigation(
        pageIndex: 36,
        zoomLevel: 1.65,
        pageLayout: 'continuous',
        readingMode: 'document',
        scrollOffset: 0.42,
        readingTheme: 'sepia',
      )
      ..updateActivePanel('annotations');

    final second = tabs.open(document, forceNew: true);
    expect(second.id, isNot(first.id));
    expect(tabs.tabs, hasLength(2));
    tabs
      ..updateActiveNavigation(
        pageIndex: 4,
        zoomLevel: 1.1,
        pageLayout: 'single',
        readingMode: 'book',
        scrollOffset: 0.08,
        readingTheme: 'dark',
      )
      ..updateActivePanel('bookmarks')
      ..activate(first.id);

    expect(tabs.activeTab?.pageIndex, 36);
    expect(tabs.activeTab?.zoomLevel, 1.65);
    expect(tabs.activeTab?.pageLayout, 'continuous');
    expect(tabs.activeTab?.scrollOffset, 0.42);
    expect(tabs.activeTab?.readingTheme, 'sepia');
    expect(tabs.activeTab?.panelSection, 'annotations');
    expect(
      tabs.tabs.where((tab) => tab.resourceState.name == 'warm'),
      hasLength(1),
    );
  });

  test('drop filtering accepts one or many unique PDFs only', () {
    expect(acceptedDroppedPdfPaths(['one.PDF']), ['one.PDF']);
    expect(
      acceptedDroppedPdfPaths([
        r'C:\books\one.pdf',
        r'C:\books\two.PDF',
        r'C:\books\one.PDF',
        r'C:\books\cover.png',
      ]),
      [r'C:\books\one.pdf', r'C:\books\two.PDF'],
    );
  });

  group('0.10 global search', () {
    late AppDatabase database;
    late DocumentRepository documents;
    late GlobalSearchRepository repository;

    setUp(() {
      database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
      documents = DocumentRepository(database);
      repository = GlobalSearchRepository(database);
    });

    tearDown(() => database.close());

    test('filters bookmark and page-text sources in SQLite', () {
      final document = _document(id: 'atlas', title: 'Atlas local');
      documents.save(document);
      repository
        ..beginDocument(document)
        ..addPage(
          document: document,
          pageIndex: 6,
          text: 'orogenesis continental',
        );
      BookmarkRepository(
        database,
      ).add(document.id, 4, label: 'tesis principal');
      repository.refreshBookmarkEntries();

      final bookmarks = repository.search(
        'tesis',
        kinds: {GlobalSearchKind.bookmark},
      );
      final pages = repository.search(
        'orogenesis',
        kinds: {GlobalSearchKind.pageText},
      );
      final excluded = repository.search(
        'orogenesis',
        kinds: {GlobalSearchKind.metadata},
      );

      expect(bookmarks.single.kind, GlobalSearchKind.bookmark);
      expect(bookmarks.single.pageIndex, 4);
      expect(pages.single.pageIndex, 6);
      expect(excluded, isEmpty);
    });

    test('explicit cancellation wins over a debounced query', () async {
      final document = _document(id: 'cancel', title: 'Atlas cancelable');
      documents.save(document);
      repository.beginDocument(document);
      final controller = GlobalLibrarySearchController(repository);

      final pending = controller.search('atlas');
      expect(controller.state, GlobalSearchState.searching);
      controller.cancel();
      await pending;

      expect(controller.state, GlobalSearchState.cancelled);
      expect(controller.results, isEmpty);
      controller.dispose();
    });

    test('ranking uses recent access only as a deterministic tie-breaker', () {
      final older = _document(
        id: 'atlas-older',
        title: 'Atlas local',
        lastOpenedAt: DateTime.utc(2026, 8, 20),
      );
      final recent = _document(
        id: 'atlas-recent',
        title: 'Atlas local',
        lastOpenedAt: DateTime.utc(2026, 8, 26),
      );
      documents
        ..save(older)
        ..save(recent);
      repository
        ..beginDocument(older)
        ..beginDocument(recent);

      final first = repository.search(
        'atlas',
        kinds: {GlobalSearchKind.metadata},
      );
      final repeated = repository.search(
        'atlas',
        kinds: {GlobalSearchKind.metadata},
      );

      expect(first.map((result) => result.documentId), [
        'atlas-recent',
        'atlas-older',
      ]);
      expect(
        repeated.map((result) => result.documentId),
        first.map((result) => result.documentId),
      );
    });
  });
}

LibraryDocument _document({
  required String id,
  required String title,
  DateTime? lastOpenedAt,
}) {
  final now = lastOpenedAt ?? DateTime.utc(2026, 8, 26);
  return LibraryDocument(
    id: id,
    path: '$id.pdf',
    normalizedPath: '$id.pdf',
    fileName: '$id.pdf',
    title: title,
    pageCount: 80,
    fileSize: 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    lastPageIndex: 19,
    readingProgress: 0.25,
    fingerprint: 'fingerprint-$id',
  );
}
