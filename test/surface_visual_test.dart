import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/shortcuts/app_commands.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/features/command_palette/presentation/command_palette.dart';
import 'package:yamantaka/features/help/presentation/welcome_dialog.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';

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

  testWidgets('captures palette, menu and dialog widget evidence', (
    tester,
  ) async {
    const paletteRoot = String.fromEnvironment('YAMANTAKA_PALETTE_MATRIX_DIR');
    const dialogsRoot = String.fromEnvironment('YAMANTAKA_DIALOGS_MATRIX_DIR');
    if (paletteRoot.isEmpty && dialogsRoot.isEmpty) return;
    final paletteOutput = paletteRoot.isEmpty
        ? null
        : (Directory(paletteRoot)..createSync(recursive: true));
    final dialogsOutput = dialogsRoot.isEmpty
        ? null
        : (Directory(dialogsRoot)..createSync(recursive: true));
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const boundaryKey = Key('surface-visual-boundary');
    final paletteCases = <Map<String, Object?>>[];
    final dialogCases = <Map<String, Object?>>[];
    final now = DateTime.utc(2026, 8, 25, 12);
    final recent = LibraryDocument(
      id: 'visual-recent',
      path: 'cuaderno-local.pdf',
      normalizedPath: 'cuaderno-local.pdf',
      fileName: 'cuaderno-local.pdf',
      title: 'Cuaderno de lectura',
      author: 'Archivo local',
      pageCount: 184,
      fileSize: 1024,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      lastPageIndex: 76,
      readingProgress: 0.42,
      fingerprint: 'visual-recent',
    );

    Future<void> capture(String path) async {
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
    }

    Future<void> paletteCase({
      required String name,
      required Size size,
      ThemeMode mode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
    }) async {
      if (paletteOutput == null) return;
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: MaterialApp(
            key: ValueKey('palette-$name'),
            debugShowCheckedModeBanner: false,
            theme: AppTheme.light(highContrast: highContrast),
            darkTheme: AppTheme.dark(highContrast: highContrast),
            themeMode: mode,
            builder: (context, child) => MediaQuery(
              data: MediaQuery.of(context).copyWith(
                highContrast: highContrast,
                disableAnimations: reduceMotion,
              ),
              child: child!,
            ),
            home: Scaffold(
              body: CommandPalette(
                actions: {
                  for (final definition in AppCommandRegistry.definitions)
                    definition.id: () {},
                },
                recentDocuments: [recent],
                onOpenRecent: (_) {},
                reduceMotion: reduceMotion,
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 250));
      final path = '${paletteOutput.path}${Platform.pathSeparator}$name.png';
      await capture(path);
      paletteCases.add({
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'theme': mode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'surface': 'palette',
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

    await paletteCase(
      name: 'palette-640x480-light-widget',
      size: const Size(640, 480),
    );
    await paletteCase(
      name: 'palette-1280x720-light-widget',
      size: const Size(1280, 720),
    );
    await paletteCase(
      name: 'palette-1280x720-dark-widget',
      size: const Size(1280, 720),
      mode: ThemeMode.dark,
    );
    await paletteCase(
      name: 'palette-1280x720-high-contrast-widget',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await paletteCase(
      name: 'palette-1280x720-reduced-motion-widget',
      size: const Size(1280, 720),
      reduceMotion: true,
    );

    if (paletteOutput != null) {
      tester.view.physicalSize = const Size(1280, 720);
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: MaterialApp(
            key: const ValueKey('menu-open'),
            theme: AppTheme.light(),
            home: const Scaffold(body: Center(child: _MenuFixture())),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('surface-menu-button')));
      await tester.pumpAndSettle();
      final path =
          '${paletteOutput.path}${Platform.pathSeparator}menu-1280x720-open-widget.png';
      await capture(path);
      paletteCases.add({
        'name': 'menu-1280x720-open-widget',
        'width': 1280,
        'height': 720,
        'theme': 'light',
        'surface': 'menu-open',
        'source': 'flutter-widget-test',
        'path': path,
      });
      File(
        '${paletteOutput.path}${Platform.pathSeparator}palette-matrix.json',
      ).writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'format': 'yamantaka-palette-widget-matrix',
          'generatedAt': DateTime.now().toUtc().toIso8601String(),
          'cases': paletteCases,
        }),
      );
    }

    Future<void> dialogCase({
      required String name,
      required Size size,
      ThemeMode mode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
    }) async {
      if (dialogsOutput == null) return;
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: MaterialApp(
            key: ValueKey('dialog-$name'),
            debugShowCheckedModeBanner: false,
            theme: AppTheme.light(highContrast: highContrast),
            darkTheme: AppTheme.dark(highContrast: highContrast),
            themeMode: mode,
            builder: (context, child) => MediaQuery(
              data: MediaQuery.of(context).copyWith(
                highContrast: highContrast,
                disableAnimations: reduceMotion,
              ),
              child: child!,
            ),
            home: Builder(
              builder: (context) => Scaffold(
                body: Center(
                  child: FilledButton(
                    onPressed: () => showDialog<WelcomeAction>(
                      context: context,
                      builder: (_) => const WelcomeDialog(),
                    ),
                    child: const Text('Open'),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      final path = '${dialogsOutput.path}${Platform.pathSeparator}$name.png';
      await capture(path);
      dialogCases.add({
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'theme': mode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'surface': 'dialog',
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

    await dialogCase(
      name: 'dialog-640x480-light-widget',
      size: const Size(640, 480),
    );
    await dialogCase(
      name: 'dialog-1280x720-light-widget',
      size: const Size(1280, 720),
    );
    await dialogCase(
      name: 'dialog-1280x720-dark-widget',
      size: const Size(1280, 720),
      mode: ThemeMode.dark,
    );
    await dialogCase(
      name: 'dialog-1280x720-high-contrast-widget',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await dialogCase(
      name: 'dialog-1280x720-reduced-motion-widget',
      size: const Size(1280, 720),
      reduceMotion: true,
    );
    if (dialogsOutput != null) {
      File(
        '${dialogsOutput.path}${Platform.pathSeparator}dialogs-matrix.json',
      ).writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'format': 'yamantaka-dialog-widget-matrix',
          'generatedAt': DateTime.now().toUtc().toIso8601String(),
          'cases': dialogCases,
        }),
      );
    }
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

final class _MenuFixture extends StatelessWidget {
  const _MenuFixture();

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<String>(
      key: const Key('surface-menu-button'),
      tooltip: 'Open menu',
      itemBuilder: (context) => const [
        PopupMenuItem(
          value: 'open',
          child: ListTile(
            contentPadding: EdgeInsets.zero,
            leading: Icon(Icons.file_open_outlined),
            title: Text('Open document'),
            trailing: Text('Ctrl+O'),
          ),
        ),
        CheckedPopupMenuItem(
          value: 'selected',
          checked: true,
          child: Text('Selected option'),
        ),
        PopupMenuDivider(),
        PopupMenuItem(
          value: 'delete',
          child: ListTile(
            contentPadding: EdgeInsets.zero,
            leading: Icon(Icons.delete_outline_rounded),
            title: Text('Remove locally'),
          ),
        ),
      ],
      icon: const Icon(Icons.more_horiz_rounded),
    );
  }
}
