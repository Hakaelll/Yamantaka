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
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

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
        ),
      );
    }
    final controller = LibraryController(documents, logger);
    await controller.load();
    addTearDown(() async {
      controller.dispose();
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
        ),
      ),
    );

    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await _capture(tester, capturePath);

    await tester.tap(find.text('En curso 2'));
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-in-progress'));

    tester.view.physicalSize = const Size(640, 760);
    await tester.pump();
    await _capture(tester, _suffix(capturePath, '-compact'));

    await tester.pumpWidget(app(mode: ThemeMode.dark));
    await tester.pumpAndSettle();
    await _capture(tester, _suffix(capturePath, '-compact-dark'));
  });
}

Future<void> _capture(WidgetTester tester, String path) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(const Key('library-capture-boundary')),
  );
  final image = await boundary.toImage(pixelRatio: 1);
  final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
  final file = File(path);
  file.parent.createSync(recursive: true);
  await file.writeAsBytes(bytes!.buffer.asUint8List(), flush: true);
}

String _suffix(String path, String suffix) =>
    path.replaceFirst(RegExp(r'\.png$'), '$suffix.png');
