// ignore_for_file: prefer_initializing_formals

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../app/app_config.dart';
import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../core/performance/progressive_open.dart';
import '../../../core/platform/window_service.dart';
import '../../../pdf_engine/cache/thumbnail_service.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/preferences_repository.dart';
import '../../../storage/repositories/reading_session_repository.dart';
import '../../annotations/application/annotation_controller.dart';
import '../../annotations/domain/annotation.dart';
import '../../bookmarks/application/bookmark_controller.dart';
import '../../library/application/document_service.dart';
import '../../library/domain/library_document.dart';
import '../../navigation/application/navigation_history_controller.dart';
import '../../search/application/search_controller.dart';
import '../domain/reader_models.dart';
import '../domain/reading_session.dart';
import '../domain/text_selection.dart';

enum ReaderStatus { idle, loading, ready, failure }

final class SelectionAnnotationResult {
  const SelectionAnnotationResult({
    required this.annotation,
    required this.usedPreciseGeometry,
  });

  final PdfAnnotation annotation;
  final bool usedPreciseGeometry;
}

final class _WarmReaderDocument {
  const _WarmReaderDocument({required this.document, required this.opened});

  final LibraryDocument document;
  final OpenedPdfDocument opened;
}

int _defaultHistoryLimit() => 250;

@visibleForTesting
DocumentPreferences normalizeOpeningReaderPreferences(
  DocumentPreferences restored,
) => restored.viewMode == ReaderMode.book
    ? restored.copyWith(
        pageLayout: PageLayoutMode.facing,
        zoomMode: ZoomMode.fitPage,
        scrollOffset: 0,
      )
    : restored;

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
    this.annotations,
    required this.bookmarks,
    this.navigationHistory,
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
  final AnnotationController? annotations;
  final BookmarkController bookmarks;
  final NavigationHistoryController? navigationHistory;
  final SearchController search;
  final PdfViewController viewController;

  ReaderStatus status = ReaderStatus.idle;
  LibraryDocument? document;
  OpenedPdfDocument? openedDocument;
  DocumentPreferences? preferences;
  AppFailure? error;
  String? pendingPath;
  List<PdfOutlineItem> outline = const [];
  final ValueNotifier<int> pageIndexListenable = ValueNotifier<int>(0);
  final ValueNotifier<ReaderDestinationCue?> destinationCue =
      ValueNotifier<ReaderDestinationCue?>(null);
  bool isFullScreen = false;
  ProgressiveOpenState openProgress = const ProgressiveOpenState(
    phase: ProgressiveOpenPhase.idle,
    generation: 0,
  );
  ReaderTemporaryPosition? temporaryPositionA;
  ReaderTemporaryPosition? temporaryPositionB;
  TextSelection? textSelection;
  String? sessionPanelSection;

  Timer? _saveTimer;
  Timer? _activityTimer;
  Timer? _destinationCueTimer;
  PdfOperationToken? _coverToken;
  ReadingSession? _session;
  DateTime _lastActivity = DateTime.now();
  int _activeSeconds = 0;
  bool _isVisible = true;
  int _openGeneration = 0;
  final Map<String, _WarmReaderDocument> _warmDocuments = {};

  int get pageCount => openedDocument?.pageCount ?? 0;
  int get currentPageIndex => pageIndexListenable.value;
  set currentPageIndex(int value) => pageIndexListenable.value = value;
  bool get canGoBack => currentPageIndex > 0;
  bool get canGoForward => currentPageIndex + 1 < pageCount;
  ReaderMode get readerMode => preferences?.viewMode ?? ReaderMode.document;
  bool get canGoToPreviousReadingUnit =>
      _previousReadingUnitTarget() != _canonicalReadingUnitPage();
  bool get canGoToNextReadingUnit =>
      _nextReadingUnitTarget() != _canonicalReadingUnitPage();
  bool get hasReliableTextSelection {
    final selection = textSelection;
    if (selection == null ||
        !selection.hasText ||
        selection.pages.length != 1) {
      return false;
    }
    final rects = selection.pages.single.rects;
    return rects.isNotEmpty && rects.every((rect) => rect.isMeaningful);
  }

  void updateTextSelection(TextSelection? value) {
    final hadSelection = textSelection?.hasText == true;
    final hasSelection = value?.hasText == true;
    textSelection = hasSelection ? value : null;
    if (hadSelection != hasSelection || hasSelection) notifyListeners();
  }

  void setSessionPanelSection(String? value) {
    sessionPanelSection = value;
  }

  SelectionAnnotationResult? createAnnotationFromSelection({
    required AnnotationType type,
    AnnotationColor color = AnnotationColor.yellow,
    String? note,
  }) {
    final annotationController = annotations;
    final selection = textSelection;
    if (annotationController == null ||
        selection == null ||
        !selection.hasText) {
      return null;
    }
    final ordered = selection.range.ordered();
    final geometryIsReliable = hasReliableTextSelection;
    final page = selection.pages.first;
    final start = ordered.start.offset.clamp(0, page.text.length);
    final end = ordered.end.pageIndex == page.pageIndex
        ? ordered.end.offset.clamp(start, page.text.length)
        : page.text.length;
    final beforeStart = (start - 80).clamp(0, page.text.length);
    final afterEnd = (end + 80).clamp(0, page.text.length);
    final created = annotationController.create(
      pageIndex: page.pageIndex,
      type: geometryIsReliable ? type : AnnotationType.pageComment,
      color: color,
      note: note,
      selectedText: selection.selectedText.trim(),
      contextBefore: page.text.substring(beforeStart, start).trim(),
      contextAfter: page.text.substring(end, afterEnd).trim(),
      rects: geometryIsReliable ? page.normalized().rects : const [],
      pageRotation: preferences?.pageRotation ?? 0,
    );
    if (created == null) return null;
    registerActivity();
    return SelectionAnnotationResult(
      annotation: created,
      usedPreciseGeometry: geometryIsReliable,
    );
  }

  Future<bool> openPath(String path, {String? password}) async {
    final openGeneration = ++_openGeneration;
    openProgress = ProgressiveOpenState(
      phase: ProgressiveOpenPhase.nativeOpen,
      generation: openGeneration,
    );
    pendingPath = path;
    status = ReaderStatus.loading;
    error = null;
    notifyListeners();
    OpenedPdfDocument? newlyOpened;
    try {
      final inspected = await _documentService.inspect(path);
      final current = document;
      if (current != null &&
          current.normalizedPath == inspected.normalizedPath &&
          current.fingerprint == inspected.fingerprint) {
        status = ReaderStatus.ready;
        pendingPath = null;
        notifyListeners();
        return true;
      }
      final registered = _documents.byNormalizedPath(inspected.normalizedPath);
      var warm = registered == null
          ? null
          : _warmDocuments.remove(registered.id);
      if (warm != null && warm.document.fingerprint != inspected.fingerprint) {
        await _engine.closeDocument(warm.opened.documentId);
        warm = null;
      }
      final OpenedPdfDocument selectedOpened;
      final LibraryDocument libraryDocument;
      if (warm != null) {
        selectedOpened = warm.opened;
        libraryDocument = _documentService.register(
          file: inspected,
          pageCount: selectedOpened.pageCount,
        );
      } else {
        newlyOpened = await _engine.openDocument(
          inspected.path,
          password: password,
        );
        selectedOpened = newlyOpened;
        libraryDocument = _documentService.register(
          file: inspected,
          pageCount: newlyOpened.pageCount,
        );
      }
      openProgress = ProgressiveOpenState(
        phase: ProgressiveOpenPhase.firstLayout,
        generation: openGeneration,
        pageIndex: 0,
      );
      if (openGeneration != _openGeneration) {
        await _engine.closeDocument(selectedOpened.documentId);
        newlyOpened = null;
        return false;
      }
      await _closeCurrent(retainNative: true);
      document = libraryDocument;
      openedDocument = selectedOpened;
      newlyOpened = null;
      final restoredPreferences = _preferences.exists(libraryDocument.id)
          ? _preferences.load(libraryDocument.id)
          : _defaultPreferences(libraryDocument.id);
      preferences = normalizeOpeningReaderPreferences(restoredPreferences);
      currentPageIndex = _shouldRestorePosition()
          ? libraryDocument.lastPageIndex.clamp(0, pageCount - 1)
          : 0;
      annotations?.load(libraryDocument.id, libraryDocument.fingerprint);
      textSelection = null;
      bookmarks.load(libraryDocument.id);
      search.attach(openedDocument!.documentId, pageCount);
      navigationHistory?.reset(
        NavigationLocation(
          pageIndex: currentPageIndex,
          offset: preferences?.scrollOffset ?? 0,
          zoom: preferences?.zoomLevel ?? 1,
          reason: NavigationReason.history,
          label: 'Posición inicial',
        ),
      );
      outline = const [];
      status = ReaderStatus.ready;
      openProgress = ProgressiveOpenState(
        phase: ProgressiveOpenPhase.firstRender,
        generation: openGeneration,
        pageIndex: currentPageIndex,
      );
      pendingPath = null;
      registerActivity();
      _startSession();
      openProgress = ProgressiveOpenState(
        phase: ProgressiveOpenPhase.enrichment,
        generation: openGeneration,
        pageIndex: currentPageIndex,
      );
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
      if (openGeneration != _openGeneration) return false;
      error = failure;
      openProgress = ProgressiveOpenPolicy.failure(
        openProgress,
        failure.kind.name,
      );
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
      if (openGeneration != _openGeneration) return false;
      _logger.error('Reader open flow failed.', exception, stackTrace);
      error = AppFailure(
        FailureKind.unknown,
        'No se pudo abrir el documento.',
        debugMessage: exception.toString(),
      );
      openProgress = ProgressiveOpenPolicy.failure(openProgress, 'unknown');
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
        openProgress = ProgressiveOpenState(
          phase: ProgressiveOpenPhase.complete,
          generation: openProgress.generation,
          pageIndex: currentPageIndex,
        );
        notifyListeners();
      }
    } on Object catch (exception) {
      _logger.warning('Outline load failed.', exception);
      if (openedDocument?.documentId == opened.documentId) {
        openProgress = ProgressiveOpenState(
          phase: ProgressiveOpenPhase.complete,
          generation: openProgress.generation,
          pageIndex: currentPageIndex,
        );
        notifyListeners();
      }
    }
  }

  Future<void> _generateCover() async {
    final libraryDocument = document;
    final opened = openedDocument;
    if (libraryDocument == null || opened == null) return;
    final token = PdfOperationToken();
    _coverToken?.cancel();
    _coverToken = token;
    try {
      await _thumbnails.generateCover(
        libraryDocumentId: libraryDocument.id,
        engineDocumentId: opened.documentId,
        cancellationToken: token,
      );
    } on Object {
      // Thumbnail failure is already logged and must never block reading.
    } finally {
      if (identical(_coverToken, token)) _coverToken = null;
    }
  }

  Future<void> closeDocument() async {
    try {
      await leaveFullScreen();
    } on Object catch (exception) {
      // Closing the PDF must remain recoverable even if Windows rejects a
      // monitor transition. Do not strand handles or persisted state.
      _logger.warning(
        'Fullscreen restore failed while closing the reader.',
        exception,
      );
    }
    _openGeneration++;
    openProgress = ProgressiveOpenPolicy.cancel(openProgress);
    await _closeCurrent();
    await retainWarmDocuments(const {});
    status = ReaderStatus.idle;
    error = null;
    pendingPath = null;
    notifyListeners();
  }

  Future<void> _closeCurrent({bool retainNative = false}) async {
    _saveTimer?.cancel();
    _coverToken?.cancel();
    _coverToken = null;
    _savePositionNow();
    _finishSession();
    search.reset();
    annotations?.clear();
    textSelection = null;
    bookmarks.clear();
    navigationHistory?.clear();
    _destinationCueTimer?.cancel();
    destinationCue.value = null;
    temporaryPositionA = null;
    temporaryPositionB = null;
    final opened = openedDocument;
    final currentDocument = document;
    document = null;
    openedDocument = null;
    preferences = null;
    viewController.detach();
    outline = const [];
    currentPageIndex = 0;
    if (opened == null) return;
    if (retainNative && currentDocument != null) {
      final replaced = _warmDocuments.remove(currentDocument.id);
      if (replaced != null && replaced.opened.documentId != opened.documentId) {
        await _engine.closeDocument(replaced.opened.documentId);
      }
      _warmDocuments[currentDocument.id] = _WarmReaderDocument(
        document: currentDocument,
        opened: opened,
      );
      while (_warmDocuments.length > AppConfig.maxWarmTabs) {
        final oldestId = _warmDocuments.keys.first;
        final oldest = _warmDocuments.remove(oldestId);
        if (oldest != null) {
          await _engine.closeDocument(oldest.opened.documentId);
        }
      }
    } else {
      await _engine.closeDocument(opened.documentId);
    }
  }

  Future<void> retainWarmDocuments(Iterable<String> documentIds) async {
    final keep = documentIds.toSet();
    final evicted = _warmDocuments.entries
        .where((entry) => !keep.contains(entry.key))
        .toList(growable: false);
    for (final entry in evicted) {
      _warmDocuments.remove(entry.key);
      await _engine.closeDocument(entry.value.opened.documentId);
    }
  }

  void restoreNavigationSnapshot({
    required int pageIndex,
    required double zoomLevel,
    required PageLayoutMode pageLayout,
    required ReaderMode readingMode,
    required double scrollOffset,
    required ReaderTheme readingTheme,
  }) {
    final currentPreferences = preferences;
    if (status != ReaderStatus.ready || currentPreferences == null) return;
    preferences = currentPreferences.copyWith(
      zoomLevel: zoomLevel,
      pageLayout: pageLayout,
      viewMode: readingMode,
      scrollOffset: scrollOffset,
      readingTheme: readingTheme,
    );
    currentPageIndex = pageIndex.clamp(0, pageCount - 1);
    notifyListeners();
  }

  void onPageChanged(int pageIndex) {
    if (pageCount == 0) return;
    final next = pageIndex.clamp(0, pageCount - 1);
    if (next == currentPageIndex) return;
    currentPageIndex = next;
    registerActivity();
    _scheduleSave();
  }

  Future<void> goToPreviousReadingUnit() async {
    final target = _previousReadingUnitTarget();
    if (target == _canonicalReadingUnitPage()) return;
    await viewController.goToPage(target);
  }

  Future<void> goToNextReadingUnit() async {
    final target = _nextReadingUnitTarget();
    if (target == _canonicalReadingUnitPage()) return;
    await viewController.goToPage(target);
  }

  int _canonicalReadingUnitPage() => ReadingUnitNavigation.canonicalPage(
    pageIndex: currentPageIndex,
    pageCount: pageCount,
    mode: readerMode,
    firstPageIsCover: preferences?.coverPolicy != ReaderCoverPolicy.noCover,
    initialSide: preferences?.initialSide ?? ReaderInitialSide.left,
  );

  int _previousReadingUnitTarget() => ReadingUnitNavigation.previousTarget(
    pageIndex: currentPageIndex,
    pageCount: pageCount,
    mode: readerMode,
    firstPageIsCover: preferences?.coverPolicy != ReaderCoverPolicy.noCover,
    initialSide: preferences?.initialSide ?? ReaderInitialSide.left,
  );

  int _nextReadingUnitTarget() => ReadingUnitNavigation.nextTarget(
    pageIndex: currentPageIndex,
    pageCount: pageCount,
    mode: readerMode,
    firstPageIsCover: preferences?.coverPolicy != ReaderCoverPolicy.noCover,
    initialSide: preferences?.initialSide ?? ReaderInitialSide.left,
  );

  Future<void> goToFirstReadingUnit() => viewController.goToPage(0);

  Future<void> goToLastReadingUnit() => viewController.goToPage(
    ReadingUnitNavigation.lastTarget(
      pageCount: pageCount,
      mode: readerMode,
      firstPageIsCover: preferences?.coverPolicy != ReaderCoverPolicy.noCover,
      initialSide: preferences?.initialSide ?? ReaderInitialSide.left,
    ),
  );

  String readingUnitIndicator({
    ReadingDirection direction = ReadingDirection.leftToRight,
  }) => ReadingUnitNavigation.indicator(
    pageIndex: currentPageIndex,
    pageCount: pageCount,
    mode: readerMode,
    direction: direction,
    firstPageIsCover: preferences?.coverPolicy != ReaderCoverPolicy.noCover,
    initialSide: preferences?.initialSide ?? ReaderInitialSide.left,
  );

  Future<void> goToPage(int oneBasedPage) {
    if (pageCount == 0) return Future<void>.value();
    return navigateTo(
      oneBasedPage.clamp(1, pageCount) - 1,
      reason: NavigationReason.distantPage,
      label: 'Página $oneBasedPage',
    );
  }

  Future<void> navigateTo(
    int pageIndex, {
    required NavigationReason reason,
    required String label,
    bool focusDestination = false,
  }) async {
    if (pageCount == 0) return;
    final target = pageIndex.clamp(0, pageCount - 1);
    navigationHistory?.record(
      NavigationLocation(
        pageIndex: target,
        offset: viewController.scrollOffset,
        zoom: viewController.zoom,
        reason: reason,
        label: label,
      ),
    );
    await viewController.goToPage(target);
    if (focusDestination) _showDestinationCue(target, label);
  }

  void _showDestinationCue(int pageIndex, String label) {
    _destinationCueTimer?.cancel();
    final sequence = (destinationCue.value?.sequence ?? 0) + 1;
    destinationCue.value = ReaderDestinationCue(
      pageIndex: pageIndex,
      label: label,
      sequence: sequence,
    );
    _destinationCueTimer = Timer(const Duration(milliseconds: 1400), () {
      destinationCue.value = null;
    });
  }

  Future<void> historyBack() async {
    final location = navigationHistory?.back();
    if (location != null) {
      await viewController.goToPage(location.pageIndex);
      await viewController.restorePosition(
        zoom: location.zoom,
        scrollOffset: location.offset,
      );
    }
  }

  Future<void> historyForward() async {
    final location = navigationHistory?.forward();
    if (location != null) {
      await viewController.goToPage(location.pageIndex);
      await viewController.restorePosition(
        zoom: location.zoom,
        scrollOffset: location.offset,
      );
    }
  }

  ReaderTemporaryPosition? temporaryPosition(
    ReaderTemporaryPositionSlot slot,
  ) => switch (slot) {
    ReaderTemporaryPositionSlot.a => temporaryPositionA,
    ReaderTemporaryPositionSlot.b => temporaryPositionB,
  };

  ReaderTemporaryPosition? saveTemporaryPosition(
    ReaderTemporaryPositionSlot slot,
  ) {
    final current = preferences;
    if (current == null || openedDocument == null) return null;
    final position = ReaderTemporaryPosition(
      pageIndex: currentPageIndex,
      zoom: viewController.zoom,
      scrollOffset: viewController.scrollOffset,
      viewMode: current.viewMode,
      pageLayout: current.pageLayout,
      rotation: current.pageRotation,
    );
    switch (slot) {
      case ReaderTemporaryPositionSlot.a:
        temporaryPositionA = position;
      case ReaderTemporaryPositionSlot.b:
        temporaryPositionB = position;
    }
    notifyListeners();
    return position;
  }

  Future<bool> goToTemporaryPosition(ReaderTemporaryPositionSlot slot) async {
    final position = temporaryPosition(slot);
    final current = preferences;
    if (position == null || current == null || pageCount == 0) return false;
    final target = position.pageIndex.clamp(0, pageCount - 1);
    navigationHistory?.record(
      NavigationLocation(
        pageIndex: target,
        offset: position.scrollOffset,
        zoom: position.zoom,
        reason: NavigationReason.history,
        label: 'Posición temporal ${slot.name.toUpperCase()}',
      ),
    );
    preferences = current.copyWith(
      viewMode: position.viewMode,
      pageLayout: position.pageLayout,
      pageRotation: position.rotation,
      zoomMode: ZoomMode.custom,
      zoomLevel: position.zoom,
      scrollOffset: position.scrollOffset,
    );
    currentPageIndex = target;
    notifyListeners();
    await viewController.goToPage(target);
    await viewController.restorePosition(
      zoom: position.zoom,
      scrollOffset: position.scrollOffset,
    );
    _scheduleSave();
    return true;
  }

  bool swapTemporaryPositions() {
    if (temporaryPositionA == null && temporaryPositionB == null) return false;
    final previousA = temporaryPositionA;
    temporaryPositionA = temporaryPositionB;
    temporaryPositionB = previousA;
    notifyListeners();
    return true;
  }

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
    if (current == null || current.viewMode == mode) return;
    final coverPolicy = current.coverPolicy;
    final initialSide = current.initialSide;
    final canonicalPage = ReadingUnitNavigation.canonicalPage(
      pageIndex: currentPageIndex,
      pageCount: pageCount,
      mode: mode,
      firstPageIsCover: coverPolicy != ReaderCoverPolicy.noCover,
      initialSide: initialSide,
    );
    currentPageIndex = canonicalPage;
    preferences = current.copyWith(
      viewMode: mode,
      pageLayout: mode == ReaderMode.book
          ? PageLayoutMode.facing
          : PageLayoutMode.single,
      coverPolicy: coverPolicy,
      initialSide: initialSide,
      // Document and Book use different geometric units. Start the new unit
      // with a deterministic fit so a custom one-page zoom cannot leave half
      // of a spread outside the viewport.
      zoomMode: ZoomMode.fitPage,
      scrollOffset: 0,
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
    final mode = layout == PageLayoutMode.facing
        ? ReaderMode.book
        : ReaderMode.document;
    final coverPolicy = current.coverPolicy;
    final initialSide = current.initialSide;
    currentPageIndex = ReadingUnitNavigation.canonicalPage(
      pageIndex: currentPageIndex,
      pageCount: pageCount,
      mode: mode,
      firstPageIsCover: coverPolicy != ReaderCoverPolicy.noCover,
      initialSide: initialSide,
    );
    preferences = current.copyWith(
      pageLayout: layout,
      viewMode: mode,
      coverPolicy: coverPolicy,
      initialSide: initialSide,
      zoomMode: ZoomMode.fitPage,
      scrollOffset: 0,
    );
    _scheduleSave();
    notifyListeners();
  }

  void setCoverPolicy(ReaderCoverPolicy policy) {
    final current = preferences;
    if (current == null || current.coverPolicy == policy) return;
    _setBookPairing(policy, current.initialSide);
  }

  void setInitialSide(ReaderInitialSide side) {
    final current = preferences;
    if (current == null || current.initialSide == side) return;
    _setBookPairing(current.coverPolicy, side);
  }

  void resetBookPairing() {
    _setBookPairing(
      ReaderCoverPolicy.firstPageIsCover,
      ReaderInitialSide.right,
    );
  }

  void _setBookPairing(
    ReaderCoverPolicy coverPolicy,
    ReaderInitialSide initialSide,
  ) {
    final current = preferences;
    if (current == null) return;
    currentPageIndex = ReadingUnitNavigation.canonicalPage(
      pageIndex: currentPageIndex,
      pageCount: pageCount,
      mode: current.viewMode,
      firstPageIsCover: coverPolicy != ReaderCoverPolicy.noCover,
      initialSide: initialSide,
    );
    preferences = current.copyWith(
      coverPolicy: coverPolicy,
      initialSide: initialSide,
      zoomMode: ZoomMode.fitPage,
      scrollOffset: 0,
    );
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
    final serviceIsFullScreen = await _windowService.isFullScreen();
    if (serviceIsFullScreen) await _windowService.setFullScreen(false);
    final changed = isFullScreen;
    isFullScreen = false;
    if (changed) notifyListeners();
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
    _destinationCueTimer?.cancel();
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
      final current = preferences;
      _sessions.finish(
        id: session.id,
        endPage: currentPageIndex,
        activeSeconds: _activeSeconds,
        context: current == null
            ? null
            : ReaderSessionContext(
                zoomLevel: viewController.zoom,
                pageLayout: current.pageLayout.name,
                readingMode: current.viewMode.name,
                readingTheme: current.readingTheme.name,
                scrollOffset: viewController.scrollOffset,
                panelSection: sessionPanelSection,
              ),
      );
    }
  }

  @override
  void dispose() {
    _openGeneration++;
    openProgress = ProgressiveOpenPolicy.cancel(openProgress);
    viewController.removeListener(_onViewerStateChanged);
    _saveTimer?.cancel();
    _activityTimer?.cancel();
    _savePositionNow();
    _finishSession();
    pageIndexListenable.dispose();
    destinationCue.dispose();
    unawaited(_closeEngineDocuments());
    super.dispose();
  }

  Future<void> _closeEngineDocuments() async {
    final opened = openedDocument;
    if (opened != null) await _engine.closeDocument(opened.documentId);
    final warm = _warmDocuments.values.toList(growable: false);
    _warmDocuments.clear();
    for (final item in warm) {
      await _engine.closeDocument(item.opened.documentId);
    }
  }
}
