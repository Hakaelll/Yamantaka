import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../application/global_search_controller.dart';
import '../application/library_index_controller.dart';
import '../domain/global_search_models.dart';

final class GlobalSearchDialog extends StatefulWidget {
  const GlobalSearchDialog({
    super.key,
    required this.controller,
    required this.indexController,
    required this.onOpen,
  });

  final GlobalLibrarySearchController controller;
  final LibraryIndexController indexController;
  final ValueChanged<GlobalSearchResult> onOpen;

  @override
  State<GlobalSearchDialog> createState() => _GlobalSearchDialogState();
}

final class _GlobalSearchDialogState extends State<GlobalSearchDialog> {
  late final TextEditingController _queryController = TextEditingController(
    text: widget.controller.query,
  );
  int _selectedIndex = 0;

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Dialog(
      insetPadding: const EdgeInsets.all(AppSpacing.space16),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 980, maxHeight: 680),
        child: ListenableBuilder(
          listenable: Listenable.merge([
            widget.controller,
            widget.indexController,
          ]),
          builder: (context, _) {
            final progress = widget.indexController.progress;
            return Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    AppSpacing.space16,
                    AppSpacing.space8,
                    AppSpacing.space12,
                  ),
                  child: AppDialogHeader(
                    icon: Icons.manage_search_rounded,
                    title: strings.searchTitle,
                    description: strings.get('search.localDescription'),
                    trailing: IconButton(
                      tooltip: strings.get('action.close'),
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(Icons.close_rounded),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppSpacing.space16,
                  ),
                  child: TextField(
                    key: const Key('global-search-field'),
                    controller: _queryController,
                    autofocus: true,
                    textInputAction: TextInputAction.search,
                    decoration: InputDecoration(
                      prefixIcon: const Icon(Icons.manage_search_rounded),
                      labelText: strings.searchField,
                      suffixIcon: _queryController.text.isEmpty
                          ? null
                          : IconButton(
                              tooltip: strings.get('action.clear'),
                              onPressed: () {
                                _queryController.clear();
                                widget.controller.clear();
                              },
                              icon: const Icon(Icons.clear_rounded),
                            ),
                    ),
                    onChanged: (value) {
                      setState(() {});
                      widget.controller.search(value);
                    },
                    onSubmitted: widget.controller.search,
                  ),
                ),
                const SizedBox(height: AppSpacing.space8),
                _IndexStatusBar(
                  controller: widget.indexController,
                  progress: progress,
                ),
                const Divider(height: 1),
                Expanded(
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      final canPreview =
                          constraints.maxWidth >= 820 &&
                          widget.controller.results.isNotEmpty &&
                          widget.controller.query.isNotEmpty;
                      if (!canPreview) return _results(showSelection: false);
                      final selected = _selectedResult;
                      return Row(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Expanded(child: _results(showSelection: true)),
                          VerticalDivider(
                            width: 1,
                            color: context.appColors.borderStrong,
                          ),
                          SizedBox(
                            width: 310,
                            child: selected == null
                                ? const SizedBox.shrink()
                                : _SearchResultPreview(
                                    result: selected,
                                    onOpen: () => _openResult(selected),
                                  ),
                          ),
                        ],
                      );
                    },
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  GlobalSearchResult? get _selectedResult {
    final results = widget.controller.results;
    if (results.isEmpty) return null;
    _selectedIndex = _selectedIndex.clamp(0, results.length - 1);
    return results[_selectedIndex];
  }

