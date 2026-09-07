import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/ui/yama_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

/// The bottom chrome is one editorial control family. The parent positions it
/// over the PDF, so resting/active state never reserves a toolbar-height strip
/// or changes viewer geometry.
final class ReaderBottomControls extends StatelessWidget {
  const ReaderBottomControls({
    super.key,
    required this.controller,
    required this.compact,
    required this.onGoToPage,
    required this.onChooseTheme,
    required this.readingDirection,
    required this.active,
    required this.onActivate,
    required this.showProgress,
    this.flatChrome = false,
    this.shortcutOverrides = '{}',
  });

  final ReaderController controller;
  final bool compact;
  final VoidCallback onGoToPage;
  final VoidCallback onChooseTheme;
  final ReadingDirection readingDirection;
  final bool active;
  final VoidCallback onActivate;
  final bool showProgress;
  final bool flatChrome;
  final String shortcutOverrides;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: Listenable.merge([
        controller.viewController,
        controller.bookmarks,
        controller.pageIndexListenable,
      ]),
      builder: (context, _) => _buildControls(context),
    );
  }

  Widget _buildControls(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final preferences = controller.preferences;
    if (preferences == null) return const SizedBox.shrink();
    final facing = preferences.viewMode == ReaderMode.book;
    final previousLabel = strings.get(
      facing ? 'reader.previousSpread' : 'command.previousPage',
    );
    final nextLabel = strings.get(
      facing ? 'reader.nextSpread' : 'command.nextPage',
    );
    final fitWidthLabel = facing
        ? strings.get('reader.fitSpreadWidth')
        : strings.get('command.fitWidth');
    final fitPageLabel = facing
        ? strings.get('reader.fitFullSpread')
        : strings.get('command.fitPage');
    final layoutLabel = strings.get(switch (preferences.pageLayout) {
      PageLayoutMode.single => 'settings.value.singlePage',
      PageLayoutMode.facing => 'settings.value.facingPages',
      PageLayoutMode.continuous => 'settings.value.continuous',
    });
    final flat = flatChrome || MediaQuery.highContrastOf(context);
    if (!active) {
      return _ReaderRestingDock(
        controller: controller,
        compact: compact,
        readingDirection: readingDirection,
        showProgress: showProgress,
        onActivate: onActivate,
      );
    }
    final navigation = Row(
      key: const Key('reader-navigation-group'),
      mainAxisSize: MainAxisSize.min,
      children: [
        YamaIconButton(
          semanticLabel: previousLabel,
          tooltip: _shortcutTooltip(
            previousLabel,
            AppCommandId.previousPage,
            strings,
          ),
          icon: Icons.chevron_left_rounded,
          onPressed: controller.canGoToPreviousReadingUnit
              ? controller.goToPreviousReadingUnit
              : null,
        ),
        Tooltip(
          message: _shortcutTooltip(
            strings.get('command.goToPage'),
            AppCommandId.goToPage,
            strings,
          ),
          child: TextButton(
            key: const Key('reader-page-indicator'),
            onPressed: onGoToPage,
            child: Text(
              controller.readingUnitIndicator(direction: readingDirection),
              style: Theme.of(context).textTheme.labelLarge?.copyWith(
                fontFeatures: AppTypeScale.tabularFigures,
              ),
            ),
          ),
        ),
        YamaIconButton(
          semanticLabel: nextLabel,
          tooltip: _shortcutTooltip(nextLabel, AppCommandId.nextPage, strings),
          icon: Icons.chevron_right_rounded,
          onPressed: controller.canGoToNextReadingUnit
              ? controller.goToNextReadingUnit
              : null,
        ),
      ],
    );
    final zoom = Row(
      key: const Key('reader-view-group'),
      mainAxisSize: MainAxisSize.min,
      children: [
        YamaIconButton(
          semanticLabel: strings.get('command.zoomOut'),
          tooltip: _shortcutTooltip(
            strings.get('command.zoomOut'),
            AppCommandId.zoomOut,
            strings,
          ),
          icon: Icons.remove_rounded,
          onPressed: () => controller.zoomBy(0.85),
        ),
        SizedBox(
          width:
              (compact ? 52 : 64) *
              MediaQuery.textScalerOf(context).scale(1).clamp(1, 2),
          child: Tooltip(
            message: strings.get('reader.customZoom'),
            child: TextButton(
              key: const Key('reader-zoom-indicator'),
              style: TextButton.styleFrom(
                minimumSize: const Size(40, AppSizes.desktopControlExtent),
                padding: EdgeInsets.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              onPressed: () => _showZoomChooser(context),
              child: Text(
                '${(controller.viewController.zoom * 100).round()} %',
                textAlign: TextAlign.center,
                maxLines: 1,
                softWrap: false,
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                  fontFeatures: AppTypeScale.tabularFigures,
                ),
              ),
            ),
          ),
        ),
        YamaIconButton(
          semanticLabel: strings.get('command.zoomIn'),
          tooltip: _shortcutTooltip(
            strings.get('command.zoomIn'),
            AppCommandId.zoomIn,
            strings,
          ),
          icon: Icons.add_rounded,
          onPressed: () => controller.zoomBy(1.18),
        ),
        if (!compact)
          PopupMenuButton<_ZoomAction>(
            tooltip: strings.get('settings.zoomMode'),
            icon: Icon(
              Icons.fit_screen_outlined,
              size: 18,
              color: preferences.zoomMode == ZoomMode.custom
                  ? context.appColors.iconNormal
                  : context.appColors.iconActive,
            ),
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
              PopupMenuItem(
                value: _ZoomAction.actualSize,
                child: AppMenuItemContent(
                  icon: Icons.looks_one_outlined,
                  label: strings.get('command.actualSize'),
                  shortcut: 'Ctrl+1',
                ),
              ),
              PopupMenuItem(
                value: _ZoomAction.fitWidth,
                child: AppMenuItemContent(
                  icon: Icons.fit_screen_outlined,
                  label: fitWidthLabel,
                  shortcut: 'Ctrl+2',
                ),
              ),
              PopupMenuItem(
                value: _ZoomAction.fitPage,
                child: AppMenuItemContent(
                  icon: Icons.crop_free_rounded,
                  label: fitPageLabel,
                  shortcut: 'Ctrl+0',
                ),
              ),
            ],
          ),
        if (!compact) ...[
          YamaIconButton(
            semanticLabel: strings.get('reader.readingTheme'),
            tooltip: strings.get('reader.readingTheme'),
            icon: Icons.tonality_outlined,
            onPressed: onChooseTheme,
          ),
        ] else
          PopupMenuButton<_CompactReaderAction>(
            tooltip: strings.get('reader.moreReadingControls'),
            icon: const Icon(Icons.more_horiz_rounded),
            onSelected: _runCompactAction,
            itemBuilder: (_) => [
              PopupMenuItem(
                value: _CompactReaderAction.actualSize,
                child: Text('${strings.get('command.actualSize')} · Ctrl+1'),
              ),
              PopupMenuItem(
                value: _CompactReaderAction.fitWidth,
                child: Text('$fitWidthLabel · Ctrl+2'),
              ),
              PopupMenuItem(
                value: _CompactReaderAction.fitPage,
                child: Text('$fitPageLabel · Ctrl+0'),
              ),
              PopupMenuItem(
                value: _CompactReaderAction.theme,
                child: Text(strings.get('reader.readingTheme')),
              ),
            ],
          ),
        YamaIconButton(
          semanticLabel:
              controller.bookmarks.containsPage(controller.currentPageIndex)
              ? strings.get('reader.removeBookmark')
              : strings.get('reader.addBookmark'),
          tooltip:
              controller.bookmarks.containsPage(controller.currentPageIndex)
              ? _shortcutTooltip(
                  strings.get('reader.removeBookmark'),
                  AppCommandId.toggleBookmark,
                  strings,
                )
              : _shortcutTooltip(
                  strings.get('reader.addBookmark'),
                  AppCommandId.toggleBookmark,
                  strings,
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

    final availableWidth = MediaQuery.sizeOf(context).width;
    if (!compact &&
        availableWidth >= 900 &&
        MediaQuery.textScalerOf(context).scale(1) <= 1.3) {
      return Align(
        alignment: Alignment.bottomCenter,
        child: SizedBox(
          height: AppSizes.bottomControlHeight,
          child: YamaToolbarGroup(
            key: const Key('reader-active-dock'),
            label:
                '${strings.get('reader.navigation')}. ${strings.get('reader.viewAndZoom')}',
            flat: flat,
            radius: AppRadii.control,
            padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space6),
            children: [
              SizedBox(
                width: (availableWidth - AppSpacing.space32).clamp(0, 760),
                height: AppSizes.desktopControlExtent,
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Align(
                      alignment: Alignment.centerLeft,
                      child: Padding(
                        padding: const EdgeInsets.only(left: AppSpacing.space8),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              Icons.menu_book_outlined,
                              size: 16,
                              color: context.appColors.textSecondary,
                            ),
                            const SizedBox(width: AppSpacing.space6),
                            Text(
                              layoutLabel,
                              style: Theme.of(context).textTheme.labelSmall,
                            ),
                          ],
                        ),
                      ),
                    ),
                    Center(child: navigation),
                    Align(
                      alignment: Alignment.centerRight,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [const YamaToolbarDivider(), zoom],
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
    return Align(
      alignment: Alignment.bottomCenter,
      child: SizedBox(
        height: MediaQuery.textScalerOf(context).scale(1) > 1.3
            ? null
            : AppSizes.bottomControlHeight,
        child: YamaToolbarGroup(
          key: const Key('reader-active-dock'),
          label:
              '${strings.get('reader.navigation')}. ${strings.get('reader.viewAndZoom')}',
          flat: flat,
          radius: AppRadii.control,
          padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space6),
          children: [
            if (MediaQuery.textScalerOf(context).scale(1) > 1.3)
              Flexible(
                child: Wrap(
                  alignment: WrapAlignment.center,
                  spacing: 12,
                  runSpacing: 4,
                  children: [navigation, zoom],
                ),
              )
            else ...[
              navigation,
              const YamaToolbarDivider(),
              zoom,
            ],
          ],
        ),
      ),
    );
  }

  String _shortcutTooltip(
    String label,
    AppCommandId id,
    YamantakaLocalizations strings,
  ) {
    final shortcut = AppCommandRegistry.shortcutLabel(
      id,
      shortcutOverrides,
      strings: strings,
    );
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
    final strings = YamantakaLocalizations.of(context);
    final value = TextEditingController(
      text: '${(controller.viewController.zoom * 100).round()}',
    );
    final result = await showDialog<double>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('reader.zoom')),
        content: TextField(
          controller: value,
          autofocus: true,
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            suffixText: '%',
            labelText: strings.get('reader.scale'),
          ),
          onSubmitted: (text) => Navigator.pop(context, double.tryParse(text)),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () =>
                Navigator.pop(context, double.tryParse(value.text)),
            child: Text(strings.get('action.apply')),
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

final class _ReaderRestingDock extends StatelessWidget {
  const _ReaderRestingDock({
    required this.controller,
    required this.compact,
    required this.readingDirection,
    required this.showProgress,
    required this.onActivate,
  });

  final ReaderController controller;
  final bool compact;
  final ReadingDirection readingDirection;
  final bool showProgress;
  final VoidCallback onActivate;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final progress = controller.pageCount == 0
        ? 0.0
        : ReadingProgress.calculate(
            pageIndex: controller.currentPageIndex,
            pageCount: controller.pageCount,
          );
    final position = controller.readingUnitIndicator(
      direction: readingDirection,
    );
    return Align(
      alignment: Alignment.bottomCenter,
      child: Semantics(
        button: true,
        label: strings.get(
          'reader.expandReadingControls',
          parameters: {'position': position},
        ),
        child: Material(
          color: context.appColors.surfaceFloating,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(AppRadii.field),
            side: BorderSide(color: context.appColors.borderSubtle),
          ),
          child: InkWell(
            key: const Key('reader-resting-dock'),
            borderRadius: BorderRadius.circular(AppRadii.field),
            onTap: onActivate,
            child: SizedBox(
              width: compact ? 236 : 336,
              height: (MediaQuery.textScalerOf(context).scale(14) * 1.3 + 12)
                  .clamp(AppSizes.readerRestingDockHeight, 100),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space12,
                  vertical: AppSpacing.space6,
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (showProgress) ...[
                      Expanded(
                        child: ClipRRect(
                          borderRadius: BorderRadius.circular(
                            AppRadii.indicator,
                          ),
                          child: LinearProgressIndicator(
                            value: progress,
                            minHeight: 3,
                            backgroundColor: context.appColors.borderSubtle,
                          ),
                        ),
                      ),
                      const SizedBox(width: AppSpacing.space12),
                    ],
                    Text(
                      position,
                      style: Theme.of(context).textTheme.labelLarge?.copyWith(
                        fontFeatures: AppTypeScale.tabularFigures,
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    Icon(
                      Icons.keyboard_arrow_up_rounded,
                      size: 18,
                      color: context.appColors.textSecondary,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

enum _ZoomAction { actualSize, fitWidth, fitPage }

enum _CompactReaderAction { actualSize, fitWidth, fitPage, theme }
