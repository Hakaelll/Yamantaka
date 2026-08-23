import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/ui/yama_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

/// The bottom chrome is intentionally two coordinated floating groups.  The
/// parent positions this widget over the PDF, so the viewer never reserves a
/// toolbar-height strip for it.
final class ReaderBottomControls extends StatelessWidget {
  const ReaderBottomControls({
    super.key,
    required this.controller,
    required this.compact,
    required this.onGoToPage,
    required this.onChooseTheme,
    required this.readingDirection,
    this.flatChrome = false,
    this.shortcutOverrides = '{}',
  });

  final ReaderController controller;
  final bool compact;
  final VoidCallback onGoToPage;
  final VoidCallback onChooseTheme;
  final ReadingDirection readingDirection;
  final bool flatChrome;
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
    final facing = preferences.viewMode == ReaderMode.book;
    final previousLabel = facing ? 'Pliego anterior' : 'Página anterior';
    final nextLabel = facing ? 'Pliego siguiente' : 'Página siguiente';
    final fitWidthLabel = facing
        ? 'Ajustar al ancho del pliego'
        : 'Ajustar al ancho';
    final fitPageLabel = facing
        ? 'Ajustar al pliego completo'
        : 'Ajustar a la página';
    final flat = flatChrome || MediaQuery.highContrastOf(context);
    final navigation = YamaToolbarGroup(
      key: const Key('reader-navigation-group'),
      label: 'Navegación de lectura',
      flat: flat,
      children: [
        YamaIconButton(
          semanticLabel: previousLabel,
          tooltip: _shortcutTooltip(previousLabel, AppCommandId.previousPage),
          icon: Icons.chevron_left_rounded,
          onPressed: controller.canGoToPreviousReadingUnit
              ? controller.goToPreviousReadingUnit
              : null,
        ),
        Tooltip(
          message: _shortcutTooltip('Ir a una página', AppCommandId.goToPage),
          child: TextButton(
            key: const Key('reader-page-indicator'),
            onPressed: onGoToPage,
            child: Text(
              controller.readingUnitIndicator(direction: readingDirection),
              style: Theme.of(context).textTheme.labelMedium,
            ),
          ),
        ),
        YamaIconButton(
          semanticLabel: nextLabel,
          tooltip: _shortcutTooltip(nextLabel, AppCommandId.nextPage),
          icon: Icons.chevron_right_rounded,
          onPressed: controller.canGoToNextReadingUnit
              ? controller.goToNextReadingUnit
              : null,
        ),
      ],
    );
    final zoom = YamaToolbarGroup(
      key: const Key('reader-view-group'),
      label: 'Vista y zoom',
      flat: flat,
      children: [
        YamaIconButton(
          semanticLabel: 'Reducir zoom',
          tooltip: _shortcutTooltip('Reducir zoom', AppCommandId.zoomOut),
          icon: Icons.remove_rounded,
          onPressed: () => controller.zoomBy(0.85),
        ),
        SizedBox(
          width: compact ? 52 : 64,
          child: Tooltip(
            message: 'Zoom personalizado',
            child: TextButton(
              key: const Key('reader-zoom-indicator'),
              onPressed: () => _showZoomChooser(context),
              child: Text(
                '${(controller.viewController.zoom * 100).round()} %',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ),
          ),
        ),
        YamaIconButton(
          semanticLabel: 'Aumentar zoom',
          tooltip: _shortcutTooltip('Aumentar zoom', AppCommandId.zoomIn),
          icon: Icons.add_rounded,
          onPressed: () => controller.zoomBy(1.18),
        ),
        PopupMenuButton<_ZoomAction>(
          tooltip: 'Ajuste de zoom',
          icon: const Icon(Icons.fit_screen_outlined, size: 18),
          onSelected: (action) {
            switch (action) {
              case _ZoomAction.actualSize:
                controller.viewController.actualSize();
              case _ZoomAction.fitWidth:
                controller.fitWidth();
              case _ZoomAction.fitPage:
                controller.fitPage();
            }
          },
          itemBuilder: (_) => [
            const PopupMenuItem(
              value: _ZoomAction.actualSize,
              child: Text('Tamaño real · Ctrl+1'),
            ),
            PopupMenuItem(
              value: _ZoomAction.fitWidth,
              child: Text('$fitWidthLabel · Ctrl+2'),
            ),
            PopupMenuItem(
              value: _ZoomAction.fitPage,
              child: Text('$fitPageLabel · Ctrl+0'),
            ),
          ],
        ),
        if (!compact) ...[
          YamaIconButton(
            semanticLabel: 'Tema de lectura',
            tooltip: 'Tema de lectura',
            icon: Icons.tonality_outlined,
            onPressed: onChooseTheme,
          ),
        ] else
          PopupMenuButton<_CompactReaderAction>(
            tooltip: 'Más controles de lectura',
            icon: const Icon(Icons.more_horiz_rounded),
            onSelected: _runCompactAction,
            itemBuilder: (_) => [
              const PopupMenuItem(
                value: _CompactReaderAction.actualSize,
                child: Text('Tamaño real · Ctrl+1'),
              ),
              PopupMenuItem(
                value: _CompactReaderAction.fitWidth,
                child: Text('$fitWidthLabel · Ctrl+2'),
              ),
              PopupMenuItem(
                value: _CompactReaderAction.fitPage,
                child: Text('$fitPageLabel · Ctrl+0'),
              ),
              const PopupMenuItem(
                value: _CompactReaderAction.theme,
                child: Text('Tema de lectura'),
              ),
            ],
          ),
        YamaIconButton(
          semanticLabel:
              controller.bookmarks.containsPage(controller.currentPageIndex)
              ? 'Retirar marcador'
              : 'Añadir marcador',
          tooltip:
              controller.bookmarks.containsPage(controller.currentPageIndex)
              ? _shortcutTooltip(
                  'Retirar marcador',
                  AppCommandId.toggleBookmark,
                )
              : _shortcutTooltip(
                  'Añadir marcador',
                  AppCommandId.toggleBookmark,
                ),
          icon: controller.bookmarks.containsPage(controller.currentPageIndex)
              ? Icons.bookmark_rounded
              : Icons.bookmark_border_rounded,
          selected: controller.bookmarks.containsPage(
            controller.currentPageIndex,
          ),
          onPressed: controller.toggleBookmark,
        ),
      ],
    );

    return LayoutBuilder(
      builder: (context, constraints) {
        if (compact || constraints.maxWidth < 900) {
          return Align(
            alignment: Alignment.bottomCenter,
            child: Wrap(
              alignment: WrapAlignment.center,
              runAlignment: WrapAlignment.center,
              spacing: AppSpacing.space12,
              runSpacing: AppSpacing.space8,
              children: [navigation, zoom],
            ),
          );
        }
        return SizedBox(
          height: AppSizes.bottomControlHeight + AppSpacing.space8,
          child: Stack(
            children: [
              Align(alignment: Alignment.bottomCenter, child: navigation),
              Align(alignment: Alignment.bottomRight, child: zoom),
            ],
          ),
        );
      },
    );
  }

