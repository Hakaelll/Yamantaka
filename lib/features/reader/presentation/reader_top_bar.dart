import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/ui/yama_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
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
    final history = controller.navigationHistory;
    if (history == null) return _buildBar(context);
    return ListenableBuilder(
      listenable: history,
      builder: (context, _) => _buildBar(context),
    );
  }

  Widget _buildBar(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final preferences = controller.preferences;
    final document = controller.document;
    if (preferences == null || document == null) {
      return const SizedBox(height: AppSizes.topBarHeight);
    }
    final colors = context.appColors;
    final veilColor = canvasColor ?? colors.readerBackground;
    return DecoratedBox(
      key: const Key('reader-top-surface'),
      decoration: BoxDecoration(
        color: veilColor.withValues(alpha: flatChrome ? 1 : 0.92),
      ),
      child: SizedBox(
        height: AppSizes.topBarHeight,
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
              Expanded(
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: YamaToolbarGroup(
                    label: strings.get('reader.currentDocument'),
                    flat: flatChrome,
                    children: [
                      YamaIconButton(
                        semanticLabel: strings.get('reader.backToLibrary'),
                        tooltip: strings.get('reader.backToLibrary'),
                        icon: Icons.arrow_back_rounded,
                        onPressed: onBack,
                      ),
                      const SizedBox(width: AppSpacing.space8),
                      ConstrainedBox(
                        constraints: BoxConstraints(
                          maxWidth: compact
                              ? 112
                              : expanded
                              ? 280
                              : 180,
                        ),
                        child: Tooltip(
                          message: document.fileName,
                          child: Text(
                            document.effectiveTitle,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.bodyMedium
                                ?.copyWith(
                                  color: colors.textSecondary,
                                  fontWeight: FontWeight.w500,
                                ),
                          ),
                        ),
                      ),
                      if (!compact && controller.navigationHistory != null) ...[
                        YamaIconButton(
                          semanticLabel: strings.get('command.navigationBack'),
                          tooltip:
                              '${strings.get('command.navigationBack')} (Alt+←)',
                          icon: Icons.undo_rounded,
                          onPressed: controller.navigationHistory!.canGoBack
                              ? controller.historyBack
                              : null,
                        ),
                        YamaIconButton(
                          semanticLabel: strings.get(
                            'command.navigationForward',
                          ),
                          tooltip:
                              '${strings.get('command.navigationForward')} (Alt+→)',
                          icon: Icons.redo_rounded,
                          onPressed: controller.navigationHistory!.canGoForward
                              ? controller.historyForward
                              : null,
                        ),
                      ],
                    ],
                  ),
                ),
              ),
              Expanded(
                child: Align(
                  alignment: Alignment.centerRight,
                  child: YamaToolbarGroup(
                    label: strings.get('reader.tools'),
                    flat: flatChrome,
                    children: [
                      YamaIconButton(
                        semanticLabel: strings.get('command.search'),
                        tooltip:
                            '${strings.get('command.search')} (${AppCommandRegistry.shortcutLabel(AppCommandId.search, shortcutOverrides, strings: strings)})',
                        icon: Icons.search_rounded,
                        onPressed: onSearch,
                        selected: activePanel == ReaderPanelSection.search,
                      ),
                      if (expanded && showPanelControls)
                        YamaIconButton(
                          semanticLabel: strings.get('reader.openThumbnails'),
                          tooltip:
                              '${strings.get('reader.thumbnails')} (${AppCommandRegistry.shortcutLabel(AppCommandId.togglePanel, shortcutOverrides, strings: strings)})',
                          icon: Icons.grid_view_outlined,
                          selected:
                              activePanel == ReaderPanelSection.thumbnails,
                          onPressed: onShowThumbnails,
                        ),
                      if (showPanelControls)
                        PopupMenuButton<_ReaderToolsAction>(
                          tooltip: strings.get('reader.tools'),
                          onSelected: _runToolsAction,
                          itemBuilder: (context) => [
                            PopupMenuItem(
                              value: _ReaderToolsAction.thumbnails,
                              child: _ReaderMenuItem(
                                icon: Icons.grid_view_outlined,
                                label: strings.get('reader.thumbnails'),
                                shortcut: AppCommandRegistry.shortcutLabel(
                                  AppCommandId.togglePanel,
                                  shortcutOverrides,
                                  strings: strings,
                                ),
                                selected:
                                    activePanel ==
                                    ReaderPanelSection.thumbnails,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderToolsAction.outline,
                              child: _ReaderMenuItem(
                                icon: Icons.format_list_bulleted_rounded,
                                label: strings.get('reader.outline'),
                                selected:
                                    activePanel == ReaderPanelSection.outline,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderToolsAction.bookmarks,
                              child: _ReaderMenuItem(
                                icon: Icons.bookmarks_outlined,
                                label: strings.get('reader.bookmarks'),
                                selected:
                                    activePanel == ReaderPanelSection.bookmarks,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderToolsAction.annotations,
                              child: _ReaderMenuItem(
                                icon: Icons.rate_review_outlined,
                                label: strings.get('reader.annotations'),
                                selected:
                                    activePanel ==
                                    ReaderPanelSection.annotations,
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
                          semanticLabel: strings.get('command.commandPalette'),
                          tooltip:
                              '${strings.get('command.commandPalette')} (${AppCommandRegistry.shortcutLabel(AppCommandId.commandPalette, shortcutOverrides, strings: strings)})',
                          icon: Icons.keyboard_command_key_rounded,
                          onPressed: onCommandPalette,
                        ),
                      PopupMenuButton<_ReaderMenuAction>(
                        tooltip: strings.get('reader.moreOptions'),
                        onSelected: _runAction,
                        constraints: const BoxConstraints(
                          minWidth: 252,
                          maxWidth: 292,
                        ),
                        itemBuilder: (context) => [
                          PopupMenuItem(
                            value: _ReaderMenuAction.openDocument,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.file_open_outlined,
                              label: strings.get('reader.openAnotherPdf'),
                              shortcut: AppCommandRegistry.shortcutLabel(
                                AppCommandId.openDocument,
                                shortcutOverrides,
                                strings: strings,
                              ),
                            ),
                          ),
                          const PopupMenuDivider(height: 9),
                          if (preferences.pageLayout ==
                              PageLayoutMode.facing) ...[
                            CheckedPopupMenuItem(
                              value: _ReaderMenuAction.coverAlone,
                              checked:
                                  preferences.coverPolicy ==
                                  ReaderCoverPolicy.firstPageIsCover,
                              child: Text(strings.get('reader.coverAlone')),
                            ),
                            CheckedPopupMenuItem(
                              value: _ReaderMenuAction.firstPageRight,
                              checked:
                                  preferences.initialSide ==
                                  ReaderInitialSide.right,
                              child: Text(strings.get('reader.firstPageRight')),
                            ),
                            PopupMenuItem(
                              value: _ReaderMenuAction.resetPairing,
                              child: Text(strings.get('reader.resetPairing')),
                            ),
                            const PopupMenuDivider(height: 9),
                          ],
                          if (compact) ...[
                            PopupMenuItem(
                              value: _ReaderMenuAction.commandPalette,
                              height: 40,
                              child: _ReaderMenuItem(
                                icon: Icons.keyboard_command_key_rounded,
                                label: strings.get('command.commandPalette'),
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
                                label: strings.get('reader.thumbnails'),
                                selected:
                                    activePanel ==
                                    ReaderPanelSection.thumbnails,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderMenuAction.outline,
                              height: 40,
                              child: _ReaderMenuItem(
                                icon: Icons.format_list_bulleted_rounded,
                                label: strings.get('reader.outline'),
                                selected:
                                    activePanel == ReaderPanelSection.outline,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderMenuAction.bookmarks,
                              height: 40,
                              child: _ReaderMenuItem(
                                icon: Icons.bookmarks_outlined,
                                label: strings.get('reader.bookmarks'),
                                selected:
                                    activePanel == ReaderPanelSection.bookmarks,
                              ),
                            ),
                            PopupMenuItem(
                              value: _ReaderMenuAction.annotations,
                              height: 40,
                              child: _ReaderMenuItem(
                                icon: Icons.rate_review_outlined,
                                label: strings.get('reader.annotations'),
                                selected:
                                    activePanel ==
                                    ReaderPanelSection.annotations,
                              ),
                            ),
                            const PopupMenuDivider(height: 9),
                          ],
                          PopupMenuItem(
                            value: _ReaderMenuAction.rotate,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.rotate_right_rounded,
                              label: strings.get('reader.rotatePage'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _ReaderMenuAction.margins,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.width_normal_rounded,
                              label: strings.get('reader.readingMargins'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _ReaderMenuAction.fullscreen,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: controller.isFullScreen
                                  ? Icons.fullscreen_exit_rounded
                                  : Icons.fullscreen_rounded,
                              label: controller.isFullScreen
                                  ? strings.get('reader.exitFullScreen')
                                  : strings.get('command.fullScreen'),
                              shortcut: AppCommandRegistry.shortcutLabel(
                                AppCommandId.fullScreen,
                                shortcutOverrides,
                                strings: strings,
                              ),
                            ),
                          ),
                          PopupMenuItem(
                            value: _ReaderMenuAction.toggleChrome,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.visibility_off_outlined,
                              label: strings.get('reader.hideInterface'),
                              shortcut: AppCommandRegistry.shortcutLabel(
                                AppCommandId.toggleChrome,
                                shortcutOverrides,
                                strings: strings,
                              ),
                            ),
                          ),
                          PopupMenuItem(
                            value: _ReaderMenuAction.toggleImmersive,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.center_focus_strong_rounded,
                              label: strings.get('reader.immersiveReading'),
                              shortcut: AppCommandRegistry.shortcutLabel(
                                AppCommandId.toggleImmersive,
                                shortcutOverrides,
                                strings: strings,
                              ),
                            ),
                          ),
                          const PopupMenuDivider(height: 9),
                          PopupMenuItem(
                            value: _ReaderMenuAction.documentInfo,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.info_outline_rounded,
                              label: strings.get('command.documentInfo'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _ReaderMenuAction.settings,
                            height: 40,
                            child: _ReaderMenuItem(
                              icon: Icons.settings_outlined,
                              label: strings.settingsTitle,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(width: AppSpacing.space4),
                    ],
                  ),
                ),
              ),
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
      case _ReaderMenuAction.coverAlone:
        controller.setCoverPolicy(
          controller.preferences?.coverPolicy ==
                  ReaderCoverPolicy.firstPageIsCover
              ? ReaderCoverPolicy.noCover
              : ReaderCoverPolicy.firstPageIsCover,
        );
      case _ReaderMenuAction.firstPageRight:
        controller.setInitialSide(
          controller.preferences?.initialSide == ReaderInitialSide.right
              ? ReaderInitialSide.left
              : ReaderInitialSide.right,
        );
      case _ReaderMenuAction.resetPairing:
        controller.resetBookPairing();
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

enum _ReaderMenuAction {
  openDocument,
  coverAlone,
  firstPageRight,
  resetPairing,
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
