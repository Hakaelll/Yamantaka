import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/errors/failure_presentation.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/platform/window_service.dart';
import 'package:yamantaka/features/annotations/application/annotation_controller.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/bookmarks/application/bookmark_controller.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/reader/application/reader_controller.dart';
import 'package:yamantaka/features/reader/domain/document_lifecycle.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/domain/text_selection.dart'
    as reader_selection;
import 'package:yamantaka/features/navigation/application/navigation_history_controller.dart';
import 'package:yamantaka/features/reader/presentation/document_navigator_dialog.dart';
import 'package:yamantaka/features/reader/presentation/reader_screen.dart';
import 'package:yamantaka/features/search/application/search_controller.dart'
    as app_search;
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/pdf_engine/pdf_document_engine.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    final windowsRoot = Platform.environment['WINDIR'] ?? r'C:\Windows';
    final flutterRoot = Platform.environment['FLUTTER_ROOT'];
    await _loadTestFont('Segoe UI Variable', [
      '$windowsRoot/Fonts/segoeui.ttf',
      '$windowsRoot/Fonts/SegoeUIVF.ttf',
    ]);
    await _loadTestFont('MaterialIcons', [
      r'.flutter-sdk\bin\cache\artifacts\material_fonts\MaterialIcons-Regular.otf',
      if (flutterRoot != null)
        '$flutterRoot/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
    ]);
  });

  test('opening a saved book starts from a complete fitted unit', () {
    const restored = DocumentPreferences(
      documentId: 'book',
      viewMode: ReaderMode.book,
      pageLayout: PageLayoutMode.facing,
      coverPolicy: ReaderCoverPolicy.firstPageIsCover,
      initialSide: ReaderInitialSide.right,
      zoomMode: ZoomMode.custom,
      zoomLevel: 1.56,
      scrollOffset: 0.4,
    );

    final normalized = normalizeOpeningReaderPreferences(restored);

    expect(normalized.zoomMode, ZoomMode.fitPage);
    expect(normalized.scrollOffset, 0);
    expect(normalized.zoomLevel, 1.56);
    expect(normalized.coverPolicy, ReaderCoverPolicy.firstPageIsCover);
    expect(normalized.initialSide, ReaderInitialSide.right);
  });

  test('only the latest overlapping open publishes reader state', () async {
    final engine = _ControlledOpenPdfEngine();
    final harness = await _ReaderHarness.create(
      ready: false,
      engineOverride: engine,
    );
    final directory = Directory(
      'build/test-reader-open-race-${DateTime.now().microsecondsSinceEpoch}',
    )..createSync(recursive: true);
    final first = File('${directory.path}/first.pdf')
      ..writeAsBytesSync(const [0x25, 0x50, 0x44, 0x46, 0x31]);
    final second = File('${directory.path}/second.pdf')
      ..writeAsBytesSync(const [0x25, 0x50, 0x44, 0x46, 0x32, 0x32]);

    try {
      final firstOpen = harness.controller.openPath(first.absolute.path);
      await engine.waitForRequest(first.absolute.path);
      final secondOpen = harness.controller.openPath(second.absolute.path);
      await engine.waitForRequest(second.absolute.path);

      engine.complete(second.absolute.path, documentId: 'native-second');
      expect(await secondOpen, isTrue);
      engine.complete(first.absolute.path, documentId: 'native-first');
      expect(await firstOpen, isFalse);

      expect(harness.controller.document?.fileName, 'second.pdf');
      expect(harness.controller.openedDocument?.documentId, 'native-second');
      expect(harness.controller.lifecycle.state, DocumentLifecycleState.ready);
      expect(engine.closedDocumentIds, contains('native-first'));
      expect(engine.closedDocumentIds, isNot(contains('native-second')));
    } finally {
      await harness.dispose();
      directory.deleteSync(recursive: true);
    }
  });

  test('closing during native open cancels publication and handle', () async {
    final engine = _ControlledOpenPdfEngine();
    final harness = await _ReaderHarness.create(
      ready: false,
      engineOverride: engine,
    );
    final directory = Directory(
      'build/test-reader-close-race-${DateTime.now().microsecondsSinceEpoch}',
    )..createSync(recursive: true);
    final file = File('${directory.path}/closing.pdf')
      ..writeAsBytesSync(const [0x25, 0x50, 0x44, 0x46]);

    try {
      final opening = harness.controller.openPath(file.absolute.path);
      await engine.waitForRequest(file.absolute.path);
      await harness.controller.closeDocument();
      engine.complete(file.absolute.path, documentId: 'native-cancelled');

      expect(await opening, isFalse);
      expect(harness.controller.document, isNull);
      expect(harness.controller.openedDocument, isNull);
      expect(harness.controller.lifecycle.state, DocumentLifecycleState.closed);
      expect(engine.closedDocumentIds, contains('native-cancelled'));
    } finally {
      await harness.dispose();
      directory.deleteSync(recursive: true);
    }
  });

  testWidgets('reader presents a recoverable error state', (tester) async {
    final harness = await _ReaderHarness.create(ready: false);
    FailureAction? selectedAction;
    harness.controller
      ..status = ReaderStatus.failure
      ..error = const AppFailure(
        FailureKind.corruptPdf,
        'El PDF de prueba está dañado.',
      );

    await tester.pumpWidget(
      _app(harness, onFailureAction: (action) async => selectedAction = action),
    );

    expect(find.text('El PDF no se puede abrir'), findsOneWidget);
    expect(
      find.text(
        'El archivo puede estar dañado o usar una variante no compatible.',
      ),
      findsOneWidget,
    );
    expect(find.text('Localizar'), findsOneWidget);
    expect(find.text('Volver a la biblioteca'), findsOneWidget);
    await tester.tap(find.text('Localizar'));
    expect(selectedAction, FailureAction.chooseAnotherFile);
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
    final topSurface = tester.widget<DecoratedBox>(
      find.byKey(const Key('reader-top-surface')),
    );
    final topDecoration = topSurface.decoration as BoxDecoration;
    expect(topDecoration.color, isNotNull);
    expect(topDecoration.color!.a, lessThan(1));
    expect(topDecoration.border, isNull);
    expect(topDecoration.gradient, isNull);

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

    await tester.tap(find.byTooltip('Página siguiente (→)'));
    await tester.pump();
    expect(harness.controller.currentPageIndex, 1);
    expect(find.text('2 / 3'), findsOneWidget);

    await tester.tap(find.byTooltip('Buscar en el documento (Ctrl+F)'));
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

  testWidgets(
    'Ctrl+J unifies document destinations without rebuilding the viewer',
    (tester) async {
      tester.view.physicalSize = const Size(1280, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final harness = await _ReaderHarness.create(ready: true, pageCount: 100);
      final adapter = _CountingViewerAdapter();
      harness.bookmarks.add(23, label: 'Tesis principal');

      await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
      await tester.pump();
      expect(adapter.buildCount, 1);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyJ);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();

      expect(
        find.byKey(const Key('document-navigator-filter')),
        findsOneWidget,
      );
      tester.binding.focusManager.primaryFocus?.unfocus();
      await tester.pump(const Duration(seconds: 1));
      await tester.pump();
      expect(find.text('Miniaturas'), findsWidgets);
      expect(find.text('Marcadores'), findsWidgets);
      await tester.enterText(
        find.byKey(const Key('document-navigator-filter')),
        'Tesis principal',
      );
      await tester.pump(const Duration(milliseconds: 180));
      final destination = find.descendant(
        of: find.byKey(const Key('document-navigator-list')),
        matching: find.text('Tesis principal'),
      );
      expect(destination, findsOneWidget);
      await tester.tap(destination);
      await tester.pump();
      await tester.tap(find.byKey(const Key('document-navigator-open')));
      await tester.pump();
      expect(harness.controller.currentPageIndex, 23);
      expect(adapter.buildCount, 1);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await tester.pump();
      expect(harness.controller.currentPageIndex, 0);

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  test(
    'selection annotations use real geometry and degrade explicitly',
    () async {
      final harness = await _ReaderHarness.create(ready: true);
      const rect = NormalizedRect(
        left: 0.1,
        top: 0.2,
        right: 0.4,
        bottom: 0.24,
      );
      harness.controller.updateTextSelection(
        const reader_selection.TextSelection(
          range: reader_selection.TextSelectionRange(
            start: reader_selection.TextSelectionPosition(
              pageIndex: 0,
              offset: 6,
            ),
            end: reader_selection.TextSelectionPosition(
              pageIndex: 0,
              offset: 11,
            ),
          ),
          pages: [
            reader_selection.TextSelectionPageGeometry(
              pageIndex: 0,
              text: 'Antes texto después',
              rects: [rect],
            ),
          ],
        ),
      );
      final precise = harness.controller.createAnnotationFromSelection(
        type: AnnotationType.highlight,
      );
      expect(precise?.usedPreciseGeometry, isTrue);
      expect(precise?.annotation.type, AnnotationType.highlight);
      expect(precise?.annotation.rects.single.toJson(), rect.toJson());
      expect(precise?.annotation.selectedText, 'texto');

      harness.controller.updateTextSelection(
        const reader_selection.TextSelection(
          range: reader_selection.TextSelectionRange(
            start: reader_selection.TextSelectionPosition(
              pageIndex: 0,
              offset: 0,
            ),
            end: reader_selection.TextSelectionPosition(
              pageIndex: 1,
              offset: 4,
            ),
          ),
          pages: [
            reader_selection.TextSelectionPageGeometry(
              pageIndex: 0,
              text: 'uno',
              rects: [],
            ),
            reader_selection.TextSelectionPageGeometry(
              pageIndex: 1,
              text: 'dos',
              rects: [],
            ),
          ],
        ),
      );
      final fallback = harness.controller.createAnnotationFromSelection(
        type: AnnotationType.underline,
      );
      expect(fallback?.usedPreciseGeometry, isFalse);
      expect(fallback?.annotation.type, AnnotationType.pageComment);
      expect(fallback?.annotation.rects, isEmpty);
      await harness.dispose();
    },
  );

  testWidgets(
    'reader overflow is compact and replaces the document tab strip',
    (tester) async {
      tester.view.physicalSize = const Size(1280, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final harness = await _ReaderHarness.create(ready: true);

      await tester.pumpWidget(_app(harness));
      await tester.pump();
      await tester.tap(find.byTooltip('Más opciones'));
      await tester.pump(const Duration(milliseconds: 500));
      await tester.pump();

      expect(find.text('Abrir otro PDF…'), findsOneWidget);
      expect(find.text('Información del documento'), findsOneWidget);
      expect(find.text('Miniaturas'), findsNothing);
      expect(find.textContaining('pestaña'), findsNothing);

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  testWidgets('fullscreen keeps chrome orthogonal to immersive mode', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);

    await tester.pumpWidget(_app(harness));
    await tester.pump();
    expect(find.byKey(const Key('reader-top-bar')), findsOneWidget);
    expect(find.byKey(const Key('reader-bottom-controls')), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.f11);
    await tester.pump();

    expect(harness.windowService.fullScreen, isTrue);
    expect(harness.controller.isFullScreen, isTrue);
    expect(find.byKey(const Key('test-pdf-viewer')), findsOneWidget);
    expect(find.byKey(const Key('reader-top-bar')), findsOneWidget);
    expect(find.byKey(const Key('reader-bottom-controls')), findsOneWidget);
    expect(find.byKey(const Key('reader-progress')), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();

    expect(harness.windowService.fullScreen, isFalse);
    expect(harness.controller.isFullScreen, isFalse);
    expect(find.byKey(const Key('reader-top-bar')), findsOneWidget);
    expect(find.byKey(const Key('reader-bottom-controls')), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.f11);
    await tester.pump();
    expect(harness.windowService.fullScreen, isTrue);

    await harness.controller.closeDocument();
    expect(harness.windowService.fullScreen, isFalse);
    expect(harness.controller.isFullScreen, isFalse);

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('book mode pairs page one immediately and navigates by spreads', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 6);
    final adapter = _RecordingViewerAdapter();

    harness.controller.setReaderMode(ReaderMode.book);
    await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
    await tester.pump();

    expect(harness.controller.preferences!.pageLayout, PageLayoutMode.facing);
    expect(adapter.configuration?.flow, PdfViewFlow.facing);
    expect(find.byTooltip('Pliego siguiente (→)'), findsOneWidget);
    expect(adapter.configuration?.coverPolicy, ReaderCoverPolicy.noCover);
    expect(adapter.configuration?.initialSide, ReaderInitialSide.left);
    expect(find.text('1–2 / 6'), findsOneWidget);
    expect(find.byKey(const Key('reader-mode-selector')), findsNothing);
    expect(
      tester.getCenter(find.byKey(const Key('reader-navigation-group'))).dx,
      closeTo(640, 1),
    );

    for (final expectedPage in const [2, 4, 4, 4]) {
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await tester.pump();
      expect(harness.controller.currentPageIndex, expectedPage);
    }
    expect(find.text('5–6 / 6'), findsOneWidget);
    for (final expectedPage in const [2, 0]) {
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump();
      expect(harness.controller.currentPageIndex, expectedPage);
    }

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('book reader matches the maintained editorial v0.15 baseline', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 6);
    harness.controller.setReaderMode(ReaderMode.book);

    await tester.pumpWidget(
      _app(harness, viewerAdapter: const _BookPreviewViewerAdapter()),
    );
    await tester.pump(const Duration(milliseconds: 300));

    await expectLater(
      find.byType(ReaderScreen),
      matchesGoldenFile(
        '../docs/screenshots/windows-reader-editorial-v015.png',
      ),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('document navigator matches the maintained v0.15 baseline', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 100);
    harness.bookmarks.add(23, label: 'Tesis principal');
    harness.annotations.create(
      pageIndex: 41,
      note: 'Revisar este argumento antes de continuar.',
    );
    const boundaryKey = Key('navigator-golden-boundary');

    await tester.pumpWidget(
      _visualApp(
        harness,
        captureKey: boundaryKey,
        themeMode: ThemeMode.light,
        highContrast: false,
        reduceMotion: true,
        scenarioKey: 'navigator-golden',
        panelOpen: false,
        navigatorOpen: true,
        textScale: 1,
      ),
    );
    await tester.pump();
    expect(find.byKey(const Key('document-navigator-filter')), findsOneWidget);

    await expectLater(
      find.byKey(boundaryKey),
      matchesGoldenFile(
        '../docs/screenshots/windows-document-navigator-v015.png',
      ),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('reader chrome uses the compact 0.9 geometry and 40 px targets', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 6);

    for (final entry in const [
      (Size(640, 480), AppSizes.readerCompactTopBarHeight),
      (Size(1280, 720), AppSizes.topBarHeight),
    ]) {
      tester.view.physicalSize = entry.$1;
      await tester.pumpWidget(_app(harness));
      await tester.pump();

      expect(
        tester.getSize(find.byKey(const Key('reader-top-surface'))).height,
        entry.$2,
      );
      expect(
        tester.getSize(find.byTooltip('Volver a la biblioteca')),
        const Size.square(AppSizes.desktopControlExtent),
      );
      expect(
        tester.getSize(find.byKey(const Key('reader-active-dock'))).height,
        AppSizes.bottomControlHeight,
      );
      expect(find.text('book.pdf'), findsOneWidget);
      expect(tester.takeException(), isNull, reason: '${entry.$1}');
    }

    await tester.pump(const Duration(seconds: 4));
    await tester.pump();
    expect(
      tester.getSize(find.byKey(const Key('reader-resting-dock'))).height,
      AppSizes.readerRestingDockHeight,
    );

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('Ctrl+K exposes all layouts without a persistent selector', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 6);
    final adapter = _RecordingViewerAdapter();
    await harness.controller.zoomBy(1.56);
    harness.controller.preferences = harness.controller.preferences!.copyWith(
      coverPolicy: ReaderCoverPolicy.firstPageIsCover,
      initialSide: ReaderInitialSide.right,
    );
    expect(harness.controller.preferences!.zoomMode, ZoomMode.custom);

    await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
    await tester.pump();
    expect(find.byKey(const Key('reader-mode-selector')), findsNothing);
    expect(adapter.configuration?.flow, PdfViewFlow.single);

    Future<void> openPalette(String query) async {
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      final search = find.byKey(const Key('command-palette-search'));
      expect(search, findsOneWidget);
      await tester.enterText(search, query);
      await tester.pump();
    }

    await openPalette('disposición');
    expect(
      find.byKey(const ValueKey('command-singlePageLayout')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('command-facingPageLayout')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('command-continuousPageLayout')),
      findsOneWidget,
    );
    await tester.tap(find.byKey(const ValueKey('command-facingPageLayout')));
    await tester.pump();
    expect(harness.controller.preferences!.viewMode, ReaderMode.book);
    expect(harness.controller.preferences!.pageLayout, PageLayoutMode.facing);
    expect(harness.controller.preferences!.zoomMode, ZoomMode.fitPage);
    expect(
      harness.controller.preferences!.coverPolicy,
      ReaderCoverPolicy.firstPageIsCover,
    );
    expect(
      harness.controller.preferences!.initialSide,
      ReaderInitialSide.right,
    );
    expect(adapter.configuration?.flow, PdfViewFlow.facing);
    expect(adapter.configuration?.initialZoomMode, PdfViewZoomMode.fitPage);
    await tester.tap(find.byTooltip('Más opciones'));
    await tester.pump();
    expect(find.text('Portada sola'), findsOneWidget);
    expect(find.text('Primera página a la derecha'), findsOneWidget);
    expect(find.text('Restablecer emparejado'), findsOneWidget);
    await tester.tapAt(Offset.zero);
    await tester.pump();

    await openPalette('continua');
    await tester.tap(
      find.byKey(const ValueKey('command-continuousPageLayout')),
    );
    await tester.pump();
    expect(harness.controller.preferences!.viewMode, ReaderMode.document);
    expect(
      harness.controller.preferences!.pageLayout,
      PageLayoutMode.continuous,
    );
    expect(adapter.configuration?.flow, PdfViewFlow.continuous);

    await openPalette('una página');
    await tester.tap(find.byKey(const ValueKey('command-singlePageLayout')));
    await tester.pump();
    expect(harness.controller.preferences!.viewMode, ReaderMode.document);
    expect(harness.controller.preferences!.pageLayout, PageLayoutMode.single);
    expect(adapter.configuration?.flow, PdfViewFlow.single);

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('all reading keys share navigation and text input keeps them', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 6);
    harness.controller.setReaderMode(ReaderMode.book);

    await tester.pumpWidget(_app(harness));
    await tester.pump();

    await tester.sendKeyEvent(LogicalKeyboardKey.pageDown);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 2);
    expect(find.text('3–4 / 6'), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.pageUp);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 0);

    await tester.sendKeyEvent(LogicalKeyboardKey.space);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 2);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.space);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 0);

    await tester.sendKeyEvent(LogicalKeyboardKey.end);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 4);
    await tester.sendKeyEvent(LogicalKeyboardKey.home);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 0);

    await tester.tap(find.byTooltip('Buscar en el documento (Ctrl+F)'));
    await tester.pump();
    final searchField = find.byKey(const Key('pdf-search-field'));
    expect(searchField, findsOneWidget);
    await tester.tap(searchField);
    await tester.enterText(searchField, 'consulta');
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.sendKeyEvent(LogicalKeyboardKey.pageDown);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 0);

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets('arrow keys navigate while the PDF canvas owns focus', (
    tester,
  ) async {
    final harness = await _ReaderHarness.create(ready: true);
    final viewerFocus = FocusNode(debugLabel: 'embedded PDF canvas');
    final adapter = _KeyboardConsumingViewerAdapter(viewerFocus);

    await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
    await tester.pump();
    viewerFocus.requestFocus();
    await tester.pump();
    expect(viewerFocus.hasPrimaryFocus, isTrue);

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 1);

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
    await tester.pump();
    expect(harness.controller.currentPageIndex, 0);

    await tester.pumpWidget(const SizedBox.shrink());
    viewerFocus.dispose();
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
      expect(find.byKey(const Key('reader-mode-selector')), findsNothing);
    }

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets(
    'document and book expose the same common actions in compact and expanded layouts',
    (tester) async {
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      for (final size in const [Size(640, 760), Size(1280, 800)]) {
        final compact = size.width < AppBreakpoints.medium;
        for (final mode in ReaderMode.values) {
          final harness = await _ReaderHarness.create(
            ready: true,
            pageCount: 6,
          );
          if (mode == ReaderMode.book) {
            harness.controller.setReaderMode(ReaderMode.book);
          }
          tester.view.physicalSize = size;
          await tester.pumpWidget(_app(harness));
          await tester.pump();

          expect(find.byTooltip('Volver a la biblioteca'), findsOneWidget);
          expect(
            find.byTooltip('Buscar en el documento (Ctrl+F)'),
            findsOneWidget,
          );
          expect(
            find.byTooltip('Ajuste de zoom'),
            compact ? findsNothing : findsOneWidget,
          );
          if (compact) {
            expect(find.byTooltip('Más controles de lectura'), findsOneWidget);
          }
          expect(
            find.byTooltip(
              mode == ReaderMode.book
                  ? 'Pliego siguiente (→)'
                  : 'Página siguiente (→)',
            ),
            findsOneWidget,
          );

          await tester.tap(find.byTooltip('Más opciones'));
          await tester.pump();
          for (final label in const [
            'Abrir otro PDF…',
            'Girar página',
            'Márgenes de lectura',
            'Pantalla completa',
            'Ocultar interfaz',
            'Modo de concentración',
            'Información del documento',
            'Ajustes',
          ]) {
            expect(
              find.text(label),
              findsOneWidget,
              reason: '$mode · $size · $label',
            );
          }
          if (compact) {
            for (final label in const [
              'Abrir paleta de comandos',
              'Miniaturas',
              'Índice',
              'Marcadores',
              'Anotaciones',
            ]) {
              expect(
                find.text(label),
                findsOneWidget,
                reason: '$mode · $size · $label',
              );
            }
          }
          await tester.sendKeyEvent(LogicalKeyboardKey.escape);
          await tester.pump();

          if (compact) {
            await tester.tap(find.byTooltip('Más controles de lectura'));
            await tester.pump();
            expect(find.text('Tema de lectura'), findsOneWidget);
            expect(find.textContaining('Tamaño real'), findsOneWidget);
            expect(find.textContaining('Ajustar al'), findsWidgets);
            await tester.sendKeyEvent(LogicalKeyboardKey.escape);
            await tester.pump();
          } else {
            expect(find.byTooltip('Tema de lectura'), findsOneWidget);
            expect(
              find.byTooltip('Abrir paleta de comandos (Ctrl+K)'),
              findsOneWidget,
            );
            await tester.tap(find.byTooltip('Herramientas del lector'));
            await tester.pump();
            for (final label in const [
              'Miniaturas',
              'Índice',
              'Marcadores',
              'Anotaciones',
            ]) {
              expect(find.text(label), findsOneWidget);
            }
            await tester.sendKeyEvent(LogicalKeyboardKey.escape);
            await tester.pump();
          }

          await tester.pumpWidget(const SizedBox.shrink());
          await harness.dispose();
        }
      }
    },
  );

  testWidgets('book flow remains a compact spread at every reader width', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);
    final adapter = _RecordingViewerAdapter();
    harness.controller.setReaderMode(ReaderMode.book);

    tester.view.physicalSize = const Size(640, 800);
    await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.facing);
    expect(adapter.configuration?.coverPolicy, ReaderCoverPolicy.noCover);
    expect(adapter.configuration?.initialSide, ReaderInitialSide.left);
    final configuredMargin = adapter.configuration?.margin;

    tester.view.physicalSize = const Size(1200, 800);
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.facing);
    expect(find.byKey(const Key('reader-left-zone')), findsOneWidget);
    expect(find.byKey(const Key('reader-right-zone')), findsOneWidget);

    tester.view.physicalSize = const Size(1200, 460);
    await tester.pump();
    expect(adapter.configuration?.flow, PdfViewFlow.facing);
    expect(adapter.configuration?.margin, configuredMargin);

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
      expect(
        find.byKey(const Key('reader-panel-section-rail')),
        findsOneWidget,
      );
      expect(find.bySemanticsLabel('Ancho del panel lateral'), findsOneWidget);
      expect(find.byKey(const Key('reader-panel-scrim')), findsNothing);
      expect(
        tester.getSize(find.byKey(const ValueKey('reader-viewer-pdf-1'))).width,
        closeTo(1440 - AppSizes.sidePanelWidth, 1),
      );

      final handle = find.byKey(const Key('reader-panel-resize-handle'));
      await tester.drag(handle, const Offset(-96, 0));
      await tester.pump();
      final draggedWidth = tester
          .getSize(find.byKey(const Key('reader-side-panel')))
          .width;
      expect(draggedWidth, inInclusiveRange(400, 440));
      expect(
        tester.getSize(find.byKey(const ValueKey('reader-viewer-pdf-1'))).width,
        closeTo(1440 - draggedWidth, 1),
      );
      expect(adapter.buildCount, 1);
      expect(persistedWidth, isNull);
      await tester.pump(const Duration(milliseconds: 260));
      expect(persistedWidth, closeTo(draggedWidth, 1));

      await tester.tap(find.byTooltip('Miniaturas'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));
      await tester.pump();
      expect(find.byKey(const Key('pdf-search-field')), findsNothing);
      expect(adapter.buildCount, 1);

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

      await tester.tap(find.byTooltip('Miniaturas (Ctrl+Mayús+J)'));
      await tester.pump();
      await tester.pump();
      expect(find.byKey(const Key('pdf-search-field')), findsNothing);
      expect(find.text('No disponible'), findsWidgets);
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      expect(find.byKey(const Key('reader-side-panel')), findsNothing);
      tester.view.physicalSize = const Size(800, 600);
      await tester.pump();
      expect(adapter.buildCount, 1);
      tester.view.physicalSize = const Size(1440, 900);
      await tester.pump();
      expect(adapter.buildCount, 1);

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  testWidgets('side panel overlays medium layouts without shrinking the PDF', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(800, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true);

    await tester.pumpWidget(
      _app(harness, initialPanelOpen: true, initialPanelSection: 'search'),
    );
    await tester.pump();

    expect(find.byKey(const Key('reader-panel-scrim')), findsOneWidget);
    expect(
      tester.getSize(find.byKey(const ValueKey('reader-viewer-pdf-1'))).width,
      closeTo(800, 1),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets(
    'combined notes filter orders entries and marks a jumped destination',
    (tester) async {
      tester.view.physicalSize = const Size(1280, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final harness = await _ReaderHarness.create(ready: true, pageCount: 8);
      harness.annotations.create(pageIndex: 5, note: 'Conclusión local');
      harness.bookmarks.add(2, label: 'Tesis principal');

      await tester.pumpWidget(
        _app(
          harness,
          initialPanelOpen: true,
          initialPanelSection: 'annotations',
        ),
      );
      await tester.pump();

      expect(find.byKey(const Key('reader-note-kind-filter')), findsOneWidget);
      expect(find.text('Tesis principal'), findsOneWidget);
      expect(find.text('Conclusión local'), findsOneWidget);
      final bookmarkY = tester.getTopLeft(find.text('Tesis principal')).dy;
      final annotationY = tester.getTopLeft(find.text('Conclusión local')).dy;
      expect(bookmarkY, lessThan(annotationY));

      await tester.tap(find.byKey(const Key('reader-note-kind-filter')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Solo marcadores').last);
      await tester.pump();
      expect(find.text('Tesis principal'), findsOneWidget);
      expect(find.text('Conclusión local'), findsNothing);

      await tester.tap(find.text('Tesis principal'));
      await tester.pump();
      expect(harness.controller.currentPageIndex, 2);
      expect(find.byKey(const Key('reader-destination-cue')), findsOneWidget);
      await tester.pump(const Duration(milliseconds: 1500));
      expect(find.byKey(const Key('reader-destination-cue')), findsNothing);

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  testWidgets('annotation filters remain reachable at 640x480 and 200% text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(640, 480);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 8);
    harness.annotations.create(pageIndex: 2, note: 'Nota legible');
    await tester.pumpWidget(
      _visualApp(
        harness,
        captureKey: const Key('notes-stress'),
        themeMode: ThemeMode.light,
        highContrast: false,
        reduceMotion: true,
        scenarioKey: 'notes-stress',
        panelOpen: true,
        panelSection: 'annotations',
        textScale: 2,
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    await tester.ensureVisible(
      find.byKey(const Key('reader-note-advanced-filters')),
    );
    await tester.tap(find.byKey(const Key('reader-note-advanced-filters')));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.byKey(const Key('reader-note-filter-scroll')), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets(
    'reader controls follow global theme independently of PDF canvas',
    (tester) async {
      tester.view.physicalSize = const Size(1280, 720);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final harness = await _ReaderHarness.create(ready: true);
      for (final mode in [ThemeMode.dark, ThemeMode.light]) {
        harness.controller.setTheme(
          mode == ThemeMode.dark ? ReaderTheme.ivory : ReaderTheme.dark,
        );
        await tester.pumpWidget(
          _visualApp(
            harness,
            captureKey: const Key('theme-scope'),
            themeMode: mode,
            highContrast: false,
            reduceMotion: true,
            scenarioKey: mode.name,
            panelOpen: true,
            textScale: 1,
          ),
        );
        await tester.pumpAndSettle();
        expect(
          Theme.of(
            tester.element(find.byKey(const Key('reader-top-surface'))),
          ).brightness,
          mode == ThemeMode.dark ? Brightness.dark : Brightness.light,
        );
        expect(tester.takeException(), isNull);
      }
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
    expect(
      find.byTooltip('Buscar en el documento (Ctrl+F)').hitTestable(),
      findsNothing,
    );
    expect(
      find.byKey(const Key('reader-resting-dock')).hitTestable(),
      findsOneWidget,
    );

    await tester.tap(find.byKey(const Key('reader-resting-dock')));
    await tester.pump();
    expect(
      find.byTooltip('Buscar en el documento (Ctrl+F)').hitTestable(),
      findsOneWidget,
    );
    await tester.pump(const Duration(seconds: 4));

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump();

    expect(harness.controller.currentPageIndex, 2);
    expect(
      find.byTooltip('Buscar en el documento (Ctrl+F)').hitTestable(),
      findsOneWidget,
    );
    await tester.pumpWidget(const SizedBox.shrink());
    await harness.dispose();
  });

  testWidgets(
    'page changes rebuild page surfaces without notifying the reader shell',
    (tester) async {
      final harness = await _ReaderHarness.create(ready: true);
      final adapter = _CountingViewerAdapter();
      var shellNotifications = 0;
      var pageNotifications = 0;
      void onShellChanged() => shellNotifications++;
      void onPageChanged() => pageNotifications++;
      harness.controller.addListener(onShellChanged);
      harness.controller.pageIndexListenable.addListener(onPageChanged);

      await tester.pumpWidget(_app(harness, viewerAdapter: adapter));
      await tester.pump();
      shellNotifications = 0;
      pageNotifications = 0;

      harness.controller.onPageChanged(1);
      await tester.pump();

      expect(shellNotifications, 0);
      expect(pageNotifications, 1);
      expect(adapter.buildCount, 1);
      expect(find.text('2 / 3'), findsOneWidget);

      harness.controller.removeListener(onShellChanged);
      harness.controller.pageIndexListenable.removeListener(onPageChanged);
      await tester.pumpWidget(const SizedBox.shrink());
      await harness.dispose();
    },
  );

  testWidgets(
    'temporary positions A and B restore every layout in LTR and RTL',
    (tester) async {
      final harness = await _ReaderHarness.create(ready: true);
      for (final direction in ReadingDirection.values) {
        for (final layout in PageLayoutMode.values) {
          final mode = layout == PageLayoutMode.facing
              ? ReaderMode.book
              : ReaderMode.document;
          await tester.pumpWidget(_app(harness, readingDirection: direction));
          harness.controller.preferences = harness.controller.preferences!
              .copyWith(viewMode: mode, pageLayout: layout, pageRotation: 90);
          harness.controller.onPageChanged(1);
          harness.viewController.update(zoom: 1.75, scrollOffset: 0.42);
          final saved = harness.controller.saveTemporaryPosition(
            ReaderTemporaryPositionSlot.a,
          );
          expect(saved, isNotNull);

          harness.controller.preferences = harness.controller.preferences!
              .copyWith(
                viewMode: ReaderMode.document,
                pageLayout: PageLayoutMode.single,
                pageRotation: 0,
              );
          harness.controller.onPageChanged(0);
          harness.viewController.update(zoom: 1, scrollOffset: 0);

          expect(
            await harness.controller.goToTemporaryPosition(
              ReaderTemporaryPositionSlot.a,
            ),
            isTrue,
          );
          await tester.pump();
          expect(harness.controller.currentPageIndex, 1);
          expect(harness.controller.preferences!.viewMode, mode);
          expect(harness.controller.preferences!.pageLayout, layout);
          expect(harness.controller.preferences!.pageRotation, 90);
          expect(harness.viewController.zoom, 1.75);
          expect(harness.viewController.scrollOffset, 0.42);
        }
      }

      harness.controller.saveTemporaryPosition(ReaderTemporaryPositionSlot.b);
      final previousA = harness.controller.temporaryPositionA;
      final previousB = harness.controller.temporaryPositionB;
      expect(harness.controller.swapTemporaryPositions(), isTrue);
      expect(harness.controller.temporaryPositionA, same(previousB));
      expect(harness.controller.temporaryPositionB, same(previousA));

      await tester.pumpWidget(const SizedBox.shrink());
      await harness.controller.closeDocument();
      expect(harness.controller.temporaryPositionA, isNull);
      expect(harness.controller.temporaryPositionB, isNull);
      await harness.dispose();
    },
  );

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

  testWidgets('reader errors switch immediately between Spanish and English', (
    tester,
  ) async {
    final harness = await _ReaderHarness.create(ready: false);
    addTearDown(harness.dispose);
    harness.controller
      ..status = ReaderStatus.failure
      ..error = const AppFailure(FailureKind.fileNotFound, 'private path');

    for (final language in AppLanguage.values) {
      await tester.pumpWidget(_app(harness, locale: Locale(language.code)));
      await tester.pump();

      expect(
        find.text(
          language == AppLanguage.spanish
              ? 'El archivo ya no está disponible'
              : 'The file is no longer available',
        ),
        findsOneWidget,
      );
    }
  });

  testWidgets('captures the requested reader widget visual matrix', (
    tester,
  ) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_READER_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final harness = await _ReaderHarness.create(ready: true, pageCount: 12);
    addTearDown(harness.dispose);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const captureKey = Key('reader-visual-matrix-boundary');
    final evidence = <Map<String, Object?>>[];

    Future<void> capture({
      required String name,
      required Size size,
      ThemeMode themeMode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
      bool panelOpen = false,
      String panelSection = 'search',
      bool resting = false,
      bool navigatorOpen = false,
      double textScale = 1,
    }) async {
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        _visualApp(
          harness,
          captureKey: captureKey,
          themeMode: themeMode,
          highContrast: highContrast,
          reduceMotion: reduceMotion,
          scenarioKey: name,
          panelOpen: panelOpen,
          panelSection: panelSection,
          navigatorOpen: navigatorOpen,
          textScale: textScale,
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));
      // Finish nested theme/control transitions before capturing evidence.
      await tester.pumpAndSettle();
      if (navigatorOpen) {
        expect(
          find.byKey(const Key('document-navigator-filter')),
          findsOneWidget,
        );
      }
      if (resting) {
        await tester.pump(const Duration(seconds: 4));
        await tester.pump(const Duration(milliseconds: 250));
      }
      await tester.pump();
      final path = '${output.path}${Platform.pathSeparator}$name.png';
      await _captureBoundary(tester, captureKey, path);
      evidence.add({
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'theme': themeMode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'panelOpen': panelOpen,
        'panelSection': panelOpen ? panelSection : null,
        'resting': resting,
        'navigatorOpen': navigatorOpen,
        'captureScope': navigatorOpen ? 'dialog' : 'surface',
        'textScale': textScale,
        'mode': harness.controller.preferences!.viewMode.name,
        'layout': harness.controller.preferences!.pageLayout.name,
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

    harness.controller
      ..setReaderMode(ReaderMode.document)
      ..setPageLayout(PageLayoutMode.single)
      ..setTheme(ReaderTheme.ivory);
    for (final entry in const [
      ('640x480', Size(640, 480)),
      ('800x600', Size(800, 600)),
      ('1280x720', Size(1280, 720)),
      ('1600x900', Size(1600, 900)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ]) {
      await capture(
        name: 'reader-document-${entry.$1}-light-widget',
        size: entry.$2,
      );
    }

    const profileSize = Size(1280, 720);
    harness.controller.setReaderMode(ReaderMode.book);
    await capture(name: 'reader-book-1280x720-light-widget', size: profileSize);
    harness.controller
      ..setReaderMode(ReaderMode.document)
      ..setPageLayout(PageLayoutMode.continuous);
    await capture(
      name: 'reader-continuous-1280x720-light-widget',
      size: profileSize,
    );
    harness.controller
      ..setPageLayout(PageLayoutMode.single)
      ..setTheme(ReaderTheme.dark);
    await capture(
      name: 'reader-document-1280x720-dark-widget',
      size: profileSize,
      themeMode: ThemeMode.dark,
    );
    harness.controller.setTheme(ReaderTheme.light);
    await capture(
      name: 'reader-document-1280x720-high-contrast-widget',
      size: profileSize,
      highContrast: true,
    );
    await capture(
      name: 'reader-document-1280x720-reduced-motion-widget',
      size: profileSize,
      reduceMotion: true,
    );
    await capture(
      name: 'reader-document-800x600-panel-overlay-widget',
      size: const Size(800, 600),
      panelOpen: true,
    );
    await capture(
      name: 'reader-document-1600x900-panel-docked-widget',
      size: const Size(1600, 900),
      panelOpen: true,
    );
    for (final entry in const [
      ('640x480', Size(640, 480)),
      ('1280x720', Size(1280, 720)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ]) {
      await capture(
        name: 'reader-document-${entry.$1}-panel-light-widget',
        size: entry.$2,
        panelOpen: true,
      );
    }
    harness.annotations.create(
      pageIndex: 6,
      note: 'Relacionar este argumento con la conclusión.',
    );
    harness.bookmarks.add(2, label: 'Tesis principal');
    for (final entry in const [
      ('640x480', Size(640, 480)),
      ('800x600', Size(800, 600)),
      ('1280x720', Size(1280, 720)),
      ('1600x900', Size(1600, 900)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ]) {
      await capture(
        name: 'navigator-${entry.$1}-light-widget',
        size: entry.$2,
        navigatorOpen: true,
      );
    }
    await capture(
      name: 'navigator-1280x720-dark-widget',
      size: profileSize,
      themeMode: ThemeMode.dark,
      navigatorOpen: true,
    );
    await capture(
      name: 'navigator-1280x720-high-contrast-widget',
      size: profileSize,
      highContrast: true,
      navigatorOpen: true,
    );
    await capture(
      name: 'navigator-1280x720-reduced-motion-widget',
      size: profileSize,
      reduceMotion: true,
      navigatorOpen: true,
    );
    await capture(
      name: 'reader-document-1600x900-notes-docked-widget',
      size: const Size(1600, 900),
      panelOpen: true,
      panelSection: 'annotations',
    );
    await capture(
      name: 'reader-document-1280x720-panel-dark-widget',
      size: profileSize,
      themeMode: ThemeMode.dark,
      panelOpen: true,
      panelSection: 'annotations',
    );
    await capture(
      name: 'reader-document-1280x720-panel-high-contrast-widget',
      size: profileSize,
      highContrast: true,
      panelOpen: true,
      panelSection: 'annotations',
    );
    await capture(
      name: 'reader-document-1280x720-panel-reduced-motion-widget',
      size: profileSize,
      reduceMotion: true,
      panelOpen: true,
      panelSection: 'annotations',
    );
    for (final entry in const [(1.3, '130'), (1.5, '150'), (2.0, '200')]) {
      await capture(
        name: 'reader-document-1280x720-panel-text-${entry.$2}-widget',
        size: profileSize,
        panelOpen: true,
        panelSection: 'annotations',
        textScale: entry.$1,
      );
    }
    await capture(
      name: 'reader-document-1600x900-resting-widget',
      size: const Size(1600, 900),
      resting: true,
    );
    await capture(
      name: 'reader-document-1280x720-text-130-widget',
      size: profileSize,
      textScale: 1.3,
    );
    await capture(
      name: 'reader-document-1280x720-text-150-widget',
      size: profileSize,
      textScale: 1.5,
    );
    await capture(
      name: 'reader-document-1280x720-text-200-widget',
      size: profileSize,
      textScale: 2,
    );
    File(
      '${output.path}${Platform.pathSeparator}reader-matrix.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-reader-widget-matrix',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'source': 'flutter-widget-test',
        'cases': evidence,
      }),
    );
  });
}

Future<void> _loadTestFont(String family, List<String> candidates) async {
  File? fontFile;
  for (final candidate in candidates) {
    final file = File(candidate);
    if (file.existsSync()) {
      fontFile = file;
      break;
    }
  }
  if (fontFile == null) return;
  final bytes = await fontFile.readAsBytes();
  final data = ByteData.sublistView(bytes);
  await (FontLoader(family)..addFont(Future.value(data))).load();
}

Widget _app(
  _ReaderHarness harness, {
  PdfViewerAdapter viewerAdapter = const _FakeViewerAdapter(),
  ReaderVisualProfile visualProfile = ReaderVisualProfile.standard,
  bool initialPanelOpen = false,
  String initialPanelSection = '',
  ValueChanged<double>? onPanelWidthChanged,
  Future<void> Function(FailureAction action)? onFailureAction,
  ReadingDirection readingDirection = ReadingDirection.leftToRight,
  Locale? locale,
}) => MaterialApp(
  locale: locale ?? const Locale('es'),
  supportedLocales: const [Locale('es'), Locale('en')],
  localizationsDelegates: const [
    YamantakaLocalizations.delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
  ],
  home: ReaderScreen(
    controller: harness.controller,
    engine: harness.engine,
    viewerAdapter: viewerAdapter,
    onBack: () async {},
    onOpenDocument: () async {},
    onOpenSettings: () {},
    reduceMotion: true,
    showProgress: true,
    readingDirection: readingDirection,
    visualProfile: visualProfile,
    initialPanelOpen: initialPanelOpen,
    initialPanelSection: initialPanelSection,
    onPanelWidthChanged: onPanelWidthChanged,
    onFailureAction: onFailureAction,
  ),
);

Widget _visualApp(
  _ReaderHarness harness, {
  required Key captureKey,
  required ThemeMode themeMode,
  required bool highContrast,
  required bool reduceMotion,
  required String scenarioKey,
  required bool panelOpen,
  String panelSection = 'search',
  bool navigatorOpen = false,
  required double textScale,
}) => RepaintBoundary(
  key: captureKey,
  child: MaterialApp(
    debugShowCheckedModeBanner: false,
    locale: const Locale('es'),
    supportedLocales: const [Locale('es'), Locale('en')],
    localizationsDelegates: const [
      YamantakaLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    theme: AppTheme.light(highContrast: highContrast),
    darkTheme: AppTheme.dark(highContrast: highContrast),
    themeMode: themeMode,
    builder: (context, child) => MediaQuery(
      data: MediaQuery.of(context).copyWith(
        highContrast: highContrast,
        disableAnimations: reduceMotion,
        textScaler: TextScaler.linear(textScale),
      ),
      child: child!,
    ),
    home: Stack(
      children: [
        ReaderScreen(
          key: ValueKey(scenarioKey),
          controller: harness.controller,
          engine: harness.engine,
          viewerAdapter: const _VisualViewerAdapter(),
          onBack: () async {},
          onOpenDocument: () async {},
          onOpenSettings: () {},
          reduceMotion: reduceMotion,
          showProgress: true,
          readingDirection: ReadingDirection.leftToRight,
          initialPanelOpen: panelOpen,
          initialPanelSection: panelOpen ? panelSection : '',
        ),
        if (navigatorOpen) ...[
          const Positioned.fill(child: ColoredBox(color: Color(0x52000000))),
          Positioned.fill(
            child: DocumentNavigatorDialog(controller: harness.controller),
          ),
        ],
      ],
    ),
  ),
);

Future<void> _captureBoundary(WidgetTester tester, Key key, String path) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(find.byKey(key));
  await tester.runAsync(() async {
    final image = await boundary.toImage(pixelRatio: 1);
    try {
      final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
      File(path).writeAsBytesSync(bytes!.buffer.asUint8List(), flush: true);
    } finally {
      image.dispose();
    }
  });
}

final class _ReaderHarness {
  _ReaderHarness({
    required this.controller,
    required this.engine,
    required this.viewController,
    required this.database,
    required this.bookmarks,
    required this.annotations,
    required this.search,
    required this.windowService,
    required this.navigationHistory,
  });

  final ReaderController controller;
  final PdfDocumentEngine engine;
  final PdfViewController viewController;
  final AppDatabase database;
  final BookmarkController bookmarks;
  final AnnotationController annotations;
  final app_search.SearchController search;
  final FakeWindowService windowService;
  final NavigationHistoryController navigationHistory;

  static Future<_ReaderHarness> create({
    required bool ready,
    int pageCount = 3,
    PdfDocumentEngine? engineOverride,
  }) async {
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final engine = engineOverride ?? _FakePdfEngine();
    final viewController = PdfViewController();
    final bookmarks = BookmarkController(BookmarkRepository(database));
    final annotations = AnnotationController(AnnotationRepository(database));
    final search = app_search.SearchController(engine, viewController, logger);
    final windowService = FakeWindowService();
    final navigationHistory = NavigationHistoryController();
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
      windowService: windowService,
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
    if (ready) {
      final now = DateTime.utc(2026, 7, 28);
      final document = LibraryDocument(
        id: 'doc-1',
        path: '${directory.path}/book.pdf',
        normalizedPath: '${directory.path}/book.pdf',
        fileName: 'book.pdf',
        pageCount: pageCount,
        fileSize: 100,
        fileModifiedAt: now,
        addedAt: now,
        lastOpenedAt: now,
        fingerprint: 'fingerprint',
      );
      documents.save(document);
      controller
        ..document = document
        ..openedDocument = OpenedPdfDocument(
          documentId: 'pdf-1',
          path: 'book.pdf',
          pageCount: pageCount,
          pageSizes: List.filled(pageCount, const PdfPageSize(600, 800)),
          isEncrypted: false,
        )
        ..preferences = const DocumentPreferences(documentId: 'doc-1')
        ..status = ReaderStatus.ready;
      bookmarks.load(document.id);
      annotations.load(document.id, document.fingerprint);
      viewController.attach(
        pageCount: pageCount,
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
        restorePosition: (zoom, scrollOffset) async =>
            viewController.update(zoom: zoom, scrollOffset: scrollOffset),
      );
      navigationHistory.reset(
        const NavigationLocation(
          pageIndex: 0,
          offset: 0,
          zoom: 1,
          reason: NavigationReason.history,
          label: 'Inicio',
        ),
      );
    }
    return _ReaderHarness(
      controller: controller,
      engine: engine,
      viewController: viewController,
      database: database,
      bookmarks: bookmarks,
      annotations: annotations,
      search: search,
      windowService: windowService,
      navigationHistory: navigationHistory,
    );
  }

  Future<void> dispose() async {
    await controller.closeDocument();
    controller.dispose();
    bookmarks.dispose();
    annotations.dispose();
    search.dispose();
    navigationHistory.dispose();
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

final class _VisualViewerAdapter implements PdfViewerAdapter {
  const _VisualViewerAdapter();

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) => ColoredBox(
    key: const Key('visual-pdf-viewer'),
    color: configuration.backgroundColor,
    child: LayoutBuilder(
      builder: (context, constraints) {
        final availableHeight = (constraints.maxHeight - 36).clamp(
          180.0,
          760.0,
        );
        final availableWidth = (constraints.maxWidth - 48).clamp(180.0, 1400.0);
        if (configuration.flow == PdfViewFlow.continuous) {
          final width = (availableWidth * 0.54).clamp(220.0, 580.0);
          return ListView.separated(
            padding: const EdgeInsets.symmetric(vertical: 28),
            itemCount: 4,
            separatorBuilder: (_, _) => const SizedBox(height: 18),
            itemBuilder: (context, index) => Center(
              child: _VisualPage(
                pageNumber: index + 1,
                width: width,
                height: width / 0.72,
                shadow: configuration.pageShadow,
              ),
            ),
          );
        }
        if (configuration.flow == PdfViewFlow.facing) {
          final width = math.min(
            availableWidth / 2 - 8,
            availableHeight * 0.72,
          );
          return Center(
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                _VisualPage(
                  pageNumber: initialPageIndex + 1,
                  width: width,
                  height: width / 0.72,
                  shadow: configuration.pageShadow,
                ),
                const SizedBox(width: 8),
                _VisualPage(
                  pageNumber: initialPageIndex + 2,
                  width: width,
                  height: width / 0.72,
                  shadow: configuration.pageShadow,
                ),
              ],
            ),
          );
        }
        final height = math.min(availableHeight, availableWidth / 0.72);
        return Center(
          child: _VisualPage(
            pageNumber: initialPageIndex + 1,
            width: height * 0.72,
            height: height,
            shadow: configuration.pageShadow,
          ),
        );
      },
    ),
  );
}

final class _VisualPage extends StatelessWidget {
  const _VisualPage({
    required this.pageNumber,
    required this.width,
    required this.height,
    required this.shadow,
  });

  final int pageNumber;
  final double width;
  final double height;
  final BoxShadow? shadow;

  @override
  Widget build(BuildContext context) => Container(
    width: width,
    height: height,
    padding: EdgeInsets.all((width * 0.1).clamp(18.0, 52.0)),
    decoration: BoxDecoration(
      color: const Color(0xFFFDFDFC),
      boxShadow: [?shadow],
      border: Border.all(color: const Color(0x1F000000)),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'YAMANTAKA · $pageNumber',
          style: TextStyle(
            color: const Color(0xFF272823),
            fontSize: (width * 0.045).clamp(11.0, 22.0),
            fontWeight: FontWeight.w600,
            letterSpacing: 1.2,
          ),
        ),
        SizedBox(height: (height * 0.08).clamp(16.0, 42.0)),
        Container(
          width: width * 0.58,
          height: 4,
          color: const Color(0xFF575A50),
        ),
        SizedBox(height: (height * 0.04).clamp(10.0, 26.0)),
        for (final factor in const [0.92, 0.82, 0.88, 0.68]) ...[
          Container(
            width: width * factor,
            height: 2,
            color: const Color(0xFFB6B6AF),
          ),
          SizedBox(height: (height * 0.025).clamp(7.0, 16.0)),
        ],
      ],
    ),
  );
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

final class _BookPreviewViewerAdapter implements PdfViewerAdapter {
  const _BookPreviewViewerAdapter();

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) => ColoredBox(
    key: const Key('test-pdf-viewer'),
    color: configuration.backgroundColor,
    child: LayoutBuilder(
      builder: (context, constraints) {
        final pageHeight = (constraints.maxHeight - 190).clamp(360.0, 640.0);
        final pageWidth = pageHeight * 0.72;
        return Center(
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _PreviewPage(
                pageNumber: initialPageIndex + 1,
                width: pageWidth,
                height: pageHeight,
                innerEdge: Alignment.centerRight,
              ),
              _PreviewPage(
                pageNumber: initialPageIndex + 2,
                width: pageWidth,
                height: pageHeight,
                innerEdge: Alignment.centerLeft,
              ),
            ],
          ),
        );
      },
    ),
  );
}

final class _PreviewPage extends StatelessWidget {
  const _PreviewPage({
    required this.pageNumber,
    required this.width,
    required this.height,
    required this.innerEdge,
  });

  final int pageNumber;
  final double width;
  final double height;
  final Alignment innerEdge;

  @override
  Widget build(BuildContext context) {
    final innerEdgeOnRight = innerEdge == Alignment.centerRight;
    return Container(
      width: width,
      height: height,
      decoration: const BoxDecoration(
        color: Color(0xFFFDFDFC),
        boxShadow: [
          BoxShadow(
            color: Color(0x52000000),
            blurRadius: 18,
            offset: Offset(0, 8),
          ),
        ],
      ),
      child: Stack(
        children: [
          Center(
            child: Text(
              'Yamantaka · página $pageNumber',
              style: const TextStyle(
                color: Color(0xFF20231F),
                fontFamily: 'Segoe UI Variable',
                fontSize: 24,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          Positioned(
            top: 0,
            bottom: 0,
            left: innerEdgeOnRight ? null : 0,
            right: innerEdgeOnRight ? 0 : null,
            width: 22,
            child: DecoratedBox(
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: innerEdgeOnRight
                      ? Alignment.centerLeft
                      : Alignment.centerRight,
                  end: innerEdge,
                  colors: const [
                    Colors.transparent,
                    Color(0x16000000),
                    Color(0x40000000),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

final class _KeyboardConsumingViewerAdapter implements PdfViewerAdapter {
  const _KeyboardConsumingViewerAdapter(this.focusNode);

  final FocusNode focusNode;

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) => Focus(
    focusNode: focusNode,
    onKeyEvent: (_, event) =>
        event.logicalKey == LogicalKeyboardKey.arrowLeft ||
            event.logicalKey == LogicalKeyboardKey.arrowRight
        ? KeyEventResult.handled
        : KeyEventResult.ignored,
    child: const ColoredBox(key: Key('test-pdf-viewer'), color: Colors.white),
  );
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

final class _ControlledOpenPdfEngine extends StubPdfDocumentEngine {
  final Map<String, Completer<void>> _requests = {};
  final Map<String, Completer<OpenedPdfDocument>> _opens = {};
  final List<String> closedDocumentIds = [];

  Future<void> waitForRequest(String path) =>
      (_requests[path] ??= Completer<void>()).future;

  void complete(String path, {required String documentId}) {
    final completer = _opens[path];
    if (completer == null) {
      throw StateError('No pending open request for $path.');
    }
    completer.complete(
      OpenedPdfDocument(
        documentId: documentId,
        path: path,
        pageCount: 2,
        pageSizes: const [PdfPageSize(600, 800), PdfPageSize(600, 800)],
        isEncrypted: false,
      ),
    );
  }

  @override
  Future<OpenedPdfDocument> openDocument(String path, {String? password}) {
    final completer = Completer<OpenedPdfDocument>();
    _opens[path] = completer;
    (_requests[path] ??= Completer<void>()).complete();
    return completer.future;
  }

  @override
  Future<void> closeDocument(String documentId) async {
    closedDocumentIds.add(documentId);
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
    double devicePixelRatio = 1,
    int? generation,
    PdfRenderPriority priority = PdfRenderPriority.visible,
  }) async {
    throw StateError('No bitmap in the widget harness.');
  }
}
