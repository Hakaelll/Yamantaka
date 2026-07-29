import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

final class ReaderBottomControls extends StatelessWidget {
  const ReaderBottomControls({
    super.key,
    required this.controller,
    required this.compact,
    required this.onGoToPage,
    required this.onChooseTheme,
    this.shortcutOverrides = '{}',
  });

  final ReaderController controller;
  final bool compact;
  final VoidCallback onGoToPage;
  final VoidCallback onChooseTheme;
  final String shortcutOverrides;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: Listenable.merge([
        controller.viewController,
        controller.bookmarks,
      ]),
      builder: (context, _) => _buildControls(context),
    );
  }

  Widget _buildControls(BuildContext context) {
    final preferences = controller.preferences;
    if (preferences == null) return const SizedBox.shrink();
    return Center(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: compact ? 520 : 680),
        child: Material(
          color: context.appColors.surfaceElevated,
          elevation: 3,
          shadowColor: Colors.black.withValues(alpha: 0.14),
          shape: RoundedRectangleBorder(
            side: BorderSide(color: context.appColors.borderSubtle),
            borderRadius: BorderRadius.circular(AppRadii.field),
          ),
          clipBehavior: Clip.antiAlias,
          child: SizedBox(
            height: AppSizes.bottomControlHeight,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const SizedBox(width: AppSpacing.space4),
                IconButton(
                  tooltip: _shortcutTooltip(
                    'Página anterior',
                    AppCommandId.previousPage,
                  ),
                  onPressed: controller.canGoBack
                      ? controller.previousPage
                      : null,
                  icon: const Icon(Icons.chevron_left_rounded),
                ),
                Tooltip(
                  message: _shortcutTooltip(
                    'Ir a una página',
                    AppCommandId.goToPage,
                  ),
                  child: TextButton(
                    key: const Key('reader-page-indicator'),
                    onPressed: onGoToPage,
                    child: Text(
                      '${controller.currentPageIndex + 1} / '
                      '${controller.pageCount}',
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ),
                ),
                IconButton(
                  tooltip: _shortcutTooltip(
                    'Página siguiente',
                    AppCommandId.nextPage,
                  ),
                  onPressed: controller.canGoForward
                      ? controller.nextPage
                      : null,
                  icon: const Icon(Icons.chevron_right_rounded),
                ),
                const _ControlDivider(),
                IconButton(
                  tooltip: _shortcutTooltip(
                    'Reducir zoom',
                    AppCommandId.zoomOut,
                  ),
                  onPressed: () => controller.zoomBy(0.85),
                  icon: const Icon(Icons.remove_rounded),
                ),
                if (!compact)
                  SizedBox(
                    width: 48,
                    child: Text(
                      '${(controller.viewController.zoom * 100).round()} %',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ),
                IconButton(
                  tooltip: _shortcutTooltip(
                    'Aumentar zoom',
                    AppCommandId.zoomIn,
                  ),
                  onPressed: () => controller.zoomBy(1.18),
                  icon: const Icon(Icons.add_rounded),
                ),
                if (!compact) ...[
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
                  const _ControlDivider(),
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
                ] else
                  PopupMenuButton<_CompactReaderAction>(
                    tooltip: 'Más controles de lectura',
                    icon: const Icon(Icons.more_horiz_rounded),
                    onSelected: (action) =>
                        _runCompactAction(action, controller),
                    itemBuilder: (_) => [
                      const PopupMenuItem(
                        value: _CompactReaderAction.fitWidth,
                        child: Text('Ajustar al ancho'),
                      ),
                      const PopupMenuItem(
                        value: _CompactReaderAction.fitPage,
                        child: Text('Ajustar a la página'),
                      ),
                      const PopupMenuDivider(),
                      CheckedPopupMenuItem(
                        value: _CompactReaderAction.single,
                        checked:
                            preferences.pageLayout == PageLayoutMode.single,
                        child: const Text('Una página'),
                      ),
                      CheckedPopupMenuItem(
                        value: _CompactReaderAction.continuous,
                        checked:
                            preferences.pageLayout == PageLayoutMode.continuous,
                        child: const Text('Vista continua'),
                      ),
                      const PopupMenuItem(
                        value: _CompactReaderAction.theme,
                        child: Text('Tema de lectura'),
                      ),
                    ],
                  ),
                IconButton(
                  tooltip:
                      controller.bookmarks.containsPage(
                        controller.currentPageIndex,
                      )
                      ? _shortcutTooltip(
                          'Retirar marcador',
                          AppCommandId.toggleBookmark,
                        )
                      : _shortcutTooltip(
                          'Añadir marcador',
                          AppCommandId.toggleBookmark,
                        ),
                  onPressed: controller.toggleBookmark,
                  icon: Icon(
                    controller.bookmarks.containsPage(
                          controller.currentPageIndex,
                        )
                        ? Icons.bookmark_rounded
                        : Icons.bookmark_border_rounded,
                  ),
                ),
                const SizedBox(width: AppSpacing.space4),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _shortcutTooltip(String label, AppCommandId id) {
    final shortcut = AppCommandRegistry.shortcutLabel(id, shortcutOverrides);
    return shortcut.isEmpty ? label : '$label ($shortcut)';
  }

  void _runCompactAction(
    _CompactReaderAction action,
    ReaderController controller,
  ) {
    switch (action) {
      case _CompactReaderAction.fitWidth:
        controller.fitWidth();
      case _CompactReaderAction.fitPage:
        controller.fitPage();
      case _CompactReaderAction.single:
        controller.setPageLayout(PageLayoutMode.single);
      case _CompactReaderAction.continuous:
        controller.setPageLayout(PageLayoutMode.continuous);
      case _CompactReaderAction.theme:
        onChooseTheme();
    }
  }
}

final class _ControlDivider extends StatelessWidget {
  const _ControlDivider();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 1,
      height: 20,
      margin: const EdgeInsets.symmetric(horizontal: AppSpacing.space4),
      color: context.appColors.borderSubtle,
    );
  }
}

enum _CompactReaderAction { fitWidth, fitPage, single, continuous, theme }
