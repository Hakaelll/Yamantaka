import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:pdfrx/pdfrx.dart' as pdfrx;
import 'package:url_launcher/url_launcher.dart';

import '../../app/app_config.dart';
import '../../app/theme/app_tokens.dart';
import '../../core/security/local_input_policy.dart';
import '../../features/reader/domain/reader_models.dart';
import '../../features/annotations/domain/annotation.dart';
import '../../features/reader/domain/text_selection.dart' as reader;
import '../models/pdf_models.dart';
import '../presentation/pdf_viewer_adapter.dart';
import '../presentation/reader_viewport_metrics.dart';
import 'pdfrx_document_engine.dart';

final class PdfrxViewerAdapter implements PdfViewerAdapter {
  const PdfrxViewerAdapter(this._engine);

  final PdfrxDocumentEngine _engine;

  @override
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) => _PdfrxViewer(
    key: ValueKey((
      document.documentId,
      configuration.flow,
      configuration.rotation,
      configuration.readingDirection,
    )),
    engine: _engine,
    document: document,
    bridgeController: controller,
    configuration: configuration,
    initialPageIndex: initialPageIndex,
    onPageChanged: onPageChanged,
    onError: onError,
  );
}

final class _PdfrxViewer extends StatefulWidget {
  const _PdfrxViewer({
    super.key,
    required this.engine,
    required this.document,
    required this.bridgeController,
    required this.configuration,
    required this.initialPageIndex,
    required this.onPageChanged,
    required this.onError,
  });

  final PdfrxDocumentEngine engine;
  final OpenedPdfDocument document;
  final PdfViewController bridgeController;
  final PdfViewConfiguration configuration;
  final int initialPageIndex;
  final ValueChanged<int> onPageChanged;
  final ValueChanged<Object> onError;

  @override
  State<_PdfrxViewer> createState() => _PdfrxViewerState();
}

final class _PdfrxViewerState extends State<_PdfrxViewer> {
  late final pdfrx.PdfViewerController _controller;
  late final pdfrx.PdfDocumentRefDirect _documentRef;
  Size? _viewportSize;
  bool _recenterScheduled = false;
  bool _firstPageReported = false;
  final Stopwatch _firstPageStopwatch = Stopwatch()..start();
  Timer? _interactionIdleTimer;
  final Object _attachmentOwner = Object();

  Duration get _motionDuration => widget.configuration.reduceMotion
      ? Duration.zero
      : const Duration(milliseconds: 120);

  double get _effectiveMargin => ReaderViewportMetrics.adaptiveMargin(
    viewport: _viewportSize ?? Size.zero,
    preferred: widget.configuration.margin,
  );

  @override
  void initState() {
    super.initState();
    _controller = pdfrx.PdfViewerController();
    _documentRef = pdfrx.PdfDocumentRefDirect(
      widget.engine.documentFor(widget.document.documentId),
      autoDispose: false,
    );
    _controller.addListener(_onTransformChanged);
  }

