// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:io';

import '../core/async/job_coordinator.dart';
import '../core/lifecycle/lifecycle_tracker.dart';
import '../core/logging/app_logger.dart';
import '../core/memory/memory_governor.dart';
import '../core/performance/startup_timeline.dart';
import '../core/performance/local_performance_monitor.dart';
import '../core/privacy/local_storage_service.dart';
import '../core/platform/window_service.dart';
import '../features/annotations/application/annotation_controller.dart';
import '../features/annotations/application/annotation_export_service.dart';
import '../features/automations/application/local_automation_engine.dart';
import '../features/bookmarks/application/bookmark_controller.dart';
import '../features/library/application/document_service.dart';
import '../features/library/application/document_enrichment_controller.dart';
import '../features/library/application/duplicate_resolution_service.dart';
import '../features/library/application/library_bulk_operation_service.dart';
import '../features/library/application/library_controller.dart';
import '../features/library/application/library_file_validation_controller.dart';
import '../features/library/application/folder_import_controller.dart';
import '../features/library/application/folder_watch_controller.dart';
import '../features/library/application/document_recovery_service.dart';
import '../features/navigation/application/navigation_history_controller.dart';
import '../features/diagnostics/application/diagnostics_service.dart';
import '../features/organization/application/organization_controller.dart';
import '../features/profiles/application/reading_profiles_controller.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/domain/reader_models.dart';
import '../features/search/application/search_controller.dart';
import '../features/search/application/global_search_controller.dart';
import '../features/search/application/library_index_controller.dart';
import '../features/settings/application/settings_controller.dart';
import '../pdf_engine/adapters/pdfrx_document_engine.dart';
import '../pdf_engine/adapters/pdfrx_viewer_adapter.dart';
import '../pdf_engine/cache/thumbnail_service.dart';
import '../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../pdf_engine/pdf_document_engine.dart';
import '../storage/database/app_database.dart';
import '../storage/backup/local_backup_service.dart';
import '../storage/paths/app_paths.dart';
import '../storage/repositories/app_settings_repository.dart';
import '../storage/repositories/annotation_repository.dart';
import '../storage/repositories/automation_repository.dart';
import '../storage/repositories/bookmark_repository.dart';
import '../storage/repositories/document_repository.dart';
import '../storage/repositories/document_relation_repository.dart';
import '../storage/repositories/global_search_repository.dart';
import '../storage/repositories/import_queue_repository.dart';
import '../storage/repositories/organization_repository.dart';
import '../storage/repositories/preferences_repository.dart';
import '../storage/repositories/reading_session_repository.dart';
import '../storage/repositories/recovery_repository.dart';
import '../storage/repositories/workspace_repository.dart';
import '../features/workspaces/application/document_tabs_controller.dart';
import '../features/workspaces/application/workspace_controller.dart';

final class AppServices {
  AppServices._({
    required this.paths,
    required this.database,
    required this.logger,
    required this.library,
    required this.fileValidation,
    required this.reader,
    required this.settings,
    required this.viewerAdapter,
    required this.thumbnails,
    required this.documents,
    required this.readingSessions,
    required this.annotations,
    required this.annotationExporter,
    required this.annotationRepository,
    required this.automationRepository,
    required this.automations,
    required this.relations,
    required this.recovery,
    required this.tabs,
    required this.workspaces,
    required this.organization,
    required this.globalSearch,
    required this.libraryIndex,
    required this.enrichment,
    required this.folderImport,
    required this.folderWatch,
    required this.documentRecovery,
    required this.diagnostics,
    required this.readingProfiles,
    required this.safeMode,
    required this.backups,
    required this.documentService,
    required this.bulkOperations,
    required this.duplicateResolution,
    required this.jobs,
    required this.memory,
    required this.lifecycle,
    required this.startup,
    required this.performance,
    required this.storage,
    required void Function() memorySettingsListener,
    required PdfrxDocumentEngine engine,
  }) : _engine = engine,
       _memorySettingsListener = memorySettingsListener;

