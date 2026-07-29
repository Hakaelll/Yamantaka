import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';
import 'reader_side_panel.dart';

final class ReaderTopBar extends StatelessWidget
    implements PreferredSizeWidget {
  const ReaderTopBar({
    super.key,
    required this.controller,
    required this.compact,
    required this.expanded,
    required this.activePanel,
    required this.showModeControls,
    required this.showPanelControls,
    required this.onBack,
    required this.onSearch,
    required this.onCommandPalette,
    required this.onShowThumbnails,
    required this.onShowOutline,
    required this.onShowBookmarks,
    required this.onShowAnnotations,
    required this.onAdjustMargins,
    required this.onDocumentInfo,
    required this.onToggleChrome,
    required this.onOpenSettings,
    this.shortcutOverrides = '{}',
  });

  final ReaderController controller;
  final bool compact;
  final bool expanded;
  final ReaderPanelSection? activePanel;
  final bool showModeControls;
  final bool showPanelControls;
  final VoidCallback onBack;
  final VoidCallback onSearch;
  final VoidCallback onCommandPalette;
  final VoidCallback onShowThumbnails;
  final VoidCallback onShowOutline;
  final VoidCallback onShowBookmarks;
  final VoidCallback onShowAnnotations;
  final VoidCallback onAdjustMargins;
  final VoidCallback onDocumentInfo;
  final VoidCallback onToggleChrome;
  final VoidCallback onOpenSettings;
  final String shortcutOverrides;

  @override
  Size get preferredSize => const Size.fromHeight(AppSizes.topBarHeight);

  @override
  Widget build(BuildContext context) {
    final preferences = controller.preferences;
    final document = controller.document;
    if (preferences == null || document == null) {
      return const SizedBox(height: AppSizes.topBarHeight);
    }
    return Material(
      color: context.appColors.surfacePrimary,
      child: Container(
        height: AppSizes.topBarHeight,
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(color: context.appColors.borderSubtle),
          ),
        ),
        child: Row(
          children: [
            const SizedBox(width: AppSpacing.space4),
            IconButton(
              tooltip: 'Volver a la biblioteca',
              onPressed: onBack,
              icon: const Icon(Icons.arrow_back_rounded),
            ),
            const SizedBox(width: AppSpacing.space4),
            Expanded(
              child: Tooltip(
                message: document.fileName,
                child: Text(
                  document.title ?? document.fileName,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
              ),
            ),
            if (!compact && controller.navigationHistory != null) ...[
              IconButton(
                tooltip: 'Volver a la ubicación anterior (Alt+←)',
                onPressed: controller.navigationHistory!.canGoBack
                    ? controller.historyBack
                    : null,
                icon: const Icon(Icons.arrow_back_ios_new_rounded, size: 17),
              ),
              IconButton(
                tooltip: 'Avanzar en el historial (Alt+→)',
                onPressed: controller.navigationHistory!.canGoForward
                    ? controller.historyForward
                    : null,
                icon: const Icon(Icons.arrow_forward_ios_rounded, size: 17),
              ),
            ],
            if (!compact && showModeControls) ...[
              const SizedBox(width: AppSpacing.space12),
              SegmentedButton<ReaderMode>(
                segments: const [
                  ButtonSegment(
                    value: ReaderMode.document,
                    icon: Icon(Icons.description_outlined, size: 17),
                    label: Text('Documento'),
                    tooltip: 'Modo documento',
                  ),
                  ButtonSegment(
                    value: ReaderMode.book,
                    icon: Icon(Icons.menu_book_outlined, size: 17),
                    label: Text('Libro'),
                    tooltip: 'Modo libro',
                  ),
                ],
                selected: {preferences.viewMode},
                showSelectedIcon: false,
                onSelectionChanged: (selection) =>
                    controller.setReaderMode(selection.first),
                style: const ButtonStyle(
                  visualDensity: VisualDensity.compact,
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
              ),
              const SizedBox(width: AppSpacing.space8),
            ],
            IconButton(
              tooltip:
                  'Buscar (${AppCommandRegistry.shortcutLabel(AppCommandId.search, shortcutOverrides)})',
              onPressed: onSearch,
              icon: Icon(
                Icons.search_rounded,
                color: activePanel == ReaderPanelSection.search
                    ? context.appColors.accentPrimary
                    : null,
              ),
            ),
            if (expanded && showPanelControls) ...[
              _PanelIconButton(
                tooltip:
                    'Miniaturas (${AppCommandRegistry.shortcutLabel(AppCommandId.togglePanel, shortcutOverrides)})',
                icon: Icons.grid_view_outlined,
                selected: activePanel == ReaderPanelSection.thumbnails,
                onPressed: onShowThumbnails,
              ),
              _PanelIconButton(
                tooltip: 'Índice',
                icon: Icons.format_list_bulleted_rounded,
                selected: activePanel == ReaderPanelSection.outline,
                onPressed: onShowOutline,
              ),
              _PanelIconButton(
                tooltip: 'Marcadores',
                icon: Icons.bookmarks_outlined,
                selected: activePanel == ReaderPanelSection.bookmarks,
                onPressed: onShowBookmarks,
              ),
              _PanelIconButton(
                tooltip: 'Anotaciones',
                icon: Icons.rate_review_outlined,
                selected: activePanel == ReaderPanelSection.annotations,
                onPressed: onShowAnnotations,
              ),
            ],
            if (!compact)
              IconButton(
                tooltip:
                    'Paleta de comandos (${AppCommandRegistry.shortcutLabel(AppCommandId.commandPalette, shortcutOverrides)})',
                onPressed: onCommandPalette,
                icon: const Icon(Icons.keyboard_command_key_rounded),
              ),
            PopupMenuButton<_ReaderMenuAction>(
              tooltip: 'Más opciones',
              onSelected: _runAction,
              itemBuilder: (context) => [
                if (compact) ...[
                  CheckedPopupMenuItem(
                    value: _ReaderMenuAction.documentMode,
                    checked: preferences.viewMode == ReaderMode.document,
                    child: const Text('Modo documento'),
                  ),
                  CheckedPopupMenuItem(
                    value: _ReaderMenuAction.bookMode,
                    checked: preferences.viewMode == ReaderMode.book,
                    child: const Text('Modo libro'),
                  ),
                  const PopupMenuDivider(),
                  const PopupMenuItem(
                    value: _ReaderMenuAction.commandPalette,
                    child: Text('Paleta de comandos'),
                  ),
                ],
                CheckedPopupMenuItem(
                  value: _ReaderMenuAction.thumbnails,
                  checked: activePanel == ReaderPanelSection.thumbnails,
                  child: const Text('Miniaturas'),
                ),
                CheckedPopupMenuItem(
                  value: _ReaderMenuAction.outline,
                  checked: activePanel == ReaderPanelSection.outline,
                  child: Text('Índice'),
                ),
                CheckedPopupMenuItem(
                  value: _ReaderMenuAction.bookmarks,
                  checked: activePanel == ReaderPanelSection.bookmarks,
                  child: const Text('Marcadores'),
                ),
                CheckedPopupMenuItem(
                  value: _ReaderMenuAction.annotations,
                  checked: activePanel == ReaderPanelSection.annotations,
                  child: const Text('Anotaciones'),
                ),
                const PopupMenuDivider(),
                const PopupMenuItem(
                  value: _ReaderMenuAction.rotate,
                  child: Text('Girar página'),
                ),
                const PopupMenuItem(
                  value: _ReaderMenuAction.margins,
                  child: Text('Márgenes de lectura'),
                ),
                PopupMenuItem(
                  value: _ReaderMenuAction.fullscreen,
                  child: Text(
                    'Pantalla completa '
                    '(${AppCommandRegistry.shortcutLabel(AppCommandId.fullScreen, shortcutOverrides)})',
                  ),
                ),
                PopupMenuItem(
                  value: _ReaderMenuAction.toggleChrome,
                  child: Text(
                    'Ocultar interfaz '
                    '(${AppCommandRegistry.shortcutLabel(AppCommandId.toggleChrome, shortcutOverrides)})',
                  ),
                ),
                const PopupMenuItem(
                  value: _ReaderMenuAction.documentInfo,
                  child: Text('Información del documento'),
                ),
                const PopupMenuItem(
                  value: _ReaderMenuAction.settings,
                  child: Text('Ajustes'),
                ),
              ],
            ),
            const SizedBox(width: AppSpacing.space4),
          ],
        ),
      ),
    );
  }

  void _runAction(_ReaderMenuAction action) {
    switch (action) {
      case _ReaderMenuAction.documentMode:
        controller.setReaderMode(ReaderMode.document);
      case _ReaderMenuAction.bookMode:
        controller.setReaderMode(ReaderMode.book);
      case _ReaderMenuAction.commandPalette:
        onCommandPalette();
      case _ReaderMenuAction.thumbnails:
        onShowThumbnails();
      case _ReaderMenuAction.outline:
        onShowOutline();
      case _ReaderMenuAction.bookmarks:
        onShowBookmarks();
      case _ReaderMenuAction.annotations:
        onShowAnnotations();
      case _ReaderMenuAction.rotate:
        controller.rotateClockwise();
      case _ReaderMenuAction.margins:
        onAdjustMargins();
      case _ReaderMenuAction.fullscreen:
        controller.toggleFullScreen();
      case _ReaderMenuAction.toggleChrome:
        onToggleChrome();
      case _ReaderMenuAction.documentInfo:
        onDocumentInfo();
      case _ReaderMenuAction.settings:
        onOpenSettings();
    }
  }
}

enum _ReaderMenuAction {
  documentMode,
  bookMode,
  commandPalette,
  thumbnails,
  outline,
  bookmarks,
  annotations,
  rotate,
  margins,
  fullscreen,
  toggleChrome,
  documentInfo,
  settings,
}

final class _PanelIconButton extends StatelessWidget {
  const _PanelIconButton({
    required this.tooltip,
    required this.icon,
    required this.selected,
    required this.onPressed,
  });

  final String tooltip;
  final IconData icon;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      selected: selected,
      child: IconButton(
        tooltip: tooltip,
        onPressed: onPressed,
        style: selected
            ? IconButton.styleFrom(
                backgroundColor: context.appColors.selectionBackground,
                foregroundColor: context.appColors.accentPrimary,
              )
            : null,
        icon: Icon(icon),
      ),
    );
  }
}