  Widget _results({required bool showSelection}) {
    final controller = widget.controller;
    final strings = YamantakaLocalizations.of(context);
    if (controller.isSearching && controller.results.isEmpty) {
      return AppStatusView(
        icon: Icons.manage_search_rounded,
        title: strings.get('search.searchingTitle'),
        message: strings.get('search.searchingBody'),
        loading: true,
      );
    }
    if (controller.query.isEmpty) {
      return AppStatusView(
        icon: Icons.search_rounded,
        title: strings.searchEmptyTitle,
        message: strings.searchEmptyBody,
      );
    }
    if (controller.results.isEmpty) {
      return AppStatusView(
        icon: Icons.search_off_rounded,
        title: strings.searchNoResultsTitle,
        message: strings.searchNoResultsBody,
      );
    }
    return ListView.builder(
      key: const PageStorageKey('global-search-results'),
      itemCount: controller.results.length + (controller.hasMore ? 1 : 0),
      itemBuilder: (context, index) {
        if (index == controller.results.length) {
          return Padding(
            padding: const EdgeInsets.all(AppSpacing.space16),
            child: OutlinedButton(
              onPressed: () =>
                  controller.search(controller.query, loadMore: true),
              child: Text(strings.get('action.loadMore')),
            ),
          );
        }
        final result = controller.results[index];
        final tile = ListTile(
          selected: showSelection && index == _selectedIndex,
          selectedTileColor: context.appColors.surfaceActive,
          shape: Border(
            left: BorderSide(
              color: showSelection && index == _selectedIndex
                  ? context.appColors.focusRing
                  : Colors.transparent,
              width: 2,
            ),
            bottom: BorderSide(color: context.appColors.borderSubtle),
          ),
          leading: Icon(_icon(result.kind)),
          title: Text(
            result.title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          subtitle: Text(
            [
              strings.get('search.kind.${result.kind.name}'),
              if (result.pageIndex != null)
                strings.get(
                  'reader.page',
                  parameters: {'page': result.pageIndex! + 1},
                ),
              if (result.snippet.isNotEmpty) result.snippet,
            ].join(' · '),
            maxLines: 3,
            overflow: TextOverflow.ellipsis,
          ),
          onTap: () => _openResult(result),
        );
        if (!showSelection) return tile;
        return MouseRegion(
          onEnter: (_) => setState(() => _selectedIndex = index),
          child: Focus(
            onFocusChange: (focused) {
              if (focused) setState(() => _selectedIndex = index);
            },
            child: tile,
          ),
        );
      },
    );
  }

  void _openResult(GlobalSearchResult result) {
    Navigator.pop(context);
    widget.onOpen(result);
  }

  static IconData _icon(GlobalSearchKind kind) => switch (kind) {
    GlobalSearchKind.metadata => Icons.picture_as_pdf_outlined,
    GlobalSearchKind.pageText => Icons.subject_rounded,
    GlobalSearchKind.annotation => Icons.rate_review_outlined,
    GlobalSearchKind.collection => Icons.folder_outlined,
    GlobalSearchKind.tag => Icons.label_outline_rounded,
  };
}

final class _SearchResultPreview extends StatelessWidget {
  const _SearchResultPreview({required this.result, required this.onOpen});

  final GlobalSearchResult result;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return ColoredBox(
      color: context.appColors.surfaceSecondary,
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.space20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            AppStatusLabel(
              icon: _GlobalSearchDialogState._icon(result.kind),
              label: strings.get('search.kind.${result.kind.name}'),
            ),
            const SizedBox(height: AppSpacing.space20),
            Text(result.title, style: Theme.of(context).textTheme.titleLarge),
            if (result.pageIndex case final page?) ...[
              const SizedBox(height: AppSpacing.space8),
              Text(
                strings.get('reader.page', parameters: {'page': page + 1}),
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ],
            const SizedBox(height: AppSpacing.space16),
            Text(
              result.snippet.isEmpty
                  ? strings.get('search.previewNoSnippet')
                  : result.snippet,
              maxLines: 12,
              overflow: TextOverflow.ellipsis,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: context.appColors.textSecondary,
              ),
            ),
            const Spacer(),
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: onOpen,
                icon: const Icon(Icons.arrow_forward_rounded, size: 18),
                label: Text(strings.get('search.openResult')),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _IndexStatusBar extends StatelessWidget {
  const _IndexStatusBar({required this.controller, required this.progress});

  final LibraryIndexController controller;
  final LibraryIndexProgress progress;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space16),
      child: Row(
        children: [
          Switch(value: controller.enabled, onChanged: controller.setEnabled),
          const SizedBox(width: AppSpacing.space4),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  controller.enabled
                      ? progress.isRunning
                            ? strings.get(
                                'search.indexing',
                                parameters: {
                                  'current': progress.completedDocuments,
                                  'total': progress.totalDocuments,
                                },
                              )
                            : strings.get('search.indexEnabled')
                      : strings.get('search.indexDisabled'),
                  style: Theme.of(context).textTheme.labelMedium,
                ),
                if (progress.isRunning)
                  LinearProgressIndicator(value: progress.fraction),
              ],
            ),
          ),
          if (progress.isRunning)
            IconButton(
              tooltip: strings.get(
                progress.isPaused ? 'action.resume' : 'action.pause',
              ),
              onPressed: progress.isPaused
                  ? controller.resume
                  : controller.pause,
              icon: Icon(
                progress.isPaused
                    ? Icons.play_arrow_rounded
                    : Icons.pause_rounded,
              ),
            )
          else if (controller.enabled)
            IconButton(
              tooltip: strings.get('search.updateIndex'),
              onPressed: controller.indexIncremental,
              icon: const Icon(Icons.refresh_rounded),
            ),
          PopupMenuButton<_IndexAction>(
            tooltip: strings.get('search.indexMaintenance'),
            onSelected: (action) {
              switch (action) {
                case _IndexAction.rebuild:
                  controller.rebuild();
                case _IndexAction.clear:
                  controller.clearIndex();
              }
            },
            itemBuilder: (context) => [
              PopupMenuItem(
                value: _IndexAction.rebuild,
                child: AppMenuItemContent(
                  icon: Icons.replay_rounded,
                  label: strings.get('search.rebuildIndex'),
                ),
              ),
              PopupMenuItem(
                value: _IndexAction.clear,
                child: AppMenuItemContent(
                  icon: Icons.delete_sweep_outlined,
                  label: strings.get('search.clearIndex'),
                  destructive: true,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

enum _IndexAction { rebuild, clear }
