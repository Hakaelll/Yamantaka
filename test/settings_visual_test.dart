import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/app_services.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/settings/application/settings_controller.dart';
import 'package:yamantaka/features/settings/presentation/settings_screen.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

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

  testWidgets('captures the requested settings widget visual matrix', (
    tester,
  ) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_SETTINGS_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final directory = Directory.systemTemp.createTempSync(
      'yamantaka-settings-matrix-',
    );
    final cache = Directory('${directory.path}/cache')..createSync();
    final thumbnails = Directory('${directory.path}/covers')..createSync();
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final settings = SettingsController(
      AppSettingsRepository(_MemoryPreferencesStore()),
      logger,
    );
    late AppServices services;
    await tester.runAsync(() async {
      await settings.load();
      services = await AppServices.create(
        paths: AppPaths(
          dataDirectory: directory,
          cacheDirectory: cache,
          thumbnailDirectory: thumbnails,
        ),
        logger: logger,
        preloadedSettings: settings,
      );
    });
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const captureKey = Key('settings-visual-matrix-boundary');
    final evidence = <Map<String, Object?>>[];

    Future<void> capture({
      required String sizeName,
      required Size size,
      ThemeMode themeMode = ThemeMode.light,
      bool highContrast = false,
      bool reduceMotion = false,
    }) async {
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        MaterialApp(
          debugShowCheckedModeBanner: false,
          theme: AppTheme.light(highContrast: highContrast),
          darkTheme: AppTheme.dark(highContrast: highContrast),
          themeMode: themeMode,
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context).copyWith(
              highContrast: highContrast,
              disableAnimations: reduceMotion,
            ),
            child: child!,
          ),
          home: RepaintBoundary(
            key: captureKey,
            child: SettingsScreen(services: services, onBack: () {}),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 400));
      final qualifiers = [
        themeMode == ThemeMode.dark ? 'dark' : 'light',
        if (highContrast) 'high-contrast',
        if (reduceMotion) 'reduced-motion',
        'widget',
      ].join('-');
      final name = 'settings-$sizeName-$qualifiers';
      final path = '${output.path}${Platform.pathSeparator}$name.png';
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(captureKey),
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
      evidence.add({
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'theme': themeMode.name,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'source': 'flutter-widget-test',
        'path': path,
      });
    }

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
      await capture(sizeName: entry.$1, size: entry.$2);
    }
    await capture(
      sizeName: '1280x720',
      size: const Size(1280, 720),
      themeMode: ThemeMode.dark,
    );
    await capture(
      sizeName: '1280x720',
      size: const Size(1280, 720),
      highContrast: true,
    );
    await capture(
      sizeName: '1280x720',
      size: const Size(1280, 720),
      reduceMotion: true,
    );
    File(
      '${output.path}${Platform.pathSeparator}settings-matrix.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-settings-widget-matrix',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'source': 'flutter-widget-test',
        'cases': evidence,
      }),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    services.fileValidation.dispose();
    services.enrichment.dispose();
    services.folderWatch.dispose();
    services.reader.dispose();
    services.globalSearch.dispose();
    services.libraryIndex.dispose();
    services.library.dispose();
    services.performance.stopFrameMonitoring();
    services.lifecycle.dispose();
    services.settings.dispose();
    services.database.markCleanShutdown();
    services.database.close();
    directory.deleteSync(recursive: true);
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

final class _MemoryPreferencesStore implements AppPreferencesStore {
  final Map<String, Object> _values = {};

  @override
  Future<bool?> getBool(String key) async => _values[key] as bool?;

  @override
  Future<double?> getDouble(String key) async => _values[key] as double?;

  @override
  Future<int?> getInt(String key) async => _values[key] as int?;

  @override
  Future<String?> getString(String key) async => _values[key] as String?;

  @override
  Future<void> setBool(String key, bool value) async => _values[key] = value;

  @override
  Future<void> setDouble(String key, double value) async =>
      _values[key] = value;

  @override
  Future<void> setInt(String key, int value) async => _values[key] = value;

  @override
  Future<void> setString(String key, String value) async =>
      _values[key] = value;

  @override
  Future<void> clear() async => _values.clear();
}
