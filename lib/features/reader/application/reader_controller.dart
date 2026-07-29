// ignore_for_file: prefer_initializing_formals

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../app/app_config.dart';
import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../core/platform/window_service.dart';
import '../../../pdf_engine/cache/thumbnail_service.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/preferences_repository.dart';
import '../../../storage/repositories/reading_session_repository.dart';
import '../../bookmarks/application/bookmark_controller.dart';
import '../../library/application/document_service.dart';
import '../../library/domain/library_document.dart';
import '../../search/application/search_controller.dart';
import '../domain/reader_models.dart';
import '../domain/reading_session.dart';

enum ReaderStatus { idle, loading, ready, failure }

int _defaultHistoryLimit() => 250;

final class ReaderController extends ChangeNotifier {
  ReaderController({
    required DocumentService documentService,
    required DocumentRepository documents,
    required DocumentPreferencesRepository preferences,
    required ReadingSessionRepository sessions,
    required PdfDocumentEngine engine,
    required ThumbnailService thumbnails,
    required WindowService windowService,
    required AppLogger logger,
    required this.bookmarks,
    required this.search,
    required this.viewController,
    required bool Function() shouldGenerateCovers,
    required bool Function() shouldRecordSessions,
    int Function()? readingHistoryLimit,
    required bool Function() shouldRestorePosition,
    required DocumentPreferences Function(String documentId) defaultPreferences,
  }) : _documentService = documentService,
       _documents = documents,
       _preferences = preferences,
       _sessions = sessions,
       _engine = engine,
       _thumbnails = thumbnails,
       _windowService = windowService,
       _logger = logger,
       _shouldGenerateCovers = shouldGenerateCovers,
       _shouldRecordSessions = shouldRecordSessions,
       _readingHistoryLimit = readingHistoryLimit ?? _defaultHistoryLimit,
       _shouldRestorePosition = shouldRestorePosition,
       _defaultPreferences = defaultPreferences {
    viewController.addListener(_onViewerStateChanged);
  }

  final DocumentService _documentService;
  final DocumentRepository _documents;
  final DocumentPreferencesRepository _preferences;
  final ReadingSessionRepository _sessions;
  final PdfDocumentEngine _engine;
  final ThumbnailService _thumbnails;
  final WindowService _windowService;
  final AppLogger _logger;
  final bool Function() _shouldGenerateCovers;
  final bool Function() _shouldRecordSessions;
  final int Function() _readingHistoryLimit;
  final bool Function() _shouldRestorePosition;
  final DocumentPreferences Function(String documentId) _defaultPreferences;
  final BookmarkController bookmarks;
  final SearchController search;
  final PdfViewController viewController;

  ReaderStatus status = ReaderStatus.idle;
  LibraryDocument? document;
  OpenedPdfDocument? openedDocument;
  DocumentPreferences? preferences;
  AppFailure? error;
  String? pendingPath;
  List<PdfOutlineItem> outline = const [];
  int currentPageIndex = 0;
  bool isFullScreen = false;

  Timer? _saveTimer;
  Timer? _activityTimer;
  ReadingSession? _session;
  DateTime _lastActivity = DateTime.now();
  int _activeSeconds = 0;
  bool _isVisible = true;

  int get pageCount => openedDocument?.pageCount ?? 0;
  bool get canGoBack => currentPageIndex > 0;
  bool get canGoForward => currentPageIndex + 1 < pageCount;

