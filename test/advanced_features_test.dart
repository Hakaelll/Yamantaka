import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:sqlite3/sqlite3.dart';
import 'package:yamantaka/app/app_config.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/annotations/application/annotation_controller.dart';
import 'package:yamantaka/features/annotations/application/annotation_export_service.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/diagnostics/application/diagnostics_service.dart';
import 'package:yamantaka/features/library/application/document_recovery_service.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/application/folder_import_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/navigation/application/navigation_history_controller.dart';
import 'package:yamantaka/features/organization/domain/organization_models.dart';
import 'package:yamantaka/features/profiles/domain/reading_profile.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/search/application/library_index_controller.dart';
import 'package:yamantaka/features/workspaces/application/document_tabs_controller.dart';
import 'package:yamantaka/features/workspaces/domain/workspace_models.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/pdf_engine/adapters/pdfrx_document_engine.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';
import 'package:yamantaka/storage/repositories/organization_repository.dart';
import 'package:yamantaka/storage/repositories/workspace_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  group('database schema v2', () {
    test('creates strict capability tables and FTS index', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);

      expect(
        database.database.select('PRAGMA user_version').first.columnAt(0),
        AppConfig.databaseVersion,
      );
      final names = database.database
          .select(
            "SELECT name FROM sqlite_master WHERE type IN ('table', 'view')",
          )
          .map((row) => row['name'])
          .toSet();
      expect(
        names,
        containsAll({
          'annotations',
          'tags',
          'collections',
          'workspaces',
          'workspace_tabs',
          'library_search',
          'search_index_state',
        }),
      );
      expect(database.integrityCheck(), 'ok');
    });

    test('safe diagnostics redact the local data path', () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-diagnostics-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final database = AppDatabase.open(
        '${directory.path}${Platform.pathSeparator}state.sqlite3',
        AppLogger(),
      );
      addTearDown(database.close);
      final engine = PdfrxDocumentEngine(AppLogger());
      addTearDown(engine.dispose);
      final report = await DiagnosticsService(
        database: database,
        paths: AppPaths(
          dataDirectory: directory,
          cacheDirectory: directory,
          thumbnailDirectory: directory,
        ),
        engine: engine,
        safeMode: false,
      ).collect();
      final encoded = report.toJson();

      expect(report.values['integrity'], 'ok');
      expect(encoded, isNot(contains(directory.path)));
      expect(report.values['contains_paths_titles_or_queries'], isFalse);
    });

    test('safe mode starts after three consecutive unclean restarts', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);

      expect(database.recordStartup(), isFalse);
      expect(database.recordStartup(), isFalse);
      expect(database.recordStartup(), isFalse);
      expect(database.recordStartup(), isTrue);
      database.markCleanShutdown();
      expect(database.recordStartup(), isFalse);
    });

    test('migrates version one transactionally and creates a safety copy', () {
      final directory = Directory.systemTemp.createTempSync(
        'yamantaka-migration-',
      );
      addTearDown(() => directory.deleteSync(recursive: true));
      final path = '${directory.path}${Platform.pathSeparator}legacy.sqlite3';
      final legacy = sqlite3.open(path)
        ..execute('CREATE TABLE documents(id TEXT NOT NULL PRIMARY KEY) STRICT')
        ..execute('PRAGMA user_version = 1');
      legacy.close();

      final migrated = AppDatabase.open(path, AppLogger());
      addTearDown(migrated.close);

      expect(
        migrated.database.select('PRAGMA user_version').first.columnAt(0),
        2,
      );
      expect(
        directory
            .listSync()
            .where((entry) => entry.path.contains('pre-migration-v1'))
            .length,
        1,
      );
      expect(migrated.integrityCheck(), 'ok');
    });
  });

  group('annotations', () {
    late AppDatabase database;
    late AnnotationRepository repository;
    late AnnotationController controller;
    late LibraryDocument document;

    setUp(() {
      database = AppDatabase.inMemory(AppLogger());
      final documents = DocumentRepository(database);
      document = _document();
      documents.save(document);
      repository = AnnotationRepository(database);
      controller = AnnotationController(repository)
        ..load(document.id, document.fingerprint);
    });

    tearDown(() {
      controller.dispose();
      database.close();
    });

    test('creates, updates, filters, deletes and undoes', () {
      final created = controller.create(
        pageIndex: 2,
        type: AnnotationType.highlight,
        color: AnnotationColor.green,
        note: 'Idea principal',
        selectedText: 'texto elegido',
        rects: const [
          NormalizedRect(left: 0.1, top: 0.2, right: 0.8, bottom: 0.3),
        ],
        tags: const ['Estudio'],
      )!;

      expect(controller.annotations, hasLength(1));
      expect(controller.pageHasAnnotations(2), isTrue);
      expect(controller.annotations.single.tags, ['Estudio']);

      controller.update(
        created.copyWith(
          color: AnnotationColor.blue,
          note: 'Comentario editado',
        ),
      );
      expect(controller.annotations.single.color, AnnotationColor.blue);
      expect(controller.annotations.single.note, 'Comentario editado');

      controller.setFilters(search: 'editado');
      expect(controller.annotations, hasLength(1));
      controller.setFilters(search: 'ausente');
      expect(controller.annotations, isEmpty);
      controller.setFilters(search: '');

      controller.delete(controller.annotations.single);
      expect(controller.annotations, isEmpty);
      expect(controller.canUndoDelete, isTrue);
      expect(controller.undoDelete(), isTrue);
      expect(controller.annotations, hasLength(1));
    });

    test('rotates normalized coordinates without pixel coupling', () {
      const rect = NormalizedRect(left: 0.1, top: 0.2, right: 0.4, bottom: 0.5);
      final rotated = rect.rotatedClockwise();

      expect(rotated.left, closeTo(0.5, 0.0001));
      expect(rotated.top, closeTo(0.1, 0.0001));
      expect(rotated.right, closeTo(0.8, 0.0001));
      expect(rotated.bottom, closeTo(0.4, 0.0001));
    });

    test('keeps annotations for changed documents and marks review', () {
      controller.create(pageIndex: 0, note: 'Conservar');
      controller.load(document.id, 'different-fingerprint');

      expect(controller.annotations, hasLength(1));
      expect(
        controller.annotations.single.status,
        AnnotationAttachmentStatus.reviewRequired,
      );
      expect(controller.reviewRequiredCount, 1);
    });

    test('exports readable Markdown and versioned JSON', () {
      controller.create(
        pageIndex: 1,
        type: AnnotationType.underline,
        note: 'Comentario',
        selectedText: 'Fragmento',
        tags: const ['Tema'],
      );
      const exporter = AnnotationExportService();
      final markdown = exporter.toMarkdown(
        title: 'Documento',
        author: 'Autora',
        progress: 0.5,
        annotations: controller.annotations,
        exportedAt: DateTime.utc(2026, 7, 29),
      );
      final json =
          jsonDecode(
                exporter.toVersionedJson(
                  documentId: document.id,
                  title: 'Documento',
                  progress: 0.5,
                  annotations: controller.annotations,
                  exportedAt: DateTime.utc(2026, 7, 29),
                ),
              )
              as Map<String, dynamic>;

      expect(markdown, contains('Página 2 — Subrayado'));
      expect(markdown, contains('Fragmento'));
      expect(markdown, isNot(contains(document.path)));
      expect(json['format'], 'yamantaka-annotations');
      expect(json['version'], 1);
      expect((json['annotations'] as List).single['pageIndex'], 1);
    });
  });

  group('organization and global search', () {
    late AppDatabase database;
    late DocumentRepository documents;
    late LibraryDocument document;

    setUp(() {
      database = AppDatabase.inMemory(AppLogger());
      documents = DocumentRepository(database);
      document = _document();
      documents.save(document);
    });

    tearDown(() => database.close());

    test('collection membership and smart collections never copy files', () {
      final organization = OrganizationRepository(database);
      final collection = organization.createCollection('Referencias');
      organization.addDocument(collection.id, document.id);

      expect(organization.documentIds(collection.id), {document.id});
      expect(organization.collections().single.documentCount, 1);
      expect(
        organization.smartDocumentIds(SmartCollection.notStarted),
        contains(document.id),
      );

      organization.deleteCollection(collection.id);
      expect(documents.byId(document.id), isNotNull);
    });

    test(
      'tags merge transactionally and deleting tags preserves documents',
      () {
        final annotations = AnnotationRepository(database);
        annotations
          ..create(
            documentId: document.id,
            pageIndex: 0,
            type: AnnotationType.pageComment,
            color: AnnotationColor.yellow,
            documentFingerprint: document.fingerprint,
            note: 'Primera',
            tags: const ['Fuente'],
          )
          ..create(
            documentId: document.id,
            pageIndex: 1,
            type: AnnotationType.pageComment,
            color: AnnotationColor.blue,
            documentFingerprint: document.fingerprint,
            note: 'Segunda',
            tags: const ['Revisar'],
          );
        final organization = OrganizationRepository(database);
        final tags = organization.tags();
        final source = tags.firstWhere((tag) => tag.name == 'Fuente');
        final target = tags.firstWhere((tag) => tag.name == 'Revisar');
        organization.assignTagToDocument(source.id, document.id);

        organization.mergeTags(source.id, target.id);
        expect(organization.tags(), hasLength(1));
        expect(organization.tags().single.usageCount, greaterThanOrEqualTo(2));

        organization.deleteTag(target.id);
        expect(organization.tags(), isEmpty);
        expect(documents.byId(document.id), isNotNull);
      },
    );

    test('FTS5 finds metadata, page text and annotation text', () {
      final search = GlobalSearchRepository(database);
      search.beginDocument(document);
      search.addPage(
        document: document,
        pageIndex: 3,
        text: 'Yamantaka contiene una búsqueda completamente local.',
      );
      search.finishDocument(document.id, hasExtractableText: true);

      final annotations = AnnotationRepository(database);
      annotations.create(
        documentId: document.id,
        pageIndex: 4,
        type: AnnotationType.pageComment,
        color: AnnotationColor.yellow,
        documentFingerprint: document.fingerprint,
        note: 'Revisar arquitectura modular',
      );
      search.refreshAnnotationEntries();

      final textResults = search.search('completamente');
      final noteResults = search.search('arquitectura');
      expect(textResults.single.pageIndex, 3);
      expect(textResults.single.kind.name, 'pageText');
      expect(noteResults.single.pageIndex, 4);
      expect(noteResults.single.kind.name, 'annotation');
      expect(search.indexedDocumentCount, 1);

      search.clear();
      expect(search.search('arquitectura'), isEmpty);
    });

    test(
      'incremental indexing honors exclusions, cancellation and closes handles',
      () async {
        final second = _document(id: 'second', path: r'C:\second.pdf');
        documents.save(second);
        final search = GlobalSearchRepository(database);
        final engine = _RecordingPdfEngine(
          pageCount: 3,
          pageText: (page) => 'contenido local página $page',
        );
        final index = LibraryIndexController(documents, search, engine)
          ..setEnabled(true);
        addTearDown(index.dispose);

        await index.indexIncremental(excludedDocumentIds: {second.id});
        expect(search.indexedDocumentCount, 1);
        expect(engine.openedPaths, [document.path]);
        expect(engine.closedDocumentIds, ['fake-1']);

        search.clear();
        engine.onExtract = (page) {
          if (page == 0) index.cancel();
        };
        await index.indexIncremental(excludedDocumentIds: {second.id});
        expect(index.progress.isRunning, isFalse);
        expect(search.indexedDocumentCount, 0);
        expect(engine.closedDocumentIds, ['fake-1', 'fake-2']);
      },
    );
  });

  group('folder import, recovery and profiles', () {
    test(
      'folder preview detects duplicates and import closes every PDF',
      () async {
        final directory = await Directory.systemTemp.createTemp(
          'yamantaka-folder-import-',
        );
        addTearDown(() => directory.delete(recursive: true));
        final existing = File('${directory.path}/existing.pdf')
          ..writeAsBytesSync('%PDF-existing'.codeUnits);
        File(
          '${directory.path}/duplicate.pdf',
        ).writeAsBytesSync('%PDF-existing'.codeUnits);
        File(
          '${directory.path}/new.pdf',
        ).writeAsBytesSync('%PDF-new'.codeUnits);
        File('${directory.path}/empty.pdf').writeAsBytesSync(const []);
        File('${directory.path}/ignored.txt').writeAsStringSync('not a PDF');

        final database = AppDatabase.inMemory(AppLogger());
        addTearDown(database.close);
        final documents = DocumentRepository(database);
        final service = DocumentService(documents);
        service.register(
          file: await service.inspect(existing.path),
          pageCount: 1,
        );
        final engine = _RecordingPdfEngine(pageCount: 2);
        final controller = FolderImportController(documents, service, engine);
        addTearDown(controller.dispose);

        final candidates = await controller.scan(directory.path);
        expect(candidates, hasLength(4));
        expect(
          candidates.where(
            (item) => item.status == FolderCandidateStatus.duplicate,
          ),
          hasLength(2),
        );
        expect(
          candidates.where(
            (item) => item.status == FolderCandidateStatus.newDocument,
          ),
          hasLength(1),
        );
        expect(
          candidates.where(
            (item) => item.status == FolderCandidateStatus.invalid,
          ),
          hasLength(1),
        );

        final summary = await controller.importNew();
        expect(summary.imported, 1);
        expect(engine.openedPaths.single, endsWith('new.pdf'));
        expect(engine.closedDocumentIds, ['fake-1']);
      },
    );

    test('recovery only auto-associates exact fingerprints', () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-recovery-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final exact = File('${directory.path}/moved.pdf')
        ..writeAsBytesSync('%PDF-exact'.codeUnits);
      final uncertain = File('${directory.path}/uncertain.pdf')
        ..writeAsBytesSync('%PDF-uncertain'.codeUnits);

      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final service = DocumentService(documents);
      final inspected = await service.inspect(exact.path);
      final uncertainStat = await uncertain.stat();
      final missingExact =
          _document(
            id: 'missing-exact',
            path: '${directory.path}/missing.pdf',
          ).copyWith(
            fileName: inspected.fileName,
            fileSize: inspected.fileSize,
            fileModifiedAt: inspected.modifiedAt,
            fingerprint: inspected.fingerprint,
            isMissing: true,
          );
      final missingUncertain =
          _document(
            id: 'missing-uncertain',
            path: '${directory.path}/uncertain.pdf.missing',
          ).copyWith(
            fileName: 'uncertain.pdf',
            fileSize: uncertainStat.size,
            fileModifiedAt: uncertainStat.modified,
            fingerprint: 'non-matching-fingerprint',
            isMissing: true,
          );
      documents
        ..save(missingExact)
        ..save(missingUncertain);
      final recovery = DocumentRecoveryService(documents, service);

      final matches = await recovery.findMatches(directory.path);
      expect(
        matches
            .firstWhere((item) => item.document.id == missingExact.id)
            .confidence,
        RecoveryConfidence.exact,
      );
      expect(
        matches
            .firstWhere((item) => item.document.id == missingUncertain.id)
            .confidence,
        isNot(RecoveryConfidence.exact),
      );
      final summary = await recovery.recoverExact(matches);
      expect(summary.recovered, 1);
      expect(
        documents.byId(missingExact.id)?.normalizedPath,
        inspected.normalizedPath,
      );
      expect(documents.byId(missingUncertain.id)?.isMissing, isTrue);
    });

    test('six built-in profiles apply bounded preferences', () {
      expect(BuiltinReadingProfiles.all, hasLength(6));
      const base = DocumentPreferences(documentId: 'document');
      for (final profile in BuiltinReadingProfiles.all) {
        final applied = profile.apply(base);
        expect(applied.documentId, base.documentId);
        expect(applied.horizontalMargin, inInclusiveRange(0, 64));
        expect(applied.readingTheme, profile.theme);
        if (profile.electronicInk) {
          expect(applied.pageLayout, PageLayoutMode.single);
        }
      }
    });
  });

  group('tabs, workspaces, history and split policy', () {
    test('tabs deduplicate, suspend old resources and reopen', () {
      final tabs = DocumentTabsController(maxTabs: 4, maxWarmTabs: 1);
      addTearDown(tabs.dispose);
      final first = tabs.open(_document(id: 'one', path: r'C:\one.pdf'));
      final second = tabs.open(_document(id: 'two', path: r'C:\two.pdf'));
      final third = tabs.open(_document(id: 'three', path: r'C:\three.pdf'));

      expect(tabs.tabs, hasLength(3));
      expect(tabs.activeTabId, third.id);
      expect(tabs.tabs.first.resourceState, TabResourceState.suspended);
      expect(tabs.tabs[1].resourceState, TabResourceState.warm);
      expect(
        tabs.open(_document(id: 'two', path: r'C:\two.pdf')).id,
        second.id,
      );
      expect(tabs.tabs, hasLength(3));

      tabs.close(first.id);
      expect(tabs.canReopenClosed, isTrue);
      expect(tabs.reopenLastClosed()?.documentId, 'one');
      expect(tabs.tabs, hasLength(3));
    });

    test('workspace round-trip restores lightweight tab state', () {
      final database = AppDatabase.inMemory(AppLogger());
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final firstDocument = _document(id: 'one', path: r'C:\one.pdf');
      final secondDocument = _document(id: 'two', path: r'C:\two.pdf');
      documents
        ..save(firstDocument)
        ..save(secondDocument);
      final tabs = DocumentTabsController()
        ..open(firstDocument)
        ..open(secondDocument);
      addTearDown(tabs.dispose);
      final repository = WorkspaceRepository(database, documents);
      final saved = repository.save(
        name: 'Estudio',
        tabs: tabs.tabs,
        activeTabId: tabs.activeTabId,
        panelSection: 'annotations',
        panelWidth: 360,
        split: const SplitViewState(
          enabled: true,
          syncZoom: true,
          comparisonMode: true,
          overlayOpacity: 0.5,
        ),
      );
      final loaded = repository.all().single;

      expect(loaded.id, saved.id);
      expect(loaded.tabs, hasLength(2));
      expect(loaded.panelSection, 'annotations');
      expect(loaded.panelWidth, 360);
      expect(loaded.split.enabled, isTrue);
      expect(loaded.split.hasSafeOverlay, isTrue);
    });

    test('navigation history ignores nearby repeats and supports forward', () {
      final history = NavigationHistoryController();
      addTearDown(history.dispose);
      history.reset(_location(0, NavigationReason.history));
      history.record(_location(10, NavigationReason.searchResult));
      history.record(_location(20, NavigationReason.annotation));

      expect(history.back()?.pageIndex, 10);
      expect(history.back()?.pageIndex, 0);
      expect(history.forward()?.pageIndex, 10);
    });

    test(
      'split policy has exact responsive boundary and conservative budget',
      () {
        expect(SplitViewPolicy.isAvailable(1099.9), isFalse);
        expect(SplitViewPolicy.isAvailable(1100), isTrue);
        expect(SplitViewPolicy.cacheBudgetFactor(true), lessThan(1));
        expect(const SplitViewState().copyWith(ratio: 0.95).ratio, 0.8);
      },
    );
  });
}

