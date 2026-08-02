import 'dart:collection';

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import '../../../app/app_config.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../annotations/domain/annotation.dart';
import '../../bookmarks/domain/bookmark.dart';
import '../../navigation/application/navigation_history_controller.dart';
import '../application/reader_controller.dart';

enum ReaderPanelSection { thumbnails, outline, bookmarks, annotations, search }

final class ReaderSidePanel extends StatefulWidget {
  const ReaderSidePanel({
    super.key,
    required this.section,
    required this.controller,
    required this.engine,
    required this.width,
    required this.onClose,
    required this.onWidthChanged,
    this.pinned = false,
    this.allowPin = false,
    this.onPinnedChanged,
    this.flatChrome = false,
    this.onExportAnnotations,
  });

  final ReaderPanelSection section;
  final ReaderController controller;
  final PdfDocumentEngine engine;
  final double width;
  final VoidCallback onClose;
  final ValueChanged<double> onWidthChanged;
  final bool pinned;
  final bool allowPin;
  final ValueChanged<bool>? onPinnedChanged;
  final bool flatChrome;
  final Future<void> Function()? onExportAnnotations;

  @override
  State<ReaderSidePanel> createState() => _ReaderSidePanelState();
}

final class _ReaderSidePanelState extends State<ReaderSidePanel> {
  final TextEditingController _searchTextController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _searchTextController.text = widget.controller.search.query;
  }

  @override
  void dispose() {
    _searchTextController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: 'Panel de $_title',
      child: FocusTraversalGroup(
        child: Material(
          key: const Key('reader-side-panel'),
          color: context.appColors.surfaceElevated,
          elevation: 0,
          child: Container(
            width: widget.width,
            decoration: BoxDecoration(
              border: Border(
                left: BorderSide(color: context.appColors.borderStrong),
              ),
              boxShadow: widget.flatChrome
                  ? null
                  : [
                      BoxShadow(
                        color: context.appColors.ambientShadow,
                        blurRadius: AppShadow.readerPanelBlur,
                        offset: const Offset(-2, 0),
                      ),
                    ],
            ),
            child: Stack(
              children: [
                Column(
                  children: [
                    SizedBox(
                      height: AppSizes.topBarHeight,
                      child: Row(
                        children: [
                          const SizedBox(width: AppSpacing.space16),
                          Expanded(
                            child: Text(
                              _title,
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                          ),
                          if (widget.allowPin)
                            IconButton(
                              tooltip: widget.pinned
                                  ? 'Desanclar panel'
                                  : 'Anclar panel',
                              onPressed: widget.onPinnedChanged == null
                                  ? null
                                  : () => widget.onPinnedChanged!(!widget.pinned),
                              icon: Icon(
                                widget.pinned
                                    ? Icons.push_pin_rounded
                                    : Icons.push_pin_outlined,
                              ),
                            ),
                          IconButton(
                            autofocus:
                                widget.section != ReaderPanelSection.search,
                            tooltip: 'Cerrar panel',
                            onPressed: widget.onClose,
                            icon: const Icon(Icons.close_rounded),
                          ),
                          const SizedBox(width: AppSpacing.space4),
                        ],
                      ),
                    ),
                    const Divider(height: 1),
                    Expanded(child: _content()),
                  ],
                ),
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  child: _PanelResizeHandle(
                    width: widget.width,
                    onWidthChanged: widget.onWidthChanged,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String get _title => switch (widget.section) {
    ReaderPanelSection.thumbnails => 'Miniaturas',
    ReaderPanelSection.outline => 'Índice',
    ReaderPanelSection.bookmarks => 'Marcadores',
    ReaderPanelSection.annotations => 'Anotaciones',
    ReaderPanelSection.search => 'Buscar',
  };

  Widget _content() => switch (widget.section) {
    ReaderPanelSection.thumbnails => _ThumbnailPanel(
      controller: widget.controller,
      engine: widget.engine,
      panelWidth: widget.width,
    ),
    ReaderPanelSection.outline => _OutlinePanel(controller: widget.controller),
    ReaderPanelSection.bookmarks => _BookmarksPanel(
      controller: widget.controller,
    ),
    ReaderPanelSection.annotations => _AnnotationsPanel(
      controller: widget.controller,
      onExport: widget.onExportAnnotations,
    ),
    ReaderPanelSection.search => _SearchPanel(
      controller: widget.controller,
      textController: _searchTextController,
    ),
  };
}

final class _PanelResizeHandle extends StatelessWidget {
  const _PanelResizeHandle({required this.width, required this.onWidthChanged});

  final double width;
  final ValueChanged<double> onWidthChanged;

  double _clamp(double value) => value
      .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
      .toDouble();

  @override
  Widget build(BuildContext context) {
    void changeBy(double delta) => onWidthChanged(_clamp(width + delta));
    return Semantics(
      label: 'Ancho del panel lateral',
      value: '${width.round()} píxeles',
      increasedValue: '${_clamp(width + 16).round()} píxeles',
      decreasedValue: '${_clamp(width - 16).round()} píxeles',
      onIncrease: () => changeBy(16),
      onDecrease: () => changeBy(-16),
      child: Focus(
        onKeyEvent: (_, event) {
          if (event is! KeyDownEvent) return KeyEventResult.ignored;
          if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
            changeBy(16);
            return KeyEventResult.handled;
          }
          if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
            changeBy(-16);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Builder(
          builder: (handleContext) => MouseRegion(
            cursor: SystemMouseCursors.resizeColumn,
            child: GestureDetector(
              key: const Key('reader-panel-resize-handle'),
              behavior: HitTestBehavior.opaque,
              dragStartBehavior: DragStartBehavior.down,
              onTapDown: (_) => Focus.of(handleContext).requestFocus(),
              onHorizontalDragStart: (_) =>
                  Focus.of(handleContext).requestFocus(),
              onHorizontalDragUpdate: (details) => changeBy(-details.delta.dx),
              child: SizedBox(
                width: 10,
                child: Center(
                  child: Container(
                    width: 2,
                    height: 36,
                    color: context.appColors.borderStrong,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _ThumbnailPanel extends StatefulWidget {
  const _ThumbnailPanel({
    required this.controller,
    required this.engine,
    required this.panelWidth,
  });

  final ReaderController controller;
  final PdfDocumentEngine engine;
  final double panelWidth;

  @override
  State<_ThumbnailPanel> createState() => _ThumbnailPanelState();
}

final class _ThumbnailPanelState extends State<_ThumbnailPanel> {
  static const _maximumRetainedRequests = 18;
  final LinkedHashMap<
    ({int pageIndex, int scaleMilli}),
    Future<RenderedPdfPage>
  >
  _renderRequests = LinkedHashMap();
  final Map<({int pageIndex, int scaleMilli}), PdfOperationToken>
  _renderTokens = {};
  String? _documentId;

  @override
  void initState() {
    super.initState();
    _documentId = widget.controller.openedDocument?.documentId;
  }

  @override
  void dispose() {
    _cancelThumbnailRequests();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ThumbnailPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    final nextDocumentId = widget.controller.openedDocument?.documentId;
    if (_documentId != nextDocumentId) {
      _cancelThumbnailRequests();
      _renderRequests.clear();
      _documentId = nextDocumentId;
    }
  }

  @override
  Widget build(BuildContext context) {
    final opened = widget.controller.openedDocument;
    if (opened == null) return const SizedBox.shrink();
    final pixelRatio = MediaQuery.devicePixelRatioOf(context).clamp(1.0, 2.0);
    return ListView.builder(
      key: const PageStorageKey('reader-thumbnails'),
      padding: const EdgeInsets.all(AppSpacing.space12),
      itemCount: opened.pageCount,
      itemBuilder: (context, index) {
        final selected = index == widget.controller.currentPageIndex;
        final pageSize = opened.pageSizes[index];
        final renderScale =
            ((widget.panelWidth - AppSpacing.space40) *
                    pixelRatio *
                    1.12 /
                    pageSize.width)
                .clamp(0.45, 1.2)
                .toDouble();
        return Semantics(
          button: true,
          selected: selected,
          label: 'Página ${index + 1}',
          child: Material(
            color: selected
                ? context.appColors.selectionBackground
                : Colors.transparent,
            child: InkWell(
              onTap: () => widget.controller.navigateTo(
                index,
                reason: NavigationReason.thumbnail,
                label: 'Miniatura de página ${index + 1}',
              ),
              child: Container(
                margin: const EdgeInsets.only(bottom: AppSpacing.space12),
                padding: const EdgeInsets.all(AppSpacing.space6),
                decoration: BoxDecoration(
                  border: Border(
                    left: BorderSide(
                      color: selected
                          ? context.appColors.accentPrimary
                          : context.appColors.borderSubtle,
                      width: selected ? 3 : 1,
                    ),
                    top: BorderSide(color: context.appColors.borderSubtle),
                    right: BorderSide(color: context.appColors.borderSubtle),
                    bottom: BorderSide(color: context.appColors.borderSubtle),
                  ),
                ),
                child: Column(
                  children: [
                    AspectRatio(
                      aspectRatio:
                          opened.pageSizes[index].width /
                          opened.pageSizes[index].height,
                      child: FutureBuilder<RenderedPdfPage>(
                        future: _renderRequests.putIfAbsent(
                          (
                            pageIndex: index,
                            scaleMilli: (renderScale * 1000).round(),
                          ),
                          () => _requestThumbnail(
                            documentId: opened.documentId,
                            pageIndex: index,
                            scale: renderScale,
                          ),
                        ),
                        builder: (context, snapshot) {
                          if (snapshot.hasData) {
                            return RawImage(
                              image: snapshot.data!.image,
                              fit: BoxFit.contain,
                              filterQuality: FilterQuality.high,
                            );
                          }
                          if (snapshot.hasError) {
                            return _ThumbnailState(
                              icon: Icons.broken_image_outlined,
                              label: 'No disponible',
                              color: context.appColors.danger,
                            );
                          }
                          return const _ThumbnailState(
                            icon: Icons.hourglass_empty_rounded,
                            label: 'Generando…',
                          );
                        },
                      ),
                    ),
                    const SizedBox(height: AppSpacing.space6),
                    Text(
                      '${index + 1}',
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                    if (widget.controller.annotations?.pageHasAnnotations(
                          index,
                        ) ??
                        false)
                      const Padding(
                        padding: EdgeInsets.only(top: AppSpacing.space4),
                        child: Icon(
                          Icons.rate_review_outlined,
                          size: 15,
                          semanticLabel: 'Página con anotaciones',
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Future<RenderedPdfPage> _requestThumbnail({
    required String documentId,
    required int pageIndex,
    required double scale,
  }) {
    while (_renderRequests.length >= _maximumRetainedRequests) {
      final oldestKey = _renderRequests.keys.first;
      _renderTokens.remove(oldestKey)?.cancel();
      _renderRequests.remove(oldestKey);
    }
    final key = (pageIndex: pageIndex, scaleMilli: (scale * 1000).round());
    final token = PdfOperationToken();
    _renderTokens[key] = token;
    return widget.engine.renderPage(
      documentId: documentId,
      pageIndex: pageIndex,
      scale: scale,
      cancellationToken: token,
    );
  }

  void _cancelThumbnailRequests() {
    for (final token in _renderTokens.values) {
      token.cancel();
    }
    _renderTokens.clear();
  }
}

final class _ThumbnailState extends StatelessWidget {
  const _ThumbnailState({required this.icon, required this.label, this.color});

  final IconData icon;
  final String label;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final resolvedColor = color ?? context.appColors.textMuted;
    return ColoredBox(
      color: context.appColors.surfaceSunken,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 18, color: resolvedColor),
            const SizedBox(height: AppSpacing.space4),
            Text(
              label,
              style: Theme.of(
                context,
              ).textTheme.labelSmall?.copyWith(color: resolvedColor),
            ),
          ],
        ),
      ),
    );
  }
}

final class _OutlinePanel extends StatelessWidget {
  const _OutlinePanel({required this.controller});

  final ReaderController controller;

  @override
  Widget build(BuildContext context) {
    if (controller.outline.isEmpty) {
      return const _PanelEmptyState(
        icon: Icons.account_tree_outlined,
        title: 'Sin índice',
        message: 'Este documento no incluye una tabla de contenidos.',
      );
    }
    final flattened = <({PdfOutlineItem item, int depth})>[];
    void append(List<PdfOutlineItem> items, int depth) {
      for (final item in items) {
        flattened.add((item: item, depth: depth));
        append(item.children, depth + 1);
      }
    }

    append(controller.outline, 0);
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: AppSpacing.space8),
      itemCount: flattened.length,
      itemBuilder: (context, index) {
        final entry = flattened[index];
        return ListTile(
          contentPadding: EdgeInsets.only(
            left: AppSpacing.space12 + entry.depth * AppSpacing.space16,
            right: AppSpacing.space8,
          ),
          title: Text(
            entry.item.title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: entry.item.pageIndex == null
              ? null
              : Text(
                  '${entry.item.pageIndex! + 1}',
                  style: Theme.of(context).textTheme.labelMedium,
                ),
          onTap: entry.item.pageIndex == null
              ? null
              : () => controller.navigateTo(
                  entry.item.pageIndex!,
                  reason: NavigationReason.outline,
                  label: 'Sección del índice',
                ),
        );
      },
    );
  }
}

final class _BookmarksPanel extends StatelessWidget {
  const _BookmarksPanel({required this.controller});

  final ReaderController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller.bookmarks,
      builder: (context, _) {
        final bookmarks = controller.bookmarks.bookmarks;
        return Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space12),
              child: SizedBox(
                width: double.infinity,
                child: OutlinedButton.icon(
                  onPressed: () => _addNamedBookmark(context),
                  icon: const Icon(Icons.bookmark_add_outlined, size: 18),
                  label: Text(
                    'Marcar página ${controller.currentPageIndex + 1}',
                  ),
                ),
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: bookmarks.isEmpty
                  ? const _PanelEmptyState(
                      icon: Icons.bookmark_border_rounded,
                      title: 'Sin marcadores',
                      message: 'Guarda una página para volver a ella después.',
                    )
                  : ListView.separated(
                      padding: const EdgeInsets.symmetric(
                        vertical: AppSpacing.space8,
                      ),
                      itemCount: bookmarks.length,
                      separatorBuilder: (_, _) => const Divider(
                        indent: AppSpacing.space16,
                        endIndent: AppSpacing.space16,
                      ),
                      itemBuilder: (context, index) => _BookmarkTile(
                        bookmark: bookmarks[index],
                        onOpen: () => controller.navigateTo(
                          bookmarks[index].pageIndex,
                          reason: NavigationReason.bookmark,
                          label:
                              'Marcador en página ${bookmarks[index].pageIndex + 1}',
                        ),
                        onDelete: () =>
                            controller.bookmarks.remove(bookmarks[index]),
                      ),
                    ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _addNamedBookmark(BuildContext context) async {
    final textController = TextEditingController();
    final label = await showDialog<String?>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Añadir marcador'),
        content: SizedBox(
          width: 360,
          child: TextField(
            controller: textController,
            autofocus: true,
            maxLength: 60,
            decoration: const InputDecoration(
              labelText: 'Nombre corto',
              hintText: 'Opcional',
            ),
            onSubmitted: (value) => Navigator.pop(context, value),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, textController.text),
            child: const Text('Guardar'),
          ),
        ],
      ),
    );
    textController.dispose();
    if (label != null) {
      controller.bookmarks.add(controller.currentPageIndex, label: label);
    }
  }
}

final class _BookmarkTile extends StatelessWidget {
  const _BookmarkTile({
    required this.bookmark,
    required this.onOpen,
    required this.onDelete,
  });

  final Bookmark bookmark;
  final VoidCallback onOpen;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(
        Icons.bookmark_rounded,
        color: context.appColors.accentPrimary,
      ),
      title: Text(bookmark.label ?? 'Página ${bookmark.pageIndex + 1}'),
      subtitle: bookmark.label == null
          ? null
          : Text('Página ${bookmark.pageIndex + 1}'),
      onTap: onOpen,
      trailing: IconButton(
        tooltip: 'Eliminar marcador',
        onPressed: onDelete,
        icon: const Icon(Icons.delete_outline_rounded),
      ),
    );
  }
}

final class _AnnotationsPanel extends StatefulWidget {
  const _AnnotationsPanel({required this.controller, this.onExport});

  final ReaderController controller;
  final Future<void> Function()? onExport;

  @override
  State<_AnnotationsPanel> createState() => _AnnotationsPanelState();
}

final class _AnnotationsPanelState extends State<_AnnotationsPanel> {
  final TextEditingController _queryController = TextEditingController();

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final annotations = widget.controller.annotations;
    if (annotations == null) {
      return const _PanelEmptyState(
        icon: Icons.rate_review_outlined,
        title: 'Anotaciones no disponibles',
        message: 'El almacén local de anotaciones no está conectado.',
      );
    }
    return ListenableBuilder(
      listenable: annotations,
      builder: (context, _) => Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(AppSpacing.space12),
            child: Column(
              children: [
                Row(
                  children: [
                    Expanded(
                      child: FilledButton.tonalIcon(
                        key: const Key('annotation-add-page-note'),
                        onPressed: () => _editAnnotation(),
                        icon: const Icon(Icons.add_comment_outlined, size: 18),
                        label: Text(
                          'Anotar página ${widget.controller.currentPageIndex + 1}',
                        ),
                      ),
                    ),
                    if (widget.onExport != null) ...[
                      const SizedBox(width: AppSpacing.space8),
                      IconButton.outlined(
                        tooltip: 'Exportar anotaciones',
                        onPressed: widget.onExport,
                        icon: const Icon(Icons.ios_share_outlined),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: AppSpacing.space8),
                TextField(
                  controller: _queryController,
                  decoration: const InputDecoration(
                    isDense: true,
                    prefixIcon: Icon(Icons.search_rounded, size: 18),
                    labelText: 'Buscar en notas',
                  ),
                  onChanged: (value) => annotations.setFilters(search: value),
                ),
                const SizedBox(height: AppSpacing.space8),
                Row(
                  children: [
                    Expanded(
                      child: DropdownButtonFormField<AnnotationType?>(
                        initialValue: annotations.typeFilter,
                        isExpanded: true,
                        decoration: const InputDecoration(
                          isDense: true,
                          labelText: 'Tipo',
                        ),
                        items: [
                          const DropdownMenuItem(
                            value: null,
                            child: Text('Todos'),
                          ),
                          ...AnnotationType.values.map(
                            (type) => DropdownMenuItem(
                              value: type,
                              child: Text(
                                type.label,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                        ],
                        onChanged: (value) => annotations.setFilters(
                          type: value,
                          clearType: value == null,
                        ),
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    Expanded(
                      child: DropdownButtonFormField<AnnotationColor?>(
                        initialValue: annotations.colorFilter,
                        isExpanded: true,
                        decoration: const InputDecoration(
                          isDense: true,
                          labelText: 'Color',
                        ),
                        items: [
                          const DropdownMenuItem(
                            value: null,
                            child: Text('Todos'),
                          ),
                          ...AnnotationColor.values.map(
                            (color) => DropdownMenuItem(
                              value: color,
                              child: Text(
                                color.label,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                        ],
                        onChanged: (value) => annotations.setFilters(
                          color: value,
                          clearColor: value == null,
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          if (annotations.reviewRequiredCount > 0)
            MaterialBanner(
              content: Text(
                '${annotations.reviewRequiredCount} anotaciones necesitan '
                'revisión porque cambió el documento.',
              ),
              leading: const Icon(Icons.warning_amber_rounded),
              actions: const [SizedBox.shrink()],
            ),
          const Divider(height: 1),
          Expanded(
            child: annotations.annotations.isEmpty
                ? const _PanelEmptyState(
                    icon: Icons.rate_review_outlined,
                    title: 'Sin anotaciones',
                    message:
                        'Añade una nota de página o una marca local para '
                        'volver a esta idea.',
                  )
                : ListView.builder(
                    key: const PageStorageKey('reader-annotations'),
                    padding: const EdgeInsets.symmetric(
                      vertical: AppSpacing.space8,
                    ),
                    itemCount: annotations.annotations.length,
                    itemBuilder: (context, index) {
                      final annotation = annotations.annotations[index];
                      return Semantics(
                        button: true,
                        label:
                            '${annotation.type.label}, página '
                            '${annotation.pageIndex + 1}, '
                            '${annotation.color.label}',
                        child: ListTile(
                          leading: _AnnotationColorMarker(
                            color: Color(annotation.color.argb),
                            type: annotation.type,
                          ),
                          title: Text(
                            annotation.note ??
                                annotation.selectedText ??
                                annotation.type.label,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          subtitle: Text(
                            [
                              'Página ${annotation.pageIndex + 1}',
                              if (annotation.tags.isNotEmpty)
                                annotation.tags.take(2).join(', '),
                              if (annotation.status ==
                                  AnnotationAttachmentStatus.reviewRequired)
                                'Revisar posición',
                            ].join(' · '),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          onTap: () => widget.controller.navigateTo(
                            annotation.pageIndex,
                            reason: NavigationReason.annotation,
                            label:
                                'Anotación en página ${annotation.pageIndex + 1}',
                          ),
                          trailing: PopupMenuButton<_AnnotationAction>(
                            tooltip: 'Acciones de anotación',
                            onSelected: (action) =>
                                _runAction(action, annotation),
                            itemBuilder: (context) => const [
                              PopupMenuItem(
                                value: _AnnotationAction.edit,
                                child: Text('Editar'),
                              ),
                              PopupMenuItem(
                                value: _AnnotationAction.copy,
                                child: Text('Copiar contenido'),
                              ),
                              PopupMenuItem(
                                value: _AnnotationAction.delete,
                                child: Text('Eliminar'),
                              ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }

  Future<void> _runAction(
    _AnnotationAction action,
    PdfAnnotation annotation,
  ) async {
    final controller = widget.controller.annotations!;
    switch (action) {
      case _AnnotationAction.edit:
        await _editAnnotation(annotation);
      case _AnnotationAction.copy:
        await Clipboard.setData(
          ClipboardData(text: annotation.note ?? annotation.selectedText ?? ''),
        );
      case _AnnotationAction.delete:
        controller.delete(annotation);
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: const Text('Anotación eliminada'),
            action: SnackBarAction(
              label: 'Deshacer',
              onPressed: () => controller.undoDelete(),
            ),
          ),
        );
    }
  }

  Future<void> _editAnnotation([PdfAnnotation? annotation]) async {
    final noteController = TextEditingController(text: annotation?.note ?? '');
    final tagsController = TextEditingController(
      text: annotation?.tags.join(', ') ?? '',
    );
    var type = annotation?.type ?? AnnotationType.pageComment;
    var color = annotation?.color ?? AnnotationColor.yellow;
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(
            annotation == null ? 'Nueva anotación' : 'Editar anotación',
          ),
          content: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 460),
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  DropdownButtonFormField<AnnotationType>(
                    initialValue: type,
                    decoration: const InputDecoration(labelText: 'Tipo'),
                    items: AnnotationType.values
                        .map(
                          (value) => DropdownMenuItem(
                            value: value,
                            child: Text(value.label),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => type = value);
                      }
                    },
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  DropdownButtonFormField<AnnotationColor>(
                    initialValue: color,
                    decoration: const InputDecoration(labelText: 'Color'),
                    items: AnnotationColor.values
                        .map(
                          (value) => DropdownMenuItem(
                            value: value,
                            child: Text(value.label),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => color = value);
                      }
                    },
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  TextField(
                    controller: noteController,
                    autofocus: true,
                    maxLength: AppConfig.maxAnnotationNoteLength,
                    maxLines: 6,
                    decoration: const InputDecoration(
                      labelText: 'Comentario',
                      alignLabelWithHint: true,
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  TextField(
                    controller: tagsController,
                    maxLength: 160,
                    decoration: const InputDecoration(
                      labelText: 'Etiquetas opcionales',
                      hintText: 'estudio, revisar',
                    ),
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancelar'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Guardar'),
            ),
          ],
        ),
      ),
    );
    if (result == true) {
      final tags = tagsController.text
          .split(',')
          .map((tag) => tag.trim())
          .where((tag) => tag.isNotEmpty)
          .toList(growable: false);
      if (annotation == null) {
        widget.controller.annotations!.create(
          pageIndex: widget.controller.currentPageIndex,
          type: type,
          color: color,
          note: noteController.text,
          pageRotation: widget.controller.preferences?.pageRotation ?? 0,
          tags: tags,
        );
      } else {
        widget.controller.annotations!.update(
          annotation.copyWith(
            type: type,
            color: color,
            note: noteController.text,
            tags: tags,
          ),
        );
      }
    }
    noteController.dispose();
    tagsController.dispose();
  }
}

enum _AnnotationAction { edit, copy, delete }

final class _AnnotationColorMarker extends StatelessWidget {
  const _AnnotationColorMarker({required this.color, required this.type});

  final Color color;
  final AnnotationType type;

  @override
  Widget build(BuildContext context) => Tooltip(
    message: type.label,
    child: Container(
      width: 28,
      height: 28,
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: context.appColors.borderStrong),
      ),
      child: Icon(
        type == AnnotationType.pageComment
            ? Icons.chat_bubble_outline_rounded
            : Icons.format_color_fill_rounded,
        size: 16,
      ),
    ),
  );
}

final class _SearchPanel extends StatelessWidget {
  const _SearchPanel({required this.controller, required this.textController});

  final ReaderController controller;
  final TextEditingController textController;

  @override
  Widget build(BuildContext context) {
    final search = controller.search;
    return ListenableBuilder(
      listenable: search,
      builder: (context, _) => Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(AppSpacing.space12),
            child: TextField(
              key: const Key('pdf-search-field'),
              controller: textController,
              autofocus: true,
              textInputAction: TextInputAction.search,
              decoration: InputDecoration(
                labelText: 'Buscar en el documento',
                hintText: 'Palabra o frase',
                suffixIcon: search.isSearching
                    ? IconButton(
                        tooltip: 'Cancelar búsqueda',
                        onPressed: search.cancel,
                        icon: const Icon(Icons.close_rounded),
                      )
                    : IconButton(
                        tooltip: 'Buscar',
                        onPressed: () => search.search(textController.text),
                        icon: const Icon(Icons.search_rounded),
                      ),
              ),
              onSubmitted: search.search,
            ),
          ),
          if (search.isSearching)
            Semantics(
              liveRegion: true,
              label: 'Buscando en el documento',
              child: const Column(
                children: [
                  LinearProgressIndicator(minHeight: 2),
                  Padding(
                    padding: EdgeInsets.symmetric(
                      horizontal: AppSpacing.space12,
                      vertical: AppSpacing.space8,
                    ),
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: Text('Buscando… Puedes cancelar la operación.'),
                    ),
                  ),
                ],
              ),
            ),
          if (!search.isSearching && search.matches.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(
                AppSpacing.space12,
                0,
                AppSpacing.space8,
                AppSpacing.space8,
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      '${search.activeIndex + 1} de ${search.matches.length}',
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ),
                  IconButton(
                    tooltip: 'Resultado anterior',
                    onPressed: search.previous,
                    icon: const Icon(Icons.keyboard_arrow_up_rounded),
                  ),
                  IconButton(
                    tooltip: 'Resultado siguiente',
                    onPressed: search.next,
                    icon: const Icon(Icons.keyboard_arrow_down_rounded),
                  ),
                ],
              ),
            ),
          if (search.noExtractableText)
            const _PanelEmptyState(
              icon: Icons.image_search_outlined,
              title: 'Sin texto extraíble',
              message:
                  'Este PDF puede ser un escaneo. '
                  '${AppConfig.productName} todavía no incluye OCR.',
            )
          else if (!search.isSearching &&
              search.query.isNotEmpty &&
              search.matches.isEmpty)
            const _PanelEmptyState(
              icon: Icons.search_off_rounded,
              title: 'Sin resultados',
              message: 'Prueba otra palabra o una frase más corta.',
            ),
          if (search.error case final error?)
            Builder(
              builder: (context) {
                final presentation = FailurePresentations.forFailure(error);
                return Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: AppStatusLabel(
                    icon: Icons.error_outline_rounded,
                    label: '${presentation.title}: ${presentation.message}',
                    tone: AppStatusTone.error,
                  ),
                );
              },
            ),
          Expanded(
            child: ListView.separated(
              itemCount: search.matches.length,
              separatorBuilder: (_, _) => const Divider(
                indent: AppSpacing.space12,
                endIndent: AppSpacing.space12,
              ),
              itemBuilder: (context, index) {
                final match = search.matches[index];
                return ListTile(
                  selected: index == search.activeIndex,
                  title: Text(
                    match.snippet.isEmpty ? 'Coincidencia' : match.snippet,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                  ),
                  subtitle: Text('Página ${match.pageIndex + 1}'),
                  onTap: () => search.select(index),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

final class _PanelEmptyState extends StatelessWidget {
  const _PanelEmptyState({
    required this.icon,
    required this.title,
    required this.message,
  });

  final IconData icon;
  final String title;
  final String message;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.space24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, color: context.appColors.textMuted, size: 24),
            const SizedBox(height: AppSpacing.space12),
            Text(title, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: AppSpacing.space6),
            Text(
              message,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}
