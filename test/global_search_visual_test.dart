import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/search/application/global_search_controller.dart';
import 'package:yamantaka/features/search/application/library_index_controller.dart';
import 'package:yamantaka/features/search/domain/global_search_models.dart';
import 'package:yamantaka/features/search/presentation/global_search_dialog.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    final windowsRoot = Platform.environment['WINDIR'] ?? r'C:\Windows';
    await _loadTestFont('Segoe UI Variable', [
      '$windowsRoot/Fonts/segoeui.ttf',
      '$windowsRoot/Fonts/SegoeUIVF.ttf',
    ]);
    await _loadTestFont('MaterialIcons', [
      r'.flutter-sdk\bin\cache\artifacts\material_fonts\MaterialIcons-Regular.otf',
    ]);
  });

  testWidgets('captures global search 0.12 visual evidence', (tester) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_SEARCH_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = GlobalSearchRepository(database);
    final controller = GlobalLibrarySearchController(repository)
      ..query = 'archivo'
      ..results = const [
        GlobalSearchResult(
          documentId: 'doc-1',
          pageIndex: null,
          kind: GlobalSearchKind.metadata,
          title: 'Cuaderno de lectura',
          snippet: 'Archivo local · edición revisada',
          rank: 0.1,
        ),
        GlobalSearchResult(
          documentId: 'doc-1',
          pageIndex: 23,
          kind: GlobalSearchKind.pageText,
          title: 'Cuaderno de lectura',
          snippet: '… el archivo conserva la memoria del documento …',
          rank: 0.2,
        ),
        GlobalSearchResult(
          documentId: 'doc-2',
          pageIndex: 8,
          kind: GlobalSearchKind.annotation,
          title: 'Notas de campo',
          snippet: 'Revisar esta anotación antes de continuar.',
          rank: 0.3,
        ),
        GlobalSearchResult(
          documentId: 'collection-1',
          pageIndex: null,
          kind: GlobalSearchKind.collection,
          title: 'Lecturas pendientes',
          snippet: 'Colección local · 12 documentos',
          rank: 0.4,
        ),
        GlobalSearchResult(
          documentId: 'tag-1',
          pageIndex: null,
          kind: GlobalSearchKind.tag,
          title: 'investigación',
          snippet: 'Etiqueta local',
          rank: 0.5,
        ),
      ];
    addTearDown(controller.dispose);
    final index = LibraryIndexController(
      DocumentRepository(database),
      repository,
      StubPdfDocumentEngine(),
    )..setEnabled(true);
    addTearDown(index.dispose);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const boundaryKey = Key('global-search-visual-boundary');
    final cases = <Map<String, Object?>>[];

    Future<void> capture({
      required String name,
      required Size size,
      ThemeMode themeMode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
      double textScale = 1,
      bool compactDetail = false,
    }) async {
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: MaterialApp(
            key: ValueKey(name),
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
            home: Scaffold(
              body: GlobalSearchDialog(
                controller: controller,
                indexController: index,
                onOpen: (_) {},
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 250));
      if (compactDetail) {
        await tester.tap(find.text('Cuaderno de lectura').first);
        await tester.pump();
      }
      expect(tester.takeException(), isNull, reason: name);
      final closeRect = tester.getRect(find.byIcon(Icons.close_rounded));
      expect(
        closeRect.top >= 0 &&
            closeRect.left >= 0 &&
            closeRect.right <= size.width &&
            closeRect.bottom <= size.height,
        isTrue,
        reason:
            '$name must keep its close action inside the viewport: '
            '$closeRect',
      );
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
        'theme': themeMode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'textScale': textScale,
        'view': compactDetail ? 'detail' : 'list',
        'closeActionBounds': {
          'left': closeRect.left,
          'top': closeRect.top,
          'right': closeRect.right,
          'bottom': closeRect.bottom,
        },
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

    await capture(
      name: 'search-640x480-light-widget',
      size: const Size(640, 480),
    );
    await capture(
      name: 'search-640x480-detail-light-widget',
      size: const Size(640, 480),
      compactDetail: true,
    );
    await capture(
      name: 'search-640x480-detail-text-200-widget',
      size: const Size(640, 480),
      textScale: 2,
      compactDetail: true,
    );
    await capture(
      name: 'search-1280x720-light-widget',
      size: const Size(1280, 720),
    );
    for (final entry in const [
      ('800x600', Size(800, 600)),
      ('1600x900', Size(1600, 900)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ]) {
      await capture(name: 'search-${entry.$1}-light-widget', size: entry.$2);
    }
    await capture(
      name: 'search-1280x720-dark-widget',
      size: const Size(1280, 720),
      themeMode: ThemeMode.dark,
    );
    await capture(
      name: 'search-1280x720-high-contrast-widget',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await capture(
      name: 'search-1280x720-reduced-motion-widget',
      size: const Size(1280, 720),
      reduceMotion: true,
    );
    await capture(
      name: 'search-1280x720-text-130-widget',
      size: const Size(1280, 720),
      textScale: 1.3,
    );
    await capture(
      name: 'search-1280x720-text-150-widget',
      size: const Size(1280, 720),
      textScale: 1.5,
    );
    await capture(
      name: 'search-1280x720-text-200-widget',
      size: const Size(1280, 720),
      textScale: 2,
    );

    File(
      '${output.path}${Platform.pathSeparator}search-matrix.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-global-search-widget-matrix',
        'productVersion': '0.12.0',
        'generatedAt': '2026-08-26T12:00:00Z',
        'cases': cases,
      }),
      flush: true,
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
