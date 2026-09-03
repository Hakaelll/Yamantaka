import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/app_config.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/automations/application/local_automation_engine.dart';
import 'package:yamantaka/features/automations/domain/local_automation.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/reader/presentation/document_comparison_dialog.dart';
import 'package:yamantaka/features/recovery/domain/recovery_models.dart';
import 'package:yamantaka/features/relations/domain/document_relation.dart';
import 'package:yamantaka/features/workspaces/application/document_tabs_controller.dart';
import 'package:yamantaka/features/workspaces/application/workspace_controller.dart';
import 'package:yamantaka/features/workspaces/domain/workspace_models.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/automation_repository.dart';
import 'package:yamantaka/storage/repositories/document_relation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/recovery_repository.dart';
import 'package:yamantaka/storage/repositories/workspace_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  group('0.12 persistence', () {
    test('schema v10 contains review, queue, relation and journal state', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);

      expect(
        database.database.select('PRAGMA user_version').first.columnAt(0),
        AppConfig.databaseVersion,
      );
      expect(AppConfig.databaseVersion, greaterThanOrEqualTo(10));
      final tables = database.database
          .select("SELECT name FROM sqlite_master WHERE type = 'table'")
          .map((row) => row['name'])
          .toSet();
      expect(
        tables,
        containsAll({
          'document_relations',
          'document_revision_groups',
          'document_revision_members',
          'automation_rules',
          'operation_journal',
        }),
      );
      final annotationColumns = database.database
          .select('PRAGMA table_info(annotations)')
          .map((row) => row['name'])
          .toSet();
      final workspaceColumns = database.database
          .select('PRAGMA table_info(workspaces)')
          .map((row) => row['name'])
          .toSet();
      final tabColumns = database.database
          .select('PRAGMA table_info(workspace_tabs)')
          .map((row) => row['name'])
          .toSet();
      expect(annotationColumns, contains('review_state'));
      expect(
        workspaceColumns,
        containsAll({
          'query_json',
          'grouping',
          'sort_mode',
          'density',
          'visible_columns_json',
          'focus_mode',
        }),
      );
      expect(
        tabColumns,
        containsAll({'queue_status', 'blocking_reason', 'next_action'}),
      );
      expect(database.integrityCheck(), 'ok');
    });

    test('annotation review filters and bounded bulk edits round trip', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final document = _document('annotations');
      DocumentRepository(database).save(document);
      final repository = AnnotationRepository(database);
      final ids = <String>[];
      for (var index = 0; index < 84; index++) {
        ids.add(
          repository
              .create(
                documentId: document.id,
                pageIndex: index,
                type: AnnotationType.highlight,
                color: AnnotationColor.yellow,
                documentFingerprint: document.fingerprint,
                note: 'Review $index',
                rects: const [
                  NormalizedRect(left: .1, top: .1, right: .2, bottom: .2),
                ],
              )
              .id,
        );
      }

      expect(repository.forDocument(document.id), hasLength(80));
      repository.updateReviewState(ids, AnnotationReviewState.accepted);
      repository.addTag(ids, 'Critical');
      expect(
        repository.all(
          reviewState: AnnotationReviewState.accepted,
          tag: 'critical',
        ),
        hasLength(80),
      );
      expect(
        repository.byId(ids.last)!.reviewState,
        AnnotationReviewState.pending,
      );
    });

    test('workspace view and 1,000-item queue state restore exactly', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final first = _document('queue-a');
      final second = _document('queue-b');
      documents
        ..save(first)
        ..save(second);
      final tabs = DocumentTabsController();
      addTearDown(tabs.dispose);
      final firstTab = tabs.open(first);
      final secondTab = tabs.open(second);
      final repository = WorkspaceRepository(database, documents);
      final controller = WorkspaceController(repository, tabs);
      addTearDown(controller.dispose);
      final saved = controller.saveCurrent(
        name: 'Review',
        queryJson: '{"reading":"pending"}',
        grouping: WorkspaceGrouping.author,
        sortMode: 'title',
        density: WorkspaceDensity.compact,
        visibleColumns: const ['document', 'author', 'status'],
        focusMode: true,
      );
      controller.updateQueueEntry(
        firstTab.id,
        status: WorkspaceQueueStatus.blocked,
        blockingReason: 'Awaiting source',
        nextAction: 'Ask reviewer',
      );
      controller.reorderQueue(0, 1);

      final restored = repository.all().single;
      expect(restored.id, saved.id);
      expect(restored.queryJson, '{"reading":"pending"}');
      expect(restored.grouping, WorkspaceGrouping.author);
      expect(restored.density, WorkspaceDensity.compact);
      expect(restored.focusMode, isTrue);
      expect(restored.tabs.first.id, secondTab.id);
      final blocked = restored.tabs.singleWhere((tab) => tab.id == firstTab.id);
      expect(blocked.queueStatus, WorkspaceQueueStatus.blocked);
      expect(blocked.blockingReason, 'Awaiting source');
      expect(blocked.nextAction, 'Ask reviewer');

      controller.delete(saved.id);
      expect(controller.workspaces, isEmpty);
      expect(controller.undoDelete()?.id, saved.id);
      expect(controller.workspaces.single.tabs, hasLength(2));
    });

    test('relations expose directional backlinks and revision choice', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final old = _document('old');
      final current = _document('current');
      documents
        ..save(old)
        ..save(current);
      final repository = DocumentRelationRepository(database);
      repository.create(
        sourceDocumentId: current.id,
        targetDocumentId: old.id,
        type: DocumentRelationType.replaces,
        sourceAnchor: const DocumentAnchor(
          kind: DocumentAnchorKind.page,
          id: '3',
        ),
      );

      expect(repository.forDocument(current.id).single.isReplacedBy, isFalse);
      expect(repository.forDocument(old.id).single.isReplacedBy, isTrue);
      final group = repository.createRevisionGroup(
        name: 'Contract revisions',
        documentIds: [old.id, current.id],
        currentDocumentId: current.id,
      );
      repository.setCurrentRevision(group.id, old.id);
      expect(
        repository
            .revisionGroupsForDocument(current.id)
            .single
            .currentDocumentId,
        old.id,
      );

      documents.remove(old.id);
      expect(repository.forDocument(current.id), isEmpty);
      expect(
        repository.revisionGroupsForDocument(current.id).single.documentIds,
        [current.id],
      );
      expect(database.integrityCheck(), 'ok');
    });

    test('atomic session and interrupted journals produce safe problems', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final recovery = RecoveryRepository(database);
      final savedAt = DateTime.utc(2026, 8, 30, 10);
      recovery.saveSession(
        AtomicSessionSnapshot(
          workspaceId: 'workspace',
          activeDocumentId: 'document',
          savedAt: savedAt,
          clean: false,
        ),
      );
      expect(recovery.session()?.workspaceId, 'workspace');
      final automations = AutomationRepository(database);
      final journal = automations.startJournal(
        kind: 'automation',
        totalItems: 9,
      );
      automations.markRunningAsInterrupted();

      final problem = recovery.problems().single;
      expect(problem.id, journal.id);
      expect(problem.code, 'operation_interrupted');
      expect(problem.canResume, isTrue);
      recovery.discardProblem(problem.id);
      expect(recovery.problems(), isEmpty);
      recovery.discardSession();
      expect(recovery.session(), isNull);
    });
  });

  group('0.12 local automation', () {
    test(
      'simulation, application, cancellation and undo are journalled',
      () async {
        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final repository = AutomationRepository(database);
        final engine = LocalAutomationEngine(repository);
        final now = DateTime.utc(2026, 8, 30);
        final rule = repository.save(
          LocalAutomationRule(
            id: '',
            name: 'PDF review',
            trigger: const AutomationTrigger(
              AutomationTriggerKind.extension,
              value: '.pdf',
            ),
            actions: const [
              AutomationAction(AutomationActionKind.addTag, value: 'review'),
            ],
            createdAt: now,
            updatedAt: now,
          ),
        );
        final documents = [
          for (var index = 0; index < 5; index++) _document('$index'),
        ];
        final simulation = engine.simulate(rule, documents);
        expect(simulation.affected, 5);
        final applied = <String>{};
        final result = await engine.apply(
          rule: rule,
          documents: documents,
          applyAction: (document, _) async {
            applied.add(document.id);
            return {'documentId': document.id};
          },
        );
        expect(result.status, JournalStatus.completed);
        expect(applied, hasLength(5));
        expect(repository.journal(result.journalId)?.completedItems, 5);
        expect(
          await engine.undo(result.journalId, (undo) async {
            applied.remove(undo['documentId']);
          }),
          isTrue,
        );
        expect(applied, isEmpty);
        expect(
          repository.journal(result.journalId)?.status,
          JournalStatus.rolledBack,
        );

        final cancellation = AutomationCancellationToken()..cancel();
        final cancelled = await engine.apply(
          rule: rule,
          documents: documents,
          cancellationToken: cancellation,
          applyAction: (_, _) async => const {},
        );
        expect(cancelled.status, JournalStatus.cancelled);
        expect(cancelled.completed, 0);
      },
    );

    test(
      '100,000-document simulation is bounded and reports truncation',
      () async {
        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final engine = LocalAutomationEngine(AutomationRepository(database));
        final now = DateTime.utc(2026, 8, 30);
        final rule = LocalAutomationRule(
          id: 'rule',
          name: 'Missing files',
          trigger: const AutomationTrigger(AutomationTriggerKind.inaccessible),
          actions: const [
            AutomationAction(AutomationActionKind.setPriority, value: 'high'),
          ],
          createdAt: now,
          updatedAt: now,
        );
        final result = engine.simulate(rule, _manyDocuments(100001));
        expect(result.scanned, AppConfig.maxAutomationDocuments);
        expect(result.truncated, isTrue);
        expect(result.affected, 50000);

        final timedOut = await engine.simulateAsync(
          rule,
          _manyDocuments(100001),
          timeLimit: Duration.zero,
        );
        expect(timedOut.scanned, 0);
        expect(timedOut.truncated, isTrue);
      },
    );
  });

  group('0.12 adaptive surfaces', () {
    testWidgets('continuity center is keyboard reachable at 200% text', (
      tester,
    ) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(500, 700);
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      var resumed = false;
      await tester.pumpWidget(
        MaterialApp(
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(
              context,
            ).copyWith(textScaler: const TextScaler.linear(2)),
            child: child!,
          ),
          home: Scaffold(
            body: ContinuityCenter(
              summary: const ContinuitySummary(
                openDocuments: 5,
                pinnedDocuments: 2,
                pendingReadings: 18,
                workspaceCount: 3,
                problemCount: 1,
              ),
              hasActiveReading: true,
              hasRecentlyClosed: true,
              onContinue: () {},
              onOpenNext: () {},
              onResumeSession: () => resumed = true,
              onDiscardContext: () {},
              onProblems: () {},
            ),
          ),
        ),
      );
      await tester.pump();

      expect(tester.takeException(), isNull);
      expect(find.byKey(const Key('continuity-continue')), findsOneWidget);
      expect(find.byKey(const Key('continuity-open-next')), findsOneWidget);
      await tester.ensureVisible(find.byKey(const Key('continuity-resume')));
      await tester.tap(find.byKey(const Key('continuity-resume')));
      expect(resumed, isTrue);
      final semantics = tester.getSemantics(
        find.textContaining('problemas').last,
      );
      expect(semantics.label, contains('problemas'));
    });

    testWidgets('comparison closes only its secondary document handle', (
      tester,
    ) async {
      final image = await _onePixelImage();
      addTearDown(image.dispose);
      final engine = _ComparisonEngine(image);
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                barrierDismissible: false,
                builder: (_) => DocumentComparisonDialog(
                  engine: engine,
                  primary: const OpenedPdfDocument(
                    documentId: 'primary',
                    path: 'primary.pdf',
                    pageCount: 2,
                    pageSizes: [PdfPageSize(100, 100), PdfPageSize(100, 100)],
                    isEncrypted: false,
                  ),
                  primaryTitle: 'Primary',
                  secondaryPath: 'secondary.pdf',
                  secondaryTitle: 'Secondary',
                  initialPage: 0,
                ),
              ),
              child: const Text('open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('open'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pump();
      expect(engine.opened, ['secondary.pdf']);
      expect(find.text('Primary'), findsOneWidget);
      expect(find.text('Secondary'), findsOneWidget);

      await tester.tap(find.byIcon(Icons.close_rounded));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      expect(engine.closed, ['secondary']);
      expect(engine.closed, isNot(contains('primary')));
    });
  });
}

