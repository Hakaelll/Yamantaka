import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/platform/window_service.dart';
import 'package:yamantaka/features/annotations/application/annotation_controller.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/bookmarks/application/bookmark_controller.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/reader/application/reader_controller.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/presentation/reader_screen.dart';
import 'package:yamantaka/features/navigation/application/navigation_history_controller.dart';
import 'package:yamantaka/features/search/application/search_controller.dart'
    as app_search;
import 'package:yamantaka/pdf_engine/adapters/pdfrx_document_engine.dart';
import 'package:yamantaka/pdf_engine/adapters/pdfrx_viewer_adapter.dart';
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

import '../test/support/pdf_fixture.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('opens, renders, searches and navigates a real local PDF', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1200, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await pdfrxFlutterInitialize();
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-integration-',
    );
    final pdf = File('${directory.path}${Platform.pathSeparator}fixture.pdf');
    await pdf.writeAsBytes(createTestPdf(includeThirdPage: true));
    final diagnosticLog = <String>[];
    final logger = AppLogger(
      minimumLevel: LogLevel.debug,
      sink: (level, message, error, stackTrace) {
        diagnosticLog.add(message);
      },
    );
    final engine = PdfrxDocumentEngine(logger);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final viewController = PdfViewController();
    final bookmarks = BookmarkController(BookmarkRepository(database));
    final annotations = AnnotationController(AnnotationRepository(database));
    final navigationHistory = NavigationHistoryController();
    final search = app_search.SearchController(engine, viewController, logger);
    final covers = Directory('${directory.path}${Platform.pathSeparator}covers')
      ..createSync();
    final reader = ReaderController(
      documentService: DocumentService(documents),
      documents: documents,
      preferences: DocumentPreferencesRepository(database),
      sessions: ReadingSessionRepository(database),
      engine: engine,
      thumbnails: ThumbnailService(
        engine: engine,
        paths: AppPaths(
          dataDirectory: directory,
          cacheDirectory: directory,
          thumbnailDirectory: covers,
        ),
        documents: documents,
        logger: logger,
        maxCacheBytes: () => 64 * 1024 * 1024,
      ),
      windowService: FakeWindowService(),
      logger: logger,
      annotations: annotations,
      bookmarks: bookmarks,
      navigationHistory: navigationHistory,
      search: search,
      viewController: viewController,
      shouldGenerateCovers: () => false,
      shouldRecordSessions: () => false,
      shouldRestorePosition: () => true,
      defaultPreferences: (id) => DocumentPreferences(documentId: id),
    );
    addTearDown(() async {
      await _bounded(reader.closeDocument(), 'reader teardown');
      reader.dispose();
      bookmarks.dispose();
      annotations.dispose();
      navigationHistory.dispose();
      search.dispose();
      await _bounded(engine.dispose(), 'engine teardown');
      database.close();
      await directory.delete(recursive: true);
    });

    expect(
      await _bounded(reader.openPath(pdf.path), 'initial PDF open'),
      isTrue,
    );
    final opened = reader.openedDocument!;
    expect(opened.pageCount, 3);
    expect(
      await _bounded(
        engine.extractPageText(documentId: opened.documentId, pageIndex: 0),
        'text extraction',
      ),
      contains('Yamantaka page one'),
    );
    final rendered = await _bounded(
      engine.renderPage(
        documentId: opened.documentId,
        pageIndex: 0,
        scale: 0.5,
      ),
      'native ui.Image render',
      diagnostics: diagnosticLog,
    );
    expect(rendered.image.width, greaterThan(0));
    final rgba = await _bounded(
      rendered.image.toByteData(format: ui.ImageByteFormat.rawRgba),
      'ui.Image pixel readback',
    );
    expect(rgba, isNotNull);
    var nonWhitePixels = 0;
    for (var offset = 0; offset < rgba!.lengthInBytes; offset += 4) {
      if (rgba.getUint8(offset) < 245 ||
          rgba.getUint8(offset + 1) < 245 ||
          rgba.getUint8(offset + 2) < 245) {
        nonWhitePixels++;
      }
    }
    expect(nonWhitePixels, greaterThan(100));

    final adapter = PdfrxViewerAdapter(engine);
    reader.preferences = reader.preferences!.copyWith(
      pageLayout: PageLayoutMode.facing,
    );
    await tester.pumpWidget(
      RepaintBoundary(
        key: const Key('native-viewer-boundary'),
        child: MaterialApp(
          debugShowCheckedModeBanner: false,
          theme: AppTheme.light(),
          home: ReaderScreen(
            controller: reader,
            engine: engine,
            viewerAdapter: adapter,
            onBack: () async {},
            onOpenDocument: () async {},
            onOpenSettings: () {},
            reduceMotion: true,
            showProgress: true,
            readingDirection: ReadingDirection.leftToRight,
          ),
        ),
      ),
    );
    await _pumpUntil(
      tester,
      () => viewController.isReady,
      reason: 'the native PDF viewer to attach',
    );
    expect(viewController.isReady, isTrue);
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 600)),
    );
    await tester.pump();
    await _verifyReaderPaint(tester);

    expect(find.byKey(const Key('reader-top-bar')), findsOneWidget);
    expect(find.byKey(const Key('reader-bottom-controls')), findsOneWidget);
    expect(find.byKey(const Key('reader-progress')), findsOneWidget);
    expect(find.text('Documento'), findsOneWidget);

    final actualDocument = reader.document!;
    reader.document = actualDocument.copyWith(
      path: r'C:\Fixtures\fixture.pdf',
      normalizedPath: r'c:\fixtures\fixture.pdf',
    );
    await tester.pump();
    await tester.tap(find.byTooltip('Más opciones'));
    await tester.pump(const Duration(milliseconds: 250));
    await tester.tap(find.text('Información del documento'));
    await tester.pump(const Duration(milliseconds: 250));
    expect(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.text('Información del documento'),
      ),
      findsOneWidget,
    );
    expect(find.text('Copiar ruta'), findsOneWidget);
    await _captureReaderState(tester, suffix: '-info');
    await tester.tap(find.text('Cerrar'));
    await tester.pump();
    reader.document = actualDocument;
    await tester.pump();

    await tester.tap(find.byTooltip('Buscar (Ctrl+F)'));
    await tester.pump();
    expect(find.byKey(const Key('pdf-search-field')), findsOneWidget);
    await tester.enterText(
      find.byKey(const Key('pdf-search-field')),
      'page two',
    );
    await tester.testTextInput.receiveAction(TextInputAction.search);
    await _pumpUntil(
      tester,
      () => !search.isSearching && search.matches.isNotEmpty,
      reason: 'the reader search panel to return a match',
    );
    expect(search.matches.single.pageIndex, 1);
    expect(find.text('1 de 1'), findsOneWidget);
    await _captureReaderState(tester, suffix: '-search');
    await tester.drag(
      find.byKey(const Key('reader-panel-resize-handle')),
      const Offset(-80, 0),
    );
    await tester.pump();
    await _captureReaderState(tester, suffix: '-panel-resized');

    await tester.tap(find.byTooltip('Cerrar panel'));
    await tester.pump();
    await _captureReaderState(tester, suffix: '-facing');
    annotations.create(
      pageIndex: reader.currentPageIndex,
      type: AnnotationType.pageComment,
      color: AnnotationColor.blue,
      note: 'Nota local de integración',
      tags: const ['estudio'],
    );
    await tester.tap(find.byTooltip('Anotaciones'));
    await tester.pump();
    expect(find.text('Nota local de integración'), findsOneWidget);
    expect(find.byKey(const Key('annotation-add-page-note')), findsOneWidget);
    await _captureReaderState(tester, suffix: '-annotations');
    await tester.tap(find.byTooltip('Cerrar panel'));
    await tester.pump();
    await tester.tap(find.byTooltip('Paleta de comandos (Ctrl+K)'));
    await tester.pump();
    expect(find.byKey(const Key('command-palette-search')), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 250));
    await _captureReaderState(tester, suffix: '-palette');
    await tester.tapAt(const Offset(8, 8));
    await tester.pump(const Duration(milliseconds: 250));
    expect(find.byKey(const Key('command-palette-search')), findsNothing);

    final bookNavigation = viewController.goToPage(0);
    await _pumpUntil(
      tester,
      () => reader.currentPageIndex == 0,
      reason: 'navigation to return to the cover for book mode',
    );
    await _bounded(bookNavigation, 'book-mode cover navigation');
    reader.preferences = reader.preferences!.copyWith(controlsAutoHide: true);
    reader.setReaderMode(ReaderMode.book);
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    await tester.pump(const Duration(seconds: 4));
    expect(find.byKey(const Key('reader-top-bar')).hitTestable(), findsNothing);
    await _captureReaderState(tester, suffix: '-book');
    reader.setReaderMode(ReaderMode.document);
    await tester.pump();

    final initialZoom = viewController.zoom;
    final zoom = viewController.zoomBy(1.1);
    await _pumpUntil(
      tester,
      () => viewController.zoom > initialZoom + 0.001,
      reason: 'zoom to increase',
    );
    await _bounded(zoom, 'zoom completion');

    var navigation = viewController.goToPage(1);
    await _pumpUntil(
      tester,
      () => reader.currentPageIndex == 1,
      reason: 'navigation to reach the second page',
    );
    await _bounded(navigation, 'navigation completion');
    expect(reader.currentPageIndex, 1);

    navigation = viewController.goToPage(0);
    await _pumpUntil(
      tester,
      () => reader.currentPageIndex == 0,
      reason: 'backward navigation to reach the first page',
    );
    await _bounded(navigation, 'backward navigation completion');
    expect(reader.currentPageIndex, 0);

    navigation = viewController.goToPage(1);
    await _pumpUntil(
      tester,
      () => reader.currentPageIndex == 1,
      reason: 'forward navigation to return to the second page',
    );
    await _bounded(navigation, 'forward navigation completion');
    expect(reader.currentPageIndex, 1);
    reader.toggleBookmark(label: 'Retomar aquí');

    final matches = await _bounded(
      engine.searchText(documentId: opened.documentId, query: 'page two'),
      'full-text search',
    );
    expect(matches.single.pageIndex, 1);

    await tester.pumpWidget(const SizedBox.shrink());
    await _bounded(reader.closeDocument(), 'reader close');
    expect(await _bounded(reader.openPath(pdf.path), 'PDF reopen'), isTrue);
    expect(reader.currentPageIndex, 1);
    expect(reader.bookmarks.containsPage(1), isTrue);
  });
}

