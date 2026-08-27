import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

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
    this.onOpenInNewTab,
  });

  final GlobalLibrarySearchController controller;
  final LibraryIndexController indexController;
  final ValueChanged<GlobalSearchResult> onOpen;
  final ValueChanged<GlobalSearchResult>? onOpenInNewTab;

  @override
  State<GlobalSearchDialog> createState() => _GlobalSearchDialogState();
}

final class _GlobalSearchDialogState extends State<GlobalSearchDialog> {
  late final TextEditingController _queryController = TextEditingController(
    text: widget.controller.query,
  );
  final FocusNode _resultsFocusNode = FocusNode(
    debugLabel: 'Global search results',
  );
  final ScrollController _resultsScrollController = ScrollController();
  int _selectedIndex = 0;
  bool _compactDetail = false;

  @override
  void dispose() {
    _queryController.dispose();
    _resultsFocusNode.dispose();
    _resultsScrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return PopScope(
      canPop: !_compactDetail,
      onPopInvokedWithResult: (didPop, _) {
        if (didPop || !_compactDetail) return;
        _showCompactResults();
      },
      child: Dialog(
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
                    child: CallbackShortcuts(
                      bindings: {
                        const SingleActivator(
                          LogicalKeyboardKey.arrowDown,
                        ): () =>
                            _moveSelection(1),
                        const SingleActivator(
                          LogicalKeyboardKey.enter,
                          control: true,
                        ): () =>
                            _openSelected(inNewTab: true),
                      },
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
                              : widget.controller.isSearching
                              ? IconButton(
                                  tooltip: strings.get('action.cancel'),
                                  onPressed: widget.controller.cancel,
                                  icon: const Icon(Icons.stop_circle_outlined),
                                )
                              : IconButton(
                                  tooltip: strings.get('action.clear'),
                                  onPressed: () {
                                    _queryController.clear();
                                    widget.controller.clear();
                                    setState(() => _compactDetail = false);
                                  },
                                  icon: const Icon(Icons.clear_rounded),
                                ),
                        ),
                        onChanged: (value) {
                          setState(() => _compactDetail = false);
                          widget.controller.search(value);
                        },
                        onSubmitted: (value) {
                          if (widget.controller.results.isNotEmpty) {
                            _openSelected();
                          } else {
                            widget.controller.search(value);
                          }
                        },
                      ),
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  _SearchFilters(controller: widget.controller),
                  const SizedBox(height: AppSpacing.space4),
                  _IndexStatusBar(
                    controller: widget.indexController,
                    progress: progress,
                  ),
                  const Divider(height: 1),
                  Expanded(
                    child: Focus(
                      focusNode: _resultsFocusNode,
                      onKeyEvent: _handleResultsKey,
                      child: LayoutBuilder(
                        builder: (context, constraints) {
                          final compact = constraints.maxWidth < 820;
                          final selected = _selectedResult;
                          if (compact && _compactDetail && selected != null) {
                            return _SearchResultPreview(
                              key: const Key('global-search-compact-detail'),
                              result: selected,
                              onBack: _showCompactResults,
                              onOpen: () => _openResult(selected),
                            );
                          }
                          final canPreview =
                              !compact &&
                              widget.controller.results.isNotEmpty &&
                              widget.controller.query.isNotEmpty;
                          if (!canPreview) {
                            return _results(
                              showSelection: true,
                              compactNavigation: compact,
                            );
                          }
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
                  ),
                ],
              );
            },
          ),
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

  Widget _results({
    required bool showSelection,
    bool compactNavigation = false,
  }) {
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
    if (controller.state == GlobalSearchState.cancelled) {
      return AppStatusView(
        icon: Icons.stop_circle_outlined,
        title: strings.get('search.cancelledTitle'),
        message: strings.get('search.cancelledBody'),
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
      controller: _resultsScrollController,
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
          subtitle: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                [
                  strings.get('search.kind.${result.kind.name}'),
                  if (result.pageIndex != null)
                    strings.get(
                      'reader.page',
                      parameters: {'page': result.pageIndex! + 1},
                    ),
                  if (result.isMissing) strings.get('search.resultMissing'),
                  if (result.isStale) strings.get('search.resultStale'),
                ].join(' · '),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              if (result.snippet.isNotEmpty)
                _HighlightedSnippet(text: result.snippet, maxLines: 2),
            ],
          ),
          onTap: compactNavigation
              ? () {
                  setState(() {
                    _selectedIndex = index;
                    _compactDetail = true;
                  });
                }
              : () => _openResult(result),
        );
        if (!showSelection) return tile;
        return MouseRegion(
          onEnter: (_) => setState(() => _selectedIndex = index),
          child: Focus(
            onKeyEvent: (_, event) {
              if (event is! KeyDownEvent ||
                  (event.logicalKey != LogicalKeyboardKey.enter &&
                      event.logicalKey != LogicalKeyboardKey.numpadEnter)) {
                return KeyEventResult.ignored;
              }
              if (HardwareKeyboard.instance.isControlPressed &&
                  widget.onOpenInNewTab != null) {
                Navigator.pop(context);
                widget.onOpenInNewTab!(result);
              } else {
                _openResult(result);
              }
              return KeyEventResult.handled;
            },
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

  void _showCompactResults() {
    setState(() => _compactDetail = false);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _resultsFocusNode.requestFocus();
    });
  }

  void _moveSelection(int delta) {
    final results = widget.controller.results;
    if (results.isEmpty) return;
    setState(() {
      _selectedIndex = (_selectedIndex + delta).clamp(0, results.length - 1);
    });
    _resultsFocusNode.requestFocus();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_resultsScrollController.hasClients) return;
      final target = (_selectedIndex * 76.0).clamp(
        0.0,
        _resultsScrollController.position.maxScrollExtent,
      );
      if (MediaQuery.disableAnimationsOf(context)) {
        _resultsScrollController.jumpTo(target);
      } else {
        _resultsScrollController.animateTo(
          target,
          duration: AppMotion.fast,
          curve: Curves.easeOutCubic,
        );
      }
    });
  }

  KeyEventResult _handleResultsKey(FocusNode _, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      _moveSelection(1);
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      _moveSelection(-1);
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.numpadEnter) {
      _openSelected(inNewTab: HardwareKeyboard.instance.isControlPressed);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _openSelected({bool inNewTab = false}) {
    final result = _selectedResult;
    if (result == null) return;
    Navigator.pop(context);
    if (inNewTab && widget.onOpenInNewTab != null) {
      widget.onOpenInNewTab!(result);
    } else {
      widget.onOpen(result);
    }
  }

  static IconData _icon(GlobalSearchKind kind) => switch (kind) {
    GlobalSearchKind.metadata => Icons.picture_as_pdf_outlined,
    GlobalSearchKind.pageText => Icons.subject_rounded,
    GlobalSearchKind.annotation => Icons.rate_review_outlined,
    GlobalSearchKind.bookmark => Icons.bookmark_outline_rounded,
    GlobalSearchKind.collection => Icons.folder_outlined,
    GlobalSearchKind.tag => Icons.label_outline_rounded,
  };
}

