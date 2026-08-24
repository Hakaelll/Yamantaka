// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:ffi/ffi.dart';
import 'package:crypto/crypto.dart';
import 'package:pdfrx/pdfrx.dart' as pdfrx;
import 'package:pdfium_dart/pdfium_dart.dart' as pdfium;

import '../../app/app_config.dart';
import '../../core/async/job_coordinator.dart';
import '../../core/errors/app_failure.dart';
import '../../core/lifecycle/lifecycle_tracker.dart';
import '../../core/logging/app_logger.dart';
import '../../core/memory/memory_governor.dart';
import '../../core/performance/local_performance_monitor.dart';
import '../../core/security/local_input_policy.dart';
import '../cache/sized_lru_cache.dart';
import '../models/pdf_models.dart';
import '../pdf_document_engine.dart';

final class PdfEngineDiagnostics {
  const PdfEngineDiagnostics({
    required this.renderRequests,
    required this.cacheHits,
    required this.cacheMisses,
    required this.deduplicatedRenders,
    required this.cancelledRenders,
    required this.cachedPages,
    required this.cachedBytes,
    required this.pendingRenders,
    required this.openDocuments,
    required this.pageCacheEntries,
    required this.pageCacheBytes,
    required this.tileCacheEntries,
    required this.tileCacheBytes,
    required this.textCacheEntries,
    required this.textCacheBytes,
    required this.staleResultsDiscarded,
    required this.viewerPreviewRenders,
    required this.viewerRefinementRenders,
    required this.viewerTilesReused,
  });

  final int renderRequests;
  final int cacheHits;
  final int cacheMisses;
  final int deduplicatedRenders;
  final int cancelledRenders;
  final int cachedPages;
  final int cachedBytes;
  final int pendingRenders;
  final int openDocuments;
  final int pageCacheEntries;
  final int pageCacheBytes;
  final int tileCacheEntries;
  final int tileCacheBytes;
  final int textCacheEntries;
  final int textCacheBytes;
  final int staleResultsDiscarded;
  final int viewerPreviewRenders;
  final int viewerRefinementRenders;
  final int viewerTilesReused;

  Map<String, int> toJson() => {
    'render_requests': renderRequests,
    'cache_hits': cacheHits,
    'cache_misses': cacheMisses,
    'deduplicated_renders': deduplicatedRenders,
    'cancelled_renders': cancelledRenders,
    'cached_pages': cachedPages,
    'cached_bytes': cachedBytes,
    'pending_renders': pendingRenders,
    'open_documents': openDocuments,
    'page_cache_entries': pageCacheEntries,
    'page_cache_bytes': pageCacheBytes,
    'tile_cache_entries': tileCacheEntries,
    'tile_cache_bytes': tileCacheBytes,
    'text_cache_entries': textCacheEntries,
    'text_cache_bytes': textCacheBytes,
    'stale_results_discarded': staleResultsDiscarded,
    'viewer_preview_renders': viewerPreviewRenders,
    'viewer_refinement_renders': viewerRefinementRenders,
    'viewer_tiles_reused': viewerTilesReused,
  };
}

final class PdfrxDocumentEngine implements PdfDocumentEngine {
  PdfrxDocumentEngine(
    this._logger, {
    LifecycleTracker? lifecycle,
    MemoryGovernor? memory,
    JobCoordinator? jobs,
    LocalPerformanceMonitor? performance,
  }) : _lifecycle = lifecycle,
       _memory = memory,
       _performance = performance {
    _renderJobs =
        jobs ??
        JobCoordinator(
          maxConcurrent: 2,
          categoryLimits: const {JobCategory.render: 2},
          lifecycle: lifecycle,
        );
    _ownsRenderJobs = jobs == null;
    _pageRenderCache = SizedLruCache<_RenderKey, ui.Image>(
      maxBytes: AppConfig.maxPageCacheBytes,
      sizeOf: (image) => image.width * image.height * 4,
      onEvict: (image) => image.dispose(),
      onEvictEntry: _onCacheEntryEvicted,
    );
    _tileRenderCache = SizedLruCache<_RenderKey, ui.Image>(
      maxBytes: AppConfig.maxTileCacheBytes,
      sizeOf: (image) => image.width * image.height * 4,
      onEvict: (image) => image.dispose(),
      onEvictEntry: _onCacheEntryEvicted,
    );
    _textCache = SizedLruCache<_TextKey, String>(
      maxBytes: AppConfig.maxTextCacheBytes ~/ 2,
      sizeOf: (text) => text.length * 2,
    );
    _structuredTextCache =
        SizedLruCache<_TextKey, List<PdfStructuredTextBlock>>(
          maxBytes: AppConfig.maxTextCacheBytes ~/ 2,
          sizeOf: _structuredTextBytes,
        );
  }