LibraryDocument _document({
  String id = 'document-1',
  String path = r'C:\library\document.pdf',
}) => LibraryDocument(
  id: id,
  path: path,
  normalizedPath: path.toLowerCase(),
  fileName: path.split(r'\').last,
  title: 'Documento de prueba',
  author: 'Autora',
  pageCount: 12,
  fileSize: 2048,
  fileModifiedAt: DateTime.utc(2026, 7, 29),
  addedAt: DateTime.utc(2026, 7, 29),
  lastOpenedAt: DateTime.utc(2026, 7, 29),
  fingerprint: 'fingerprint-$id',
);

NavigationLocation _location(int page, NavigationReason reason) =>
    NavigationLocation(
      pageIndex: page,
      offset: 0,
      zoom: 1,
      reason: reason,
      label: 'Página ${page + 1}',
    );

final class _RecordingPdfEngine extends StubPdfDocumentEngine {
  _RecordingPdfEngine({
    required this.pageCount,
    this.pageText = _emptyPageText,
  });

  final int pageCount;
  final String Function(int page) pageText;
  final List<String> openedPaths = [];
  final List<String> closedDocumentIds = [];
  void Function(int page)? onExtract;

  static String _emptyPageText(int page) => '';

  @override
  Future<OpenedPdfDocument> openDocument(
    String path, {
    String? password,
  }) async {
    openedPaths.add(path);
    final id = 'fake-${openedPaths.length}';
    return OpenedPdfDocument(
      documentId: id,
      path: path,
      pageCount: pageCount,
      pageSizes: [
        for (var index = 0; index < pageCount; index++)
          const PdfPageSize(612, 792),
      ],
      isEncrypted: false,
    );
  }

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
  }) async {
    onExtract?.call(pageIndex);
    return pageText(pageIndex);
  }

  @override
  Future<void> closeDocument(String documentId) async {
    closedDocumentIds.add(documentId);
  }
}
