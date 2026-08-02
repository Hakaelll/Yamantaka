import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/ui/yama_components.dart';
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
    required this.onOpenDocument,
    required this.onSearch,
    required this.onCommandPalette,
    required this.onShowThumbnails,
    required this.onShowOutline,
    required this.onShowBookmarks,
    required this.onShowAnnotations,
    required this.onAdjustMargins,
    required this.onModeChanged,
    required this.onDocumentInfo,
    required this.onToggleFullScreen,
    required this.onToggleChrome,
    required this.onToggleImmersive,
    required this.onOpenSettings,
    this.canvasColor,
    this.flatChrome = false,
    this.shortcutOverrides = '{}',
  });

  final ReaderController controller;
  final bool compact;
  final bool expanded;
  final ReaderPanelSection? activePanel;
  final bool showModeControls;
  final bool showPanelControls;
  final VoidCallback onBack;
  final VoidCallback onOpenDocument;
  final VoidCallback onSearch;
  final VoidCallback onCommandPalette;
  final VoidCallback onShowThumbnails;
  final VoidCallback onShowOutline;
  final VoidCallback onShowBookmarks;
  final VoidCallback onShowAnnotations;
  final VoidCallback onAdjustMargins;
  final ValueChanged<ReaderMode> onModeChanged;
  final VoidCallback onDocumentInfo;
  final VoidCallback onToggleFullScreen;
  final VoidCallback onToggleChrome;
  final VoidCallback onToggleImmersive;
  final VoidCallback onOpenSettings;
  final Color? canvasColor;
  final bool flatChrome;
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
    final colors = context.appColors;
    final veilColor = canvasColor ?? colors.readerBackground;
    final flat = flatChrome || MediaQuery.highContrastOf(context);
    return DecoratedBox(
      decoration: flat
          ? BoxDecoration(color: veilColor)
          : BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
                colors: [
                  veilColor.withValues(alpha: 0.88),
                  veilColor.withValues(alpha: 0.34),
                  Colors.transparent,
                ],
                stops: const [0, 0.58, 1],
              ),
            ),
      child: SizedBox(
        height: AppSizes.topVeilDepth,
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: compact
                ? AppSpacing.space12
                : expanded
                ? AppSpacing.space24
                : AppSpacing.space20,
          ),
          child: Row(
          children: [
            YamaIconButton(
              semanticLabel: 'Volver a la biblioteca',
              tooltip: 'Volver a la biblioteca',
              icon: Icons.arrow_back_rounded,
              onPressed: onBack,
            ),
            const SizedBox(width: AppSpacing.space8),
            Flexible(
              child: Tooltip(
                message: document.fileName,
                child: Text(
                  document.effectiveTitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: colors.textSecondary,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
            ),
            if (!compact && controller.navigationHistory != null) ...[
              YamaIconButton(
                semanticLabel: 'Volver a la ubicación anterior',
                tooltip: 'Volver a la ubicación anterior (Alt+←)',
                icon: Icons.undo_rounded,
                onPressed: controller.navigationHistory!.canGoBack
                    ? controller.historyBack
                    : null,
              ),
              YamaIconButton(
                semanticLabel: 'Avanzar en el historial',
                tooltip: 'Avanzar en el historial (Alt+→)',
                icon: Icons.redo_rounded,
                onPressed: controller.navigationHistory!.canGoForward
                    ? controller.historyForward
                    : null,
              ),
            ],
            if (!compact && showModeControls) ...[
              const SizedBox(width: AppSpacing.space12),
              _ReaderModeSelector(
                selected: preferences.viewMode,
                onSelected: onModeChanged,
              ),
              const SizedBox(width: AppSpacing.space8),
            ],
            YamaIconButton(
              semanticLabel: 'Buscar en el documento',
              tooltip:
                  'Buscar (${AppCommandRegistry.shortcutLabel(AppCommandId.search, shortcutOverrides)})',
              icon: Icons.search_rounded,
              onPressed: onSearch,
              selected: activePanel == ReaderPanelSection.search,
            ),
            if (expanded && showPanelControls)
              YamaIconButton(
                semanticLabel: 'Abrir miniaturas',
                tooltip:
                    'Miniaturas (${AppCommandRegistry.shortcutLabel(AppCommandId.togglePanel, shortcutOverrides)})',
                icon: Icons.grid_view_outlined,
                selected: activePanel == ReaderPanelSection.thumbnails,
                onPressed: onShowThumbnails,
              ),
            if (showPanelControls)
              PopupMenuButton<_ReaderToolsAction>(
                tooltip: 'Herramientas',
                onSelected: _runToolsAction,
                itemBuilder: (context) => [
                  PopupMenuItem(
                    value: _ReaderToolsAction.thumbnails,
                    child: _ReaderMenuItem(
                      icon: Icons.grid_view_outlined,
                      label: 'Miniaturas',
                      shortcut: AppCommandRegistry.shortcutLabel(
                        AppCommandId.togglePanel,
                        shortcutOverrides,
                      ),
                      selected: activePanel == ReaderPanelSection.thumbnails,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderToolsAction.outline,
                    child: _ReaderMenuItem(
                      icon: Icons.format_list_bulleted_rounded,
                      label: 'Índice',
                      selected: activePanel == ReaderPanelSection.outline,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderToolsAction.bookmarks,
                    child: _ReaderMenuItem(
                      icon: Icons.bookmarks_outlined,
                      label: 'Marcadores',
                      selected: activePanel == ReaderPanelSection.bookmarks,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderToolsAction.annotations,
                    child: _ReaderMenuItem(
                      icon: Icons.rate_review_outlined,
                      label: 'Anotaciones',
                      selected: activePanel == ReaderPanelSection.annotations,
                    ),
                  ),
                ],
                icon: Icon(
                  Icons.build_outlined,
                  color: activePanel == null
                      ? colors.iconNormal
                      : colors.iconActive,
                ),
              ),
            if (!compact && expanded)
              YamaIconButton(
                semanticLabel: 'Abrir paleta de comandos',
                tooltip:
                    'Paleta de comandos (${AppCommandRegistry.shortcutLabel(AppCommandId.commandPalette, shortcutOverrides)})',
                icon: Icons.keyboard_command_key_rounded,
                onPressed: onCommandPalette,
              ),
            PopupMenuButton<_ReaderMenuAction>(
              tooltip: 'Más opciones',
              onSelected: _runAction,
              constraints: const BoxConstraints(minWidth: 252, maxWidth: 292),
              itemBuilder: (context) => [
                PopupMenuItem(
                  value: _ReaderMenuAction.openDocument,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.file_open_outlined,
                    label: 'Abrir otro PDF…',
                    shortcut: AppCommandRegistry.shortcutLabel(
                      AppCommandId.openDocument,
                      shortcutOverrides,
                    ),
                  ),
                ),
                const PopupMenuDivider(height: 9),
                if (compact || !showModeControls) ...[
                  PopupMenuItem(
                    value: _ReaderMenuAction.documentMode,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.description_outlined,
                      label: 'Modo documento',
                      selected: preferences.viewMode == ReaderMode.document,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderMenuAction.bookMode,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.menu_book_outlined,
                      label: 'Modo libro',
                      selected: preferences.viewMode == ReaderMode.book,
                    ),
                  ),
                  const PopupMenuDivider(height: 9),
                ],
                if (compact) ...[
                  const PopupMenuItem(
                    value: _ReaderMenuAction.commandPalette,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.keyboard_command_key_rounded,
                      label: 'Paleta de comandos',
                    ),
                  ),
                  const PopupMenuDivider(height: 9),
                ],
                if (!expanded || !showPanelControls) ...[
                  PopupMenuItem(
                    value: _ReaderMenuAction.thumbnails,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.grid_view_outlined,
                      label: 'Miniaturas',
                      selected: activePanel == ReaderPanelSection.thumbnails,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderMenuAction.outline,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.format_list_bulleted_rounded,
                      label: 'Índice',
                      selected: activePanel == ReaderPanelSection.outline,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderMenuAction.bookmarks,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.bookmarks_outlined,
                      label: 'Marcadores',
                      selected: activePanel == ReaderPanelSection.bookmarks,
                    ),
                  ),
                  PopupMenuItem(
                    value: _ReaderMenuAction.annotations,
                    height: 40,
                    child: _ReaderMenuItem(
                      icon: Icons.rate_review_outlined,
                      label: 'Anotaciones',
                      selected: activePanel == ReaderPanelSection.annotations,
                    ),
                  ),
                  const PopupMenuDivider(height: 9),
                ],
                const PopupMenuItem(
                  value: _ReaderMenuAction.rotate,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.rotate_right_rounded,
                    label: 'Girar página',
                  ),
                ),
                const PopupMenuItem(
                  value: _ReaderMenuAction.margins,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.width_normal_rounded,
                    label: 'Márgenes de lectura',
                  ),
                ),
                PopupMenuItem(
                  value: _ReaderMenuAction.fullscreen,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.fullscreen_rounded,
                    label: 'Pantalla completa',
                    shortcut: AppCommandRegistry.shortcutLabel(
                      AppCommandId.fullScreen,
                      shortcutOverrides,
                    ),
                  ),
                ),
                PopupMenuItem(
                  value: _ReaderMenuAction.toggleChrome,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.visibility_off_outlined,
                    label: 'Ocultar interfaz',
                    shortcut: AppCommandRegistry.shortcutLabel(
                      AppCommandId.toggleChrome,
                      shortcutOverrides,
                    ),
                  ),
                ),
                PopupMenuItem(
                  value: _ReaderMenuAction.toggleImmersive,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.center_focus_strong_rounded,
                    label: 'Lectura inmersiva',
                    shortcut: AppCommandRegistry.shortcutLabel(
                      AppCommandId.toggleImmersive,
                      shortcutOverrides,
                    ),
                  ),
                ),
                const PopupMenuDivider(height: 9),
                const PopupMenuItem(
                  value: _ReaderMenuAction.documentInfo,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.info_outline_rounded,
                    label: 'Información del documento',
                  ),
                ),
                const PopupMenuItem(
                  value: _ReaderMenuAction.settings,
                  height: 40,
                  child: _ReaderMenuItem(
                    icon: Icons.settings_outlined,
                    label: 'Ajustes',
                  ),
                ),
              ],
            ),
            const SizedBox(width: AppSpacing.space4),
          ],
          ),
        ),
      ),
    );
  }

  void _runToolsAction(_ReaderToolsAction action) {
    switch (action) {
      case _ReaderToolsAction.thumbnails:
        onShowThumbnails();
      case _ReaderToolsAction.outline:
        onShowOutline();
      case _ReaderToolsAction.bookmarks:
        onShowBookmarks();
      case _ReaderToolsAction.annotations:
        onShowAnnotations();
    }
  }

  void _runAction(_ReaderMenuAction action) {
    switch (action) {
      case _ReaderMenuAction.openDocument:
        onOpenDocument();
      case _ReaderMenuAction.documentMode:
        onModeChanged(ReaderMode.document);
      case _ReaderMenuAction.bookMode:
        onModeChanged(ReaderMode.book);
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
        onToggleFullScreen();
      case _ReaderMenuAction.toggleChrome:
        onToggleChrome();
      case _ReaderMenuAction.toggleImmersive:
        onToggleImmersive();
      case _ReaderMenuAction.documentInfo:
        onDocumentInfo();
      case _ReaderMenuAction.settings:
        onOpenSettings();
    }
  }
}

