import 'dart:async';

import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';

final class DocumentComparisonDialog extends StatefulWidget {
  const DocumentComparisonDialog({
    super.key,
    required this.engine,
    required this.primary,
    required this.primaryTitle,
    required this.secondaryPath,
    required this.secondaryTitle,
    required this.initialPage,
  });

  final PdfDocumentEngine engine;
  final OpenedPdfDocument primary;
  final String primaryTitle;
  final String secondaryPath;
  final String secondaryTitle;
  final int initialPage;

  @override
  State<DocumentComparisonDialog> createState() =>
      _DocumentComparisonDialogState();
}

final class _DocumentComparisonDialogState
    extends State<DocumentComparisonDialog> {
  OpenedPdfDocument? _secondary;
  Object? _error;
  bool _closing = false;
  bool _swapped = false;
  bool _syncPage = true;
  bool _syncZoom = true;
  int _leftPage = 0;
  int _rightPage = 0;
  double _leftZoom = 1;
  double _rightZoom = 1;
  final Set<int> _differenceMarkers = {};

  @override
  void initState() {
    super.initState();
    _leftPage = widget.initialPage.clamp(0, widget.primary.pageCount - 1);
    _rightPage = _leftPage;
    unawaited(_openSecondary());
  }

  Future<void> _openSecondary() async {
    try {
      final opened = await widget.engine.openDocument(widget.secondaryPath);
      if (!mounted) {
        await widget.engine.closeDocument(opened.documentId);
        return;
      }
      setState(() {
        _secondary = opened;
        _rightPage = _rightPage.clamp(0, opened.pageCount - 1);
      });
    } on Object catch (error) {
      if (mounted) setState(() => _error = error);
    }
  }

  @override
  void dispose() {
    if (!_closing && _secondary != null) unawaited(_releaseSecondary());
    super.dispose();
  }

  Future<void> _releaseSecondary() async {
    if (_closing) return;
    _closing = true;
    final secondary = _secondary;
    _secondary = null;
    if (secondary != null) {
      await widget.engine.closeDocument(secondary.documentId);
    }
  }

  Future<void> _close() async {
    await _releaseSecondary();
    if (mounted) Navigator.pop(context);
  }

  OpenedPdfDocument get _leftDocument =>
      _swapped ? _secondary! : widget.primary;
  OpenedPdfDocument get _rightDocument =>
      _swapped ? widget.primary : _secondary!;
  String get _leftTitle =>
      _swapped ? widget.secondaryTitle : widget.primaryTitle;
  String get _rightTitle =>
      _swapped ? widget.primaryTitle : widget.secondaryTitle;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return PopScope(
      canPop: true,
      onPopInvokedWithResult: (didPop, _) {
        if (didPop) unawaited(_releaseSecondary());
      },
      child: Dialog.fullscreen(
        child: Column(
          children: [
            Material(
              color: Theme.of(context).colorScheme.surfaceContainer,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space12,
                  vertical: AppSpacing.space8,
                ),
                child: Column(
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.compare_rounded),
                        const SizedBox(width: AppSpacing.space8),
                        Expanded(
                          child: Text(
                            strings.get('comparison.title'),
                            style: Theme.of(context).textTheme.titleLarge,
                          ),
                        ),
                        IconButton(
                          tooltip: strings.get('action.close'),
                          onPressed: _closing
                              ? null
                              : () => unawaited(_close()),
                          icon: const Icon(Icons.close_rounded),
                        ),
                      ],
                    ),
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: Row(
                        children: [
                          FilterChip(
                            selected: _syncPage,
                            onSelected: (value) =>
                                setState(() => _syncPage = value),
                            avatar: const Icon(
                              Icons.sync_alt_rounded,
                              size: 18,
                            ),
                            label: Text(strings.get('comparison.syncPage')),
                          ),
                          const SizedBox(width: AppSpacing.space8),
                          FilterChip(
                            selected: _syncZoom,
                            onSelected: (value) =>
                                setState(() => _syncZoom = value),
                            avatar: const Icon(
                              Icons.zoom_in_map_rounded,
                              size: 18,
                            ),
                            label: Text(strings.get('comparison.syncZoom')),
                          ),
                          IconButton(
                            tooltip: strings.get('comparison.swap'),
                            onPressed: _secondary == null
                                ? null
                                : () => setState(() => _swapped = !_swapped),
                            icon: const Icon(Icons.swap_horiz_rounded),
                          ),
                          IconButton(
                            tooltip: strings.get('comparison.markDifference'),
                            onPressed: _secondary == null
                                ? null
                                : () => setState(() {
                                    final marker = _leftPage;
                                    if (!_differenceMarkers.add(marker)) {
                                      _differenceMarkers.remove(marker);
                                    }
                                  }),
                            icon: Icon(
                              _differenceMarkers.contains(_leftPage)
                                  ? Icons.flag_rounded
                                  : Icons.outlined_flag_rounded,
                            ),
                          ),
                          IconButton(
                            tooltip: strings.get(
                              'comparison.previousDifference',
                            ),
                            onPressed: _previousDifference,
                            icon: const Icon(Icons.skip_previous_rounded),
                          ),
                          IconButton(
                            tooltip: strings.get('comparison.nextDifference'),
                            onPressed: _nextDifference,
                            icon: const Icon(Icons.skip_next_rounded),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: _error != null
                  ? Center(child: Text(strings.get('comparison.openFailed')))
                  : _secondary == null
                  ? const Center(child: CircularProgressIndicator())
                  : Row(
                      children: [
                        Expanded(
                          child: _ComparisonPane(
                            key: ValueKey('left-${_leftDocument.documentId}'),
                            engine: widget.engine,
                            document: _leftDocument,
                            title: _leftTitle,
                            pageIndex: _leftPage,
                            zoom: _leftZoom,
                            marked: _differenceMarkers.contains(_leftPage),
                            onPageChanged: (page) => _setPage(true, page),
                            onZoomChanged: (zoom) => _setZoom(true, zoom),
                          ),
                        ),
                        VerticalDivider(
                          width: 8,
                          thickness: 2,
                          color: Theme.of(context).colorScheme.primary,
                        ),
                        Expanded(
                          child: _ComparisonPane(
                            key: ValueKey('right-${_rightDocument.documentId}'),
                            engine: widget.engine,
                            document: _rightDocument,
                            title: _rightTitle,
                            pageIndex: _rightPage,
                            zoom: _rightZoom,
                            marked: _differenceMarkers.contains(_rightPage),
                            onPageChanged: (page) => _setPage(false, page),
                            onZoomChanged: (zoom) => _setZoom(false, zoom),
                          ),
                        ),
                      ],
                    ),
            ),
          ],
        ),
      ),
    );
  }

  void _setPage(bool left, int page) {
    setState(() {
      if (left) {
        _leftPage = page.clamp(0, _leftDocument.pageCount - 1);
        if (_syncPage) {
          _rightPage = page.clamp(0, _rightDocument.pageCount - 1);
        }
      } else {
        _rightPage = page.clamp(0, _rightDocument.pageCount - 1);
        if (_syncPage) {
          _leftPage = page.clamp(0, _leftDocument.pageCount - 1);
        }
      }
    });
  }

  void _setZoom(bool left, double zoom) {
    setState(() {
      if (left) {
        _leftZoom = zoom;
        if (_syncZoom) _rightZoom = zoom;
      } else {
        _rightZoom = zoom;
        if (_syncZoom) _leftZoom = zoom;
      }
    });
  }

  void _previousDifference() {
    final target = _differenceMarkers
        .where((page) => page < _leftPage)
        .fold<int?>(
          null,
          (best, page) => best == null || page > best ? page : best,
        );
    if (target != null) _setPage(true, target);
  }

  void _nextDifference() {
    final target = _differenceMarkers
        .where((page) => page > _leftPage)
        .fold<int?>(
          null,
          (best, page) => best == null || page < best ? page : best,
        );
    if (target != null) _setPage(true, target);
  }
}

