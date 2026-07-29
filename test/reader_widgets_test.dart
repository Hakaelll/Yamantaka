import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/platform/window_service.dart';
import 'package:yamantaka/features/bookmarks/application/bookmark_controller.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/reader/application/reader_controller.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/presentation/reader_screen.dart';
import 'package:yamantaka/features/search/application/search_controller.dart'
    as app_search;
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('reader presents a recoverable error state', (tester) async {
    final harness = await _ReaderHarness.create(ready: false);
    harness.controller
      ..status = ReaderStatus.failure
      ..error = const AppFailure(
        FailureKind.corruptPdf,
        'El PDF de prueba está dañado.',
      );

    await tester.pumpWidget(_app(harness));

    expect(find.text('El PDF de prueba está dañado.'), findsOneWidget);
    expect(find.text('Volver a la biblioteca'), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('reader shows navigation, progress, panels and six themes', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);

    await tester.pumpWidget(_app(harness));
    await tester.pump();

    expect(find.text('book.pdf'), findsOneWidget);
    expect(find.byKey(const Key('test-pdf-viewer')), findsOneWidget);
    expect(find.byKey(const Key('reader-progress')), findsOneWidget);
    expect(find.text('1 / 3'), findsOneWidget);

    await tester.tap(find.byTooltip('Más opciones'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 250));
    expect(find.text('Márgenes de lectura'), findsOneWidget);
    await tester.tap(find.text('Márgenes de lectura'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 250));
    expect(find.byType(Slider), findsOneWidget);
    await tester.tap(find.text('Aplicar'));
    await tester.pump();
    expect(harness.controller.preferences!.horizontalMargin, 28);

    await tester.tap(find.byTooltip('Página siguiente (Av Pág)'));
    await tester.pump();
    expect(harness.controller.currentPageIndex, 1);
    expect(find.text('2 / 3'), findsOneWidget);

    await tester.tap(find.byTooltip('Buscar (Ctrl+F)'));
    await tester.pump();
    expect(find.text('Buscar'), findsWidgets);
    expect(find.byTooltip('Cerrar panel'), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(find.byTooltip('Cerrar panel'), findsNothing);
    await tester.tap(find.byTooltip('Tema de lectura'));
    // A determinate progress bar keeps its theme animation scheduled, so this
    // test advances a bounded frame instead of waiting for global quiescence.
    await tester.pump(const Duration(milliseconds: 200));
    for (final label in [
      'Claro',
      'Marfil',
      'Sepia',
      'Gris',
      'Oscuro',
      'Negro',
    ]) {
      expect(find.text(label), findsOneWidget);
    }
    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('reader composes without overflow across all window classes', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);

    for (final size in const [
      Size(640, 720),
      Size(679, 720),
      Size(680, 720),
      Size(1099, 720),
      Size(1100, 720),
      Size(1440, 900),
    ]) {
      tester.view.physicalSize = size;
      await tester.pumpWidget(_app(harness));
      await tester.pump();
      expect(tester.takeException(), isNull, reason: 'viewport $size');
      expect(find.byKey(const Key('test-pdf-viewer')), findsOneWidget);
      expect(
        find.byType(SegmentedButton<ReaderMode>),
        size.width < 680 ? findsNothing : findsOneWidget,
      );
    }

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('facing flow is enabled only when both pages remain legible', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);
    final adapter = _RecordingViewerAdapter();
    harness.controller.preferences = harness.controller.preferences!.copyWith(
      pageLayout: PageLayoutMode.facing,
    );

    tester.view.physicalSize = const Size(1099, 800);
    await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.single);

    tester.view.physicalSize = const Size(1100, 800);
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.facing);

    tester.view.physicalSize = const Size(1100, 460);
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.single);

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('electronic ink profile forces a static single-page canvas', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);
    final adapter = _RecordingViewerAdapter();
    harness.controller.preferences = harness.controller.preferences!.copyWith(
      pageLayout: PageLayoutMode.facing,
      readingTheme: ReaderTheme.sepia,
    );

    await tester.pumpWidget(
      _app(
        harness,
        viewerAdapter: adapter,
        visualProfile: ReaderVisualProfile.electronicInk,
      ),
    );
    await tester.pump();

    expect(adapter.configuration?.flow, PdfViewFlow.single);
    expect(adapter.configuration?.reduceMotion, isTrue);
    expect(adapter.configuration?.pageShadow, isNull);
    expect(adapter.configuration?.backgroundColor, Colors.white);

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets(
    'side panel resizes with debounce and does not rebuild the PDF viewer',
    (tester) async {
      tester.view.physicalSize = const Size(1440, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final harness = await _ReaderHarness.create(ready: true);
      final adapter = _CountingViewerAdapter();
      double? persistedWidth;

      await tester.pumpWidget(
        _app(
          harness,
          viewerAdapter: adapter,
          initialPanelOpen: true,
          initialPanelSection: 'search',
          onPanelWidthChanged: (value) => persistedWidth = value,
        ),
      );
      await tester.pump();
      expect(adapter.buildCount, 1);
      expect(find.byKey(const Key('pdf-search-field')), findsOneWidget);
      expect(find.bySemanticsLabel('Ancho del panel lateral'), findsOneWidget);

      final handle = find.byKey(const Key('reader-panel-resize-handle'));
      await tester.drag(handle, const Offset(-96, 0));
      await tester.pump();
      final draggedWidth = tester
          .getSize(find.byKey(const Key('reader-side-panel')))
          .width;
      expect(draggedWidth, inInclusiveRange(380, 416));
      expect(adapter.buildCount, 1);
      expect(persistedWidth, isNull);
      await tester.pump(const Duration(milliseconds: 260));
      expect(persistedWidth, closeTo(draggedWidth, 1));

      await tester.tap(handle);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump();
      expect(
        tester.getSize(find.byKey(const Key('reader-side-panel'))).width,
        closeTo(draggedWidth + 16, 1),
      );

      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      expect(find.byKey(const Key('reader-side-panel')), findsNothing);

      await tester.tap(find.byTooltip('Miniaturas (Ctrl+J)'));
      await tester.pump();
      await tester.pump();
      expect(find.byKey(const Key('pdf-search-field')), findsNothing);
      expect(find.text('No disponible'), findsWidgets);
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      expect(find.byKey(const Key('reader-side-panel')), findsNothing);

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  testWidgets('a relevant key reveals hidden book controls and navigates', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);
    harness.controller.preferences = harness.controller.preferences!.copyWith(
      viewMode: ReaderMode.book,
      controlsAutoHide: true,
    );

    await tester.pumpWidget(_app(harness));
    await tester.pump(const Duration(seconds: 4));
    expect(find.byTooltip('Buscar (Ctrl+F)').hitTestable(), findsNothing);

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump();

    expect(harness.controller.currentPageIndex, 1);
    expect(find.byTooltip('Buscar (Ctrl+F)').hitTestable(), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  test(
    'viewer state feeds debounced zoom and relative position persistence',
    () async {
      final harness = await _ReaderHarness.create(ready: true);
      addTearDown(harness.dispose);

      harness.viewController.update(zoom: 1.75, scrollOffset: 0.42);

      expect(harness.controller.preferences!.zoomLevel, 1.75);
      expect(harness.controller.preferences!.scrollOffset, 0.42);
      harness.controller.setMargins(52);
      expect(harness.controller.preferences!.horizontalMargin, 52);
      expect(harness.controller.preferences!.verticalMargin, 52);
    },
  );
}

Widget _app(
  _ReaderHarness harness, {
  PdfViewerAdapter viewerAdapter = const _FakeViewerAdapter(),
  ReaderVisualProfile visualProfile = ReaderVisualProfile.standard,
  bool initialPanelOpen = false,
  String initialPanelSection = '',
  ValueChanged<double>? onPanelWidthChanged,
}) => MaterialApp(
  home: ReaderScreen(
    controller: harness.controller,
    engine: harness.engine,
    viewerAdapter: viewerAdapter,
    onBack: () async {},
    onOpenDocument: () async {},
    onOpenSettings: () {},
    reduceMotion: true,
    showProgress: true,
    readingDirection: ReadingDirection.leftToRight,
    visualProfile: visualProfile,
    initialPanelOpen: initialPanelOpen,
    initialPanelSection: initialPanelSection,
    onPanelWidthChanged: onPanelWidthChanged,
  ),
);

final class _ReaderHarness {
  _ReaderHarness({
    required this.controller,
    required this.engine,
    required this.viewController,
    required this.database,
    required this.bookmarks,
    required this.search,
  });

  final ReaderController controller;
  final _FakePdfEngine engine;
  final PdfViewController viewController;
  final AppDatabase database;
  final BookmarkController bookmarks;
  final app_search.SearchController search;

  static Future<_ReaderHarness> create({required bool ready}) async {
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final engine = _FakePdfEngine();
    final viewController = PdfViewController();
    final bookmarks = BookmarkController(BookmarkRepository(database));
    final search = app_search.SearchController(engine, viewController, logger);
    final directory = Directory.current;
    final thumbnails = Directory('${directory.path}/build/test-covers');
    final thumbnailService = ThumbnailService(
      engine: engine,
      paths: AppPaths(
        dataDirectory: directory,
        cacheDirectory: directory,
        thumbnailDirectory: thumbnails,
      ),
      documents: documents,
      logger: logger,
      maxCacheBytes: () => 64 * 1024 * 1024,
    );
    final controller = ReaderController(
      documentService: DocumentService(documents),
      documents: documents,
      preferences: DocumentPreferencesRepository(database),
      sessions: ReadingSessionRepository(database),
      engine: engine,
      thumbnails: thumbnailService,
      windowService: FakeWindowService(),
      logger: logger,
      bookmarks: bookmarks,
      search: search,
      viewController: viewController,
      shouldGenerateCovers: () => false,
      shouldRecordSessions: () => false,
      shouldRestorePosition: () => true,
      defaultPreferences: (id) => DocumentPreferences(documentId: id),
    );
    if (ready) {
      final now = DateTime.utc(2026, 7, 28);
      final document = LibraryDocument(
        id: 'doc-1',
        path: '${directory.path}/book.pdf',
        normalizedPath: '${directory.path}/book.pdf',
        fileName: 'book.pdf',
        pageCount: 3,
        fileSize: 100,
        fileModifiedAt: now,
        addedAt: now,
        lastOpenedAt: now,
        fingerprint: 'fingerprint',
      );
      documents.save(document);
      controller
        ..document = document
        ..openedDocument = const OpenedPdfDocument(
          documentId: 'pdf-1',
          path: 'book.pdf',
          pageCount: 3,
          pageSizes: [
            PdfPageSize(600, 800),
            PdfPageSize(600, 800),
            PdfPageSize(600, 800),
          ],
          isEncrypted: false,
        )
        ..preferences = const DocumentPreferences(documentId: 'doc-1')
        ..status = ReaderStatus.ready;
      bookmarks.load(document.id);
      viewController.attach(
        pageCount: 3,
        currentPageIndex: 0,
        zoom: 1,
        goToPage: (page) async {
          viewController.update(pageIndex: page);
          controller.onPageChanged(page);
        },
        zoomBy: (factor) async {
          viewController.update(zoom: viewController.zoom * factor);
        },
        fitWidth: () async => viewController.update(zoom: 1.2),
        fitPage: () async => viewController.update(zoom: 1),
      );
    }
    return _ReaderHarness(
      controller: controller,
      engine: engine,
      viewController: viewController,
      database: database,
      bookmarks: bookmarks,
      search: search,
    );
  }

  Future<void> dispose() async {
    await controller.closeDocument();
    controller.dispose();
    bookmarks.dispose();
    search.dispose();
    database.close();
  }
}

final class _FakeViewerAdapter implements PdfViewerAdapter {
  const _FakeViewerAdapter();

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) => const ColoredBox(key: Key('test-pdf-viewer'), color: Colors.white);
}

final class _RecordingViewerAdapter implements PdfViewerAdapter {
  PdfViewConfiguration? configuration;

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) {
    this.configuration = configuration;
    return const ColoredBox(key: Key('test-pdf-viewer'), color: Colors.white);
  }
}

final class _CountingViewerAdapter implements PdfViewerAdapter {
  int buildCount = 0;

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) {
    buildCount++;
    return const ColoredBox(key: Key('test-pdf-viewer'), color: Colors.white);
  }
}

final class _FakePdfEngine extends StubPdfDocumentEngine {
  @override
  Future<RenderedPdfPage> renderPage({
    required String documentId,
    required int pageIndex,
    required double scale,
    int rotation = 0,
    PdfRenderRegion? region,
    PdfOperationToken? cancellationToken,
  }) async {
    throw StateError('No bitmap in the widget harness.');
  }
}
