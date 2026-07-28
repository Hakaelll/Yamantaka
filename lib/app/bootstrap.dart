import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:window_manager/window_manager.dart';

import '../core/logging/app_logger.dart';
import '../features/settings/application/settings_controller.dart';
import '../storage/paths/app_paths.dart';
import '../storage/repositories/app_settings_repository.dart';
import 'app.dart';
import 'app_config.dart';
import 'app_services.dart';

Future<void> bootstrap(List<String> arguments) async {
  final startupStopwatch = Stopwatch()..start();
  WidgetsFlutterBinding.ensureInitialized();
  final logger = AppLogger(
    minimumLevel: kReleaseMode ? LogLevel.warning : LogLevel.debug,
  );
  final paths = await AppPaths.create();
  final settings = SettingsController(AppSettingsRepository(), logger);
  await settings.load();
  Pdfrx.cacheDirectoryPath = paths.cacheDirectory.path;
  await pdfrxFlutterInitialize();
  if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    await windowManager.ensureInitialized();
    final stored = settings.settings;
    final hasPosition = stored.windowX != null && stored.windowY != null;
    final options = WindowOptions(
      size: Size(
        stored.windowWidth.clamp(640, 2400),
        stored.windowHeight.clamp(480, 1600),
      ),
      minimumSize: Size(640, 480),
      center: !hasPosition,
      title: AppConfig.productName,
      backgroundColor: const Color(0xFFF4F3EF),
    );
    await windowManager.waitUntilReadyToShow(options, () async {
      if (hasPosition) {
        await windowManager.setPosition(
          Offset(stored.windowX!, stored.windowY!),
        );
      }
      await windowManager.show();
      await windowManager.focus();
    });
  }
  final services = await AppServices.create(
    paths: paths,
    logger: logger,
    preloadedSettings: settings,
  );
  final initialPath = initialPdfPath(arguments);
  logger.info(
    'Bootstrap completed in ${startupStopwatch.elapsedMilliseconds} ms.',
  );
  runApp(YamantakaApp(services: services, initialPath: initialPath));
}

String? initialPdfPath(List<String> arguments) {
  for (final argument in arguments.reversed) {
    final unquoted = argument.replaceAll('"', '');
    if (unquoted.toLowerCase().endsWith('.pdf')) return unquoted;
  }
  return null;
}