final class _ReaderModeSelector extends StatelessWidget {
  const _ReaderModeSelector({required this.selected, required this.onSelected});

  final ReaderMode selected;
  final ValueChanged<ReaderMode> onSelected;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: 'Modo de visualización',
      explicitChildNodes: true,
      child: Container(
        key: const Key('reader-mode-selector'),
        padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space4),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _ReaderModeSegment(
              mode: ReaderMode.document,
              label: 'Documento',
              tooltip: 'Documento: mostrar una página',
              icon: Icons.description_outlined,
              selected: selected == ReaderMode.document,
              onTap: onSelected,
            ),
            _ReaderModeSegment(
              mode: ReaderMode.book,
              label: 'Libro',
              tooltip: 'Libro: mostrar páginas enfrentadas',
              icon: Icons.menu_book_outlined,
              selected: selected == ReaderMode.book,
              onTap: onSelected,
            ),
          ],
        ),
      ),
    );
  }
}

final class _ReaderModeSegment extends StatelessWidget {
  const _ReaderModeSegment({
    required this.mode,
    required this.label,
    required this.tooltip,
    required this.icon,
    required this.selected,
    required this.onTap,
  });

  final ReaderMode mode;
  final String label;
  final String tooltip;
  final IconData icon;
  final bool selected;
  final ValueChanged<ReaderMode> onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final motion = MediaQuery.disableAnimationsOf(context)
        ? Duration.zero
        : AppMotion.fast;
    final foreground = selected ? colors.textPrimary : colors.textSecondary;
    return Semantics(
      button: true,
      selected: selected,
      label: label,
      child: Tooltip(
        message: tooltip,
        child: InkWell(
          onTap: () => onTap(mode),
          hoverColor: colors.accentHover.withValues(alpha: 0.08),
          focusColor: colors.focusRing.withValues(alpha: 0.12),
          child: AnimatedContainer(
            duration: motion,
            curve: Curves.easeOutCubic,
            constraints: const BoxConstraints(minHeight: 32),
            padding: const EdgeInsets.symmetric(
              horizontal: AppSpacing.space12,
              vertical: AppSpacing.space6,
            ),
            decoration: BoxDecoration(
              color: selected ? colors.surfaceElevated : Colors.transparent,
              border: Border(
                bottom: BorderSide(
                  color: selected ? colors.textPrimary : Colors.transparent,
                  width: 2,
                ),
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(icon, size: 16, color: foreground),
                const SizedBox(width: AppSpacing.space6),
                AnimatedDefaultTextStyle(
                  duration: motion,
                  curve: Curves.easeOutCubic,
                  style:
                      Theme.of(context).textTheme.labelMedium?.copyWith(
                        color: foreground,
                        fontWeight: selected
                            ? FontWeight.w600
                            : FontWeight.w400,
                      ) ??
                      TextStyle(
                        color: foreground,
                        fontWeight: selected
                            ? FontWeight.w600
                            : FontWeight.w400,
                      ),
                  child: Text(label),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

enum _ReaderMenuAction {
  openDocument,
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
  toggleImmersive,
  documentInfo,
  settings,
}

enum _ReaderToolsAction { thumbnails, outline, bookmarks, annotations }

final class _ReaderMenuItem extends StatelessWidget {
  const _ReaderMenuItem({
    required this.icon,
    required this.label,
    this.shortcut,
    this.selected = false,
  });

  final IconData icon;
  final String label;
  final String? shortcut;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    return Semantics(
      selected: selected,
      child: Row(
        children: [
          Icon(
            icon,
            size: 18,
            color: selected ? colors.accentPrimary : colors.textSecondary,
          ),
          const SizedBox(width: AppSpacing.space12),
          Expanded(
            child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis),
          ),
          if (shortcut case final value?)
            Text(
              value,
              style: Theme.of(
                context,
              ).textTheme.labelSmall?.copyWith(color: colors.textMuted),
            )
          else if (selected)
            Icon(Icons.check_rounded, size: 16, color: colors.accentPrimary),
        ],
      ),
    );
  }
}