Iterable<LibraryDocument> _manyDocuments(int count) sync* {
  for (var index = 0; index < count; index++) {
    yield _document('large-$index', missing: index.isEven);
  }
}

LibraryDocument _document(String id, {bool missing = false}) {
  final now = DateTime.utc(2026, 8, 30);
  return LibraryDocument(
    id: id,
    path: 'C:\\library\\$id.pdf',
    normalizedPath: 'c:\\library\\$id.pdf',
    fileName: '$id.pdf',
    title: 'Document $id',
    pageCount: 100,
    fileSize: 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint-$id',
    preferredViewMode: ReaderMode.document,
    preferredTheme: ReaderTheme.ivory,
    isMissing: missing,
  );
}

Future<ui.Image> _onePixelImage() async {
  final recorder = ui.PictureRecorder();
  ui.Canvas(recorder).drawColor(const ui.Color(0xFFFFFFFF), ui.BlendMode.src);
  final picture = recorder.endRecording();
  final image = await picture.toImage(1, 1);
  picture.dispose();
  return image;
}

final class _ComparisonEngine extends StubPdfDocumentEngine {
  _ComparisonEngine(this.image);

  final ui.Image image;
  final List<String> opened = [];
  final List<String> closed = [];

  @override
  Future<OpenedPdfDocument> openDocument(
    String path, {
    String? password,
  }) async {
    opened.add(path);
    return const OpenedPdfDocument(
      documentId: 'secondary',
      path: 'secondary.pdf',
      pageCount: 2,
      pageSizes: [PdfPageSize(100, 100), PdfPageSize(100, 100)],
      isEncrypted: false,
    );
  }

  @override
  Future<void> closeDocument(String documentId) async {
    closed.add(documentId);
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