  final AppLogger _logger;
  final LifecycleTracker? _lifecycle;
  final MemoryGovernor? _memory;
  final LocalPerformanceMonitor? _performance;
  late final JobCoordinator _renderJobs;
  late final bool _ownsRenderJobs;
  final Map<String, pdfrx.PdfDocument> _documents = {};
  final Map<String, String> _documentCacheIdentities = {};
  final Map<String, int> _renderGenerations = {};
  final Map<String, _DocumentRenderGate> _documentRenderGates = {};
  final Map<_RenderKey, Set<JobHandle<RenderedPdfPage>>> _pendingRenderHandles =
      {};
  final Map<_RenderKey, pdfrx.PdfPageRenderCancellationToken>
  _activeRenderTokens = {};
  late final SizedLruCache<_RenderKey, ui.Image> _pageRenderCache;
  late final SizedLruCache<_RenderKey, ui.Image> _tileRenderCache;
  late final SizedLruCache<_TextKey, String> _textCache;
  late final SizedLruCache<_TextKey, List<PdfStructuredTextBlock>>
  _structuredTextCache;
  final Map<String, Set<ui.Image>> _uncachedImages = {};
  final Map<String, LifecycleLease> _documentLeases = {};
  final Map<_RenderKey, LifecycleLease> _renderLeases = {};
  final Map<_RenderKey, LifecycleLease> _cacheLeases = {};
  final Map<_RenderKey, MemoryReservation> _cacheMemory = {};
  final Map<ui.Image, MemoryReservation> _uncachedMemory = {};
  int _nextDocumentId = 0;
  int _renderCacheHits = 0;
  int _renderCacheMisses = 0;
  int _renderRequests = 0;
  int _deduplicatedRenders = 0;
  static final pdfium.PDFium _pdfiumBindings = pdfium.getPdfium();
  int _cancelledRenders = 0;
  int _staleResultsDiscarded = 0;
  int _viewerPreviewRenders = 0;
  int _viewerRefinementRenders = 0;
  int _viewerTilesReused = 0;

  void _onCacheEntryEvicted(_RenderKey key, ui.Image image) {
    _cacheLeases.remove(key)?.release();
    _cacheMemory.remove(key)?.release();
  }

  PdfEngineDiagnostics get diagnostics => PdfEngineDiagnostics(
    renderRequests: _renderRequests,
    cacheHits: _renderCacheHits,
    cacheMisses: _renderCacheMisses,
    deduplicatedRenders: _deduplicatedRenders,
    cancelledRenders: _cancelledRenders,
    cachedPages: _pageRenderCache.length + _tileRenderCache.length,
    cachedBytes: _pageRenderCache.currentBytes + _tileRenderCache.currentBytes,
    pendingRenders: _pendingRenderHandles.length,
    openDocuments: _documents.length,
    pageCacheEntries: _pageRenderCache.length,
    pageCacheBytes: _pageRenderCache.currentBytes,
    tileCacheEntries: _tileRenderCache.length,
    tileCacheBytes: _tileRenderCache.currentBytes,
    textCacheEntries: _textCache.length + _structuredTextCache.length,
    textCacheBytes: _textCache.currentBytes + _structuredTextCache.currentBytes,
    staleResultsDiscarded: _staleResultsDiscarded,
    viewerPreviewRenders: _viewerPreviewRenders,
    viewerRefinementRenders: _viewerRefinementRenders,
    viewerTilesReused: _viewerTilesReused,
  );

  pdfrx.PdfDocument documentFor(String documentId) {
    final document = _documents[documentId];
    if (document == null) {
      throw const AppFailure(
        FailureKind.restoreFailed,
        'El documento ya no está abierto.',
      );
    }
    return document;
  }

  @override
  Future<OpenedPdfDocument> openDocument(
    String path, {
    String? password,
  }) async {
    final stopwatch = Stopwatch()..start();
    final boundedPassword = password == null
        ? null
        : LocalInputPolicy.boundedPassword(password);
    try {
      if (!await _hasPdfHeader(path)) {
        throw const AppFailure(
          FailureKind.corruptPdf,
          'No se pudo abrir el PDF. Puede estar dañado o no ser compatible.',
          debugMessage: 'Missing PDF header.',
        );
      }
      final cacheIdentity = await _cacheIdentity(path);
      final document = await pdfrx.PdfDocument.openFile(
        path,
        passwordProvider: boundedPassword == null
            ? null
            : pdfrx.createSimplePasswordProvider(boundedPassword),
        firstAttemptByEmptyPassword: true,
        useProgressiveLoading: false,
      );
      if (document.pages.isEmpty) {
        await document.dispose();
        throw const AppFailure(
          FailureKind.emptyPdf,
          'El PDF no contiene páginas.',
        );
      }
      final documentId =
          'pdf-${DateTime.now().microsecondsSinceEpoch}-${_nextDocumentId++}';
      _documents[documentId] = document;
      _documentCacheIdentities[documentId] = cacheIdentity;
      _renderGenerations[documentId] = 0;
      final lease = _lifecycle?.acquire(ResourceKind.document);
      if (lease != null) _documentLeases[documentId] = lease;
      final opened = OpenedPdfDocument(
        documentId: documentId,
        path: path,
        pageCount: document.pages.length,
        pageSizes: document.pages
            .map((page) => PdfPageSize(page.width, page.height))
            .toList(growable: false),
        isEncrypted: document.isEncrypted,
        cacheIdentity: cacheIdentity,
      );
      _performance?.recordDuration('pdf_open', stopwatch.elapsed);
      _logger.info(
        'PDF opened (${opened.pageCount} pages) in '
        '${stopwatch.elapsedMilliseconds} ms.',
      );
      return opened;
    } on pdfrx.PdfPasswordException catch (error) {
      if (!await _hasEncryptionDictionary(path)) {
        throw AppFailure(
          FailureKind.corruptPdf,
          'No se pudo abrir el PDF. Puede estar dañado o no ser compatible.',
          debugMessage: error.message,
        );
      }
      throw AppFailure(
        boundedPassword == null
            ? FailureKind.passwordRequired
            : FailureKind.wrongPassword,
        boundedPassword == null
            ? 'Este PDF requiere una contraseña.'
            : 'La contraseña no es correcta.',
        debugMessage: error.message,
      );
    } on AppFailure {
      rethrow;
    } on OutOfMemoryError catch (error) {
      throw AppFailure(
        FailureKind.outOfMemory,
        'No hay memoria suficiente para abrir este PDF.',
        debugMessage: error.toString(),
      );
    } on Object catch (error, stackTrace) {
      _logger.error('PDF open failed.', error, stackTrace);
      throw AppFailure(
        FailureKind.corruptPdf,
        'No se pudo abrir el PDF. Puede estar dañado o no ser compatible.',
        debugMessage: error.toString(),
      );
    }
  }

