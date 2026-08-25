import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/organization/application/organization_controller.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/organization_repository.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('captures reading-state library variants without personal data', (
    tester,
  ) async {
    const capturePath = String.fromEnvironment(
      'YAMANTAKA_LIBRARY_CAPTURE_PATH',
    );
    if (capturePath.isEmpty) return;
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-library-capture-',
    );
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final now = DateTime.utc(2026, 7, 29, 16);
    for (var index = 0; index < 4; index++) {
      final file = File('${directory.path}/sample-$index.pdf');
      await file.writeAsBytes(const [0x25, 0x50, 0x44, 0x46]);
      final cover = await _createCover(directory, index);
      final fileStat = await file.stat();
      documents.save(
        LibraryDocument(
          id: 'sample-$index',
          path: file.path,
          normalizedPath: file.path.toLowerCase(),
          fileName: 'sample-$index.pdf',
          title: const [
            'Arquitectura serena',
            'Notas de montaña',
            'Historia del color',
            'Diseño sin ruido',
          ][index],
          author: const ['A. Uno', 'B. Dos', 'C. Tres', 'D. Cuatro'][index],
          pageCount: 120 + index * 20,
          fileSize: fileStat.size,
          fileModifiedAt: fileStat.modified,
          addedAt: now,
          lastOpenedAt: now.subtract(Duration(days: index)),
          lastPageIndex: const [0, 42, 139, 12][index],
          readingProgress: const [0.0, 0.35, 1.0, 0.08][index],
          fingerprint: 'sample-$index',
          thumbnailPath: cover.path,
        ),
      );
    }
    final controller = LibraryController(documents, logger);
    final organization = OrganizationController(
      OrganizationRepository(database),
    );
    final collection = organization.createCollection('Investigación');
    organization
      ..addDocument(collection.id, 'sample-1')
      ..addDocument(collection.id, 'sample-3');
    await controller.load();
    addTearDown(() async {
      controller.dispose();
      organization.dispose();
      database.close();
      await directory.delete(recursive: true);
    });
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1200, 800);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);

    Widget app({ThemeMode mode = ThemeMode.light}) => RepaintBoundary(
      key: const Key('library-capture-boundary'),
      child: MaterialApp(
        theme: AppTheme.light(),
        darkTheme: AppTheme.dark(),
        themeMode: mode,
        home: LibraryScreen(
          controller: controller,
          onOpenPath: (_) async {},
          onPickDocument: () async {},
          onOpenSettings: () {},
          showRecents: true,
          onDocumentInfo: (_) async {},
          organization: organization,
        ),
      ),
    );

    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('artifact-sample-3')));
    await tester.pumpAndSettle();
    await _capture(tester, capturePath);

    await tester.tap(find.byTooltip('Organizar y más acciones'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Colecciones'));
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-collections'));
    await tester.scrollUntilVisible(
      find.text('Investigación'),
      240,
      scrollable: find
          .descendant(
            of: find.byType(BottomSheet),
            matching: find.byType(Scrollable),
          )
          .first,
    );
    await tester.tap(find.text('Investigación'));
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-collection-filtered'));
    organization.clearSelection();
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('library-filter-button')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('En curso · 2'), warnIfMissed: false);
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-in-progress'));

    tester.view.physicalSize = const Size(640, 760);
    await tester.pump();
    await _capture(tester, _suffix(capturePath, '-compact'));

    await tester.pumpWidget(app(mode: ThemeMode.dark));
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-compact-dark'));
  });

  testWidgets('captures the complete requested library widget matrix', (
    tester,
  ) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_LIBRARY_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-library-matrix-',
    );
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final controller = LibraryController(documents, logger);
    addTearDown(() async {
      controller.dispose();
      database.close();
      await directory.delete(recursive: true);
    });
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    await controller.load();
    final evidence = <Map<String, Object?>>[];

    Widget app({
      required ThemeMode themeMode,
      required bool highContrast,
      required bool reduceMotion,
    }) => MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light(highContrast: highContrast),
      darkTheme: AppTheme.dark(highContrast: highContrast),
      themeMode: themeMode,
      builder: (context, child) => MediaQuery(
        data: MediaQuery.of(
          context,
        ).copyWith(highContrast: highContrast, disableAnimations: reduceMotion),
        child: child!,
      ),
      home: RepaintBoundary(
        key: const Key('library-capture-boundary'),
        child: LibraryScreen(
          controller: controller,
          onOpenPath: (_) async {},
          onPickDocument: () async {},
          onOpenSettings: () {},
          showRecents: true,
          onDocumentInfo: (_) async {},
        ),
      ),
    );

    Future<void> capture({
      required String state,
      required String sizeName,
      required Size size,
      ThemeMode themeMode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
    }) async {
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        app(
          themeMode: themeMode,
          highContrast: highContrast,
          reduceMotion: reduceMotion,
        ),
      );
      await tester.pump(const Duration(milliseconds: 250));
      final qualifiers = [
        themeMode == ThemeMode.dark ? 'dark' : 'light',
        if (highContrast) 'high-contrast',
        if (reduceMotion) 'reduced-motion',
        'widget',
      ].join('-');
      final name = 'library-$state-$sizeName-$qualifiers';
      final path = '${output.path}${Platform.pathSeparator}$name.png';
      await _capture(tester, path);
      evidence.add({
        'name': name,
        'state': state,
        'width': size.width.round(),
        'height': size.height.round(),
        'theme': themeMode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'source': 'flutter-widget-integration-test',
        'path': path,
      });
    }

    const sizes = [
      ('640x480', Size(640, 480)),
      ('800x600', Size(800, 600)),
      ('1280x720', Size(1280, 720)),
      ('1600x900', Size(1600, 900)),
      ('1920x1080', Size(1920, 1080)),
      ('1920x1200', Size(1920, 1200)),
      ('2560x1440', Size(2560, 1440)),
      ('3440x1440', Size(3440, 1440)),
    ];
    for (final size in sizes) {
      await capture(state: 'empty', sizeName: size.$1, size: size.$2);
    }

    final now = DateTime.utc(2026, 8, 24, 12);
    for (var index = 0; index < 4; index++) {
      final cover = await _createCover(directory, index);
      documents.save(
        _matrixDocument(index: index, now: now, coverPath: cover.path),
      );
    }
    await controller.load();
    controller.selectDocument('matrix-0');
    for (final size in sizes) {
      await capture(state: 'few', sizeName: size.$1, size: size.$2);
    }

    for (var index = 4; index < 40; index++) {
      documents.save(_matrixDocument(index: index, now: now));
    }
    await controller.load();
    controller.selectDocument('matrix-0');
    for (final size in sizes) {
      await capture(state: 'many', sizeName: size.$1, size: size.$2);
    }
    await capture(
      state: 'many',
      sizeName: '1280x720',
      size: const Size(1280, 720),
      themeMode: ThemeMode.dark,
    );
    await capture(
      state: 'many',
      sizeName: '1280x720',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await capture(
      state: 'many',
      sizeName: '1280x720',
      size: const Size(1280, 720),
      reduceMotion: true,
    );
    File(
      '${output.path}${Platform.pathSeparator}library-matrix.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-library-widget-matrix',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'source': 'flutter-widget-integration-test',
        'cases': evidence,
      }),
    );
  });
}

