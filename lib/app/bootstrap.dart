import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:window_manager/window_manager.dart';

import '../core/logging/app_logger.dart';
import '../core/performance/startup_timeline.dart';
import '../core/platform/single_instance_protocol.dart';
import '../features/settings/application/settings_controller.dart';
import '../storage/paths/app_paths.dart';
import '../storage/repositories/app_settings_repository.dart';
import 'app.dart';
import 'app_config.dart';
import 'app_services.dart';

Future<void> bootstrap(List<String> arguments) async {
  final startupStopwatch = Stopwatch()..start();
  final startup = StartupTimeline();
  final bindingSpan = startup.start(StartupPhase.binding);
  WidgetsFlutterBinding.ensureInitialized();
  bindingSpan.end();
  final logger = AppLogger(
    minimumLevel: kReleaseMode ? LogLevel.warning : LogLevel.debug,
  );
  final pathsSpan = startup.start(StartupPhase.paths);
  final paths = await AppPaths.create();
  pathsSpan.end();
  final settingsSpan = startup.start(StartupPhase.settings);
  final settings = SettingsController(AppSettingsRepository(), logger);
  await settings.load();
  settingsSpan.end();
  final nativePdfSpan = startup.start(StartupPhase.nativePdf);
  Pdfrx.cacheDirectoryPath = paths.cacheDirectory.path;
  await pdfrxFlutterInitialize();
  nativePdfSpan.end();
  final windowSpan = startup.start(StartupPhase.window);
  if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    await windowManager.ensureInitialized();
    final stored = settings.settings;
    final presentation = resolveInitialWindowPresentation(stored);
    final options = WindowOptions(
      size: presentation.normalSize,
      minimumSize: Size(640, 480),
      center: presentation.normalPosition == null,
      title: AppConfig.productName,
      backgroundColor: const Color(0xFFF4F3EF),
    );
    await windowManager.waitUntilReadyToShow(options, () async {
      if (presentation.normalPosition != null) {
        await windowManager.setPosition(presentation.normalPosition!);
      }
      if (presentation.maximized) await windowManager.maximize();
      await windowManager.show();
      await windowManager.focus();
    });
  }
  windowSpan.end();
  final servicesSpan = startup.start(StartupPhase.services);
  final services = await AppServices.create(
    paths: paths,
    logger: logger,
    preloadedSettings: settings,
    startupTimeline: startup,
  );
  servicesSpan.end();
  final initialPaths = initialPdfPaths(arguments);
  logger.info(
    'Bootstrap completed in ${startupStopwatch.elapsedMilliseconds} ms.',
  );
  startup.start(StartupPhase.firstFrame);
  runApp(YamantakaApp(services: services, initialPaths: initialPaths));
}

@immutable
final class InitialWindowPresentation {
  const InitialWindowPresentation({
    required this.normalSize,
    required this.normalPosition,
    required this.maximized,
  });

  final Size normalSize;
  final Offset? normalPosition;
  final bool maximized;
}

InitialWindowPresentation resolveInitialWindowPresentation(
  AppSettings settings,
) => InitialWindowPresentation(
  normalSize: Size(
    settings.windowWidth.clamp(640, 2400),
    settings.windowHeight.clamp(480, 1600),
  ),
  normalPosition: settings.windowX == null || settings.windowY == null
      ? null
      : Offset(settings.windowX!, settings.windowY!),
  maximized: true,
);

String? initialPdfPath(List<String> arguments) {
  final paths = SingleInstanceProtocol.fromArguments(arguments).pdfPaths;
  return paths.isEmpty ? null : paths.last;
}

List<String> initialPdfPaths(List<String> arguments) {
  return SingleInstanceProtocol.fromArguments(arguments).pdfPaths;
}
