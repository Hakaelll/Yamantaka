// ignore_for_file: prefer_initializing_formals

import 'dart:io';

import '../core/logging/app_logger.dart';
import '../core/platform/window_service.dart';
import '../features/bookmarks/application/bookmark_controller.dart';
import '../features/library/application/document_service.dart';
import '../features/library/application/library_controller.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/domain/reader_models.dart';
import '../features/search/application/search_controller.dart';
import '../features/settings/application/settings_controller.dart';
import '../pdf_engine/adapters/pdfrx_document_engine.dart';
import '../pdf_engine/adapters/pdfrx_viewer_adapter.dart';
import '../pdf_engine/cache/thumbnail_service.dart';
import '../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../pdf_engine/pdf_document_engine.dart';
import '../storage/database/app_database.dart';
import '../storage/paths/app_paths.dart';
import '../storage/repositories/app_settings_repository.dart';
import '../storage/repositories/bookmark_repository.dart';
import '../storage/repositories/document_repository.dart';
import '../storage/repositories/preferences_repository.dart';
import '../storage/repositories/reading_session_repository.dart';

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
  final PdfrxDocumentEngine _engine;

  PdfDocumentEngine get engine => _engine;

  static Future<AppServices> create({
    required AppPaths paths,
    required AppLogger logger,
    SettingsController? preloadedSettings,
  }) async {
    final database = AppDatabase.open(paths.databasePath, logger);
    final documents = DocumentRepository(database);
    final documentPreferences = DocumentPreferencesRepository(database);
    final bookmarksRepository = BookmarkRepository(database);
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
    final bookmarks = BookmarkController(bookmarksRepository);
    final search = SearchController(engine, viewController, logger);
    final library = LibraryController(documents, logger);
    final reader = ReaderController(
      documentService: DocumentService(documents),
      documents: documents,
      preferences: documentPreferences,
      sessions: sessions,
      engine: engine,
      thumbnails: thumbnails,
      windowService: const DesktopWindowService(),
      logger: logger,
      bookmarks: bookmarks,
      search: search,
      viewController: viewController,
      shouldGenerateCovers: () => settings.settings.generateCovers,
      shouldRecordSessions: () => settings.settings.recordReadingSessions,
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
    await library.load();
    return AppServices._(
      paths: paths,
      database: database,
      logger: logger,
      library: library,
      reader: reader,
      settings: settings,
      viewerAdapter: PdfrxViewerAdapter(engine),
      thumbnails: thumbnails,
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
    database.clearUserData();
    await settings.clear();
    await clearCache();
    await library.load();
  }

  Future<void> deleteHistory() async {
    await reader.closeDocument();
    database.clearUserData();
    await clearCache();
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

  Future<void> dispose() async {
    await reader.closeDocument();
    reader.dispose();
    library.dispose();
    settings.dispose();
    await _engine.dispose();
    database.close();
  }
}