  final AppPaths paths;
  final AppDatabase database;
  final AppLogger logger;
  final LibraryController library;
  final LibraryFileValidationController fileValidation;
  final ReaderController reader;
  final SettingsController settings;
  final PdfrxViewerAdapter viewerAdapter;
  final ThumbnailService thumbnails;
  final DocumentRepository documents;
  final ReadingSessionRepository readingSessions;
  final AnnotationController annotations;
  final AnnotationExportService annotationExporter;
  final AnnotationRepository annotationRepository;
  final AutomationRepository automationRepository;
  final LocalAutomationEngine automations;
  final DocumentRelationRepository relations;
  final RecoveryRepository recovery;
  final DocumentTabsController tabs;
  final WorkspaceController workspaces;
  final OrganizationController organization;
  final GlobalLibrarySearchController globalSearch;
  final LibraryIndexController libraryIndex;
  final DocumentEnrichmentController enrichment;
  final FolderImportController folderImport;
  final FolderWatchController folderWatch;
  final DocumentRecoveryService documentRecovery;
  final DiagnosticsService diagnostics;
  final ReadingProfilesController readingProfiles;
  final bool safeMode;
  final LocalBackupService backups;
  final DocumentService documentService;
  final LibraryBulkOperationService bulkOperations;
  final DuplicateResolutionService duplicateResolution;
  final JobCoordinator jobs;
  final MemoryGovernor memory;
  final LifecycleTracker lifecycle;
  final StartupTimeline startup;
  final LocalPerformanceMonitor performance;
  final LocalStorageService storage;
  final PdfrxDocumentEngine _engine;
  final void Function() _memorySettingsListener;

  PdfDocumentEngine get engine => _engine;