final class _ComparisonPane extends StatefulWidget {
  const _ComparisonPane({
    super.key,
    required this.engine,
    required this.document,
    required this.title,
    required this.pageIndex,
    required this.zoom,
    required this.marked,
    required this.onPageChanged,
    required this.onZoomChanged,
  });

  final PdfDocumentEngine engine;
  final OpenedPdfDocument document;
  final String title;
  final int pageIndex;
  final double zoom;
  final bool marked;
  final ValueChanged<int> onPageChanged;
  final ValueChanged<double> onZoomChanged;

  @override
  State<_ComparisonPane> createState() => _ComparisonPaneState();
}

final class _ComparisonPaneState extends State<_ComparisonPane> {
  PdfOperationToken? _token;
  Future<RenderedPdfPage>? _render;

  @override
  void initState() {
    super.initState();
    _startRender();
  }

  @override
  void didUpdateWidget(covariant _ComparisonPane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.document.documentId != widget.document.documentId ||
        oldWidget.pageIndex != widget.pageIndex ||
        oldWidget.zoom != widget.zoom) {
      _startRender();
    }
  }

  @override
  void dispose() {
    _token?.cancel();
    super.dispose();
  }

  void _startRender() {
    _token?.cancel();
    final token = PdfOperationToken();
    _token = token;
    _render = widget.engine.renderPage(
      documentId: widget.document.documentId,
      pageIndex: widget.pageIndex,
      scale: widget.zoom.clamp(.5, 1.6),
      devicePixelRatio: 1,
      cancellationToken: token,
      priority: PdfRenderPriority.visible,
    );
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Column(
      children: [
        ListTile(
          dense: true,
          title: Text(
            widget.title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          subtitle: Text(
            strings.get(
              'comparison.pageOf',
              parameters: {
                'page': widget.pageIndex + 1,
                'total': widget.document.pageCount,
              },
            ),
          ),
          trailing: widget.marked
              ? const Icon(Icons.flag_rounded, color: Colors.orange)
              : null,
        ),
        Expanded(
          child: ColoredBox(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: FutureBuilder<RenderedPdfPage>(
              future: _render,
              builder: (context, snapshot) {
                if (snapshot.hasError) {
                  return Center(
                    child: Text(strings.get('comparison.renderFailed')),
                  );
                }
                final page = snapshot.data;
                if (page == null) {
                  return const Center(child: CircularProgressIndicator());
                }
                return InteractiveViewer(
                  minScale: 1,
                  maxScale: 3,
                  child: Center(
                    child: RawImage(
                      image: page.image,
                      fit: BoxFit.contain,
                      filterQuality: FilterQuality.medium,
                    ),
                  ),
                );
              },
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.space8,
            vertical: AppSpacing.space4,
          ),
          child: Row(
            children: [
              IconButton(
                tooltip: strings.get('comparison.previousPage'),
                onPressed: widget.pageIndex <= 0
                    ? null
                    : () => widget.onPageChanged(widget.pageIndex - 1),
                icon: const Icon(Icons.chevron_left_rounded),
              ),
              Expanded(
                child: Slider(
                  value: widget.pageIndex.toDouble(),
                  min: 0,
                  max: (widget.document.pageCount - 1).toDouble(),
                  divisions: widget.document.pageCount > 1
                      ? widget.document.pageCount - 1
                      : null,
                  label: '${widget.pageIndex + 1}',
                  onChanged: (value) => widget.onPageChanged(value.round()),
                ),
              ),
              IconButton(
                tooltip: strings.get('comparison.nextPage'),
                onPressed: widget.pageIndex >= widget.document.pageCount - 1
                    ? null
                    : () => widget.onPageChanged(widget.pageIndex + 1),
                icon: const Icon(Icons.chevron_right_rounded),
              ),
              IconButton(
                tooltip: strings.get('comparison.zoomOut'),
                onPressed: widget.zoom <= .5
                    ? null
                    : () => widget.onZoomChanged(
                        (widget.zoom - .1).clamp(.5, 1.6),
                      ),
                icon: const Icon(Icons.zoom_out_rounded),
              ),
              Text('${(widget.zoom * 100).round()} %'),
              IconButton(
                tooltip: strings.get('comparison.zoomIn'),
                onPressed: widget.zoom >= 1.6
                    ? null
                    : () => widget.onZoomChanged(
                        (widget.zoom + .1).clamp(.5, 1.6),
                      ),
                icon: const Icon(Icons.zoom_in_rounded),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