  @override
  Future<void> closeDocument(String documentId) async {
    final handles = _pendingRenderHandles.entries
        .where((entry) => entry.key.documentId == documentId)
        .expand((entry) => entry.value)
        .toList(growable: false);
    final tokens = _activeRenderTokens.entries
        .where((entry) => entry.key.documentId == documentId)
        .map((entry) => entry.value)
        .toList(growable: false);
    for (final token in tokens) {
      token.cancel();
    }
    for (final handle in handles) {
      handle.cancel();
    }
    await _ignoreJobResults(handles);
    _pendingRenderHandles.removeWhere((key, _) => key.documentId == documentId);
    _activeRenderTokens.removeWhere((key, _) => key.documentId == documentId);
    _pageRenderCache.removeWhere((key) => key.documentId == documentId);
    _tileRenderCache.removeWhere((key) => key.documentId == documentId);
    _disposeUncachedImages(documentId);
    _textCache.removeWhere((key) => key.documentId == documentId);
    _structuredTextCache.removeWhere((key) => key.documentId == documentId);
    _documentCacheIdentities.remove(documentId);
    _renderGenerations.remove(documentId);
    _documentRenderGates.remove(documentId);
    final document = _documents.remove(documentId);
    _documentLeases.remove(documentId)?.release();
    await document?.dispose();
    _logger.debug('Closed PDF document.');
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
  }) {
    _renderRequests++;
    if (cancellationToken?.isCancelled ?? false) {
      _cancelledRenders++;
      return Future.error(
        const AppFailure(FailureKind.cancelled, 'Renderizado cancelado.'),
      );
    }
    final document = documentFor(documentId);
    _validatePageIndex(document, pageIndex);
    final currentGeneration = _renderGenerations[documentId] ?? 0;
    final requestedGeneration = generation ?? currentGeneration;
    if (requestedGeneration < currentGeneration) {
      _staleResultsDiscarded++;
      return Future.error(
        const AppFailure(FailureKind.cancelled, 'Renderizado obsoleto.'),
      );
    }
    if (requestedGeneration > currentGeneration) {
      beginRenderGeneration(documentId, requestedGeneration);
    }
    final page = document.pages[pageIndex];
    final safeDpr = devicePixelRatio.clamp(0.5, 4.0).toDouble();
    final safeScale = _boundedRenderScale(
      pageWidth: region?.width ?? page.width,
      pageHeight: region?.height ?? page.height,
      requestedScale: scale * safeDpr,
    );
    final key = _RenderKey(
      documentId: documentId,
      cacheIdentity: _documentCacheIdentities[documentId] ?? documentId,
      pageIndex: pageIndex,
      scale: (safeScale * 1000).round(),
      dpi: (72 * safeDpr).round(),
      rotation: _normalizedRotation(rotation),
      region: region,
      generation: requestedGeneration,
    );
    final cache = _cacheFor(key);
    final cached = cache.get(key);
    if (cached != null) {
      _renderCacheHits++;
      if (region != null) _performance?.increment('tiles_reused');
      _logger.debug(
        'Render cache hit ($_renderCacheHits hits, '
        '$_renderCacheMisses misses, '
        '${_pageRenderCache.length + _tileRenderCache.length} images).',
      );
      return Future.value(
        RenderedPdfPage(
          image: cached,
          pageIndex: key.pageIndex,
          scale: safeScale,
        ),
      );
    }
    if (_pendingRenderHandles.containsKey(key)) _deduplicatedRenders++;
    _renderCacheMisses++;
    late final JobHandle<RenderedPdfPage> handle;
    handle = _renderJobs.submit<RenderedPdfPage>(
      key: key.schedulerKey,
      category: JobCategory.render,
      priority: _jobPriority(priority),
      generation: requestedGeneration,
      task: (schedulerToken) async {
        final operationToken = PdfOperationToken();
        void cancelNative() => operationToken.cancel();
        schedulerToken.addListener(cancelNative);
        try {
          final gate = _documentRenderGates.putIfAbsent(
            documentId,
            _DocumentRenderGate.new,
          );
          return await gate.run(
            () => _render(key, safeScale, region, operationToken),
          );
        } finally {
          schedulerToken.removeListener(cancelNative);
        }
      },
    );
    (_pendingRenderHandles[key] ??= <JobHandle<RenderedPdfPage>>{}).add(handle);
    void cancelConsumer() => handle.cancel();
    cancellationToken?.addCancellationListener(cancelConsumer);
    return handle.future
        .then((page) {
          if (cancellationToken?.isCancelled ?? false) {
            _cancelledRenders++;
            throw const AppFailure(
              FailureKind.cancelled,
              'Renderizado cancelado.',
            );
          }
          return page;
        })
        .onError<JobCancelled>((_, _) {
          _cancelledRenders++;
          throw const AppFailure(
            FailureKind.cancelled,
            'Renderizado cancelado.',
          );
        })
        .whenComplete(() {
          cancellationToken?.removeCancellationListener(cancelConsumer);
          unawaited(
            handle.settled.whenComplete(() {
              final pending = _pendingRenderHandles[key];
              pending?.remove(handle);
              if (pending?.isEmpty ?? false) _pendingRenderHandles.remove(key);
            }),
          );
        });
  }

  @override
  void beginRenderGeneration(String documentId, int generation) {
    final current = _renderGenerations[documentId] ?? 0;
    if (generation <= current) return;
    _renderGenerations[documentId] = generation;
    for (final entry in _pendingRenderHandles.entries.toList(growable: false)) {
      if (entry.key.documentId != documentId ||
          entry.key.generation >= generation) {
        continue;
      }
      for (final handle in entry.value) {
        handle.cancel();
      }
    }
    for (final entry in _activeRenderTokens.entries.toList(growable: false)) {
      if (entry.key.documentId == documentId &&
          entry.key.generation < generation) {
        entry.value.cancel();
      }
    }
    _pageRenderCache.removeWhere(
      (key) => key.documentId == documentId && key.generation < generation,
    );
    _tileRenderCache.removeWhere(
      (key) => key.documentId == documentId && key.generation < generation,
    );
  }

  Future<RenderedPdfPage> _render(
    _RenderKey key,
    double scale,
    PdfRenderRegion? region,
    PdfOperationToken? operationToken,
  ) async {
    final stopwatch = Stopwatch()..start();
    if (operationToken?.isCancelled ?? false) {
      _cancelledRenders++;
      throw const AppFailure(FailureKind.cancelled, 'Renderizado cancelado.');
    }
    final document = documentFor(key.documentId);
    final renderLease = _lifecycle?.acquire(ResourceKind.render);
    if (renderLease != null) _renderLeases[key] = renderLease;
    _validatePageIndex(document, key.pageIndex);
    final page = await document.pages[key.pageIndex].ensureLoaded();
    if (!_isCurrent(key)) {
      _staleResultsDiscarded++;
      throw const AppFailure(FailureKind.cancelled, 'Renderizado obsoleto.');
    }
    final renderToken = page.createCancellationToken();
    void cancelNativeRender() => renderToken.cancel();
    operationToken?.addCancellationListener(cancelNativeRender);
    _activeRenderTokens[key] = renderToken;
    if (operationToken?.isCancelled ?? false) {
      renderToken.cancel();
    }
    final fullWidth = math.max<double>(1, page.width * scale);
    final fullHeight = math.max<double>(1, page.height * scale);
    final x = region == null ? 0 : (region.left * scale).round();
    final y = region == null ? 0 : (region.top * scale).round();
    final width = region == null
        ? fullWidth.round()
        : math.max(1, (region.width * scale).round());
    final height = region == null
        ? fullHeight.round()
        : math.max(1, (region.height * scale).round());
    try {
      final rendered = await page.render(
        x: x,
        y: y,
        width: width,
        height: height,
        fullWidth: fullWidth,
        fullHeight: fullHeight,
        rotationOverride: _rotation(key.rotation),
        backgroundColor: 0xFFFFFFFF,
        cancellationToken: renderToken,
      );
      if (rendered == null ||
          renderToken.isCanceled ||
          (operationToken?.isCancelled ?? false)) {
        rendered?.dispose();
        _cancelledRenders++;
        throw const AppFailure(FailureKind.cancelled, 'Renderizado cancelado.');
      }
      try {
        final image = await rendered.createImage();
        if (renderToken.isCanceled ||
            (operationToken?.isCancelled ?? false) ||
            !_isCurrent(key)) {
          image.dispose();
          if (!_isCurrent(key)) _staleResultsDiscarded++;
          _cancelledRenders++;
          throw const AppFailure(
            FailureKind.cancelled,
            'Renderizado cancelado.',
          );
        }
        final estimatedBytes = image.width * image.height * 4;
        final cache = _cacheFor(key);
        _syncCacheBudgets();
        final memoryReservation = _memory?.reserve(
          owner: MemoryOwner.renderCache,
          bytes: estimatedBytes,
          priority: MemoryPriority.cache,
          onEvicted: () => cache.removeWhere((candidate) => candidate == key),
        );
        if (estimatedBytes <= cache.maxBytes &&
            (_memory == null || memoryReservation != null)) {
          cache.put(key, image);
          if (memoryReservation != null) _cacheMemory[key] = memoryReservation;
          final cacheLease = _lifecycle?.acquire(ResourceKind.image);
          if (cacheLease != null) _cacheLeases[key] = cacheLease;
        } else {
          if (_memory != null && memoryReservation == null) {
            _performance?.increment('cache_pressure_events');
          }
          memoryReservation?.release();
          final activeReservation = _memory?.reserve(
            owner: MemoryOwner.activePage,
            bytes: estimatedBytes,
            priority: MemoryPriority.visible,
          );
          if (activeReservation != null) {
            _uncachedMemory[image] = activeReservation;
          }
          (_uncachedImages[key.documentId] ??= <ui.Image>{}).add(image);
        }
        _logger.debug(
          'Rendered page ${key.pageIndex + 1} in '
          '${stopwatch.elapsedMilliseconds} ms; '
          '${_pageRenderCache.currentBytes + _tileRenderCache.currentBytes} '
          'cache bytes retained.',
        );
        _performance?.recordDuration(
          region == null ? 'page_render' : 'tile_render',
          stopwatch.elapsed,
        );
        return RenderedPdfPage(
          image: image,
          pageIndex: key.pageIndex,
          scale: scale,
        );
      } finally {
        rendered.dispose();
      }
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      if (renderToken.isCanceled) {
        _cancelledRenders++;
        throw const AppFailure(FailureKind.cancelled, 'Renderizado cancelado.');
      }
      _logger.error('Page render failed.', error, stackTrace);
      throw AppFailure(
        FailureKind.renderFailed,
        'No se pudo dibujar esta página.',
        debugMessage: error.toString(),
      );
    } finally {
      operationToken?.removeCancellationListener(cancelNativeRender);
      _activeRenderTokens.remove(key);
      _renderLeases.remove(key)?.release();
    }
  }

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  }) async {
    if (cancellationToken?.isCancelled ?? false) {
      throw const AppFailure(
        FailureKind.cancelled,
        'Extracción de texto cancelada.',
      );
    }
    final document = documentFor(documentId);
    _validatePageIndex(document, pageIndex);
    final key = _TextKey(
      documentId,
      _documentCacheIdentities[documentId] ?? documentId,
      pageIndex,
    );
    final cached = _textCache.get(key);
    if (cached != null) return cached;
    final page = await document.pages[pageIndex].ensureLoaded();
    if (cancellationToken?.isCancelled ?? false) {
      throw const AppFailure(
        FailureKind.cancelled,
        'Extracción de texto cancelada.',
      );
    }
    final text = await page.loadText();
    final value = text?.fullText ?? '';
    _textCache.put(key, value);
    return value;
  }

  @override
  Future<PdfDocumentMetadata> readDocumentMetadata(String documentId) async {
    final document = documentFor(documentId);
    return document.useNativeDocumentHandle((address) {
      final handle = pdfium.FPDF_DOCUMENT.fromAddress(address);
      return PdfDocumentMetadata(
        title: _readMetadataTag(handle, 'Title'),
        author: _readMetadataTag(handle, 'Author'),
      );
    });
  }

  @override
  Future<List<PdfStructuredTextBlock>> extractStructuredPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  }) async {
    if (cancellationToken?.isCancelled ?? false) {
      throw const AppFailure(
        FailureKind.cancelled,
        'Extracción de texto cancelada.',
      );
    }
    final document = documentFor(documentId);
    _validatePageIndex(document, pageIndex);
    final key = _TextKey(
      documentId,
      _documentCacheIdentities[documentId] ?? documentId,
      pageIndex,
    );
    final cached = _structuredTextCache.get(key);
    if (cached != null) return cached;
    final page = await document.pages[pageIndex].ensureLoaded();
    final text = await page.loadStructuredText();
    if (cancellationToken?.isCancelled ?? false) {
      throw const AppFailure(
        FailureKind.cancelled,
        'Extracción de texto cancelada.',
      );
    }
    final value = List<PdfStructuredTextBlock>.unmodifiable([
      for (final fragment in text.fragments)
        if (fragment.text.trim().isNotEmpty && fragment.bounds.isNotEmpty)
          PdfStructuredTextBlock(
            text: fragment.text.trim(),
            left: fragment.bounds.left,
            top: fragment.bounds.top,
            width: fragment.bounds.width,
            height: fragment.bounds.height,
            pageWidth: page.width,
            pageHeight: page.height,
          ),
    ]);
    _structuredTextCache.put(key, value);
    return value;
  }

  @override
  Future<List<PdfSearchMatch>> searchText({
    required String documentId,
    required String query,
    PdfOperationToken? cancellationToken,
  }) async {
    final stopwatch = Stopwatch()..start();
    final trimmed = query.trim();
    if (trimmed.isEmpty) return const [];
    final document = documentFor(documentId);
    final normalizedQuery = trimmed.toLowerCase();
    final matches = <PdfSearchMatch>[];
    for (var pageIndex = 0; pageIndex < document.pages.length; pageIndex++) {
      if (cancellationToken?.isCancelled ?? false) {
        _logger.debug(
          'Search cancelled after ${stopwatch.elapsedMilliseconds} ms.',
        );
        return matches;
      }
      final text = await extractPageText(
        documentId: documentId,
        pageIndex: pageIndex,
        cancellationToken: cancellationToken,
      );
      final normalizedText = text.toLowerCase();
      var offset = 0;
      while (offset < normalizedText.length) {
        final matchIndex = normalizedText.indexOf(normalizedQuery, offset);
        if (matchIndex < 0) break;
        final snippetStart = math.max(0, matchIndex - 32);
        final snippetEnd = math.min(
          text.length,
          matchIndex + normalizedQuery.length + 48,
        );
        matches.add(
          PdfSearchMatch(
            pageIndex: pageIndex,
            start: matchIndex,
            length: normalizedQuery.length,
            snippet: text
                .substring(snippetStart, snippetEnd)
                .replaceAll(RegExp(r'\s+'), ' ')
                .trim(),
          ),
        );
        offset = matchIndex + math.max(1, normalizedQuery.length);
      }
    }
    _logger.debug(
      'Search completed in ${stopwatch.elapsedMilliseconds} ms '
      'with ${matches.length} matches.',
    );
    return matches;
  }

  @override
  Future<List<PdfOutlineItem>> getOutline(String documentId) async {
    final document = documentFor(documentId);
    final outline = await document.loadOutline();
    return outline.map(_mapOutline).toList(growable: false);
  }

  @override
  Future<List<PdfLink>> getPageLinks({
    required String documentId,
    required int pageIndex,
  }) async {
    final document = documentFor(documentId);
    _validatePageIndex(document, pageIndex);
    final page = await document.pages[pageIndex].ensureLoaded();
    final links = await page.loadLinks();
    return links
        .map(
          (link) => PdfLink(
            uri: link.url,
            destinationPageIndex: link.dest == null
                ? null
                : link.dest!.pageNumber - 1,
            rectangles: link.rects
                .map(
                  (rect) => PdfRenderRegion(
                    left: rect.left,
                    top: rect.top,
                    width: rect.width,
                    height: rect.height,
                  ),
                )
                .toList(growable: false),
          ),
        )
        .toList(growable: false);
  }

  @override
  void clearCaches() {
    for (final token in _activeRenderTokens.values) {
      token.cancel();
    }
    _pageRenderCache.clear();
    _tileRenderCache.clear();
    _textCache.clear();
    _structuredTextCache.clear();
    for (final lease in _cacheLeases.values) {
      lease.release();
    }
    _cacheLeases.clear();
    for (final reservation in _cacheMemory.values) {
      reservation.release();
    }
    _cacheMemory.clear();
    for (final images in _uncachedImages.values) {
      for (final image in images) {
        _uncachedMemory.remove(image)?.release();
        image.dispose();
      }
    }
    _uncachedImages.clear();
    _renderCacheHits = 0;
    _renderCacheMisses = 0;
    _renderRequests = 0;
    _deduplicatedRenders = 0;
    _cancelledRenders = 0;
    _staleResultsDiscarded = 0;
    _viewerPreviewRenders = 0;
    _viewerRefinementRenders = 0;
    _viewerTilesReused = 0;
  }

  @override
  Future<void> dispose() async {
    final pending = _pendingRenderHandles.values
        .expand((handles) => handles)
        .toList(growable: false);
    for (final token in _activeRenderTokens.values) {
      token.cancel();
    }
    for (final handle in pending) {
      handle.cancel();
    }
    await _ignoreJobResults(pending);
    _pendingRenderHandles.clear();
    _activeRenderTokens.clear();
    clearCaches();
    final documents = _documents.values.toList(growable: false);
    _documents.clear();
    _documentCacheIdentities.clear();
    _renderGenerations.clear();
    _documentRenderGates.clear();
    for (final lease in _documentLeases.values) {
      lease.release();
    }
    _documentLeases.clear();
    await Future.wait(documents.map((document) => document.dispose()));
    if (_ownsRenderJobs) await _renderJobs.dispose();
  }

  static Future<void> _ignoreJobResults(
    Iterable<JobHandle<RenderedPdfPage>> handles,
  ) async {
    await Future.wait(
      handles.map((handle) async {
        try {
          await handle.future;
        } on Object {
          // Closing owns cancellation; callers receive the original failure.
        }
        await handle.settled;
      }),
    );
  }

  void _disposeUncachedImages(String documentId) {
    final images = _uncachedImages.remove(documentId);
    if (images == null) return;
    for (final image in images) {
      _uncachedMemory.remove(image)?.release();
      image.dispose();
    }
  }

  SizedLruCache<_RenderKey, ui.Image> _cacheFor(_RenderKey key) =>
      key.region == null ? _pageRenderCache : _tileRenderCache;

  bool _isCurrent(_RenderKey key) =>
      _documents.containsKey(key.documentId) &&
      (_renderGenerations[key.documentId] ?? 0) == key.generation;

  int get viewerCacheBudgetBytes {
    final total = _memory?.limitBytes ?? AppConfig.maxRenderCacheBytes * 2;
    final pressure = _memory?.snapshot.pressure ?? false;
    final factor = pressure ? 0.28 : 0.46;
    return (total * factor).round().clamp(24 * 1024 * 1024, 112 * 1024 * 1024);
  }

  bool get underMemoryPressure => _memory?.snapshot.pressure ?? false;

  void recordViewerRenderEvent(pdfrx.PdfViewerRenderEvent event) {
    switch (event.type) {
      case pdfrx.PdfViewerRenderEventType.previewCompleted:
        _viewerPreviewRenders++;
        _performance?.recordDuration('viewer_preview', event.duration);
      case pdfrx.PdfViewerRenderEventType.refinementCompleted:
        _viewerRefinementRenders++;
        _performance?.recordDuration('viewer_refinement', event.duration);
      case pdfrx.PdfViewerRenderEventType.renderCancelled:
        _cancelledRenders++;
        _performance?.increment('viewer_renders_cancelled');
      case pdfrx.PdfViewerRenderEventType.tileReused:
        _viewerTilesReused++;
        _performance?.increment('tiles_reused');
    }
  }

  void recordFirstPageVisible(Duration elapsed) {
    _performance?.recordDuration('first_page_visible', elapsed);
  }

  void _syncCacheBudgets() {
    final memory = _memory;
    if (memory == null) return;
    final pressure = memory.snapshot.pressure;
    final renderBudget = math.min(
      AppConfig.maxRenderCacheBytes,
      (memory.limitBytes * (pressure ? 0.35 : 0.58)).round(),
    );
    _pageRenderCache.setMaxBytes((renderBudget * 0.74).round());
    _tileRenderCache.setMaxBytes((renderBudget * 0.26).round());
    final textBudget = math.min(
      AppConfig.maxTextCacheBytes,
      (memory.limitBytes * (pressure ? 0.025 : 0.06)).round(),
    );
    _textCache.setMaxBytes(textBudget ~/ 2);
    _structuredTextCache.setMaxBytes(textBudget ~/ 2);
  }

  static double _boundedRenderScale({
    required double pageWidth,
    required double pageHeight,
    required double requestedScale,
  }) {
    final width = math.max(1.0, pageWidth);
    final height = math.max(1.0, pageHeight);
    final dimensionScale = math.min(
      AppConfig.maxPdfiumDimension / width,
      AppConfig.maxPdfiumDimension / height,
    );
    final byteScale = math.sqrt(
      AppConfig.maxSingleRenderBytes / (width * height * 4),
    );
    final upper = math.max(
      0.01,
      math.min(16.0, math.min(dimensionScale, byteScale)),
    );
    return requestedScale.clamp(0.01, upper).toDouble();
  }

  static int _normalizedRotation(int rotation) =>
      ((rotation % 360) + 360) % 360;

  static JobPriority _jobPriority(PdfRenderPriority priority) =>
      switch (priority) {
        PdfRenderPriority.visible => JobPriority.visible,
        PdfRenderPriority.nearby => JobPriority.interactive,
        PdfRenderPriority.prefetch => JobPriority.background,
        PdfRenderPriority.thumbnail => JobPriority.idle,
      };

  static int _structuredTextBytes(List<PdfStructuredTextBlock> blocks) =>
      blocks.fold<int>(0, (bytes, block) => bytes + block.text.length * 2 + 64);

  static Future<String> _cacheIdentity(String path) async {
    final file = File(path);
    final stat = await file.stat();
    final handle = await file.open();
    try {
      const sampleBytes = 4096;
      final first = await handle.read(math.min(sampleBytes, stat.size));
      var last = const <int>[];
      if (stat.size > sampleBytes) {
        await handle.setPosition(math.max(0, stat.size - sampleBytes));
        last = await handle.read(sampleBytes);
      }
      return sha256.convert(<int>[
        ...stat.size.toString().codeUnits,
        0,
        ...stat.modified.millisecondsSinceEpoch.toString().codeUnits,
        0,
        ...first,
        ...last,
      ]).toString();
    } finally {
      await handle.close();
    }
  }

  PdfOutlineItem _mapOutline(pdfrx.PdfOutlineNode node) => PdfOutlineItem(
    title: node.title,
    pageIndex: node.dest == null ? null : node.dest!.pageNumber - 1,
    children: node.children.map(_mapOutline).toList(growable: false),
  );

  static void _validatePageIndex(pdfrx.PdfDocument document, int pageIndex) {
    if (pageIndex < 0 || pageIndex >= document.pages.length) {
      throw RangeError.range(
        pageIndex,
        0,
        document.pages.length - 1,
        'pageIndex',
      );
    }
  }

  static String? _readMetadataTag(pdfium.FPDF_DOCUMENT document, String tag) {
    final nativeTag = tag.toNativeUtf8();
    try {
      final length = _pdfiumBindings.FPDF_GetMetaText(
        document,
        nativeTag.cast<ffi.Char>(),
        ffi.nullptr,
        0,
      );
      if (length <= 2 || length > 4096) return null;
      final buffer = calloc<ffi.Uint8>(length);
      try {
        final written = _pdfiumBindings.FPDF_GetMetaText(
          document,
          nativeTag.cast<ffi.Char>(),
          buffer.cast(),
          length,
        );
        if (written <= 2) return null;
        final units = buffer.cast<ffi.Uint16>().asTypedList((written ~/ 2) - 1);
        final value = String.fromCharCodes(units).trim();
        return value.isEmpty ? null : value;
      } finally {
        calloc.free(buffer);
      }
    } finally {
      malloc.free(nativeTag);
    }
  }

  static pdfrx.PdfPageRotation? _rotation(int degrees) =>
      switch (degrees % 360) {
        90 => pdfrx.PdfPageRotation.clockwise90,
        180 => pdfrx.PdfPageRotation.clockwise180,
        270 => pdfrx.PdfPageRotation.clockwise270,
        _ => null,
      };

  static Future<bool> _hasPdfHeader(String path) async {
    final file = File(path);
    final length = await file.length();
    if (length < 5) return false;
    final bytes = await file
        .openRead(0, math.min(length, 1024))
        .fold<List<int>>(<int>[], (buffer, chunk) => buffer..addAll(chunk));
    return _containsAscii(bytes, '%PDF-');
  }

  static Future<bool> _hasEncryptionDictionary(String path) async {
    final file = File(path);
    final length = await file.length();
    final start = math.max(0, length - 256 * 1024);
    final bytes = await file
        .openRead(start, length)
        .fold<List<int>>(<int>[], (buffer, chunk) => buffer..addAll(chunk));
    return _containsAscii(bytes, '/Encrypt');
  }

  static bool _containsAscii(List<int> bytes, String marker) {
    final expected = marker.codeUnits;
    for (var offset = 0; offset + expected.length <= bytes.length; offset++) {
      var matches = true;
      for (var index = 0; index < expected.length; index++) {
        if (bytes[offset + index] != expected[index]) {
          matches = false;
          break;
        }
      }
      if (matches) return true;
    }
    return false;
  }
}