Future<void> _verifyReaderPaint(WidgetTester tester) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(const Key('native-viewer-boundary')),
  );
  final image = await boundary.toImage();
  try {
    final rgba = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    expect(rgba, isNotNull);
    var darkPixels = 0;
    for (var offset = 0; offset < rgba!.lengthInBytes; offset += 4) {
      if (rgba.getUint8(offset) < 100 &&
          rgba.getUint8(offset + 1) < 100 &&
          rgba.getUint8(offset + 2) < 100) {
        darkPixels++;
      }
    }
    expect(darkPixels, greaterThan(100));
    await _writeCapture(image);
  } finally {
    image.dispose();
  }
}

Future<void> _captureReaderState(
  WidgetTester tester, {
  required String suffix,
}) async {
  const capturePath = String.fromEnvironment('YAMANTAKA_CAPTURE_PATH');
  if (capturePath.isEmpty) return;
  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(const Key('native-viewer-boundary')),
  );
  final image = await boundary.toImage();
  try {
    await _writeCapture(image, suffix: suffix);
  } finally {
    image.dispose();
  }
}

Future<void> _writeCapture(ui.Image image, {String suffix = ''}) async {
  const capturePath = String.fromEnvironment('YAMANTAKA_CAPTURE_PATH');
  if (capturePath.isEmpty) return;
  final path = suffix.isEmpty
      ? capturePath
      : capturePath.replaceFirst(RegExp(r'\.png$'), '$suffix.png');
  final png = await image.toByteData(format: ui.ImageByteFormat.png);
  await File(path).writeAsBytes(png!.buffer.asUint8List());
}

Future<void> _pumpUntil(
  WidgetTester tester,
  bool Function() condition, {
  required String reason,
  Duration timeout = const Duration(seconds: 15),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (!condition() && DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  if (!condition()) {
    fail('Timed out waiting for $reason.');
  }
}

Future<T> _bounded<T>(
  Future<T> operation,
  String reason, {
  Duration timeout = const Duration(seconds: 30),
  Iterable<String>? diagnostics,
}) => operation.timeout(
  timeout,
  onTimeout: () {
    final entries = diagnostics?.join(' -> ');
    final detail = entries == null || entries.isEmpty ? '' : ' [$entries]';
    throw TimeoutException('Timed out during $reason.$detail', timeout);
  },
);