LibraryDocument _matrixDocument({
  required int index,
  required DateTime now,
  String? coverPath,
}) => LibraryDocument(
  id: 'matrix-$index',
  path: 'matrix-$index.pdf',
  normalizedPath: 'matrix-$index.pdf',
  fileName: 'matrix-$index.pdf',
  title: 'Documento de muestra ${index + 1}',
  author: 'Autor ${(index % 8) + 1}',
  pageCount: 80 + index * 3,
  fileSize: 1024 * (index + 1),
  fileModifiedAt: now.subtract(Duration(days: index)),
  addedAt: now.subtract(Duration(hours: index)),
  lastOpenedAt: now.subtract(Duration(days: index % 9)),
  lastPageIndex: index % 12,
  readingProgress: (index % 10) / 10,
  fingerprint: 'matrix-$index',
  thumbnailPath: coverPath,
);

Future<void> _capture(WidgetTester tester, String path) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(const Key('library-capture-boundary')),
  );
  final image = await boundary.toImage(pixelRatio: 1);
  final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
  final file = File(path);
  file.parent.createSync(recursive: true);
  file.writeAsBytesSync(bytes!.buffer.asUint8List(), flush: true);
}

String _suffix(String path, String suffix) =>
    path.replaceFirst(RegExp(r'\.png$'), '$suffix.png');

Future<File> _createCover(Directory directory, int index) async {
  const colors = [
    Color(0xFFDDD8CC),
    Color(0xFFB8C0B8),
    Color(0xFFC7BDB6),
    Color(0xFFD1CEC4),
  ];
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  canvas.drawRect(
    const Rect.fromLTWH(0, 0, 400, 560),
    Paint()..color = colors[index],
  );
  canvas.drawRect(
    Rect.fromLTWH(42, 74 + index * 18, 316, 3),
    Paint()..color = const Color(0xFF2C2D29),
  );
  canvas.drawCircle(
    Offset(200, 246 + index * 24),
    52 + index * 8,
    Paint()
      ..color = const Color(0xFF2C2D29)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 5,
  );
  canvas.drawRect(
    const Rect.fromLTWH(42, 472, 150, 10),
    Paint()..color = const Color(0xFF2C2D29),
  );
  final image = await recorder.endRecording().toImage(400, 560);
  final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
  image.dispose();
  final file = File('${directory.path}/cover-$index.png');
  await file.writeAsBytes(bytes!.buffer.asUint8List(), flush: true);
  return file;
}
