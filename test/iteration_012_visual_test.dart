import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crypto/crypto.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/annotations/presentation/global_annotations_dialog.dart';
import 'package:yamantaka/features/automations/application/local_automation_engine.dart';
import 'package:yamantaka/features/automations/domain/local_automation.dart';
import 'package:yamantaka/features/automations/presentation/automation_manager_dialog.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/presentation/document_comparison_dialog.dart';
import 'package:yamantaka/features/recovery/domain/recovery_models.dart';
import 'package:yamantaka/features/recovery/presentation/problems_screen.dart';
import 'package:yamantaka/features/relations/domain/document_relation.dart';
import 'package:yamantaka/features/relations/presentation/document_relations_dialog.dart';
import 'package:yamantaka/features/workspaces/application/document_tabs_controller.dart';
import 'package:yamantaka/features/workspaces/application/workspace_controller.dart';
import 'package:yamantaka/features/workspaces/domain/workspace_models.dart';
import 'package:yamantaka/features/workspaces/presentation/workspace_dialog.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/automation_repository.dart';
import 'package:yamantaka/storage/repositories/document_relation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/workspace_repository.dart';

import 'support/stub_pdf_engine.dart';

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

  testWidgets('captures the 0.15 continuity and workflow surfaces', (
    tester,
  ) async {
    const outputRoot = String.fromEnvironment('YAMANTAKA_WORKFLOW_MATRIX_DIR');
    if (outputRoot.isEmpty) return;
    final output = Directory(outputRoot)..createSync(recursive: true);
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final documents = DocumentRepository(database);
    final first = _document('revision-current', title: 'Manual de campo');
    final second = _document(
      'revision-old',
      title: 'Manual de campo — borrador',
    );
    final missing = _document(
      'missing-source',
      title: 'Anexo no disponible',
      missing: true,
    );
    documents
      ..save(first)
      ..save(second)
      ..save(missing);

    final annotations = AnnotationRepository(database);
    for (var index = 0; index < 7; index++) {
      final annotation = annotations.create(
        documentId: first.id,
        pageIndex: index * 3,
        type: AnnotationType.values[index % AnnotationType.values.length],
        color: AnnotationColor.values[index % AnnotationColor.values.length],
        documentFingerprint: first.fingerprint,
        note: 'Revisar este argumento con la fuente primaria.',
        selectedText: 'Fragmento seleccionado para revisión',
        rects: const [
          NormalizedRect(left: .12, top: .18, right: .72, bottom: .24),
        ],
      );
      if (index.isEven) {
        annotations.updateReviewState([
          annotation.id,
        ], AnnotationReviewState.accepted);
      }
    }

    final tabs = DocumentTabsController();
    addTearDown(tabs.dispose);
    final firstTab = tabs.open(first);
    final secondTab = tabs.open(second);
    final workspaces = WorkspaceController(
      WorkspaceRepository(database, documents),
      tabs,
    );
    addTearDown(workspaces.dispose);
    workspaces.saveCurrent(
      name: 'Revisión editorial',
      queryJson: '{"reading":"pending"}',
      grouping: WorkspaceGrouping.author,
      sortMode: 'title',
      density: WorkspaceDensity.compact,
      visibleColumns: const ['document', 'author', 'status'],
      focusMode: true,
    );
    workspaces.updateQueueEntry(
      firstTab.id,
      status: WorkspaceQueueStatus.inReview,
      nextAction: 'Resolver anotaciones',
    );
    workspaces.updateQueueEntry(
      secondTab.id,
      status: WorkspaceQueueStatus.blocked,
      blockingReason: 'Falta validar la fuente',
    );

    final relations = DocumentRelationRepository(database);
    relations.create(
      sourceDocumentId: first.id,
      targetDocumentId: second.id,
      type: DocumentRelationType.replaces,
      sourceAnchor: const DocumentAnchor(
        kind: DocumentAnchorKind.page,
        id: '4',
      ),
    );
    relations.createRevisionGroup(
      name: 'Manual de campo',
      documentIds: [first.id, second.id],
      currentDocumentId: first.id,
    );

    final automationRepository = AutomationRepository(database);
    final now = DateTime.utc(2026, 8, 31, 12);
    automationRepository.save(
      LocalAutomationRule(
        id: '',
        name: 'Preparar PDFs importados',
        trigger: const AutomationTrigger(
          AutomationTriggerKind.extension,
          value: '.pdf',
        ),
        actions: const [
          AutomationAction(AutomationActionKind.addTag, value: 'por-revisar'),
          AutomationAction(
            AutomationActionKind.addReadingQueue,
            value: 'Revisión editorial',
          ),
        ],
        createdAt: now,
        updatedAt: now,
      ),
    );

    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1280, 720);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    const boundaryKey = Key('iteration-012-workflow-boundary');
    final cases = <Map<String, Object?>>[];

    Future<void> pumpSurface(Widget surface, {bool dark = false}) async {
      await tester.pumpWidget(
        MaterialApp(
          debugShowCheckedModeBanner: false,
          locale: const Locale('es'),
          supportedLocales: const [Locale('es'), Locale('en')],
          localizationsDelegates: const [
            YamantakaLocalizations.delegate,
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          theme: AppTheme.light(),
          darkTheme: AppTheme.dark(),
          themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          home: RepaintBoundary(
            key: boundaryKey,
            child: Scaffold(body: surface),
          ),
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pumpAndSettle();
    }

    Future<void> capture(String surface, String name) async {
      final directory = Directory(
        '${output.path}${Platform.pathSeparator}$surface',
      )..createSync(recursive: true);
      final path = '${directory.path}${Platform.pathSeparator}$name.png';
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(boundaryKey),
      );
      late final List<int> data;
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        try {
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          data = bytes!.buffer.asUint8List();
          File(path).writeAsBytesSync(data, flush: true);
        } finally {
          image.dispose();
        }
      });
      cases.add({
        'surface': surface,
        'name': name,
        'width': 1280,
        'height': 720,
        'sha256': sha256.convert(data).toString(),
      });
    }

    await pumpSurface(
      const ContinuityCenter(
        summary: ContinuitySummary(
          openDocuments: 5,
          pinnedDocuments: 2,
          pendingReadings: 18,
          workspaceCount: 3,
          problemCount: 2,
        ),
        hasActiveReading: true,
        hasRecentlyClosed: true,
      ),
    );
    await tester.tap(find.byKey(const Key('continuity-menu-button')));
    await tester.pumpAndSettle();
    await capture('continuity', 'continuity-content-light');

    await pumpSurface(
      WorkspaceOverview(
        controller: workspaces,
        onRestore: (_) {},
        onOpenQueued: (_) {},
        onAction: (_, _) {},
      ),
    );
    await capture('workspaces', 'workspaces-saved-view-light');
    await pumpSurface(
      WorkspaceOverview(
        key: const Key('queue-expanded'),
        controller: workspaces,
        onRestore: (_) {},
        onOpenQueued: (_) {},
        onAction: (_, _) {},
        expandActiveWorkspace: true,
      ),
    );
    await capture('queue', 'reading-queue-expanded-light');

    await pumpSurface(
      GlobalAnnotationsDialog(
        repository: annotations,
        documentTitle: (_) => first.effectiveTitle,
        onOpen: (_) {},
        onExport: (_) async {},
      ),
    );
    await capture('annotations', 'annotation-review-content-light');

    await pumpSurface(
      DocumentRelationsDialog(
        repository: relations,
        document: first,
        candidates: [second, missing],
        onOpenDocument: (_) {},
      ),
    );
    await capture('relations', 'relations-and-revisions-light');

    await pumpSurface(
      AutomationManagerDialog(
        repository: automationRepository,
        engine: LocalAutomationEngine(automationRepository),
        documents: [first, second, missing],
        applyAction: (_, _) async => const {},
        applyUndo: (_) async {},
      ),
      dark: true,
    );
    await capture('automations', 'local-rules-content-dark');

    await pumpSurface(
      ProblemsScreen(
        problems: [
          RecoveryProblem(
            id: 'operation',
            kind: RecoveryProblemKind.interruptedOperation,
            code: 'operation_interrupted',
            createdAt: now,
            canResume: true,
          ),
          RecoveryProblem(
            id: 'missing',
            kind: RecoveryProblemKind.inaccessibleDocument,
            code: 'document_inaccessible',
            createdAt: now,
          ),
        ],
        onBack: () {},
        onCheckIntegrity: () {},
        onDiscard: (_) {},
        onResume: (_) {},
      ),
    );
    await capture('problems', 'recovery-problems-light');

    final renderedImage = await _onePixelImage();
    addTearDown(renderedImage.dispose);
    await pumpSurface(
      DocumentComparisonDialog(
        engine: _ComparisonEngine(renderedImage),
        primary: OpenedPdfDocument(
          documentId: 'primary',
          path: 'primary.pdf',
          pageCount: 12,
          pageSizes: [
            for (var index = 0; index < 12; index++) PdfPageSize(595, 842),
          ],
          isEncrypted: false,
        ),
        primaryTitle: first.effectiveTitle,
        secondaryPath: second.path,
        secondaryTitle: second.effectiveTitle,
        initialPage: 3,
      ),
    );
    await capture('comparison', 'side-by-side-linked-light');

    File(
      '${output.path}${Platform.pathSeparator}matrix-report.json',
    ).writeAsStringSync(
      const JsonEncoder.withIndent('  ').convert({
        'format': 'yamantaka-workflow-visual-matrix',
        'version': 1,
        'productVersion': '0.15.0',
        'generatedAt': DateTime.now().toUtc().toIso8601String(),
        'cases': cases,
      }),
      flush: true,
    );
  });
}

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