  static Future<AppServices> create({
    required AppPaths paths,
    required AppLogger logger,
    SettingsController? preloadedSettings,
    StartupTimeline? startupTimeline,
  }) async {
    final startup = startupTimeline ?? StartupTimeline();
    final databaseSpan = startup.start(StartupPhase.database);
    final database = AppDatabase.open(paths.databasePath, logger);
    final storage = LocalStorageService(paths);
    final safeMode = database.recordStartup();
    databaseSpan.end();
    final documents = DocumentRepository(database);
    final documentPreferences = DocumentPreferencesRepository(database);
    final bookmarksRepository = BookmarkRepository(database);
    final annotationsRepository = AnnotationRepository(database);
    final annotations = AnnotationController(annotationsRepository);
    final organizationRepository = OrganizationRepository(database);
    final organization = OrganizationController(organizationRepository);
    final globalSearchRepository = GlobalSearchRepository(database);
    final sessions = ReadingSessionRepository(database);
    final automationRepository = AutomationRepository(database)
      ..markRunningAsInterrupted();
    final automations = LocalAutomationEngine(automationRepository);
    final relations = DocumentRelationRepository(database);
    final recovery = RecoveryRepository(database);
    final settings =
        preloadedSettings ??
        SettingsController(AppSettingsRepository(), logger);
    if (!settings.isLoaded) await settings.load();
    final lifecycle = LifecycleTracker();
    final jobs = JobCoordinator(
      maxConcurrent: 4,
      categoryLimits: {
        JobCategory.render: 2,
        JobCategory.thumbnail: 1,
        JobCategory.indexing: 1,
        JobCategory.import: 2,
        JobCategory.backup: 1,
        JobCategory.recovery: 1,
        JobCategory.maintenance: 1,
      },
      lifecycle: lifecycle,
    );
    final memory = MemoryGovernor(
      limitBytes: settings.settings.maxCacheMegabytes * 1024 * 1024,
      minimumVisibleBytes: 32 * 1024 * 1024,
    );
    final performance = LocalPerformanceMonitor()..startFrameMonitoring();
    final globalSearch = GlobalLibrarySearchController(
      globalSearchRepository,
      performance,
    );
    void syncMemoryLimit() {
      memory.setLimit(settings.settings.maxCacheMegabytes * 1024 * 1024);
    }

    settings.addListener(syncMemoryLimit);
    final engine = PdfrxDocumentEngine(
      logger,
      lifecycle: lifecycle,
      memory: memory,
      jobs: jobs,
      performance: performance,
    );
    final thumbnails = ThumbnailService(
      engine: engine,
      paths: paths,
      documents: documents,
      logger: logger,
      maxCacheBytes: () => settings.settings.maxCacheMegabytes * 1024 * 1024,
      jobs: jobs,
    );
    final bulkOperations = LibraryBulkOperationService(
      documents: documents,
      organization: organizationRepository,
      regenerateCover: (documentId) async {
        final document = documents.byId(documentId);
        if (document != null && !document.isMissing) {
          await thumbnails.restoreAutomaticCover(document);
        }
      },
      cleanupBeforeRemoval: (documentId) async {
        final document = documents.byId(documentId);
        if (document != null) await thumbnails.removeForDocument(document);
      },
    );
    final viewController = PdfViewController();
    final documentService = DocumentService(documents);
    final bookmarks = BookmarkController(bookmarksRepository);
    final search = SearchController(engine, viewController, logger);
    final navigationHistory = NavigationHistoryController();
    final library = LibraryController(documents, logger, performance);
    final fileValidation = LibraryFileValidationController(
      documents: documents,
      jobs: jobs,
      logger: logger,
      onDocumentsChanged: library.refreshFileStates,
    );
    final enrichment = DocumentEnrichmentController(documents, engine);
    thumbnails.onCoverUpdated = () => unawaited(library.load());
    final reader = ReaderController(
      documentService: documentService,
      documents: documents,
      preferences: documentPreferences,
      sessions: sessions,
      engine: engine,
      thumbnails: thumbnails,
      windowService: DesktopWindowService(),
      logger: logger,
      annotations: annotations,
      bookmarks: bookmarks,
      navigationHistory: navigationHistory,
      search: search,
      viewController: viewController,
      shouldGenerateCovers: () => settings.settings.generateCovers,
      shouldRecordSessions: () => settings.settings.recordReadingSessions,
      readingHistoryLimit: () => settings.settings.readingHistoryLimit,
      shouldRestorePosition: () => settings.settings.restoreLastPosition,
      defaultPreferences: (documentId) {
        final global = settings.settings;
        T enumValue<T extends Enum>(List<T> values, String name, T fallback) =>
            values.where((value) => value.name == name).firstOrNull ?? fallback;
        return DocumentPreferences(
          documentId: documentId,
          viewMode: enumValue(
            ReaderMode.values,
            global.defaultViewMode,
            ReaderMode.document,
          ),
          readingTheme: enumValue(
            ReaderTheme.values,
            global.defaultReaderTheme,
            ReaderTheme.ivory,
          ),
          zoomMode: enumValue(
            ZoomMode.values,
            global.defaultZoomMode,
            ZoomMode.fitPage,
          ),
          pageLayout: enumValue(
            PageLayoutMode.values,
            global.defaultPageLayout,
            PageLayoutMode.single,
          ),
          horizontalMargin: global.defaultMargin,
          verticalMargin: global.defaultMargin,
          controlsAutoHide: global.autoHideControls,
        );
      },
    );
    reader.addListener(() {
      final readerActive =
          reader.status == ReaderStatus.loading ||
          reader.status == ReaderStatus.ready;
      enrichment.setReaderPaused(readerActive);
      fileValidation.setReaderPaused(readerActive);
      if (reader.status == ReaderStatus.idle &&
          !enrichment.progress.isRunning) {
        unawaited(enrichment.analyzePending().whenComplete(library.reload));
      }
    });
    final tabs = DocumentTabsController();
    final workspaces = WorkspaceController(
      WorkspaceRepository(database, documents),
      tabs,
    );
    final libraryIndex = LibraryIndexController(
      documents,
      globalSearchRepository,
      engine,
      jobs,
    );
    final folderImport = FolderImportController(
      documents,
      documentService,
      engine,
      thumbnails,
      jobs,
      ImportQueueRepository(database),
    );
    final folderWatch = FolderWatchController();
    folderWatch.onRevalidationReady = (events) {
      if (events.isNotEmpty) unawaited(library.load());
    };
    final documentRecovery = DocumentRecoveryService(
      documents,
      documentService,
    );
    final diagnostics = DiagnosticsService(
      database: database,
      paths: paths,
      engine: engine,
      safeMode: safeMode,
      jobs: jobs,
      memory: memory,
      lifecycle: lifecycle,
      startup: startup,
      performance: performance,
      thumbnails: thumbnails,
    );
    final readingProfiles = ReadingProfilesController(reader);
    final backups = LocalBackupService(
      database: database,
      paths: paths,
      settings: settings,
    );
    final duplicateResolution = DuplicateResolutionService(
      database: database,
      documents: documents,
      createPreBackup: () async {
        final backupsDirectory = Directory(
          '${paths.dataDirectory.path}/backups',
        );
        await backupsDirectory.create(recursive: true);
        final stamp = DateTime.now().toUtc().toIso8601String().replaceAll(
          ':',
          '-',
        );
        final path = '${backupsDirectory.path}/pre-merge-$stamp.json';
        await backups.exportTo(path);
        return path;
      },
    );
    final firstLibrarySpan = startup.start(StartupPhase.firstLibrary);
    await library.load();
    firstLibrarySpan.end();
    fileValidation.startIncremental();
    if (settings.settings.generateCovers) {
      for (final document in library.documents) {
        if (!document.isMissing) thumbnails.enqueueCover(document);
      }
    }
    unawaited(enrichment.analyzePending().whenComplete(library.reload));
    return AppServices._(
      paths: paths,
      database: database,
      logger: logger,
      library: library,
      fileValidation: fileValidation,
      reader: reader,
      settings: settings,
      viewerAdapter: PdfrxViewerAdapter(engine),
      thumbnails: thumbnails,
      documents: documents,
      readingSessions: sessions,
      annotations: annotations,
      annotationExporter: const AnnotationExportService(),
      annotationRepository: annotationsRepository,
      automationRepository: automationRepository,
      automations: automations,
      relations: relations,
      recovery: recovery,
      tabs: tabs,
      workspaces: workspaces,
      organization: organization,
      globalSearch: globalSearch,
      libraryIndex: libraryIndex,
      enrichment: enrichment,
      folderImport: folderImport,
      folderWatch: folderWatch,
      documentRecovery: documentRecovery,
      diagnostics: diagnostics,
      readingProfiles: readingProfiles,
      safeMode: safeMode,
      backups: backups,
      documentService: documentService,
      bulkOperations: bulkOperations,
      duplicateResolution: duplicateResolution,
      jobs: jobs,
      memory: memory,
      lifecycle: lifecycle,
      startup: startup,
      performance: performance,
      storage: storage,
      memorySettingsListener: syncMemoryLimit,
      engine: engine,
    );
  }