  Future<bool> openPath(String path, {String? password}) async {
    pendingPath = path;
    status = ReaderStatus.loading;
    error = null;
    notifyListeners();
    OpenedPdfDocument? newlyOpened;
    try {
      final inspected = await _documentService.inspect(path);
      newlyOpened = await _engine.openDocument(
        inspected.path,
        password: password,
      );
      final libraryDocument = _documentService.register(
        file: inspected,
        pageCount: newlyOpened.pageCount,
      );
      await _closeCurrent();
      document = libraryDocument;
      openedDocument = newlyOpened;
      newlyOpened = null;
      preferences = _preferences.exists(libraryDocument.id)
          ? _preferences.load(libraryDocument.id)
          : _defaultPreferences(libraryDocument.id);
      currentPageIndex = _shouldRestorePosition()
          ? libraryDocument.lastPageIndex.clamp(0, pageCount - 1)
          : 0;
      bookmarks.load(libraryDocument.id);
      search.attach(openedDocument!.documentId, pageCount);
      outline = const [];
      status = ReaderStatus.ready;
      pendingPath = null;
      registerActivity();
      _startSession();
      notifyListeners();
      unawaited(_loadOutline());
      if (_shouldGenerateCovers() &&
          (libraryDocument.thumbnailPath == null ||
              libraryDocument.isMissing)) {
        unawaited(_generateCover());
      }
      return true;
    } on AppFailure catch (failure) {
      if (newlyOpened != null) {
        await _engine.closeDocument(newlyOpened.documentId);
      }
      error = failure;
      status = openedDocument == null
          ? ReaderStatus.failure
          : ReaderStatus.ready;
      pendingPath = null;
      notifyListeners();
      return false;
    } on Object catch (exception, stackTrace) {
      if (newlyOpened != null) {
        await _engine.closeDocument(newlyOpened.documentId);
      }
      _logger.error('Reader open flow failed.', exception, stackTrace);
      error = AppFailure(
        FailureKind.unknown,
        'No se pudo abrir el documento.',
        debugMessage: exception.toString(),
      );
      status = openedDocument == null
          ? ReaderStatus.failure
          : ReaderStatus.ready;
      pendingPath = null;
      notifyListeners();
      return false;
    }
  }

  Future<void> _loadOutline() async {
    final opened = openedDocument;
    if (opened == null) return;
    try {
      final result = await _engine.getOutline(opened.documentId);
      if (openedDocument?.documentId == opened.documentId) {
        outline = result;
        notifyListeners();
      }
    } on Object catch (exception) {
      _logger.warning('Outline load failed.', exception);
    }
  }

  Future<void> _generateCover() async {
    final libraryDocument = document;
    final opened = openedDocument;
    if (libraryDocument == null || opened == null) return;
    try {
      await _thumbnails.generateCover(
        libraryDocumentId: libraryDocument.id,
        engineDocumentId: opened.documentId,
      );
    } on Object {
      // Thumbnail failure is already logged and must never block reading.
    }
  }

  Future<void> closeDocument() async {
    await _closeCurrent();
    status = ReaderStatus.idle;
    error = null;
    pendingPath = null;
    notifyListeners();
  }

  Future<void> _closeCurrent() async {
    _saveTimer?.cancel();
    _savePositionNow();
    _finishSession();
    search.reset();
    bookmarks.clear();
    final opened = openedDocument;
    document = null;
    openedDocument = null;
    preferences = null;
    viewController.detach();
    outline = const [];
    currentPageIndex = 0;
    if (opened != null) await _engine.closeDocument(opened.documentId);
  }

  void onPageChanged(int pageIndex) {
    if (pageCount == 0) return;
    currentPageIndex = pageIndex.clamp(0, pageCount - 1);
    registerActivity();
    _scheduleSave();
    notifyListeners();
  }

  Future<void> previousPage() async {
    if (!canGoBack) return;
    await viewController.goToPage(currentPageIndex - 1);
  }

  Future<void> nextPage() async {
    if (!canGoForward) return;
    await viewController.goToPage(currentPageIndex + 1);
  }

  Future<void> firstPage() => viewController.goToPage(0);

  Future<void> lastPage() =>
      viewController.goToPage((pageCount - 1).clamp(0, pageCount));

  Future<void> goToPage(int oneBasedPage) =>
      viewController.goToPage(oneBasedPage.clamp(1, pageCount) - 1);

  Future<void> resetProgress() async {
    final currentDocument = document;
    if (currentDocument == null) return;
    await viewController.goToPage(0);
    currentPageIndex = 0;
    _documents.resetProgress(currentDocument.id);
    final currentPreferences = preferences;
    if (currentPreferences != null) {
      preferences = currentPreferences.copyWith(scrollOffset: 0);
      _preferences.save(preferences!);
    }
    document = currentDocument.copyWith(lastPageIndex: 0, readingProgress: 0);
    notifyListeners();
  }

  Future<void> zoomBy(double factor) async {
    _setZoomMode(ZoomMode.custom);
    await viewController.zoomBy(factor);
  }

  Future<void> fitWidth() async {
    _setZoomMode(ZoomMode.fitWidth);
    await viewController.fitWidth();
  }

