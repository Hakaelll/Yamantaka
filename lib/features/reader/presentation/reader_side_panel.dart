import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/gestures.dart';

import '../../../app/app_config.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../bookmarks/domain/bookmark.dart';
import '../application/reader_controller.dart';

enum ReaderPanelSection { thumbnails, outline, bookmarks, search }

final class ReaderSidePanel extends StatefulWidget {
  const ReaderSidePanel({
    super.key,
    required this.section,
    required this.controller,
    required this.engine,
    required this.width,
    required this.onClose,
    required this.onWidthChanged,
  });

  final ReaderPanelSection section;
  final ReaderController controller;
  final PdfDocumentEngine engine;
  final double width;
  final VoidCallback onClose;
  final ValueChanged<double> onWidthChanged;

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
          color: context.appColors.surfacePrimary,
          elevation: 0,
          child: Container(
            width: widget.width,
            decoration: BoxDecoration(
              border: Border(
                left: BorderSide(color: context.appColors.borderStrong),
              ),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.12),
                  blurRadius: 8,
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
    ReaderPanelSection.search => 'Buscar',
  };

  Widget _content() => switch (widget.section) {
    ReaderPanelSection.thumbnails => _ThumbnailPanel(
      controller: widget.controller,
      engine: widget.engine,
    ),
    ReaderPanelSection.outline => _OutlinePanel(controller: widget.controller),
    ReaderPanelSection.bookmarks => _BookmarksPanel(
      controller: widget.controller,
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
  const _ThumbnailPanel({required this.controller, required this.engine});

  final ReaderController controller;
  final PdfDocumentEngine engine;

  @override
  State<_ThumbnailPanel> createState() => _ThumbnailPanelState();
}

final class _ThumbnailPanelState extends State<_ThumbnailPanel> {
  final Map<int, Future<RenderedPdfPage>> _renderRequests = {};
  String? _documentId;

  @override
  void initState() {
    super.initState();
    _documentId = widget.controller.openedDocument?.documentId;
  }

  @override
  void didUpdateWidget(covariant _ThumbnailPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    final nextDocumentId = widget.controller.openedDocument?.documentId;
    if (_documentId != nextDocumentId) {
      _renderRequests.clear();
      _documentId = nextDocumentId;
    }
  }

  @override
  Widget build(BuildContext context) {
    final opened = widget.controller.openedDocument;
    if (opened == null) return const SizedBox.shrink();
    return ListView.builder(
      key: const PageStorageKey('reader-thumbnails'),
      padding: const EdgeInsets.all(AppSpacing.space12),
      itemCount: opened.pageCount,
      itemBuilder: (context, index) {
        final selected = index == widget.controller.currentPageIndex;
        return Semantics(
          button: true,
          selected: selected,
          label: 'Página ${index + 1}',
          child: Material(
            color: selected
                ? context.appColors.selectionBackground
                : Colors.transparent,
            child: InkWell(
              onTap: () => widget.controller.viewController.goToPage(index),
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
                          index,
                          () => widget.engine.renderPage(
                            documentId: opened.documentId,
                            pageIndex: index,
                            scale: 0.2,
                          ),
                        ),
                        builder: (context, snapshot) {
                          if (snapshot.hasData) {
                            return RawImage(
                              image: snapshot.data!.image,
                              fit: BoxFit.contain,
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
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
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
              : () => controller.viewController.goToPage(entry.item.pageIndex!),
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
                        onOpen: () => controller.viewController.goToPage(
                          bookmarks[index].pageIndex,
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
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space16),
              child: AppStatusLabel(
                icon: Icons.error_outline_rounded,
                label: error.userMessage,
                tone: AppStatusTone.error,
              ),
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