LibraryDocument _document(
  String id, {
  required String title,
  bool missing = false,
}) {
  final now = DateTime.utc(2026, 8, 31, 12);
  return LibraryDocument(
    id: id,
    path: 'C:\\Biblioteca local\\$id.pdf',
    normalizedPath: 'c:\\biblioteca local\\$id.pdf',
    fileName: '$id.pdf',
    title: title,
    author: 'Equipo Yamantaka',
    pageCount: 120,
    fileSize: 2 * 1024 * 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint-$id',
    preferredViewMode: ReaderMode.document,
    preferredTheme: ReaderTheme.ivory,
    readingProgress: .42,
    isMissing: missing,
  );
}

Future<ui.Image> _onePixelImage() async {
  final recorder = ui.PictureRecorder();
  ui.Canvas(recorder).drawColor(const ui.Color(0xFFF9F4E8), ui.BlendMode.src);
  final picture = recorder.endRecording();
  final image = await picture.toImage(1, 1);
  picture.dispose();
  return image;
}

final class _ComparisonEngine extends StubPdfDocumentEngine {
  _ComparisonEngine(this.image);

  final ui.Image image;

  @override
  Future<OpenedPdfDocument> openDocument(
    String path, {
    String? password,
  }) async {
    return OpenedPdfDocument(
      documentId: 'secondary',
      path: 'secondary.pdf',
      pageCount: 12,
      pageSizes: [
        for (var index = 0; index < 12; index++) PdfPageSize(595, 842),
      ],
      isEncrypted: false,
    );
  }

  @override
  Future<RenderedPdfPage> renderPage({
    required String documentId,
    required int pageIndex,
    required double scale,
    int rotation = 0,
    PdfRenderRegion? region,
    PdfOperationToken? cancellationToken,
    double devicePixelRatio = 1,
    int? generation,
    PdfRenderPriority priority = PdfRenderPriority.visible,
  }) async => RenderedPdfPage(image: image, pageIndex: pageIndex, scale: scale);
}