final class _RenderKey {
  const _RenderKey({
    required this.documentId,
    required this.cacheIdentity,
    required this.pageIndex,
    required this.scale,
    required this.dpi,
    required this.rotation,
    required this.region,
    required this.generation,
  });

  final String documentId;
  final String cacheIdentity;
  final int pageIndex;
  final int scale;
  final int dpi;
  final int rotation;
  final PdfRenderRegion? region;
  final int generation;

  String get schedulerKey => Object.hash(
    cacheIdentity,
    pageIndex,
    scale,
    dpi,
    rotation,
    region?.left,
    region?.top,
    region?.width,
    region?.height,
  ).toUnsigned(32).toRadixString(16);

  @override
  int get hashCode => Object.hash(
    cacheIdentity,
    pageIndex,
    scale,
    dpi,
    rotation,
    generation,
    region?.left,
    region?.top,
    region?.width,
    region?.height,
  );

  @override
  bool operator ==(Object other) =>
      other is _RenderKey &&
      other.documentId == documentId &&
      other.cacheIdentity == cacheIdentity &&
      other.pageIndex == pageIndex &&
      other.scale == scale &&
      other.dpi == dpi &&
      other.rotation == rotation &&
      other.generation == generation &&
      other.region?.left == region?.left &&
      other.region?.top == region?.top &&
      other.region?.width == region?.width &&
      other.region?.height == region?.height;
}

/// Serializes native render entry for one PDF handle. Different documents may
/// still render concurrently; a raw document handle is never rendered from two
/// engine jobs at once.
final class _DocumentRenderGate {
  Future<void> _tail = Future<void>.value();

  Future<T> run<T>(Future<T> Function() operation) {
    final result = Completer<T>();
    _tail = _tail.then((_) async {
      try {
        result.complete(await operation());
      } on Object catch (error, stackTrace) {
        result.completeError(error, stackTrace);
      }
    });
    return result.future;
  }
}

final class _TextKey {
  const _TextKey(this.documentId, this.cacheIdentity, this.pageIndex);

  final String documentId;
  final String cacheIdentity;
  final int pageIndex;

  @override
  int get hashCode => Object.hash(cacheIdentity, pageIndex);

  @override
  bool operator ==(Object other) =>
      other is _TextKey &&
      other.documentId == documentId &&
      other.cacheIdentity == cacheIdentity &&
      other.pageIndex == pageIndex;
}
