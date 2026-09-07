import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_components.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/app/ui/yama_components.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/features/reader/presentation/password_request_dialog.dart';
import 'package:yamantaka/features/recovery/presentation/session_recovery_dialog.dart';
import 'package:yamantaka/features/settings/presentation/backup_restore_preview_dialog.dart';
import 'package:yamantaka/storage/backup/local_backup_service.dart';

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

  testWidgets('captures the 0.15 focal visual and critical-state matrix', (
    tester,
  ) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_014_VISUAL_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final boundaryKey = GlobalKey();
    final evidence = <Map<String, Object?>>[];
    addTearDown(() => tester.binding.setSurfaceSize(null));

    Future<void> capture(
      String surface,
      String name,
      Widget child, {
      Size size = const Size(1280, 800),
      double textScale = 1,
      bool highContrast = false,
      bool reduceMotion = false,
      ThemeMode themeMode = ThemeMode.light,
    }) async {
      await tester.binding.setSurfaceSize(size);
      await tester.pumpWidget(
        _evidenceApp(
          boundaryKey: boundaryKey,
          child: child,
          textScale: textScale,
          highContrast: highContrast,
          reduceMotion: reduceMotion,
          themeMode: themeMode,
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      final directory = Directory('${output.path}/$surface')
        ..createSync(recursive: true);
      final path = '${directory.path}/$name.png';
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(boundaryKey),
      );
      late final List<int> bytes;
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        try {
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          bytes = data!.buffer.asUint8List();
          File(path).writeAsBytesSync(bytes, flush: true);
        } finally {
          image.dispose();
        }
      });
      evidence.add({
        'surface': surface,
        'name': name,
        'width': size.width.round(),
        'height': size.height.round(),
        'textScale': textScale,
        'highContrast': highContrast,
        'reducedMotion': reduceMotion,
        'theme': themeMode.name,
        'sha256': sha256.convert(bytes).toString(),
        'source': 'executed-widget-surface',
        'path': path,
      });
    }

    await capture(
      'components',
      'control-and-operation-states',
      const _ComponentStateBoard(),
    );
    await capture(
      'components',
      'high-contrast-reduced-motion-dark',
      const _ComponentStateBoard(),
      highContrast: true,
      reduceMotion: true,
      themeMode: ThemeMode.dark,
    );
    await capture(
      'password',
      'password-required',
      const PasswordRequestDialog(wrongPassword: false),
    );
    await capture(
      'password',
      'wrong-password-640x480-text-200',
      const PasswordRequestDialog(wrongPassword: true),
      size: const Size(640, 480),
      textScale: 2,
    );
    await capture(
      'recovery',
      'interrupted-session-choice',
      const SessionRecoveryDialog(),
    );
    await capture(
      'restore',
      'transactional-backup-preview',
      BackupRestorePreviewDialog(preview: _backupPreview()),
    );

    File('${output.path}/focal-matrix.json').writeAsStringSync(
      const JsonEncoder.withIndent(' ').convert({
        'format': 'yamantaka-focal-visual-matrix',
        'version': 1,
        'productVersion': '0.15.0',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'cases': evidence,
      }),
      flush: true,
    );
  });
}

final class _ComponentStateBoard extends StatefulWidget {
  const _ComponentStateBoard();

  @override
  State<_ComponentStateBoard> createState() => _ComponentStateBoardState();
}

final class _ComponentStateBoardState extends State<_ComponentStateBoard> {
  final hoverStates = WidgetStatesController({WidgetState.hovered});
  final focusStates = WidgetStatesController({WidgetState.focused});

  @override
  void dispose() {
    hoverStates.dispose();
    focusStates.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(AppSpacing.space40),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const AppSectionHeader(
          title: 'Sistema visual 0.15',
          description:
              'Controles, estados operativos y acciones locales coherentes.',
        ),
        const SizedBox(height: AppSpacing.space24),
        Wrap(
          spacing: AppSpacing.space12,
          runSpacing: AppSpacing.space12,
          children: [
            const YamaIconButton(
              icon: Icons.crop_free_rounded,
              semanticLabel: 'Normal',
              onPressed: _noop,
            ),
            YamaIconButton(
              icon: Icons.touch_app_outlined,
              semanticLabel: 'Hover',
              onPressed: _noop,
              statesController: hoverStates,
            ),
            YamaIconButton(
              icon: Icons.keyboard_rounded,
              semanticLabel: 'Focus',
              onPressed: _noop,
              statesController: focusStates,
            ),
            const YamaIconButton(
              icon: Icons.check_rounded,
              semanticLabel: 'Seleccionado',
              selected: true,
              onPressed: _noop,
            ),
            const YamaIconButton(
              icon: Icons.block_rounded,
              semanticLabel: 'Deshabilitado',
              onPressed: null,
            ),
          ],
        ),
        const SizedBox(height: AppSpacing.space24),
        const AppOperationStatus(
          label: 'Aplicando automatización',
          phase: 'Actualizando documentos',
          value: 0.64,
          onCancel: _noop,
          cancelLabel: 'Cancelar',
        ),
        const SizedBox(height: AppSpacing.space16),
        const Wrap(
          spacing: AppSpacing.space8,
          runSpacing: AppSpacing.space8,
          children: [
            AppStatusLabel(icon: Icons.hourglass_top, label: 'Cargando'),
            AppStatusLabel(
              icon: Icons.cancel_outlined,
              label: 'Cancelado',
              tone: AppStatusTone.warning,
            ),
            AppStatusLabel(
              icon: Icons.check_circle_outline,
              label: 'Completado',
              tone: AppStatusTone.success,
            ),
            AppStatusLabel(
              icon: Icons.error_outline,
              label: 'Error recuperable',
              tone: AppStatusTone.error,
            ),
          ],
        ),
      ],
    ),
  );
}

void _noop() {}

BackupPreview _backupPreview() => BackupPreview(
  version: LocalBackupService.version,
  exportedAt: DateTime.utc(2026, 9, 3),
  documentCount: 48,
  bookmarkCount: 12,
  sessionCount: 5,
  annotationCount: 73,
  schemaVersion: 11,
  fileBytes: 48291,
  entityCount: 152,
  documentAdds: 3,
  documentReplacements: 44,
  documentRemovals: 2,
  identityConflicts: 1,
  tableCounts: const {'documents': 48, 'annotations': 73},
);

Widget _evidenceApp({
  required Key boundaryKey,
  required Widget child,
  required double textScale,
  required bool highContrast,
  required bool reduceMotion,
  required ThemeMode themeMode,
}) => RepaintBoundary(
  key: boundaryKey,
  child: MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: AppTheme.light(highContrast: highContrast),
    darkTheme: AppTheme.dark(highContrast: highContrast),
    themeMode: themeMode,
    locale: const Locale('es'),
    supportedLocales: const [Locale('es'), Locale('en')],
    localizationsDelegates: const [
      YamantakaLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    builder: (context, current) => MediaQuery(
      data: MediaQuery.of(context).copyWith(
        textScaler: TextScaler.linear(textScale),
        highContrast: highContrast,
        disableAnimations: reduceMotion,
      ),
      child: current!,
    ),
    home: Scaffold(body: Center(child: child)),
  ),
);

Future<void> _loadTestFont(String family, List<String> candidates) async {
  for (final candidate in candidates) {
    final file = File(candidate);
    if (!file.existsSync()) continue;
    final loader = FontLoader(family)
      ..addFont(file.readAsBytes().then(ByteData.sublistView));
    await loader.load();
    return;
  }
}