  Future<void> clearCache() async {
    _engine.clearCaches();
    await thumbnails.clear();
    await storage.clearRebuildable();
    await library.load();
  }

  Future<void> deleteAllLocalData() async {
    await reader.closeDocument();
    libraryIndex.cancel();
    enrichment.cancel();
    database.clearUserData();
    await settings.clear();
    await clearCache();
    await storage.clearBackups();
    await library.load();
  }

  Future<void> deleteHistory() async {
    await reader.closeDocument();
    database.clearReadingHistory();
    await library.load();
  }

  Future<void> openDataLocation() async {
    if (Platform.isWindows) {
      await Process.start('explorer.exe', [paths.dataDirectory.path]);
    } else if (Platform.isMacOS) {
      await Process.start('open', [paths.dataDirectory.path]);
    } else if (Platform.isLinux) {
      await Process.start('xdg-open', [paths.dataDirectory.path]);
    }
  }

  Future<void> revealDocument(String path) async {
    if (Platform.isWindows) {
      await Process.start('explorer.exe', ['/select,', path]);
    } else if (Platform.isMacOS) {
      await Process.start('open', ['-R', path]);
    } else if (Platform.isLinux) {
      await Process.start('xdg-open', [File(path).parent.path]);
    }
  }

  Future<void> dispose() async {
    await reader.closeDocument();
    reader.dispose();
    annotations.dispose();
    tabs.dispose();
    workspaces.dispose();
    organization.dispose();
    globalSearch.dispose();
    libraryIndex.dispose();
    enrichment.dispose();
    folderImport.dispose();
    folderWatch.dispose();
    fileValidation.dispose();
    readingProfiles.dispose();
    library.dispose();
    settings.removeListener(_memorySettingsListener);
    settings.dispose();
    await jobs.dispose();
    await _engine.dispose();
    performance.stopFrameMonitoring();
    lifecycle.dispose();
    database.markCleanShutdown();
    database.close();
  }
}