  Future<void> fitPage() async {
    _setZoomMode(ZoomMode.fitPage);
    await viewController.fitPage();
  }

  void _setZoomMode(ZoomMode mode) {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(zoomMode: mode);
    _scheduleSave();
    notifyListeners();
  }

  void _onViewerStateChanged() {
    final current = preferences;
    if (current == null || !viewController.isReady) return;
    final zoomChanged = (current.zoomLevel - viewController.zoom).abs() > 0.001;
    final offsetChanged =
        (current.scrollOffset - viewController.scrollOffset).abs() > 0.001;
    if (!zoomChanged && !offsetChanged) return;
    preferences = current.copyWith(
      zoomLevel: viewController.zoom,
      scrollOffset: viewController.scrollOffset,
    );
    _scheduleSave();
  }

  void setReaderMode(ReaderMode mode) {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(
      viewMode: mode,
      pageLayout: mode == ReaderMode.book
          ? PageLayoutMode.single
          : current.pageLayout,
    );
    _scheduleSave();
    notifyListeners();
  }

  void setTheme(ReaderTheme theme) {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(readingTheme: theme);
    _scheduleSave();
    notifyListeners();
  }

  void setPageLayout(PageLayoutMode layout) {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(pageLayout: layout);
    _scheduleSave();
    notifyListeners();
  }

  void rotateClockwise() {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(
      pageRotation: (current.pageRotation + 90) % 360,
    );
    _scheduleSave();
    notifyListeners();
  }

  void setMargins(double margin) {
    final current = preferences;
    if (current == null) return;
    preferences = current.copyWith(
      horizontalMargin: margin,
      verticalMargin: margin,
    );
    _scheduleSave();
    notifyListeners();
  }

  Future<void> toggleFullScreen() async {
    await _windowService.toggleFullScreen();
    isFullScreen = await _windowService.isFullScreen();
    notifyListeners();
  }

  Future<void> leaveFullScreen() async {
    if (!await _windowService.isFullScreen()) return;
    await _windowService.setFullScreen(false);
    isFullScreen = false;
    notifyListeners();
  }

  void toggleBookmark({String? label}) {
    bookmarks.toggle(currentPageIndex, label: label);
    registerActivity();
  }

  void _scheduleSave() {
    _saveTimer?.cancel();
    _saveTimer = Timer(AppConfig.positionSaveDebounce, _savePositionNow);
  }

  void _savePositionNow() {
    final libraryDocument = document;
    final currentPreferences = preferences;
    if (libraryDocument == null || currentPreferences == null) return;
    try {
      _documents.updateProgress(
        documentId: libraryDocument.id,
        pageIndex: currentPageIndex,
        pageCount: pageCount,
        viewMode: currentPreferences.viewMode,
        theme: currentPreferences.readingTheme,
      );
      _preferences.save(currentPreferences);
    } on Object catch (exception, stackTrace) {
      _logger.error('Reading position save failed.', exception, stackTrace);
    }
  }

  void registerActivity() {
    _lastActivity = DateTime.now();
  }

  void setVisible(bool value) {
    _isVisible = value;
    if (value) registerActivity();
  }

  void _startSession() {
    _activityTimer?.cancel();
    _activeSeconds = 0;
    final libraryDocument = document;
    if (libraryDocument == null || !_shouldRecordSessions()) return;
    _session = _sessions.start(libraryDocument.id, currentPageIndex);
    _sessions.trimToLimit(_readingHistoryLimit());
    _activityTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (_isVisible &&
          DateTime.now().difference(_lastActivity) <
              const Duration(seconds: 30)) {
        _activeSeconds++;
      }
    });
  }

  void _finishSession() {
    _activityTimer?.cancel();
    _activityTimer = null;
    final session = _session;
    _session = null;
    if (session != null) {
      _sessions.finish(
        id: session.id,
        endPage: currentPageIndex,
        activeSeconds: _activeSeconds,
      );
    }
  }

  @override
  void dispose() {
    viewController.removeListener(_onViewerStateChanged);
    _saveTimer?.cancel();
    _activityTimer?.cancel();
    _savePositionNow();
    _finishSession();
    unawaited(_closeEngineDocument());
    super.dispose();
  }

  Future<void> _closeEngineDocument() async {
    final opened = openedDocument;
    if (opened != null) await _engine.closeDocument(opened.documentId);
  }
}
