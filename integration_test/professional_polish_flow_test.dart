import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yamantaka/app/app.dart';
import 'package:yamantaka/app/app_services.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/application/reader_controller.dart';
import 'package:yamantaka/features/reader/presentation/reader_screen.dart';
import 'package:yamantaka/features/settings/application/settings_controller.dart';
import 'package:yamantaka/features/settings/presentation/settings_screen.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

import '../test/support/pdf_fixture.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'native app preserves reading context across library and settings',
    (tester) async {
      const outputPath = String.fromEnvironment('YAMANTAKA_NATIVE_FLOW_DIR');
      expect(
        outputPath,
        isNotEmpty,
        reason: 'An explicit evidence directory is required.',
      );
      final output = Directory(outputPath)..createSync(recursive: true);
      tester.view.physicalSize = const Size(1280, 720);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await windowManager.ensureInitialized();
      await pdfrxFlutterInitialize();
      final root = await Directory.systemTemp.createTemp(
        'yamantaka-polish-flow-',
      );
      final pdf = File('${root.path}/Cuaderno de lectura.pdf');
      await pdf.writeAsBytes(createPdfWithPageCount(12));
      final pdfHash = sha256.convert(await pdf.readAsBytes()).toString();
      final cache = await Directory('${root.path}/cache').create();
      final covers = await Directory('${root.path}/covers').create();
      final logger = AppLogger(minimumLevel: LogLevel.error);
      final settings = SettingsController(
        AppSettingsRepository(_MemoryPreferences()),
        logger,
      );
      await settings.load();
      await settings.updateStrict(
        settings.settings.copyWith(
          hasSeenWelcome: true,
          darkMode: false,
          languageCode: 'es',
          autoHideControls: false,
        ),
      );
      final services = await AppServices.create(
        paths: AppPaths(
          dataDirectory: root,
          cacheDirectory: cache,
          thumbnailDirectory: covers,
        ),
        logger: logger,
        preloadedSettings: settings,
      );
      const boundaryKey = Key('native-polish-flow-boundary');
      final captures = <Map<String, Object?>>[];
      Future<void> waitFor(bool Function() ready) async {
        for (var attempt = 0; attempt < 200; attempt++) {
          await tester.pump(const Duration(milliseconds: 100));
          if (ready()) {
            await tester.pumpAndSettle();
            return;
          }
        }
        fail('Native application did not reach its expected state.');
      }

      Future<void> capture(String name) async {
        await tester.pumpAndSettle();
        final boundary = tester.renderObject<RenderRepaintBoundary>(
          find.byKey(boundaryKey),
        );
        final rendered = await boundary.toImage(pixelRatio: 1);
        final bytes = (await rendered.toByteData(
          format: ui.ImageByteFormat.png,
        ))!.buffer.asUint8List();
        rendered.dispose();
        final path = '${output.path}/$name.png';
        await File(path).writeAsBytes(bytes, flush: true);
        captures.add({
          'name': name,
          'path': path,
          'sha256': sha256.convert(bytes).toString(),
          'source': 'native-flutter-integration-repaint-boundary-real-pdfium',
          'width': 1280,
          'height': 720,
          'textScale': 1,
        });
        expect(tester.takeException(), isNull);
      }

      await tester.pumpWidget(
        RepaintBoundary(
          key: boundaryKey,
          child: YamantakaApp(services: services, initialPath: pdf.path),
        ),
      );
      await waitFor(
        () =>
            services.reader.status == ReaderStatus.ready &&
            find.byType(ReaderScreen).evaluate().isNotEmpty,
      );
      expect(services.reader.openedDocument?.pageCount, 12);
      await tester.tap(
        find
            .byWidgetPredicate(
              (widget) =>
                  widget is Tooltip &&
                  widget.message?.startsWith('Página siguiente') == true,
            )
            .first,
      );
      await waitFor(() => services.reader.pageIndexListenable.value == 1);
      await capture('reader-document-light');
      await tester.tap(find.byTooltip('Volver a la biblioteca'));
      await waitFor(() => find.byType(LibraryScreen).evaluate().isNotEmpty);
      expect(services.library.totalDocuments, 1);
      await capture('library-populated-light');
      await tester.tap(find.byTooltip('Organizar y más acciones'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ajustes'));
      await waitFor(() => find.byType(SettingsScreen).evaluate().isNotEmpty);
      await capture('settings-light');
      await tester.tap(find.byType(Switch).first);
      await waitFor(() => services.settings.settings.darkMode);
      // Dismiss explicitly: actionable notices can persist for accessibility.
      await waitFor(() => find.byType(SnackBar).evaluate().isNotEmpty);
      await tester.tap(
        find.descendant(
          of: find.byType(SnackBar),
          matching: find.byIcon(Icons.close),
        ),
      );
      await waitFor(() => find.byType(SnackBar).evaluate().isEmpty);
      expect(services.settings.settings.darkMode, isTrue);
      await tester.tap(find.byTooltip('Volver').first);
      await waitFor(() => find.byType(LibraryScreen).evaluate().isNotEmpty);
      expect(
        Theme.of(tester.element(find.byType(LibraryScreen))).brightness,
        Brightness.dark,
      );
      await capture('library-populated-dark');
      await tester.tap(find.byKey(const Key('continue-reading-strip')));
      await waitFor(
        () =>
            services.reader.status == ReaderStatus.ready &&
            find.byType(ReaderScreen).evaluate().isNotEmpty,
      );
      expect(services.reader.pageIndexListenable.value, 1);
      await tester.sendKeyDownEvent(
        LogicalKeyboardKey.controlLeft,
        physicalKey: PhysicalKeyboardKey.controlLeft,
      );
      await tester.sendKeyEvent(
        LogicalKeyboardKey.keyF,
        physicalKey: PhysicalKeyboardKey.keyF,
      );
      await tester.sendKeyUpEvent(
        LogicalKeyboardKey.controlLeft,
        physicalKey: PhysicalKeyboardKey.controlLeft,
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('pdf-search-field')), findsOneWidget);
      await tester.sendKeyEvent(
        LogicalKeyboardKey.escape,
        physicalKey: PhysicalKeyboardKey.escape,
      );
      await capture('reader-restored-dark');
      expect(sha256.convert(await pdf.readAsBytes()).toString(), pdfHash);
      final report = <String, Object?>{
        'format': 'yamantaka-native-polish-flow',
        'productVersion': '0.15.0',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'status': 'passed',
        'pdfUnchanged': true,
        'restoredPageIndex': 1,
        'personalDataUsed': false,
        'physicalDpiValidation': false,
        'cases': captures,
      };
      await File(
        '${output.path}/native-flow.json',
      ).writeAsString(const JsonEncoder.withIndent('  ').convert(report));
      binding.reportData = report;
      // YamantakaApp owns AppServices and disposes it after saving window bounds.
      await services.reader.closeDocument();
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pumpAndSettle();
    },
  );
}

final class _MemoryPreferences implements AppPreferencesStore {
  final _values = <String, Object>{};
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
