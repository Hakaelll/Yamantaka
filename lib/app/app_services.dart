// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:io';

import '../core/logging/app_logger.dart';
import '../core/platform/window_service.dart';
import '../features/annotations/application/annotation_controller.dart';
import '../features/annotations/application/annotation_export_service.dart';
import '../features/bookmarks/application/bookmark_controller.dart';
import '../features/library/application/document_service.dart';
import '../features/library/application/library_controller.dart';
import '../features/library/application/folder_import_controller.dart';
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
import '../storage/repositories/bookmark_repository.dart';
import '../storage/repositories/document_repository.dart';
import '../storage/repositories/global_search_repository.dart';
import '../storage/repositories/organization_repository.dart';
import '../storage/repositories/preferences_repository.dart';
import '../storage/repositories/reading_session_repository.dart';
import '../storage/repositories/workspace_repository.dart';
import '../features/workspaces/application/document_tabs_controller.dart';
import '../features/workspaces/application/workspace_controller.dart';

final class AppServices {
  AppServices._({
    required this.paths,
    required this.database,
    required this.logger,
    required this.library,
    required this.reader,
    required this.settings,
    required this.viewerAdapter,
    required this.thumbnails,
    required this.documents,
    required this.readingSessions,
    required this.annotations,
    required this.annotationExporter,
    required this.tabs,
    required this.workspaces,
    required this.organization,
    required this.globalSearch,
    required this.libraryIndex,
    required this.folderImport,
    required this.documentRecovery,
    required this.diagnostics,
    required this.readingProfiles,
    required this.safeMode,
    required this.backups,
    required this.documentService,
    required PdfrxDocumentEngine engine,
  }) : _engine = engine;

  final AppPaths paths;
  final AppDatabase database;
  final AppLogger logger;
  final LibraryController library;
  final ReaderController reader;
  final SettingsController settings;
  final PdfrxViewerAdapter viewerAdapter;
  final ThumbnailService thumbnails;
  final DocumentRepository documents;
  final ReadingSessionRepository readingSessions;
  final AnnotationController annotations;
  final AnnotationExportService annotationExporter;
  final DocumentTabsController tabs;
  final WorkspaceController workspaces;
  final OrganizationController organization;
  final GlobalLibrarySearchController globalSearch;
  final LibraryIndexController libraryIndex;
  final FolderImportController folderImport;
  final DocumentRecoveryService documentRecovery;
  final DiagnosticsService diagnostics;
  final ReadingProfilesController readingProfiles;
  final bool safeMode;
  final LocalBackupService backups;
  final DocumentService documentService;
  final PdfrxDocumentEngine _engine;

  PdfDocumentEngine get engine => _engine;

  static Future<AppServices> create({
    required AppPaths paths,
    required AppLogger logger,
    SettingsController? preloadedSettings,
  }) async {
    final database = AppDatabase.open(paths.databasePath, logger);
    final safeMode = database.recordStartup();
    final documents = DocumentRepository(database);
    final documentPreferences = DocumentPreferencesRepository(database);
    final bookmarksRepository = BookmarkRepository(database);
    final annotationsRepository = AnnotationRepository(database);
    final annotations = AnnotationController(annotationsRepository);
    final organizationRepository = OrganizationRepository(database);
    final organization = OrganizationController(organizationRepository);
    final globalSearchRepository = GlobalSearchRepository(database);
    final globalSearch = GlobalLibrarySearchController(globalSearchRepository);
    final sessions = ReadingSessionRepository(database);
    final settings =
        preloadedSettings ??
        SettingsController(AppSettingsRepository(), logger);
    if (!settings.isLoaded) await settings.load();
    final engine = PdfrxDocumentEngine(logger);
    final thumbnails = ThumbnailService(
      engine: engine,
      paths: paths,
      documents: documents,
      logger: logger,
      maxCacheBytes: () => settings.settings.maxCacheMegabytes * 1024 * 1024,
    );
    final viewController = PdfViewController();
    final documentService = DocumentService(documents);
    final bookmarks = BookmarkController(bookmarksRepository);
    final search = SearchController(engine, viewController, logger);
    final navigationHistory = NavigationHistoryController();
    final library = LibraryController(documents, logger);
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
            PageLayoutMode.continuous,
          ),
          horizontalMargin: global.defaultMargin,
          verticalMargin: global.defaultMargin,
          controlsAutoHide: global.autoHideControls,
        );
      },
    );
    final tabs = DocumentTabsController();
    final workspaces = WorkspaceController(
      WorkspaceRepository(database, documents),
      tabs,
    );
    final libraryIndex = LibraryIndexController(
      documents,
      globalSearchRepository,
      engine,
    );
    final folderImport = FolderImportController(
      documents,
      documentService,
      engine,
      thumbnails,
    );
    final documentRecovery = DocumentRecoveryService(
      documents,
      documentService,
    );
    final diagnostics = DiagnosticsService(
      database: database,
      paths: paths,
      engine: engine,
      safeMode: safeMode,
    );
    final readingProfiles = ReadingProfilesController(reader);
    final backups = LocalBackupService(
      database: database,
      paths: paths,
      settings: settings,
    );
    await library.load();
    if (settings.settings.generateCovers) {
      for (final document in library.documents) {
        if (!document.isMissing) thumbnails.enqueueCover(document);
      }
    }
    return AppServices._(
      paths: paths,
      database: database,
      logger: logger,
      library: library,
      reader: reader,
      settings: settings,
      viewerAdapter: PdfrxViewerAdapter(engine),
      thumbnails: thumbnails,
      documents: documents,
      readingSessions: sessions,
      annotations: annotations,
      annotationExporter: const AnnotationExportService(),
      tabs: tabs,
      workspaces: workspaces,
      organization: organization,
      globalSearch: globalSearch,
      libraryIndex: libraryIndex,
      folderImport: folderImport,
      documentRecovery: documentRecovery,
      diagnostics: diagnostics,
      readingProfiles: readingProfiles,
      safeMode: safeMode,
      backups: backups,
      documentService: documentService,
      engine: engine,
    );
  }

  Future<void> clearCache() async {
    _engine.clearCaches();
    await thumbnails.clear();
    await library.load();
  }

  Future<void> deleteAllLocalData() async {
    await reader.closeDocument();
    libraryIndex.cancel();
    database.clearUserData();
    await settings.clear();
    await clearCache();
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
    folderImport.dispose();
    readingProfiles.dispose();
    library.dispose();
    settings.dispose();
    await _engine.dispose();
    database.markCleanShutdown();
    database.close();
  }
}
