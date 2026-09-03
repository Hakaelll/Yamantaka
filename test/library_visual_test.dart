import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/domain/reading_session.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

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

  testWidgets('captures the 0.13 library matrix', (tester) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_LIBRARY_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final repository = DocumentRepository(database);
    final now = DateTime.utc(2026, 8, 26, 12);
    for (var index = 0; index < 14; index++) {
      repository.save(
        _document(
          index: index,
          now: now.subtract(Duration(days: index)),
          missing: index == 3,
          changed: index == 6,
        ),
      );
    }
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    await controller.load();
    addTearDown(() {
      controller.dispose();
      database.close();
    });
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const boundaryKey = Key('library-visual-boundary');
    final cases = <Map<String, Object?>>[];

    Future<void> captureCase({
      required String name,
      required Size size,
      bool empty = false,
      bool list = false,
      bool selected = false,
      bool filtered = false,
      bool highContrast = false,
      bool reduceMotion = false,
      ThemeMode themeMode = ThemeMode.light,
      double textScale = 1,
      LibraryGrouping grouping = LibraryGrouping.none,
    }) async {
      controller
        ..query = ''
        ..readingState = filtered ? 'inProgress' : 'all'
        ..sortOrder = 'lastOpened'
        ..favoritesOnly = false
        ..selectedAuthors = const {}
        ..selectedTopics = const {}
        ..grouping = grouping
        ..selectDocument(null)
        ..scrollOffset = 0;
      await controller.reload();
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: MaterialApp(
            key: ValueKey('library-$name'),
            debugShowCheckedModeBanner: false,
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
            home: empty
                ? Theme(
                    data: AppTheme.library(highContrast: highContrast),
                    child: Scaffold(body: EmptyLibraryState(onOpen: () {})),
                  )
                : LibraryScreen(
                    controller: controller,
                    onOpenPath: (_) async {},
                    onPickDocument: () async {},
                    onOpenSettings: () {},
                    showRecents: true,
                    recentlyClosed: [
                      RecentlyClosedDocument(
                        documentId: 'recent-1',
                        path: r'C:\Archivo privado\cerrado.pdf',
                        title: 'Ensayo cerrado recientemente',
                        pageIndex: 18,
                        zoomLevel: 1.2,
                        pageLayout: 'single',
                        readingMode: 'document',
                        readingTheme: 'ivory',
                        scrollOffset: 0.3,
                        closedAt: DateTime.utc(2026, 8, 27),
                      ),
                    ],
                    onRestoreClosed: (_) {},
                    onDismissClosed: (_) {},
                  ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 250));
      if (list) {
        await tester.tap(find.byKey(const Key('library-list-view-button')));
        await tester.pumpAndSettle();
      }
      if (selected) {
        final key = ValueKey(
          list ? 'artifact-list-document-0' : 'artifact-document-0',
        );
        await tester.tap(find.byKey(key), warnIfMissed: false);
        await tester.pump(const Duration(milliseconds: 250));
      }
      expect(tester.takeException(), isNull, reason: name);
      final path = '${output.path}${Platform.pathSeparator}$name.png';
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(boundaryKey),
      );
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        try {
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          File(path).writeAsBytesSync(bytes!.buffer.asUint8List(), flush: true);
        } finally {
          image.dispose();
        }
      });
      cases.add({
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'view': list ? 'list' : 'grid',
        'empty': empty,
        'selected': selected,
        'filtered': filtered,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'theme': themeMode.name,
        'textScale': textScale,
        'grouping': grouping.name,
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

    await captureCase(
      name: 'library-empty-640x480-light-widget',
      size: const Size(640, 480),
      empty: true,
    );
    await captureCase(
      name: 'library-empty-1280x720-light-widget',
      size: const Size(1280, 720),
      empty: true,
    );
    await captureCase(
      name: 'library-many-640x480-light-widget',
      size: const Size(640, 480),
    );
    await captureCase(
      name: 'library-many-1280x720-light-widget',
      size: const Size(1280, 720),
    );
    await captureCase(
      name: 'library-many-1600x900-light-widget',
      size: const Size(1600, 900),
    );
    for (final entry in const [
      ('800x600', Size(800, 600)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ]) {
      await captureCase(
        name: 'library-many-${entry.$1}-light-widget',
        size: entry.$2,
      );
    }
    await captureCase(
      name: 'library-selected-1280x720-light-widget',
      size: const Size(1280, 720),
      selected: true,
    );
    await captureCase(
      name: 'library-filtered-1280x720-light-widget',
      size: const Size(1280, 720),
      filtered: true,
    );
    await captureCase(
      name: 'library-grouped-state-1280x720-light-widget',
      size: const Size(1280, 720),
      list: true,
      grouping: LibraryGrouping.readingState,
    );
    await captureCase(
      name: 'library-list-1280x720-light-widget',
      size: const Size(1280, 720),
      list: true,
    );
    await captureCase(
      name: 'library-list-selected-1600x900-light-widget',
      size: const Size(1600, 900),
      list: true,
      selected: true,
    );
    await captureCase(
      name: 'library-many-1280x720-dark-widget',
      size: const Size(1280, 720),
      themeMode: ThemeMode.dark,
    );
    await captureCase(
      name: 'library-many-1280x720-high-contrast-widget',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await captureCase(
      name: 'library-many-1280x720-reduced-motion-widget',
      size: const Size(1280, 720),
      reduceMotion: true,
    );
    await captureCase(
      name: 'library-list-1280x720-text-130-widget',
      size: const Size(1280, 720),
      list: true,
      textScale: 1.3,
    );
    await captureCase(
      name: 'library-list-1100x720-text-150-widget',
      size: const Size(1100, 720),
      list: true,
      textScale: 1.5,
    );
    await captureCase(
      name: 'library-list-1280x720-text-200-widget',
      size: const Size(1280, 720),
      list: true,
      textScale: 2,
    );

    File(
      '${output.path}${Platform.pathSeparator}matrix-report.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-library-visual-matrix',
        'version': 1,
        'productVersion': '0.13.0',
        'generatedAt': '2026-08-26T12:00:00Z',
        'cases': cases,
      }),
      flush: true,
    );
  });
}

LibraryDocument _document({
  required int index,
  required DateTime now,
  required bool missing,
  required bool changed,
}) {
  final progress = switch (index % 4) {
    0 => 0.64,
    1 => 0.0,
    2 => 1.0,
    _ => 0.22,
  };
  return LibraryDocument(
    id: 'document-$index',
    path: 'C:\\Archivo privado\\documento-$index.pdf',
    normalizedPath: 'c:/archivo privado/documento-$index.pdf',
    fileName: 'documento-$index.pdf',
    title: [
      'Atlas de lectura privada',
      'Notas sobre arquitectura',
      'Historia del archivo doméstico',
      'Ensayo de precisión editorial',
    ][index % 4],
    author: ['Ana Vidal', 'Luis Serra', 'Mara León'][index % 3],
    pageCount: 84 + index * 13,
    fileSize: 1024 * 1024 * (2 + index),
    fileModifiedAt: now,
    addedAt: now.subtract(const Duration(days: 12)),
    lastOpenedAt: now,
    lastPageIndex: (progress * (84 + index * 13)).floor(),
    readingProgress: progress,
    isMissing: missing,
    hasFileChanged: changed,
    isFavorite: index == 1 || index == 8,
    fingerprint: 'visual-$index',
    libraryNote: index == 0 ? 'Revisar el capítulo de navegación.' : null,
  );
}

Future<void> _loadTestFont(String family, List<String> candidates) async {
  File? fontFile;
  for (final path in candidates) {
    final file = File(path);
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
