import 'package:flutter/material.dart';

import '../../../app/app_config.dart';
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
    required this.onClose,
  });

  final ReaderPanelSection section;
  final ReaderController controller;
  final PdfDocumentEngine engine;
  final VoidCallback onClose;

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
    return Material(
      elevation: 8,
      color: Theme.of(context).colorScheme.surface,
      child: SizedBox(
        width: 304,
        child: Column(
          children: [
            SizedBox(
              height: 48,
              child: Row(
                children: [
                  const SizedBox(width: 16),
                  Expanded(
                    child: Text(
                      _title,
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                  ),
                  IconButton(
                    tooltip: 'Cerrar panel',
                    onPressed: widget.onClose,
                    icon: const Icon(Icons.close_rounded),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            Expanded(child: _content()),
          ],
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

final class _ThumbnailPanel extends StatelessWidget {
  const _ThumbnailPanel({required this.controller, required this.engine});

  final ReaderController controller;
  final PdfDocumentEngine engine;

  @override
  Widget build(BuildContext context) {
    final opened = controller.openedDocument;
    if (opened == null) return const SizedBox.shrink();
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: opened.pageCount,
      itemBuilder: (context, index) {
        final selected = index == controller.currentPageIndex;
        return InkWell(
          onTap: () => controller.viewController.goToPage(index),
          child: Container(
            margin: const EdgeInsets.only(bottom: 12),
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              border: Border.all(
                color: selected
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).dividerColor,
                width: selected ? 2 : 1,
              ),
            ),
            child: Column(
              children: [
                AspectRatio(
                  aspectRatio:
                      opened.pageSizes[index].width /
                      opened.pageSizes[index].height,
                  child: FutureBuilder<RenderedPdfPage>(
                    future: engine.renderPage(
                      documentId: opened.documentId,
                      pageIndex: index,
                      scale: 0.2,
                    ),
                    builder: (context, snapshot) {
                      if (snapshot.hasData) {
                        return RawImage(
                          image: snapshot.data!.image,
                          fit: BoxFit.contain,
                        );
                      }
                      if (snapshot.hasError) {
                        return const Center(
                          child: Icon(Icons.broken_image_outlined),
                        );
                      }
                      return const Center(
                        child: SizedBox.square(
                          dimension: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        ),
                      );
                    },
                  ),
                ),
                const SizedBox(height: 4),
                Text('${index + 1}'),
              ],
            ),
          ),
        );
      },
    );
  }
}

final class _OutlinePanel extends StatelessWidget {
  const _OutlinePanel({required this.controller});

  final ReaderController controller;

  @override
  Widget build(BuildContext context) {
    if (controller.outline.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Text(
            'Este documento no contiene un índice.',
            textAlign: TextAlign.center,
          ),
        ),
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
      itemCount: flattened.length,
      itemBuilder: (context, index) {
        final entry = flattened[index];
        return ListTile(
          contentPadding: EdgeInsets.only(
            left: 12 + entry.depth * 16,
            right: 8,
          ),
          title: Text(
            entry.item.title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: entry.item.pageIndex == null
              ? null
              : Text('${entry.item.pageIndex! + 1}'),
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
              padding: const EdgeInsets.all(12),
              child: SizedBox(
                width: double.infinity,
                child: OutlinedButton.icon(
                  onPressed: () => _addNamedBookmark(context),
                  icon: const Icon(Icons.bookmark_add_outlined),
                  label: Text(
                    'Marcar página ${controller.currentPageIndex + 1}',
                  ),
                ),
              ),
            ),
            Expanded(
              child: bookmarks.isEmpty
                  ? const Center(
                      child: Padding(
                        padding: EdgeInsets.all(24),
                        child: Text(
                          'Aún no hay marcadores.',
                          textAlign: TextAlign.center,
                        ),
                      ),
                    )
                  : ListView.builder(
                      itemCount: bookmarks.length,
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
        content: TextField(
          controller: textController,
          autofocus: true,
          maxLength: 60,
          decoration: const InputDecoration(
            labelText: 'Nombre corto (opcional)',
          ),
          onSubmitted: (value) => Navigator.pop(context, value),
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
      leading: const Icon(Icons.bookmark_rounded),
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
            padding: const EdgeInsets.all(12),
            child: TextField(
              key: const Key('pdf-search-field'),
              controller: textController,
              autofocus: true,
              textInputAction: TextInputAction.search,
              decoration: InputDecoration(
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
          if (search.isSearching) const LinearProgressIndicator(minHeight: 2),
          if (!search.isSearching && search.matches.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 0, 12, 8),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      '${search.activeIndex + 1} de ${search.matches.length}',
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
            Padding(
              padding: EdgeInsets.all(20),
              child: Text(
                'No hay texto extraíble. Este PDF puede ser un escaneo; '
                '${AppConfig.productName} todavía no incluye OCR.',
                textAlign: TextAlign.center,
              ),
            )
          else if (!search.isSearching &&
              search.query.isNotEmpty &&
              search.matches.isEmpty)
            const Padding(
              padding: EdgeInsets.all(20),
              child: Text('No se encontraron resultados.'),
            ),
          if (search.error case final error?)
            Padding(
              padding: const EdgeInsets.all(20),
              child: Text(error.userMessage),
            ),
          Expanded(
            child: ListView.builder(
              itemCount: search.matches.length,
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
