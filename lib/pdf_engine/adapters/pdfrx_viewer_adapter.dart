import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:pdfrx/pdfrx.dart' as pdfrx;
import 'package:url_launcher/url_launcher.dart';

import '../../app/app_config.dart';
import '../models/pdf_models.dart';
import '../presentation/pdf_viewer_adapter.dart';
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
    key: ValueKey(document.documentId),
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

  Duration get _motionDuration => widget.configuration.reduceMotion
      ? Duration.zero
      : const Duration(milliseconds: 120);

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
    _controller.removeListener(_onTransformChanged);
    widget.bridgeController.detach();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final viewer = pdfrx.PdfViewer(
      _documentRef,
      controller: _controller,
      initialPageNumber: widget.initialPageIndex + 1,
      params: pdfrx.PdfViewerParams(
        backgroundColor: widget.configuration.backgroundColor,
        margin: widget.configuration.margin,
        layoutPages: switch (widget.configuration.flow) {
          PdfViewFlow.continuous => null,
          PdfViewFlow.single => _singlePageLayout,
          PdfViewFlow.facing => _facingPageLayout,
        },
        maxImageBytesCachedOnMemory: 96 * 1024 * 1024,
        horizontalCacheExtent: 0.5,
        verticalCacheExtent: 0.75,
        enableKeyboardNavigation: false,
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
            goToPage: (pageIndex) => controller.goToPage(
              pageNumber: pageIndex.clamp(0, document.pages.length - 1) + 1,
              duration: _motionDuration,
            ),
            zoomBy: (factor) => controller.setZoom(
              controller.centerPosition,
              (controller.currentZoom * factor).clamp(
                controller.minScale,
                controller.maxScale,
              ),
              duration: _motionDuration,
            ),
            fitWidth: () async {
              final pageNumber = controller.pageNumber ?? 1;
              final matrix = controller.calcMatrixFitWidthForPage(
                pageNumber: pageNumber,
              );
              if (matrix != null) {
                await controller.goTo(matrix, duration: _motionDuration);
              }
            },
            fitPage: () async {
              final pageNumber = controller.pageNumber ?? 1;
              await controller.goTo(
                controller.calcMatrixForPage(pageNumber: pageNumber),
                duration: _motionDuration,
              );
            },
          );
          unawaited(_restoreInitialView(controller));
        },
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
        await controller.setZoom(
          controller.centerPosition,
          widget.configuration.initialZoom.clamp(
            controller.minScale,
            controller.maxScale,
          ),
          duration: Duration.zero,
        );
      case PdfViewZoomMode.fitWidth:
        final matrix = controller.calcMatrixFitWidthForPage(
          pageNumber: widget.initialPageIndex + 1,
        );
        if (matrix != null) {
          await controller.goTo(matrix, duration: Duration.zero);
        }
      case PdfViewZoomMode.fitPage:
        await controller.goTo(
          controller.calcMatrixForPage(pageNumber: widget.initialPageIndex + 1),
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
    }
    _onTransformChanged();
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
    final scheme = uri.scheme.toLowerCase();
    if (scheme != 'http' && scheme != 'https' && scheme != 'mailto') {
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
          'Este enlace saldrá de ${AppConfig.productName}:\n\n$uri',
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
    final launched = await launchUrl(uri, mode: LaunchMode.externalApplication);
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
    if (pages.isEmpty) {
      return pdfrx.PdfPageLayout(
        pageLayouts: const [],
        documentSize: Size.zero,
      );
    }
    final margin = params.margin;
    final widestSpread = pages.length == 1
        ? pages.first.width
        : pages
                      .skip(1)
                      .fold<double>(
                        0,
                        (widest, page) => math.max(widest, page.width),
                      ) *
                  2 +
              margin;
    final spreadWidth = math.max(pages.first.width, widestSpread) + margin * 2;
    final documentHeight =
        pages.fold<double>(0, (height, page) => math.max(height, page.height)) +
        margin * 2;
    final rects = List<Rect>.filled(pages.length, Rect.zero);
    var x = margin;
    rects[0] = Rect.fromLTWH(
      x + (spreadWidth - margin * 2 - pages[0].width) / 2,
      (documentHeight - pages[0].height) / 2,
      pages[0].width,
      pages[0].height,
    );
    x += spreadWidth;
    for (var index = 1; index < pages.length; index += 2) {
      final first = pages[index];
      final second = index + 1 < pages.length ? pages[index + 1] : null;
      if (second == null) {
        rects[index] = Rect.fromLTWH(
          x + (spreadWidth - first.width) / 2,
          (documentHeight - first.height) / 2,
          first.width,
          first.height,
        );
        x += spreadWidth;
        continue;
      }
      final left =
          widget.configuration.readingDirection ==
              PdfViewReadingDirection.leftToRight
          ? first
          : second;
      final right =
          widget.configuration.readingDirection ==
              PdfViewReadingDirection.leftToRight
          ? second
          : first;
      final pairWidth = left.width + margin + right.width;
      final startX = x + (spreadWidth - pairWidth) / 2;
      final leftIndex =
          widget.configuration.readingDirection ==
              PdfViewReadingDirection.leftToRight
          ? index
          : index + 1;
      rects[leftIndex] = Rect.fromLTWH(
        startX,
        (documentHeight - left.height) / 2,
        left.width,
        left.height,
      );
      final rightIndex =
          widget.configuration.readingDirection ==
              PdfViewReadingDirection.leftToRight
          ? index + 1
          : index;
      rects[rightIndex] = Rect.fromLTWH(
        startX + left.width + margin,
        (documentHeight - right.height) / 2,
        right.width,
        right.height,
      );
      x += spreadWidth;
    }
    return pdfrx.PdfPageLayout(
      pageLayouts: rects,
      documentSize: Size(x + margin, documentHeight),
    );
  }

  pdfrx.PdfPageLayout _singlePageLayout(
    List<pdfrx.PdfPage> pages,
    pdfrx.PdfViewerParams params,
  ) {
    if (pages.isEmpty) {
      return pdfrx.PdfPageLayout(
        pageLayouts: const [],
        documentSize: Size.zero,
      );
    }
    final margin = params.margin;
    final slotWidth =
        pages.fold<double>(0, (width, page) => math.max(width, page.width)) +
        margin * 2;
    final documentHeight =
        pages.fold<double>(0, (height, page) => math.max(height, page.height)) +
        margin * 2;
    final rects = <Rect>[];
    for (var index = 0; index < pages.length; index++) {
      final page = pages[index];
      rects.add(
        Rect.fromLTWH(
          index * slotWidth + (slotWidth - page.width) / 2,
          (documentHeight - page.height) / 2,
          page.width,
          page.height,
        ),
      );
    }
    return pdfrx.PdfPageLayout(
      pageLayouts: rects,
      documentSize: Size(slotWidth * pages.length, documentHeight),
    );
  }
}
