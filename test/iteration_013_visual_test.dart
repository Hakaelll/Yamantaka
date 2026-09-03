import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_components.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/features/library/application/duplicate_resolution_service.dart';
import 'package:yamantaka/features/library/domain/duplicate_resolution.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/duplicate_review_dialog.dart';
import 'package:yamantaka/features/reader/presentation/password_request_dialog.dart';
import 'package:yamantaka/features/recovery/domain/recovery_models.dart';
import 'package:yamantaka/features/recovery/presentation/problems_screen.dart';
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

  testWidgets('duplicate review stays semantic and usable at 200% text', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final key = GlobalKey();
    final first = _document('kept', title: 'Existing local title');
    final second = _document('incoming', title: 'Incoming title');
    final candidate = DuplicateCandidate(
      existing: first,
      incoming: second,
      evidence: DuplicateClassifier.classify(first, second),
    );
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        theme: AppTheme.light(),
        locale: const Locale('es'),
        supportedLocales: const [Locale('es'), Locale('en')],
        localizationsDelegates: const [
          YamantakaLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(
            context,
          ).copyWith(textScaler: const TextScaler.linear(2)),
          child: child!,
        ),
        home: RepaintBoundary(
          key: key,
          child: Scaffold(
            body: Center(
              child: DuplicateReviewDialog(
                candidates: [candidate],
                impactFor: (_) => const DuplicateMergeImpact(
                  localRecords: 17,
                  conflictFields: ['title'],
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Duplicados locales'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('Evidencias del duplicado')),
      findsOneWidget,
    );
    expect(find.text('Mantener ambos'), findsOneWidget);
    expect(find.text('Crear copia y consolidar'), findsOneWidget);
    expect(tester.takeException(), isNull);

    const outputRoot = String.fromEnvironment('YAMANTAKA_013_VISUAL_DIR');
    if (outputRoot.isNotEmpty) {
      final output = Directory(outputRoot)..createSync(recursive: true);
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(key),
      );
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        try {
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          File(
            '${output.path}/duplicate-review-200-percent.png',
          ).writeAsBytesSync(bytes!.buffer.asUint8List(), flush: true);
        } finally {
          image.dispose();
        }
      });
    }
    semantics.dispose();
  });

  testWidgets('captures the 0.13 document resilience surfaces', (tester) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_013_VISUAL_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final boundaryKey = GlobalKey();
    final evidence = <Map<String, Object?>>[];

    Future<void> capture(
      String surface,
      String name,
      Widget child, {
      double textScale = 1,
    }) async {
      await tester.pumpWidget(
        _evidenceApp(
          boundaryKey: boundaryKey,
          child: child,
          textScale: textScale,
        ),
      );
      await tester.pumpAndSettle();
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
      expect(tester.takeException(), isNull);
      evidence.add({
        'surface': surface,
        'name': name,
        'width': 1280,
        'height': 800,
        'textScale': textScale,
        'sha256': sha256.convert(bytes).toString(),
        'source': 'executed-widget-surface',
        'path': path,
      });
    }

    await capture(
      'password',
      'password-required',
      const PasswordRequestDialog(wrongPassword: false),
    );
    final passwordField = tester.widget<TextField>(find.byType(TextField));
    expect(passwordField.obscureText, isTrue);
    expect(passwordField.enableSuggestions, isFalse);
    expect(passwordField.autocorrect, isFalse);
    await capture(
      'password',
      'wrong-password-text-200',
      const PasswordRequestDialog(wrongPassword: true),
      textScale: 2,
    );
    await capture(
      'missing',
      'file-missing-actions',
      Builder(
        builder: (context) {
          final strings = YamantakaLocalizations.of(context);
          return AppStatusView(
            icon: Icons.error_outline_rounded,
            title: strings.get('failure.fileNotFound.title'),
            message: strings.get('failure.fileNotFound.message'),
            action: () {},
            actionLabel: strings.get('library.locate'),
            secondaryAction: () {},
            secondaryActionLabel: strings.get('reader.backToLibrary'),
            tone: AppStatusTone.error,
          );
        },
      ),
    );
    await capture(
      'restore',
      'backup-dry-run-plan',
      BackupRestorePreviewDialog(
        preview: BackupPreview(
          version: LocalBackupService.version,
          exportedAt: DateTime.utc(2026, 9, 1),
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
        ),
      ),
    );
    await capture(
      'problems',
      'interrupted-and-missing',
      ProblemsScreen(
        problems: [
          RecoveryProblem(
            id: 'journal',
            kind: RecoveryProblemKind.interruptedOperation,
            code: 'operation_interrupted',
            createdAt: DateTime.utc(2026, 9, 1),
            canResume: true,
            canRollback: true,
            phase: 'apply-batch-3',
          ),
          RecoveryProblem(
            id: 'missing',
            kind: RecoveryProblemKind.inaccessibleDocument,
            code: 'document_inaccessible',
            createdAt: DateTime.utc(2026, 9, 1),
          ),
        ],
        onBack: () {},
        onCheckIntegrity: () {},
        onDiscard: (_) {},
        onResume: (_) {},
        onRollback: (_) {},
        onReviewDuplicates: () {},
      ),
    );
    await capture(
      'recovery',
      'interrupted-session-choice',
      const SessionRecoveryDialog(),
    );

    File('${output.path}/resilience-matrix.json').writeAsStringSync(
      const JsonEncoder.withIndent(' ').convert({
        'format': 'yamantaka-document-resilience-visual-matrix',
        'version': 1,
        'productVersion': '0.13.0',
        'generatedAt': '2026-09-01T12:00:00Z',
        'cases': evidence,
      }),
      flush: true,
    );
  });
}

Widget _evidenceApp({
  required Key boundaryKey,
  required Widget child,
  required double textScale,
}) => RepaintBoundary(
  key: boundaryKey,
  child: MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: AppTheme.light(),
    locale: const Locale('es'),
    supportedLocales: const [Locale('es'), Locale('en')],
    localizationsDelegates: const [
      YamantakaLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    builder: (context, current) => MediaQuery(
      data: MediaQuery.of(
        context,
      ).copyWith(textScaler: TextScaler.linear(textScale)),
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

LibraryDocument _document(String id, {required String title}) {
  final now = DateTime.utc(2026, 8, 31);
  return LibraryDocument(
    id: id,
    path: '$id.pdf',
    normalizedPath: '$id.pdf',
    fileName: 'same.pdf',
    title: title,
    pageCount: 12,
    fileSize: 4096,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'same-fingerprint',
  );
}