final class _SearchResultPreview extends StatelessWidget {
  const _SearchResultPreview({
    super.key,
    required this.result,
    required this.onOpen,
    this.onBack,
  });

  final GlobalSearchResult result;
  final VoidCallback onOpen;
  final VoidCallback? onBack;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return ColoredBox(
      color: context.appColors.surfaceSecondary,
      child: CustomScrollView(
        slivers: [
          SliverPadding(
            padding: const EdgeInsets.all(AppSpacing.space20),
            sliver: SliverFillRemaining(
              hasScrollBody: false,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (onBack case final back?) ...[
                    TextButton.icon(
                      key: const Key('global-search-back-to-results'),
                      onPressed: back,
                      icon: const Icon(Icons.arrow_back_rounded, size: 18),
                      label: Text(strings.get('search.backToResults')),
                    ),
                    const SizedBox(height: AppSpacing.space8),
                  ],
                  AppStatusLabel(
                    icon: _GlobalSearchDialogState._icon(result.kind),
                    label: strings.get('search.kind.${result.kind.name}'),
                  ),
                  const SizedBox(height: AppSpacing.space20),
                  Text(
                    result.title,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                  if (result.pageIndex case final page?) ...[
                    const SizedBox(height: AppSpacing.space8),
                    Text(
                      strings.get(
                        'reader.page',
                        parameters: {'page': page + 1},
                      ),
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ],
                  const SizedBox(height: AppSpacing.space16),
                  if (result.snippet.isEmpty)
                    Text(
                      strings.get('search.previewNoSnippet'),
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: context.appColors.textSecondary,
                      ),
                    )
                  else
                    _HighlightedSnippet(text: result.snippet, maxLines: 12),
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
          ),
        ],
      ),
    );
  }
}

final class _SearchFilters extends StatelessWidget {
  const _SearchFilters({required this.controller});

  final GlobalLibrarySearchController controller;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return SizedBox(
      height: 42,
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space16),
        children: [
          for (final kind in GlobalSearchKind.values) ...[
            FilterChip(
              key: ValueKey('global-search-filter-${kind.name}'),
              label: Text(strings.get('search.kind.${kind.name}')),
              selected: controller.selectedKinds.contains(kind),
              onSelected: (_) => controller.toggleKind(kind),
              visualDensity: VisualDensity.compact,
            ),
            const SizedBox(width: AppSpacing.space6),
          ],
        ],
      ),
    );
  }
}

final class _HighlightedSnippet extends StatelessWidget {
  const _HighlightedSnippet({required this.text, required this.maxLines});

  final String text;
  final int maxLines;

  @override
  Widget build(BuildContext context) {
    final base = Theme.of(
      context,
    ).textTheme.bodySmall?.copyWith(color: context.appColors.textSecondary);
    final highlighted = base?.copyWith(
      color: context.appColors.textPrimary,
      fontWeight: FontWeight.w700,
      backgroundColor: context.appColors.accentSoft,
    );
    final spans = <InlineSpan>[];
    var cursor = 0;
    while (cursor < text.length) {
      final start = text.indexOf('‹', cursor);
      if (start < 0) {
        spans.add(TextSpan(text: text.substring(cursor), style: base));
        break;
      }
      if (start > cursor) {
        spans.add(TextSpan(text: text.substring(cursor, start), style: base));
      }
      final end = text.indexOf('›', start + 1);
      if (end < 0) {
        spans.add(
          TextSpan(text: text.substring(start + 1), style: highlighted),
        );
        break;
      }
      spans.add(
        TextSpan(text: text.substring(start + 1, end), style: highlighted),
      );
      cursor = end + 1;
    }
    return Text.rich(
      TextSpan(children: spans),
      maxLines: maxLines,
      overflow: TextOverflow.ellipsis,
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
