import 'package:flutter/material.dart';

import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

final class ReaderBottomControls extends StatelessWidget {
  const ReaderBottomControls({
    super.key,
    required this.controller,
    required this.onGoToPage,
    required this.onChooseTheme,
  });

  final ReaderController controller;
  final VoidCallback onGoToPage;
  final VoidCallback onChooseTheme;

  @override
  Widget build(BuildContext context) {
    final preferences = controller.preferences;
    if (preferences == null) return const SizedBox.shrink();
    return Material(
      color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.97),
      child: SizedBox(
        height: 48,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            IconButton(
              tooltip: 'Página anterior',
              onPressed: controller.canGoBack ? controller.previousPage : null,
              icon: const Icon(Icons.chevron_left_rounded),
            ),
            TextButton(
              onPressed: onGoToPage,
              child: Text(
                '${controller.currentPageIndex + 1} / ${controller.pageCount}',
              ),
            ),
            IconButton(
              tooltip: 'Página siguiente',
              onPressed: controller.canGoForward ? controller.nextPage : null,
              icon: const Icon(Icons.chevron_right_rounded),
            ),
            const VerticalDivider(indent: 10, endIndent: 10),
            IconButton(
              tooltip: 'Reducir zoom',
              onPressed: () => controller.zoomBy(0.85),
              icon: const Icon(Icons.zoom_out_rounded),
            ),
            Text(
              '${(controller.viewController.zoom * 100).round()}%',
              style: Theme.of(context).textTheme.labelMedium,
            ),
            IconButton(
              tooltip: 'Aumentar zoom',
              onPressed: () => controller.zoomBy(1.18),
              icon: const Icon(Icons.zoom_in_rounded),
            ),
            PopupMenuButton<ZoomMode>(
              tooltip: 'Ajuste de zoom',
              icon: const Icon(Icons.fit_screen_rounded),
              onSelected: (mode) {
                if (mode == ZoomMode.fitWidth) {
                  controller.fitWidth();
                } else {
                  controller.fitPage();
                }
              },
              itemBuilder: (_) => const [
                PopupMenuItem(
                  value: ZoomMode.fitWidth,
                  child: Text('Ajustar al ancho'),
                ),
                PopupMenuItem(
                  value: ZoomMode.fitPage,
                  child: Text('Ajustar a la página'),
                ),
              ],
            ),
            const VerticalDivider(indent: 10, endIndent: 10),
            PopupMenuButton<PageLayoutMode>(
              tooltip: 'Disposición de páginas',
              initialValue: preferences.pageLayout,
              icon: const Icon(Icons.view_carousel_outlined),
              onSelected: controller.setPageLayout,
              itemBuilder: (_) => const [
                PopupMenuItem(
                  value: PageLayoutMode.single,
                  child: Text('Una página'),
                ),
                PopupMenuItem(
                  value: PageLayoutMode.facing,
                  child: Text('Dos páginas enfrentadas'),
                ),
                PopupMenuItem(
                  value: PageLayoutMode.continuous,
                  child: Text('Vista continua'),
                ),
              ],
            ),
            IconButton(
              tooltip: 'Tema de lectura',
              onPressed: onChooseTheme,
              icon: const Icon(Icons.tonality_outlined),
            ),
            IconButton(
              tooltip:
                  controller.bookmarks.containsPage(controller.currentPageIndex)
                  ? 'Retirar marcador'
                  : 'Añadir marcador',
              onPressed: controller.toggleBookmark,
              icon: Icon(
                controller.bookmarks.containsPage(controller.currentPageIndex)
                    ? Icons.bookmark_rounded
                    : Icons.bookmark_border_rounded,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