  @override
  void dispose() {
    _interactionIdleTimer?.cancel();
    widget.configuration.onInteractionChanged?.call(false);
    _controller.removeListener(_onTransformChanged);
    widget.bridgeController.detach(owner: _attachmentOwner);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      _handleViewportSize(constraints.biggest);
      return _buildViewer(context);
    },
  );

  Widget _buildViewer(BuildContext context) {
    final viewer = pdfrx.PdfViewer(
      _documentRef,
      controller: _controller,
      initialPageNumber: widget.initialPageIndex + 1,
      params: pdfrx.PdfViewerParams(
        backgroundColor: widget.configuration.backgroundColor,
        margin: _effectiveMargin,
        pageDropShadow: widget.configuration.pageShadow,
        textSelectionParams: pdfrx.PdfTextSelectionParams(
          onTextSelectionChange: _onTextSelectionChanged,
        ),
        pageOverlaysBuilder: widget.configuration.flow == PdfViewFlow.facing
            ? _buildBookPageOverlays
            : null,
        normalizeMatrix: widget.configuration.flow == PdfViewFlow.continuous
            ? null
            : (matrix, viewSize, layout, controller) {
                if (controller == null ||
                    !controller.isReady ||
                    layout.pageLayouts.isEmpty) {
                  return matrix;
                }
                final zoom = pdfrx.PdfMatrix4Ext(matrix).zoom
                    .clamp(controller.minScale, controller.maxScale)
                    .toDouble();
                final proposedCenter = pdfrx.PdfMatrix4Ext(
                  matrix,
                ).calcPosition(viewSize);
                final unitRect = ReaderViewportMetrics.nearestReadingUnitRect(
                  pageRects: layout.pageLayouts,
                  proposedCenter: proposedCenter,
                  facing: widget.configuration.flow == PdfViewFlow.facing,
                  coverPolicy: widget.configuration.coverPolicy,
                  initialSide: widget.configuration.initialSide,
                );

                final normalizedCenter = ReaderViewportMetrics.normalizeCenter(
                  proposedCenter: proposedCenter,
                  viewport: viewSize,
                  contentRect: unitRect,
                  zoom: zoom,
                  margin: widget.configuration.viewportMargin / zoom,
                );
                return controller.calcMatrixFor(
                  normalizedCenter,
                  zoom: zoom,
                  viewSize: viewSize,
                );
              },
        layoutPages: switch (widget.configuration.flow) {
          PdfViewFlow.continuous => null,
          PdfViewFlow.single => _singlePageLayout,
          PdfViewFlow.facing => _facingPageLayout,
        },
        maxImageBytesCachedOnMemory: widget.engine.viewerCacheBudgetBytes,
        getPageRenderingScale: _safeRenderingScale,
        onePassRenderingSizeThreshold: AppConfig.maxPdfiumDimension.toDouble(),
        onRenderEvent: _onRenderEvent,
        onInteractionStart: (_) {
          _interactionIdleTimer?.cancel();
          widget.configuration.onInteractionChanged?.call(true);
        },
        onInteractionEnd: (_) {
          _interactionIdleTimer?.cancel();
          _interactionIdleTimer = Timer(const Duration(milliseconds: 350), () {
            widget.configuration.onInteractionChanged?.call(false);
          });
        },
        behaviorControlParams: pdfrx.PdfViewerBehaviorControlParams(
          trailingPageLoadingDelay: const Duration(milliseconds: 160),
          enableLowResolutionPagePreview: true,
          pageImageCachingDelay: AppConfig.settledRenderDelay,
          partialImageLoadingDelay: AppConfig.settledRenderDelay,
          progressiveFadeDuration: widget.configuration.reduceMotion
              ? Duration.zero
              : const Duration(milliseconds: 90),
        ),
        horizontalCacheExtent:
            widget.configuration.flow == PdfViewFlow.continuous ? 0.5 : 1.1,
        verticalCacheExtent: widget.engine.underMemoryPressure
            ? 0.2
            : widget.configuration.flow == PdfViewFlow.continuous
            ? 0.9
            : 0.5,
        enableVerticalPageVirtualization:
            widget.configuration.flow == PdfViewFlow.continuous,
        pageVirtualizationRadius:
            widget.configuration.flow == PdfViewFlow.continuous ? 0 : 6,
        enableKeyboardNavigation: false,
        // A small range below native fit-page leaves room for the persistent
        // chrome-safe margin. The custom normalizer still constrains the
        // active page/spread, so adjacent horizontal slots cannot take focus.
        sizeDelegateProvider: const pdfrx.PdfViewerSizeDelegateProviderSmart(
          maxPagesVisible: 1.25,
          onePassRenderingScaleThreshold: AppConfig.previewRenderDpi / 72,
        ),
        linkHandlerParams: pdfrx.PdfLinkHandlerParams(
          enableAutoLinkDetection: false,
          onLinkTap: _handleLink,
        ),
        scrollHorizontallyByMouseWheel:
            widget.configuration.flow != PdfViewFlow.continuous,
        onPageChanged: (pageNumber) {
          if (pageNumber == null) return;
          final pageIndex = pageNumber - 1;
          widget.bridgeController.update(
            pageIndex: pageIndex,
            zoom: _controller.currentZoom,
          );
          widget.onPageChanged(pageIndex);
        },
        onViewerReady: (document, controller) {
          widget.bridgeController.attach(
            pageCount: document.pages.length,
            currentPageIndex: widget.initialPageIndex,
            zoom: controller.currentZoom,
            goToPage: (pageIndex) {
              final pageNumber =
                  pageIndex.clamp(0, document.pages.length - 1) + 1;
              if (widget.configuration.flow == PdfViewFlow.facing &&
                  widget.configuration.initialZoomMode ==
                      PdfViewZoomMode.fitPage) {
                return _fitPage(
                  controller,
                  pageNumber: pageNumber,
                  duration: _motionDuration,
                );
              }
              return controller.goToPage(
                pageNumber: pageNumber,
                duration: _motionDuration,
              );
            },
            zoomBy: (factor) => _setCustomZoom(
              controller,
              pageNumber: controller.pageNumber ?? 1,
              zoom: (controller.currentZoom * factor)
                  .clamp(controller.minScale, controller.maxScale)
                  .toDouble(),
              duration: _motionDuration,
            ),
            fitWidth: () => _fitWidth(
              controller,
              pageNumber: controller.pageNumber ?? 1,
              duration: _motionDuration,
            ),
            fitPage: () => _fitPage(
              controller,
              pageNumber: controller.pageNumber ?? 1,
              duration: _motionDuration,
            ),
            actualSize: () => _setCustomZoom(
              controller,
              pageNumber: controller.pageNumber ?? 1,
              zoom: 1,
              duration: _motionDuration,
            ),
            restorePosition: (zoom, scrollOffset) => _restorePosition(
              controller,
              pageNumber: controller.pageNumber ?? 1,
              zoom: zoom,
              scrollOffset: scrollOffset,
            ),
            owner: _attachmentOwner,
          );
          unawaited(_restoreInitialView(controller));
        },
        loadingBannerBuilder: (context, bytesDownloaded, totalBytes) =>
            Semantics(
              label: 'Preparando páginas',
              child: ColoredBox(color: widget.configuration.backgroundColor),
            ),
        errorBannerBuilder: (context, error, stackTrace, documentRef) {
          WidgetsBinding.instance.addPostFrameCallback(
            (_) => widget.onError(error),
          );
          return Center(
            child: Text(
              'No se pudo mostrar el documento.',
              style: Theme.of(context).textTheme.bodyLarge,
            ),
          );
        },
      ),
    );
    final turns = (widget.configuration.rotation ~/ 90) % 4;
    return turns == 0 ? viewer : RotatedBox(quarterTurns: turns, child: viewer);
  }

  void _onTextSelectionChanged(pdfrx.PdfTextSelection selection) {
    final callback = widget.configuration.onTextSelectionChanged;
    if (callback == null) return;
    if (!selection.isCopyAllowed || !selection.hasSelectedText) {
      callback(null);
      return;
    }
    unawaited(_readTextSelection(selection, callback));
  }

  Future<void> _readTextSelection(
    pdfrx.PdfTextSelection selection,
    ValueChanged<reader.TextSelection?> callback,
  ) async {
    try {
      final ranges = await selection.getSelectedTextRanges();
      if (!mounted || ranges.isEmpty || !selection.hasSelectedText) {
        callback(null);
        return;
      }
      ranges.sort((left, right) => left.pageNumber.compareTo(right.pageNumber));
      final pages = <reader.TextSelectionPageGeometry>[];
      for (final range in ranges) {
        final pageIndex = range.pageNumber - 1;
        if (pageIndex < 0 || pageIndex >= widget.document.pageSizes.length) {
          callback(null);
          return;
        }
        final size = widget.document.pageSizes[pageIndex];
        if (!size.width.isFinite ||
            !size.height.isFinite ||
            size.width <= 0 ||
            size.height <= 0) {
          callback(null);
          return;
        }
        final rects = <NormalizedRect>[];
        for (final fragment in range.enumerateFragmentBoundingRects()) {
          final bounds = fragment.bounds;
          final left = (bounds.left / size.width).clamp(0.0, 1.0);
          final top = (bounds.top / size.height).clamp(0.0, 1.0);
          final right = (bounds.right / size.width).clamp(0.0, 1.0);
          final bottom = (bounds.bottom / size.height).clamp(0.0, 1.0);
          if (![left, top, right, bottom].every((value) => value.isFinite) ||
              left >= right ||
              top >= bottom) {
            continue;
          }
          rects.add(
            NormalizedRect(left: left, top: top, right: right, bottom: bottom),
          );
        }
        pages.add(
          reader.TextSelectionPageGeometry(
            pageIndex: pageIndex,
            text: range.pageText.fullText,
            rects: rects,
          ),
        );
      }
      final first = ranges.first;
      final last = ranges.last;
      callback(
        reader.TextSelection(
          range: reader.TextSelectionRange(
            start: reader.TextSelectionPosition(
              pageIndex: first.pageNumber - 1,
              offset: first.start,
            ),
            end: reader.TextSelectionPosition(
              pageIndex: last.pageNumber - 1,
              offset: last.end,
            ),
          ),
          pages: pages,
        ),
      );
    } on Object {
      if (mounted) callback(null);
    }
  }

  double _safeRenderingScale(
    BuildContext context,
    pdfrx.PdfPage page,
    pdfrx.PdfViewerController controller,
    double estimatedScale,
  ) {
    final dimensionScale = math.min(
      AppConfig.maxPdfiumDimension / math.max(1, page.width),
      AppConfig.maxPdfiumDimension / math.max(1, page.height),
    );
    final byteScale = math.sqrt(
      AppConfig.maxSingleRenderBytes /
          (math.max(1, page.width) * math.max(1, page.height) * 4),
    );
    return estimatedScale
        .clamp(0.01, math.max(0.01, math.min(dimensionScale, byteScale)))
        .toDouble();
  }

  void _onRenderEvent(pdfrx.PdfViewerRenderEvent event) {
    widget.engine.recordViewerRenderEvent(event);
    if (!_firstPageReported &&
        (event.type == pdfrx.PdfViewerRenderEventType.previewCompleted ||
            event.type == pdfrx.PdfViewerRenderEventType.refinementCompleted)) {
      _firstPageReported = true;
      _firstPageStopwatch.stop();
      widget.engine.recordFirstPageVisible(_firstPageStopwatch.elapsed);
    }
  }

  void _handleViewportSize(Size nextSize) {
    if (!nextSize.width.isFinite ||
        !nextSize.height.isFinite ||
        nextSize.isEmpty) {
      return;
    }
    final previous = _viewportSize;
    _viewportSize = nextSize;
    if (previous == null ||
        ((previous.width - nextSize.width).abs() < 0.5 &&
            (previous.height - nextSize.height).abs() < 0.5) ||
        _recenterScheduled) {
      return;
    }
    _recenterScheduled = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _recenterScheduled = false;
      if (!mounted || !_controller.isReady) return;
      unawaited(
        _recenterAfterResize().catchError((Object _) {
          // A resize can race with closing the document.
        }),
      );
    });
  }

  Future<void> _recenterAfterResize() async {
    final pageNumber = _controller.pageNumber ?? widget.initialPageIndex + 1;
    switch (widget.configuration.initialZoomMode) {
      case PdfViewZoomMode.custom:
        await _setCustomZoom(
          _controller,
          pageNumber: pageNumber,
          zoom: _controller.currentZoom,
          duration: Duration.zero,
          preserveFocalPoint: true,
        );
      case PdfViewZoomMode.fitWidth:
        await _fitWidth(
          _controller,
          pageNumber: pageNumber,
          duration: Duration.zero,
        );
      case PdfViewZoomMode.fitPage:
        await _fitPage(
          _controller,
          pageNumber: pageNumber,
          duration: Duration.zero,
        );
    }
  }

  void _onTransformChanged() {
    if (!_controller.isReady) return;
    final pageIndex = (_controller.pageNumber ?? 1) - 1;
    if (pageIndex < 0 || pageIndex >= _controller.layout.pageLayouts.length) {
      return;
    }
    final pageRect = _controller.layout.pageLayouts[pageIndex];
    final offset = pageRect.height <= 0
        ? 0.0
        : ((_controller.visibleRect.top - pageRect.top) / pageRect.height)
              .clamp(0.0, 1.0);
    widget.bridgeController.update(
      pageIndex: pageIndex,
      zoom: _controller.currentZoom,
      scrollOffset: offset,
    );
  }

  Future<void> _restoreInitialView(pdfrx.PdfViewerController controller) async {
    switch (widget.configuration.initialZoomMode) {
      case PdfViewZoomMode.custom:
        await _setCustomZoom(
          controller,
          pageNumber: widget.initialPageIndex + 1,
          zoom: widget.configuration.initialZoom
              .clamp(controller.minScale, controller.maxScale)
              .toDouble(),
          duration: Duration.zero,
        );
      case PdfViewZoomMode.fitWidth:
        await _fitWidth(
          controller,
          pageNumber: widget.initialPageIndex + 1,
          duration: Duration.zero,
        );
      case PdfViewZoomMode.fitPage:
        await _fitPage(
          controller,
          pageNumber: widget.initialPageIndex + 1,
          duration: Duration.zero,
        );
    }
    if (widget.configuration.flow == PdfViewFlow.continuous &&
        widget.configuration.initialScrollOffset > 0 &&
        widget.initialPageIndex < controller.layout.pageLayouts.length) {
      final pageRect = controller.layout.pageLayouts[widget.initialPageIndex];
      await controller.goToPosition(
        documentOffset: Offset(
          pageRect.left,
          pageRect.top +
              pageRect.height * widget.configuration.initialScrollOffset,
        ),
        zoom: controller.currentZoom,
        duration: Duration.zero,
      );
    } else if (widget.configuration.initialScrollOffset > 0) {
      await _restoreInitialPageOffset(controller);
    }
    _onTransformChanged();
  }

  Future<void> _restoreInitialPageOffset(
    pdfrx.PdfViewerController controller,
  ) async {
    final rect = _readingUnitRect(controller, widget.initialPageIndex + 1);
    if (rect.isEmpty) return;
    final zoom = controller.currentZoom;
    final visibleHeight = controller.viewSize.height / zoom;
    final margin = widget.configuration.viewportMargin / zoom;
    final boundedRect = rect.inflate(margin);
    if (boundedRect.height <= visibleHeight) return;
    final targetTop =
        rect.top +
        rect.height * widget.configuration.initialScrollOffset.clamp(0.0, 1.0);
    final targetCenter = Offset(rect.center.dx, targetTop + visibleHeight / 2);
    final normalizedCenter = ReaderViewportMetrics.normalizeCenter(
      proposedCenter: targetCenter,
      viewport: controller.viewSize,
      contentRect: rect,
      zoom: zoom,
      margin: margin,
    );
    await controller.goTo(
      controller.calcMatrixFor(normalizedCenter, zoom: zoom),
      duration: Duration.zero,
    );
  }

  Future<void> _setCustomZoom(
    pdfrx.PdfViewerController controller, {
    required int pageNumber,
    required double zoom,
    required Duration duration,
    bool preserveFocalPoint = false,
  }) {
    final rect = _readingUnitRect(controller, pageNumber);
    if (rect.isEmpty) return Future<void>.value();
    final proposedCenter = preserveFocalPoint
        ? controller.centerPosition
        : rect.center;
    final normalizedCenter = ReaderViewportMetrics.normalizeCenter(
      proposedCenter: proposedCenter,
      viewport: controller.viewSize,
      contentRect: rect,
      zoom: zoom,
      margin: widget.configuration.viewportMargin / zoom,
    );
    return controller.goTo(
      controller.calcMatrixFor(normalizedCenter, zoom: zoom),
      duration: duration,
    );
  }

  Future<void> _fitPage(
    pdfrx.PdfViewerController controller, {
    required int pageNumber,
    required Duration duration,
  }) {
    final rect = _readingUnitRect(controller, pageNumber);
    if (rect.isEmpty) return Future<void>.value();
    final fitMargin = _effectiveMargin > widget.configuration.viewportMargin
        ? _effectiveMargin
        : widget.configuration.viewportMargin;
    final zoom = ReaderViewportMetrics.fitPageZoom(
      viewport: controller.viewSize,
      contentRect: rect,
      margin: fitMargin,
      minimum: controller.minScale,
      maximum: controller.maxScale,
    );
    return controller.goTo(
      controller.calcMatrixFor(rect.center, zoom: zoom),
      duration: duration,
    );
  }

  Future<void> _fitWidth(
    pdfrx.PdfViewerController controller, {
    required int pageNumber,
    required Duration duration,
  }) async {
    final rect = _readingUnitRect(controller, pageNumber);
    if (rect.isEmpty) return;
    final fitMargin = _effectiveMargin > widget.configuration.viewportMargin
        ? _effectiveMargin
        : widget.configuration.viewportMargin;
    final zoom = ReaderViewportMetrics.fitWidthZoom(
      viewport: controller.viewSize,
      contentRect: rect,
      margin: fitMargin,
      minimum: controller.minScale,
      maximum: controller.maxScale,
    );
    final visibleHeight = controller.viewSize.height / zoom;
    final topAlignedCenter = Offset(
      rect.center.dx,
      rect.top - fitMargin / zoom + visibleHeight / 2,
    );
    final normalizedCenter = ReaderViewportMetrics.normalizeCenter(
      proposedCenter: topAlignedCenter,
      viewport: controller.viewSize,
      contentRect: rect,
      zoom: zoom,
      margin: fitMargin / zoom,
      centerVerticallyWhenFits: false,
    );
    await controller.goTo(
      controller.calcMatrixFor(normalizedCenter, zoom: zoom),
      duration: duration,
    );
  }

  Future<void> _restorePosition(
    pdfrx.PdfViewerController controller, {
    required int pageNumber,
    required double zoom,
    required double scrollOffset,
  }) async {
    final rect = _readingUnitRect(controller, pageNumber);
    if (rect.isEmpty) return;
    final safeZoom = zoom
        .clamp(controller.minScale, controller.maxScale)
        .toDouble();
    if (widget.configuration.flow == PdfViewFlow.continuous) {
      final pageIndex = (pageNumber - 1).clamp(
        0,
        controller.layout.pageLayouts.length - 1,
      );
      final pageRect = controller.layout.pageLayouts[pageIndex];
      await controller.goToPosition(
        documentOffset: Offset(
          pageRect.left,
          pageRect.top + pageRect.height * scrollOffset.clamp(0.0, 1.0),
        ),
        zoom: safeZoom,
        duration: Duration.zero,
      );
      return;
    }
    final visibleHeight = controller.viewSize.height / safeZoom;
    final targetCenter = Offset(
      rect.center.dx,
      rect.top + rect.height * scrollOffset.clamp(0.0, 1.0) + visibleHeight / 2,
    );
    final normalizedCenter = ReaderViewportMetrics.normalizeCenter(
      proposedCenter: targetCenter,
      viewport: controller.viewSize,
      contentRect: rect,
      zoom: safeZoom,
      margin: widget.configuration.viewportMargin / safeZoom,
    );
    await controller.goTo(
      controller.calcMatrixFor(normalizedCenter, zoom: safeZoom),
      duration: Duration.zero,
    );
  }

  Rect _readingUnitRect(pdfrx.PdfViewerController controller, int pageNumber) {
    final layouts = controller.layout.pageLayouts;
    return ReaderViewportMetrics.readingUnitRect(
      pageRects: layouts,
      pageIndex: pageNumber - 1,
      facing: widget.configuration.flow == PdfViewFlow.facing,
      coverPolicy: widget.configuration.coverPolicy,
      initialSide: widget.configuration.initialSide,
    );
  }

  List<Widget> _buildBookPageOverlays(
    BuildContext _,
    Rect pageRect,
    pdfrx.PdfPage page,
  ) {
    final pageIndex = page.pageNumber - 1;
    final direction =
        widget.configuration.readingDirection ==
            PdfViewReadingDirection.rightToLeft
        ? ReadingDirection.rightToLeft
        : ReadingDirection.leftToRight;
    final spread = FacingPageSpread.forPage(
      pageIndex,
      widget.document.pageCount,
      direction: direction,
      firstPageIsCover:
          widget.configuration.coverPolicy != ReaderCoverPolicy.noCover,
      initialSide: widget.configuration.initialSide,
    );
    if (spread.leftPageIndex == null || spread.rightPageIndex == null) {
      return const [];
    }

    final pageIsLeft = spread.leftPageIndex == pageIndex;
    final channelWidth = (pageRect.width * 0.035).clamp(10.0, 24.0);
    return [
      Positioned(
        top: 0,
        bottom: 0,
        left: pageIsLeft ? null : 0,
        right: pageIsLeft ? 0 : null,
        width: channelWidth,
        child: IgnorePointer(
          child: DecoratedBox(
            key: ValueKey('book-inner-edge-$pageIndex'),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: pageIsLeft
                    ? Alignment.centerLeft
                    : Alignment.centerRight,
                end: pageIsLeft ? Alignment.centerRight : Alignment.centerLeft,
                colors: const [
                  Colors.transparent,
                  Color(0x12000000),
                  Color(0x38000000),
                ],
                stops: const [0, 0.58, 1],
              ),
            ),
          ),
        ),
      ),
    ];
  }

  void _handleLink(pdfrx.PdfLink link) {
    if (link.dest != null) {
      unawaited(_controller.goToDest(link.dest, duration: _motionDuration));
      return;
    }
    final uri = link.url;
    if (uri != null) unawaited(_confirmExternalLink(uri));
  }

  Future<void> _confirmExternalLink(Uri uri) async {
    if (!mounted) return;
    final authorized = LocalInputPolicy.authorizedExternalUri(uri);
    if (authorized == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            '${AppConfig.productName} bloqueó un enlace no seguro.',
          ),
        ),
      );
      return;
    }
    final approved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Abrir enlace externo'),
        content: SelectableText(
          'Este enlace saldrá de ${AppConfig.productName}:\n\n$authorized',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Abrir'),
          ),
        ],
      ),
    );
    if (approved != true || !mounted) return;
    final launched = await launchUrl(
      authorized,
      mode: LaunchMode.externalApplication,
    );
    if (!launched && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No se pudo abrir el enlace.')),
      );
    }
  }

  pdfrx.PdfPageLayout _facingPageLayout(
    List<pdfrx.PdfPage> pages,
    pdfrx.PdfViewerParams params,
  ) {
    final geometry = ReaderViewportMetrics.facingPageLayout(
      pageSizes: pages
          .map((page) => Size(page.width, page.height))
          .toList(growable: false),
      outerMargin: params.margin,
      pageGap: AppSizes.readerSpreadGutter,
      coverPolicy: widget.configuration.coverPolicy,
      initialSide: widget.configuration.initialSide,
      rightToLeft:
          widget.configuration.readingDirection ==
          PdfViewReadingDirection.rightToLeft,
    );
    return pdfrx.PdfPageLayout(
      pageLayouts: geometry.pageRects,
      documentSize: geometry.documentSize,
    );
  }

  pdfrx.PdfPageLayout _singlePageLayout(
    List<pdfrx.PdfPage> pages,
    pdfrx.PdfViewerParams params,
  ) {
    final geometry = ReaderViewportMetrics.singlePageLayout(
      pageSizes: pages
          .map((page) => Size(page.width, page.height))
          .toList(growable: false),
      outerMargin: params.margin,
    );
    return pdfrx.PdfPageLayout(
      pageLayouts: geometry.pageRects,
      documentSize: geometry.documentSize,
    );
  }
}