  String _shortcutTooltip(String label, AppCommandId id) {
    final shortcut = AppCommandRegistry.shortcutLabel(id, shortcutOverrides);
    return shortcut.isEmpty ? label : '$label ($shortcut)';
  }

  void _runCompactAction(_CompactReaderAction action) {
    switch (action) {
      case _CompactReaderAction.actualSize:
        controller.viewController.actualSize();
      case _CompactReaderAction.fitWidth:
        controller.fitWidth();
      case _CompactReaderAction.fitPage:
        controller.fitPage();
      case _CompactReaderAction.theme:
        onChooseTheme();
    }
  }

  Future<void> _showZoomChooser(BuildContext context) async {
    final value = TextEditingController(
      text: '${(controller.viewController.zoom * 100).round()}',
    );
    final result = await showDialog<double>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Zoom'),
        content: TextField(
          controller: value,
          autofocus: true,
          keyboardType: TextInputType.number,
          decoration: const InputDecoration(
            suffixText: '%',
            labelText: 'Escala',
          ),
          onSubmitted: (text) => Navigator.pop(context, double.tryParse(text)),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () =>
                Navigator.pop(context, double.tryParse(value.text)),
            child: const Text('Aplicar'),
          ),
        ],
      ),
    );
    value.dispose();
    if (result != null) {
      final zoom = (result / 100).clamp(0.1, 8.0).toDouble();
      final current = controller.viewController.zoom;
      if (current > 0) await controller.zoomBy(zoom / current);
    }
  }
}

enum _ZoomAction { actualSize, fitWidth, fitPage }

enum _CompactReaderAction { actualSize, fitWidth, fitPage, theme }
