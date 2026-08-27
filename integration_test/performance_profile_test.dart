import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:yamantaka/app/theme/app_density.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/platform/window_service.dart';
import 'package:yamantaka/features/bookmarks/application/bookmark_controller.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/application/reader_controller.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/presentation/reader_screen.dart';
import 'package:yamantaka/features/search/application/search_controller.dart'
    as app_search;
import 'package:yamantaka/pdf_engine/adapters/pdfrx_document_engine.dart';
import 'package:yamantaka/pdf_engine/adapters/pdfrx_viewer_adapter.dart';
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

import '../test/support/pdf_fixture.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('profiles the repeatable library and 5000-page workflow', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1.25;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await pdfrxFlutterInitialize();
    final root = await Directory.systemTemp.createTemp(
      'yamantaka-performance-',
    );
    final cover = File('${root.path}/cover.png');
    await cover.writeAsBytes(
      base64Decode(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNgYAAAAAMAASsJTYQAAAAASUVORK5CYII=',
      ),
    );
    final longPdf = File('${root.path}/long.pdf');
    await longPdf.writeAsBytes(createPdfWithPageCount(100));
    final hugePdf = File('${root.path}/huge.pdf');
    await hugePdf.writeAsBytes(createPdfWithPageCount(5000));
    final heavyPdf = File('${root.path}/heavy-vector.pdf');
    await heavyPdf.writeAsBytes(createHeavyVectorPdf());
    final logger = AppLogger(
      minimumLevel: LogLevel.error,
      sink: (_, _, _, _) {},
    );
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final now = DateTime.utc(2026, 7, 29);
    for (var index = 0; index < 40; index++) {
      final path = '${root.path}/fixture-$index.pdf';
      await File(path).writeAsBytes(createPdfWithPageCount(2 + index % 4));
      final fileStat = await File(path).stat();
      documents.save(
        LibraryDocument(
          id: 'fixture-$index',
          path: path,
          normalizedPath: path.toLowerCase(),
          fileName: 'fixture-$index.pdf',
          title: 'Fixture ${index.toString().padLeft(2, '0')}',
          pageCount: 2 + index % 4,
          fileSize: File(path).lengthSync(),
          fileModifiedAt: fileStat.modified,
          addedAt: now,
          lastOpenedAt: now.subtract(Duration(minutes: index)),
          lastPageIndex: index % 3,
          readingProgress: switch (index % 3) {
            0 => 0,
            1 => 0.5,
            _ => 1,
          },
          fingerprint: 'fixture-$index',
          thumbnailPath: cover.path,
        ),
      );
    }
    final library = LibraryController(documents, logger);
    await library.load();
    final paths = AppPaths(
      dataDirectory: root,
      cacheDirectory: root,
      thumbnailDirectory: Directory('${root.path}/covers')
        ..createSync(recursive: true),
    );
    final engine = PdfrxDocumentEngine(logger);
    final viewController = PdfViewController();
    final bookmarks = BookmarkController(BookmarkRepository(database));
    final search = app_search.SearchController(engine, viewController, logger);
    final reader = ReaderController(
      documentService: DocumentService(documents),
      documents: documents,
      preferences: DocumentPreferencesRepository(database),
      sessions: ReadingSessionRepository(database),
      engine: engine,
      thumbnails: ThumbnailService(
        engine: engine,
        paths: paths,
        documents: documents,
        logger: logger,
        maxCacheBytes: () => 96 * 1024 * 1024,
      ),
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
    addTearDown(() async {
      await reader.closeDocument();
      reader.dispose();
      library.dispose();
      bookmarks.dispose();
      search.dispose();
      await engine.dispose();
      database.close();
      await root.delete(recursive: true);
    });

    Widget libraryApp(AppInterfaceDensity density) => MaterialApp(
      theme: AppTheme.light(density: density),
      home: LibraryScreen(
        controller: library,
        onOpenPath: (_) async {},
        onPickDocument: () async {},
        onOpenSettings: () {},
        showRecents: true,
      ),
    );

    await tester.pumpWidget(libraryApp(AppInterfaceDensity.comfortable));
    await tester.pumpAndSettle();
    final baselineRss = ProcessInfo.currentRss;
    final libraryDurations = <int>[];
    for (var run = 0; run < 5; run++) {
      await binding.watchPerformance(() async {
        final stopwatch = Stopwatch()..start();
        if (find.byKey(const Key('library-filter')).evaluate().isEmpty) {
          await tester.tap(find.byKey(const Key('library-search-button')));
          await tester.pump();
        }
        await tester.enterText(
          find.byKey(const Key('library-filter')),
          'Fixture 3',
        );
        await tester.pump();
        await tester.tap(find.byKey(const Key('library-filter-clear')));
        await tester.pumpAndSettle();
        await tester.pumpWidget(
          libraryApp(
            run.isEven
                ? AppInterfaceDensity.compact
                : AppInterfaceDensity.comfortable,
          ),
        );
        await tester.pump();
        stopwatch.stop();
        libraryDurations.add(stopwatch.elapsedMicroseconds);
      }, reportKey: 'library_run_$run');
    }

    final openDurations = <int>[];
    final heavyRenderDurations = <int>[];
    final heavyCloseRss = <int>[];
    final warmup = await engine.openDocument(heavyPdf.path);
    await engine.renderPage(
      documentId: warmup.documentId,
      pageIndex: 0,
      scale: 1,
    );
    await engine.closeDocument(warmup.documentId);
    engine.clearCaches();
    var peakRss = ProcessInfo.currentRss;
    for (var run = 0; run < 15; run++) {
      final openWatch = Stopwatch()..start();
      final openedHeavy = await engine.openDocument(heavyPdf.path);
      openWatch.stop();
      openDurations.add(openWatch.elapsedMicroseconds);

      final renderWatch = Stopwatch()..start();
      final firstRender = engine.renderPage(
        documentId: openedHeavy.documentId,
        pageIndex: run % openedHeavy.pageCount,
        scale: 1,
      );
      final duplicateRender = engine.renderPage(
        documentId: openedHeavy.documentId,
        pageIndex: run % openedHeavy.pageCount,
        scale: 1,
      );
      await Future.wait([firstRender, duplicateRender]);
      await engine.renderPage(
        documentId: openedHeavy.documentId,
        pageIndex: run % openedHeavy.pageCount,
        scale: 1,
      );
      renderWatch.stop();
      heavyRenderDurations.add(renderWatch.elapsedMicroseconds);
      peakRss = peakRss > ProcessInfo.currentRss
          ? peakRss
          : ProcessInfo.currentRss;
      await engine.closeDocument(openedHeavy.documentId);
      heavyCloseRss.add(ProcessInfo.currentRss);
    }
    expect(engine.diagnostics.openDocuments, 0);
    expect(engine.diagnostics.pendingRenders, 0);
    expect(engine.diagnostics.cachedBytes, 0);
    final heavyCloseGrowth = heavyCloseRss.last - heavyCloseRss.first;
    expect(heavyCloseGrowth, lessThan(16 * 1024 * 1024));

    final firstPage100Watch = Stopwatch()..start();
    expect(await reader.openPath(longPdf.path), isTrue);
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: ReaderScreen(
          controller: reader,
          engine: engine,
          viewerAdapter: PdfrxViewerAdapter(engine),
          onBack: () async {},
          onOpenDocument: () async {},
          onOpenSettings: () {},
          reduceMotion: true,
          showProgress: true,
          readingDirection: ReadingDirection.leftToRight,
        ),
      ),
    );
    await _pumpUntil(tester, () => viewController.isReady);
    firstPage100Watch.stop();
    peakRss = peakRss > ProcessInfo.currentRss
        ? peakRss
        : ProcessInfo.currentRss;
    final readerDurations = <int>[];
    for (var run = 0; run < 5; run++) {
      await binding.watchPerformance(() async {
        final stopwatch = Stopwatch()..start();
        await tester.tap(find.byTooltip('Buscar en el documento (Ctrl+F)'));
        await tester.pump();
        await tester.drag(
          find.byKey(const Key('reader-panel-resize-handle')),
          const Offset(-64, 0),
        );
        await tester.pump();
        await tester.tap(find.byTooltip('Cerrar panel'));
        await tester.pump();
        await viewController.zoomBy(run.isEven ? 1.04 : 1 / 1.04);
        await tester.pump();
        await reader.goToNextReadingUnit();
        await tester.pump();
        stopwatch.stop();
        readerDurations.add(stopwatch.elapsedMicroseconds);
      }, reportKey: 'reader_run_$run');
      peakRss = peakRss > ProcessInfo.currentRss
          ? peakRss
          : ProcessInfo.currentRss;
    }

    await reader.closeDocument();
    final firstPage5000Watch = Stopwatch()..start();
    expect(await reader.openPath(hugePdf.path), isTrue);
    await tester.pump();
    await _pumpUntil(tester, () => viewController.isReady);
    firstPage5000Watch.stop();
    peakRss = peakRss > ProcessInfo.currentRss
        ? peakRss
        : ProcessInfo.currentRss;

    final layoutDurations = <int>[];
    for (final layout in [
      PageLayoutMode.continuous,
      PageLayoutMode.facing,
      PageLayoutMode.single,
      PageLayoutMode.continuous,
    ]) {
      final stopwatch = Stopwatch()..start();
      reader.setPageLayout(layout);
      await tester.pump();
      await _pumpUntil(tester, () => viewController.isReady);
      stopwatch.stop();
      layoutDurations.add(stopwatch.elapsedMicroseconds);
    }

    final continuousNavigationDurations = <int>[];
    for (final target in [49, 499, 2499, 4999, 0]) {
      final stopwatch = Stopwatch()..start();
      await viewController.goToPage(target);
      await tester.pump();
      stopwatch.stop();
      continuousNavigationDurations.add(stopwatch.elapsedMicroseconds);
    }
    final scrollWatch = Stopwatch()..start();
    await tester.drag(find.byType(PdfViewer), const Offset(0, -640));
    await tester.pump();
    scrollWatch.stop();

    final resizeDurations = <int>[];
    for (final size in const [Size(1280, 800), Size(1600, 1000)]) {
      final stopwatch = Stopwatch()..start();
      tester.view.physicalSize = size;
      await tester.pump();
      stopwatch.stop();
      resizeDurations.add(stopwatch.elapsedMicroseconds);
    }
    tester.view.physicalSize = const Size(1440, 900);
    await tester.pump();
    peakRss = peakRss > ProcessInfo.currentRss
        ? peakRss
        : ProcessInfo.currentRss;
    await reader.closeDocument();
    final engineDiagnostics = engine.diagnostics.toJson();
    expect(engineDiagnostics['open_documents'], 0);
    expect(engineDiagnostics['pending_renders'], 0);
    expect(engineDiagnostics['cached_bytes'], 0);
    engine.clearCaches();
    await tester.pumpWidget(libraryApp(AppInterfaceDensity.comfortable));
    await tester.pumpAndSettle();
    final afterReturnSamples = <int>[];
    for (var sample = 0; sample < 5; sample++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 500)),
      );
      await tester.pump();
      final sample = ProcessInfo.currentRss;
      afterReturnSamples.add(sample);
      peakRss = peakRss > sample ? peakRss : sample;
    }
    final afterReturnRss = afterReturnSamples.reduce(
      (left, right) => left < right ? left : right,
    );
    binding.reportData ??= <String, dynamic>{};
    binding.reportData!['memory_bytes'] = {
      'baseline': baselineRss,
      'peak': peakRss,
      'after_return_minimum': afterReturnRss,
      'after_return_samples': afterReturnSamples,
      'released_after_return': peakRss - afterReturnRss,
      'heavy_close_samples': heavyCloseRss,
      'heavy_close_growth': heavyCloseGrowth,
    };
    binding.reportData!['interaction_microseconds'] = {
      'library_median': _percentile(libraryDurations, 0.50),
      'library_p95': _percentile(libraryDurations, 0.95),
      'reader_median': _percentile(readerDurations, 0.50),
      'reader_p95': _percentile(readerDurations, 0.95),
      'first_page_visible_100_pages': firstPage100Watch.elapsedMicroseconds,
      'first_page_visible_5000_pages': firstPage5000Watch.elapsedMicroseconds,
      'layout_change_median': _percentile(layoutDurations, 0.50),
      'layout_change_p95': _percentile(layoutDurations, 0.95),
      'continuous_navigation_median': _percentile(
        continuousNavigationDurations,
        0.50,
      ),
      'continuous_navigation_p95': _percentile(
        continuousNavigationDurations,
        0.95,
      ),
      'continuous_scroll': scrollWatch.elapsedMicroseconds,
      'resize_p95': _percentile(resizeDurations, 0.95),
    };
    binding.reportData!['pdf_engine_microseconds'] = {
      'heavy_open_median': _percentile(openDurations, 0.50),
      'heavy_open_p95': _percentile(openDurations, 0.95),
      'heavy_render_median': _percentile(heavyRenderDurations, 0.50),
      'heavy_render_p95': _percentile(heavyRenderDurations, 0.95),
    };
    binding.reportData!['pdf_engine_diagnostics'] = engineDiagnostics;
    final libraryFrames = _frameSamples(
      binding.reportData!,
      'library_run_',
      'frame_build_times',
    );
    final readerFrames = _frameSamples(
      binding.reportData!,
      'reader_run_',
      'frame_build_times',
    );
    binding.reportData!['frame_build_microseconds'] = {
      'library_median': _percentile(libraryFrames, 0.50),
      'library_p95': _percentile(libraryFrames, 0.95),
      'reader_median': _percentile(readerFrames, 0.50),
      'reader_p95': _percentile(readerFrames, 0.95),
      'slow_over_16_7ms': [
        ...libraryFrames,
        ...readerFrames,
      ].where((duration) => duration > 16667).length,
    };
    binding.reportData!['environment'] = {
      'build_mode': 'profile',
      'operating_system': Platform.operatingSystem,
      'dpi_scale': 1.25,
      'physical_size': '1440x900',
      'logical_size': '1152x720',
      'library_documents': 40,
      'comparison_document_pages': 100,
      'huge_document_pages': 5000,
      'heavy_document_pages': 8,
      'heavy_shapes_per_page': 1200,
      'interaction_runs': 5,
      'engine_runs': 15,
      'contains_paths_or_titles': false,
    };
  });
}

List<int> _frameSamples(
  Map<String, dynamic> report,
  String prefix,
  String field,
) {
  final values = <int>[];
  for (var run = 0; run < 5; run++) {
    final section = report['$prefix$run'] as Map<String, dynamic>;
    values.addAll(
      (section[field] as List).map((value) => (value as num).toInt()),
    );
  }
  return values;
}

int _percentile(List<int> samples, double percentile) {
  final sorted = samples.toList()..sort();
  final index = ((sorted.length - 1) * percentile).ceil();
  return sorted[index];
}

Future<void> _pumpUntil(WidgetTester tester, bool Function() predicate) async {
  for (var attempt = 0; attempt < 80; attempt++) {
    if (predicate()) return;
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 50)),
    );
    await tester.pump();
  }
  fail('Timed out waiting for the native viewer.');
}
