import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:pdfrx/pdfrx.dart' as pdfrx;

import '../../app/app_config.dart';
import '../../core/errors/app_failure.dart';
import '../../core/logging/app_logger.dart';
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
  };
}

final class PdfrxDocumentEngine implements PdfDocumentEngine {
  PdfrxDocumentEngine(this._logger)
    : _renderCache = SizedLruCache<_RenderKey, RenderedPdfPage>(
        maxBytes: AppConfig.maxRenderCacheBytes,
        sizeOf: (page) => page.estimatedBytes,
        onEvict: (page) => page.dispose(),
      );

  final AppLogger _logger;
  final Map<String, pdfrx.PdfDocument> _documents = {};
  final Map<String, Map<int, String>> _textCache = {};
  final Map<_RenderKey, Future<RenderedPdfPage>> _pendingRenders = {};
  final Map<_RenderKey, pdfrx.PdfPageRenderCancellationToken>
  _activeRenderTokens = {};
  final SizedLruCache<_RenderKey, RenderedPdfPage> _renderCache;
  int _nextDocumentId = 0;
  int _renderCacheHits = 0;
  int _renderCacheMisses = 0;
  int _renderRequests = 0;
  int _deduplicatedRenders = 0;
  int _cancelledRenders = 0;

  PdfEngineDiagnostics get diagnostics => PdfEngineDiagnostics(
    renderRequests: _renderRequests,
    cacheHits: _renderCacheHits,
    cacheMisses: _renderCacheMisses,
    deduplicatedRenders: _deduplicatedRenders,
    cancelledRenders: _cancelledRenders,
    cachedPages: _renderCache.length,
    cachedBytes: _renderCache.currentBytes,
    pendingRenders: _pendingRenders.length,
    openDocuments: _documents.length,
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
    try {
      if (!await _hasPdfHeader(path)) {
        throw const AppFailure(
          FailureKind.corruptPdf,
          'No se pudo abrir el PDF. Puede estar dañado o no ser compatible.',
          debugMessage: 'Missing PDF header.',
        );
      }
      final document = await pdfrx.PdfDocument.openFile(
        path,
        passwordProvider: password == null
            ? null
            : pdfrx.createSimplePasswordProvider(password),
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
      _textCache[documentId] = {};
      final opened = OpenedPdfDocument(
        documentId: documentId,
        path: path,
        pageCount: document.pages.length,
        pageSizes: document.pages
            .map((page) => PdfPageSize(page.width, page.height))
            .toList(growable: false),
        isEncrypted: document.isEncrypted,
      );
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
        password == null
            ? FailureKind.passwordRequired
            : FailureKind.wrongPassword,
        password == null
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
    final pending = _pendingRenders.entries
        .where((entry) => entry.key.documentId == documentId)
        .map((entry) => entry.value)
        .toList(growable: false);
    final tokens = _activeRenderTokens.entries
        .where((entry) => entry.key.documentId == documentId)
        .map((entry) => entry.value)
        .toList(growable: false);
    for (final token in tokens) {
      token.cancel();
    }
    await _ignoreRenderResults(pending);
    _pendingRenders.removeWhere((key, _) => key.documentId == documentId);
    _activeRenderTokens.removeWhere((key, _) => key.documentId == documentId);
    _renderCache.removeWhere((key) => key.documentId == documentId);
    _textCache.remove(documentId);
    final document = _documents.remove(documentId);
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
  }) {
    _renderRequests++;
    final safeScale = scale.clamp(0.1, 6.0);
    final key = _RenderKey(
      documentId: documentId,
      pageIndex: pageIndex,
      scale: (safeScale * 1000).round(),
      rotation: rotation % 360,
      region: region,
    );
    final cached = _renderCache.get(key);
    if (cached != null) {
      _renderCacheHits++;
      _logger.debug(
        'Render cache hit ($_renderCacheHits hits, '
        '$_renderCacheMisses misses, ${_renderCache.length} pages).',
      );
      return Future.value(cached);
    }
    final pending = _pendingRenders[key];
    if (pending != null) {
      _deduplicatedRenders++;
      return pending;
    }
    _renderCacheMisses++;
    final render = _render(key, safeScale, region, cancellationToken)
        .whenComplete(() {
          _pendingRenders.remove(key);
        });
    _pendingRenders[key] = render;
    return render;
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
    _validatePageIndex(document, key.pageIndex);
    final page = await document.pages[key.pageIndex].ensureLoaded();
    final renderToken = page.createCancellationToken();
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
      if (rendered == null || (operationToken?.isCancelled ?? false)) {
        rendered?.dispose();
        _cancelledRenders++;
        throw const AppFailure(FailureKind.cancelled, 'Renderizado cancelado.');
      }
      final image = await rendered.createImage();
      rendered.dispose();
      final result = RenderedPdfPage(
        image: image,
        pageIndex: key.pageIndex,
        scale: scale,
      );
      _renderCache.put(key, result);
      _logger.debug(
        'Rendered page ${key.pageIndex + 1} in '
        '${stopwatch.elapsedMilliseconds} ms; '
        '${_renderCache.currentBytes} cache bytes retained.',
      );
      return result;
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
      _activeRenderTokens.remove(key);
    }
  }

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
  }) async {
    final document = documentFor(documentId);
    _validatePageIndex(document, pageIndex);
    final cached = _textCache[documentId]?[pageIndex];
    if (cached != null) return cached;
    final page = await document.pages[pageIndex].ensureLoaded();
    final text = await page.loadText();
    final value = text?.fullText ?? '';
    _textCache[documentId]?[pageIndex] = value;
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
    _renderCache.clear();
    _renderCacheHits = 0;
    _renderCacheMisses = 0;
    _renderRequests = 0;
    _deduplicatedRenders = 0;
    _cancelledRenders = 0;
    for (final cache in _textCache.values) {
      cache.clear();
    }
  }

  @override
  Future<void> dispose() async {
    final pending = _pendingRenders.values.toList(growable: false);
    for (final token in _activeRenderTokens.values) {
      token.cancel();
    }
    await _ignoreRenderResults(pending);
    _pendingRenders.clear();
    _activeRenderTokens.clear();
    clearCaches();
    _textCache.clear();
    final documents = _documents.values.toList(growable: false);
    _documents.clear();
    await Future.wait(documents.map((document) => document.dispose()));
  }

  static Future<void> _ignoreRenderResults(
    Iterable<Future<RenderedPdfPage>> futures,
  ) async {
    await Future.wait(
      futures.map((future) async {
        try {
          await future;
        } on Object {
          // Closing owns cancellation; callers receive the original failure.
        }
      }),
    );
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
    required this.pageIndex,
    required this.scale,
    required this.rotation,
    required this.region,
  });

  final String documentId;
  final int pageIndex;
  final int scale;
  final int rotation;
  final PdfRenderRegion? region;

  @override
  int get hashCode => Object.hash(
    documentId,
    pageIndex,
    scale,
    rotation,
    region?.left,
    region?.top,
    region?.width,
    region?.height,
  );

  @override
  bool operator ==(Object other) =>
      other is _RenderKey &&
      other.documentId == documentId &&
      other.pageIndex == pageIndex &&
      other.scale == scale &&
      other.rotation == rotation &&
      other.region?.left == region?.left &&
      other.region?.top == region?.top &&
      other.region?.width == region?.width &&
      other.region?.height == region?.height;
}
