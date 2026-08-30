part of 'artifact_library.dart';

Map<LibraryReadingState, int> _materializedReadingStateCounts(
  Iterable<LibraryDocument> documents,
) {
  final counts = <LibraryReadingState, int>{
    for (final state in LibraryReadingState.values) state: 0,
  };
  for (final document in documents) {
    final state = readingStateOf(document);
    counts[LibraryReadingState.all] = counts[LibraryReadingState.all]! + 1;
    counts[state] = counts[state]! + 1;
  }
  return counts;
}

enum _LibraryViewMode { grid, list }

enum _ActiveFilterKind {
  query,
  readingState,
  favorites,
  sort,
  authors,
  topics,
  grouping,
}

final class _LibraryCommandBar extends StatelessWidget {
  const _LibraryCommandBar({
    required this.resultCount,
    required this.totalCount,
    required this.query,
    required this.counts,
    required this.readingState,
    required this.sortOrder,
    required this.favoritesOnly,
    required this.activeFilter,
    required this.activeFilterCount,
    required this.selectionMode,
    required this.selectedCount,
    required this.viewMode,
    required this.selectedAuthorCount,
    required this.selectedTopicCount,
    required this.grouping,
    required this.onSearch,
    required this.onFilter,
    required this.onFacets,
    required this.onToggleSelectionMode,
    required this.onViewModeChanged,
    required this.onClearActiveFilter,
  });

  final int resultCount;
  final int totalCount;
  final String query;
  final Map<LibraryReadingState, int> counts;
  final LibraryReadingState readingState;
  final LibrarySortOrder sortOrder;
  final bool favoritesOnly;
  final bool activeFilter;
  final int activeFilterCount;
  final bool selectionMode;
  final int selectedCount;
  final _LibraryViewMode viewMode;
  final int selectedAuthorCount;
  final int selectedTopicCount;
  final LibraryGrouping grouping;
  final VoidCallback onSearch;
  final ValueChanged<_FilterCommand> onFilter;
  final VoidCallback onFacets;
  final VoidCallback onToggleSelectionMode;
  final ValueChanged<_LibraryViewMode> onViewModeChanged;
  final ValueChanged<_ActiveFilterKind> onClearActiveFilter;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final summary = _LibraryCollectionSummary(
      resultCount: resultCount,
      totalCount: totalCount,
      filtered: query.isNotEmpty || activeFilter,
    );
    final expandedFilterContext = activeFilter
        ? TextButton.icon(
            key: const Key('library-reset-filters-button'),
            onPressed: () => onFilter(_FilterCommand.reset),
            icon: Badge(
              label: Text('$activeFilterCount'),
              child: const Icon(Icons.filter_alt_off_outlined, size: 18),
            ),
            label: Text(
              strings.get(
                'library.filtersApplied',
                parameters: {'count': activeFilterCount},
              ),
            ),
          )
        : null;
    final tools = YamaToolbarGroup(
      label: strings.get('library.commandBar'),
      flat: MediaQuery.highContrastOf(context),
      children: [
        YamaIconButton(
          key: const Key('library-grid-view-button'),
          semanticLabel: strings.get('library.view.grid'),
          selected: viewMode == _LibraryViewMode.grid,
          onPressed: () => onViewModeChanged(_LibraryViewMode.grid),
          icon: Icons.grid_view_rounded,
        ),
        YamaIconButton(
          key: const Key('library-list-view-button'),
          semanticLabel: strings.get('library.view.list'),
          selected: viewMode == _LibraryViewMode.list,
          onPressed: () => onViewModeChanged(_LibraryViewMode.list),
          icon: Icons.view_list_rounded,
        ),
        const YamaToolbarDivider(),
        YamaIconButton(
          key: const Key('library-selection-mode-button'),
          semanticLabel: selectionMode
              ? strings.get('library.exitSelectionMode')
              : strings.get('library.selectMultiple'),
          tooltip: selectionMode
              ? strings.get('library.exitSelectionMode')
              : strings.get('library.selectMultiple'),
          selected: selectionMode,
          onPressed: onToggleSelectionMode,
          icon: selectionMode
              ? Icons.check_box_outlined
              : Icons.checklist_rounded,
          badge: selectedCount > 0
              ? Badge(label: Text('$selectedCount'))
              : null,
        ),
        YamaIconButton(
          key: const Key('library-search-button'),
          semanticLabel: strings.librarySearch,
          tooltip: '${strings.librarySearch} (Ctrl+F)',
          onPressed: onSearch,
          icon: Icons.search_rounded,
        ),
        YamaIconButton(
          key: const Key('library-facet-button'),
          semanticLabel: strings.get('library.filterAuthorsTopics'),
          tooltip: strings.get('library.authorsTopicsGrouping'),
          onPressed: onFacets,
          selected:
              selectedAuthorCount > 0 ||
              selectedTopicCount > 0 ||
              grouping != LibraryGrouping.none,
          icon: Icons.account_tree_outlined,
        ),
        PopupMenuButton<_FilterCommand>(
          key: const Key('library-filter-button'),
          tooltip: strings.get('library.filterSort'),
          onSelected: onFilter,
          icon: Badge(
            isLabelVisible: activeFilter,
            smallSize: 6,
            backgroundColor: context.appColors.textSecondary,
            child: const Icon(Icons.tune_rounded),
          ),
          itemBuilder: (context) => [
            PopupMenuItem(
              enabled: false,
              height: 32,
              child: Text(strings.get('library.state')),
            ),
            for (final state in LibraryReadingState.values)
              CheckedPopupMenuItem(
                value: switch (state) {
                  LibraryReadingState.all => _FilterCommand.all,
                  LibraryReadingState.inProgress => _FilterCommand.inProgress,
                  LibraryReadingState.notStarted => _FilterCommand.notStarted,
                  LibraryReadingState.completed => _FilterCommand.completed,
                },
                checked: readingState == state,
                child: Text(
                  '${strings.get('library.readingState.${state.name}')} · ${counts[state] ?? 0}',
                ),
              ),
            const PopupMenuDivider(),
            CheckedPopupMenuItem(
              value: _FilterCommand.favorites,
              checked: favoritesOnly,
              child: Text(strings.get('library.favoritesOnly')),
            ),
            const PopupMenuDivider(),
            PopupMenuItem(
              enabled: false,
              height: 32,
              child: Text(strings.get('library.sort')),
            ),
            for (final order in LibrarySortOrder.values)
              CheckedPopupMenuItem(
                value: switch (order) {
                  LibrarySortOrder.lastOpened => _FilterCommand.lastOpened,
                  LibrarySortOrder.dateAdded => _FilterCommand.dateAdded,
                  LibrarySortOrder.title => _FilterCommand.title,
                  LibrarySortOrder.progress => _FilterCommand.progress,
                },
                checked: sortOrder == order,
                child: Text(strings.get('library.sortOrder.${order.name}')),
              ),
            if (activeFilter) ...[
              const PopupMenuDivider(),
              PopupMenuItem(
                value: _FilterCommand.reset,
                child: Text(strings.get('library.resetFilters')),
              ),
            ],
          ],
        ),
      ],
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        // Medium windows still need the two-row composition: the command bar
        // carries four 40 px tools plus a localized result summary.
        final compact = constraints.maxWidth < 820;
        final compactFilterContext = activeFilter
            ? Semantics(
                label: strings.get(
                  'library.filtersApplied',
                  parameters: {'count': activeFilterCount},
                ),
                button: true,
                child: IconButton(
                  key: const Key('library-reset-filters-button'),
                  tooltip: strings.get('library.resetFilters'),
                  onPressed: () => onFilter(_FilterCommand.reset),
                  icon: Badge(
                    label: Text('$activeFilterCount'),
                    child: const Icon(Icons.filter_alt_off_outlined),
                  ),
                ),
              )
            : null;
        final commandRow = compact
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Expanded(child: summary),
                      ?compactFilterContext,
                    ],
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  Row(
                    children: [
                      if (selectedCount > 0)
                        _LibrarySelectionCount(count: selectedCount),
                      const Spacer(),
                      tools,
                    ],
                  ),
                ],
              )
            : Row(
                children: [
                  summary,
                  const Spacer(),
                  if (selectedCount > 0) ...[
                    _LibrarySelectionCount(count: selectedCount),
                    const SizedBox(width: AppSpacing.space8),
                  ],
                  if (expandedFilterContext != null) ...[
                    expandedFilterContext,
                    const SizedBox(width: AppSpacing.space8),
                  ],
                  tools,
                ],
              );
        return Padding(
          padding: const EdgeInsets.fromLTRB(
            AppSpacing.space20,
            AppSpacing.space12,
            AppSpacing.space20,
            AppSpacing.space8,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              commandRow,
              if (activeFilter) ...[
                const SizedBox(height: AppSpacing.space8),
                _ActiveFilterStrip(
                  query: query,
                  readingState: readingState,
                  sortOrder: sortOrder,
                  favoritesOnly: favoritesOnly,
                  selectedAuthorCount: selectedAuthorCount,
                  selectedTopicCount: selectedTopicCount,
                  grouping: grouping,
                  onClear: onClearActiveFilter,
                ),
              ],
            ],
          ),
        );
      },
    );
  }
}

final class _ActiveFilterStrip extends StatelessWidget {
  const _ActiveFilterStrip({
    required this.query,
    required this.readingState,
    required this.sortOrder,
    required this.favoritesOnly,
    required this.selectedAuthorCount,
    required this.selectedTopicCount,
    required this.grouping,
    required this.onClear,
  });

  final String query;
  final LibraryReadingState readingState;
  final LibrarySortOrder sortOrder;
  final bool favoritesOnly;
  final int selectedAuthorCount;
  final int selectedTopicCount;
  final LibraryGrouping grouping;
  final ValueChanged<_ActiveFilterKind> onClear;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      container: true,
      label: strings.get('library.activeFilters'),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            Text(
              strings.get('library.activeFilters'),
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: context.appColors.textMuted,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(width: AppSpacing.space8),
            if (query.isNotEmpty)
              _ActiveFilterChip(
                label: '“$query”',
                onDeleted: () => onClear(_ActiveFilterKind.query),
              ),
            if (readingState != LibraryReadingState.all)
              _ActiveFilterChip(
                label: strings.get('library.readingState.${readingState.name}'),
                onDeleted: () => onClear(_ActiveFilterKind.readingState),
              ),
            if (favoritesOnly)
              _ActiveFilterChip(
                label: strings.get('library.favoritesOnly'),
                onDeleted: () => onClear(_ActiveFilterKind.favorites),
              ),
            if (sortOrder != LibrarySortOrder.lastOpened)
              _ActiveFilterChip(
                label: strings.get('library.sortOrder.${sortOrder.name}'),
                onDeleted: () => onClear(_ActiveFilterKind.sort),
              ),
            if (selectedAuthorCount > 0)
              _ActiveFilterChip(
                label: strings.get(
                  'library.authorFilters',
                  parameters: {'count': selectedAuthorCount},
                ),
                onDeleted: () => onClear(_ActiveFilterKind.authors),
              ),
            if (selectedTopicCount > 0)
              _ActiveFilterChip(
                label: strings.get(
                  'library.topicFilters',
                  parameters: {'count': selectedTopicCount},
                ),
                onDeleted: () => onClear(_ActiveFilterKind.topics),
              ),
            if (grouping != LibraryGrouping.none)
              _ActiveFilterChip(
                label: strings.get('library.grouping.${grouping.name}'),
                onDeleted: () => onClear(_ActiveFilterKind.grouping),
              ),
          ],
        ),
      ),
    );
  }
}

final class _ActiveFilterChip extends StatelessWidget {
  const _ActiveFilterChip({required this.label, required this.onDeleted});

  final String label;
  final VoidCallback onDeleted;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(right: AppSpacing.space6),
    child: InputChip(
      label: Text(label),
      tooltip: YamantakaLocalizations.of(
        context,
      ).get('library.removeFilter', parameters: {'filter': label}),
      onDeleted: onDeleted,
      visualDensity: VisualDensity.compact,
    ),
  );
}

final class _LibraryCollectionSummary extends StatelessWidget {
  const _LibraryCollectionSummary({
    required this.resultCount,
    required this.totalCount,
    required this.filtered,
  });

  final int resultCount;
  final int totalCount;
  final bool filtered;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final label = filtered
        ? strings.get(
            'library.resultOfTotal',
            parameters: {'result': resultCount, 'total': totalCount},
          )
        : strings.count('count.documents', totalCount);
    return Semantics(
      liveRegion: filtered,
      label: label,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 28,
            height: 28,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              border: Border.all(color: context.appColors.borderSubtle),
              borderRadius: BorderRadius.circular(AppRadii.control),
            ),
            child: Icon(
              filtered
                  ? Icons.filter_list_rounded
                  : Icons.auto_stories_outlined,
              size: 16,
              color: context.appColors.textSecondary,
            ),
          ),
          const SizedBox(width: AppSpacing.space8),
          Text(
            label,
            style: Theme.of(context).textTheme.labelMedium?.copyWith(
              color: context.appColors.textSecondary,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}

final class _LibrarySelectionCount extends StatelessWidget {
  const _LibrarySelectionCount({required this.count});

  final int count;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      liveRegion: true,
      label: strings.count('count.documentsSelected', count),
      child: Text(
        strings.count('count.selected', count),
        style: Theme.of(context).textTheme.labelMedium,
      ),
    );
  }
}

final class _BulkActionBar extends StatelessWidget {
  const _BulkActionBar({
    required this.selectedCount,
    required this.onClear,
    required this.onAction,
    required this.canRun,
  });

  final int selectedCount;
  final VoidCallback onClear;
  final ValueChanged<BulkLibraryAction> onAction;
  final bool canRun;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      container: true,
      liveRegion: true,
      label: strings.get(
        'library.actionsForSelection',
        parameters: {'count': selectedCount},
      ),
      child: Material(
        color: context.appColors.surfaceSecondary,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(
            AppSpacing.space20,
            AppSpacing.space4,
            AppSpacing.space12,
            AppSpacing.space4,
          ),
          child: Row(
            children: [
              const Icon(Icons.select_all_rounded, size: 18),
              const SizedBox(width: AppSpacing.space8),
              Expanded(
                child: Text(strings.count('count.selected', selectedCount)),
              ),
              PopupMenuButton<BulkLibraryAction>(
                enabled: canRun,
                tooltip: strings.get('library.bulkActions'),
                onSelected: onAction,
                itemBuilder: (context) => [
                  PopupMenuItem(
                    value: BulkLibraryAction.favorite,
                    child: Text(strings.get('library.markFavorite')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.unfavorite,
                    child: Text(strings.get('library.unmarkFavorite')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.assignTag,
                    child: Text(strings.get('library.assignTag')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.addToCollection,
                    child: Text(strings.get('library.addToCollection')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.removeFromCollection,
                    child: Text(strings.get('library.removeFromCollection')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.archive,
                    child: Text(strings.get('library.archive')),
                  ),
                  PopupMenuItem(
                    value: BulkLibraryAction.regenerateCover,
                    child: Text(strings.get('library.regenerateCovers')),
                  ),
                  PopupMenuDivider(),
                  PopupMenuItem(
                    value: BulkLibraryAction.removeFromLibrary,
                    child: Text(strings.get('library.removeFromLibrary')),
                  ),
                ],
                icon: const Icon(Icons.more_horiz_rounded),
              ),
              IconButton(
                tooltip: strings.get('library.clearSelection'),
                onPressed: onClear,
                icon: const Icon(Icons.close_rounded),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

final class _ArtifactCanvas extends StatelessWidget {
  const _ArtifactCanvas({
    required this.controller,
    required this.documents,
    required this.selectionListenableFor,
    required this.selectedDocumentIds,
    required this.selectionMode,
    required this.reserveInfoSpace,
    required this.reduceMotion,
    required this.grouping,
    required this.collections,
    required this.tags,
    required this.collectionMemberships,
    required this.tagMemberships,
    required this.collapsedGroups,
    required this.onToggleGroup,
    required this.viewMode,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
    this.onDocumentVisible,
    this.onDocumentHidden,
  });

  final ScrollController controller;
  final List<LibraryDocument> documents;
  final ValueListenable<bool> Function(String documentId)
  selectionListenableFor;
  final Set<String> selectedDocumentIds;
  final bool selectionMode;
  final bool reserveInfoSpace;
  final bool reduceMotion;
  final LibraryGrouping grouping;
  final List<DocumentCollection> collections;
  final List<DocumentTag> tags;
  final Map<String, Set<String>> collectionMemberships;
  final Map<String, Set<String>> tagMemberships;
  final Set<String> collapsedGroups;
  final ValueChanged<String> onToggleGroup;
  final _LibraryViewMode viewMode;
  final ValueChanged<LibraryDocument> onSelect;
  final ValueChanged<LibraryDocument> onOpen;
  final ValueChanged<String?> onHover;
  final void Function(LibraryDocument document, Offset position) onMenu;
  final ValueChanged<LibraryDocument>? onDocumentVisible;
  final ValueChanged<LibraryDocument>? onDocumentHidden;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final horizontalPadding = AppSpacing.pagePadding(constraints.maxWidth);
        if (viewMode == _LibraryViewMode.list) {
          return _ArtifactList(
            controller: controller,
            documents: documents,
            horizontalPadding: horizontalPadding,
            selectionListenableFor: selectionListenableFor,
            selectedDocumentIds: selectedDocumentIds,
            selectionMode: selectionMode,
            reserveInfoSpace: reserveInfoSpace,
            grouping: grouping,
            groups: _groups(context),
            collapsedGroups: collapsedGroups,
            onToggleGroup: onToggleGroup,
            onSelect: onSelect,
            onOpen: onOpen,
            onHover: onHover,
            onMenu: onMenu,
            onDocumentVisible: onDocumentVisible,
            onDocumentHidden: onDocumentHidden,
          );
        }
        final metrics = LibraryArtifactLayout.metricsFor(
          viewportWidth: constraints.maxWidth - horizontalPadding * 2,
          documentCount: documents.length,
          compact:
              AppBreakpoints.classify(constraints.maxWidth) ==
              AppWindowClass.compact,
        );
        if (grouping != LibraryGrouping.none) {
          final groups = _groups(context);
          return CustomScrollView(
            controller: controller,
            slivers: [
              const SliverToBoxAdapter(
                child: SizedBox(height: AppSpacing.space16),
              ),
              for (final entry in groups.entries) ...[
                SliverPadding(
                  padding: EdgeInsets.fromLTRB(
                    horizontalPadding,
                    AppSpacing.space8,
                    horizontalPadding,
                    AppSpacing.space8,
                  ),
                  sliver: SliverToBoxAdapter(
                    child: _ArtifactGroupHeader(
                      label: entry.key,
                      count: entry.value.length,
                      collapsed: collapsedGroups.contains(entry.key),
                      onToggle: () => onToggleGroup(entry.key),
                    ),
                  ),
                ),
                if (!collapsedGroups.contains(entry.key))
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      horizontalPadding,
                      0,
                      horizontalPadding,
                      AppSpacing.space24,
                    ),
                    sliver: SliverGrid.builder(
                      itemCount: entry.value.length,
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: metrics.columns,
                        mainAxisExtent: metrics.cellExtent,
                        mainAxisSpacing: AppSpacing.space16,
                        crossAxisSpacing: AppSpacing.space16,
                      ),
                      itemBuilder: (context, index) {
                        final document = entry.value[index];
                        return Center(
                          child: _artifact(document, metrics.artifactWidth),
                        );
                      },
                    ),
                  ),
              ],
              SliverToBoxAdapter(
                child: SizedBox(height: reserveInfoSpace ? 156 : 48),
              ),
            ],
          );
        }
        if (documents.length <= 6) {
          final bottomPadding = reserveInfoSpace ? 116.0 : AppSpacing.space24;
          return SingleChildScrollView(
            controller: controller,
            padding: EdgeInsets.fromLTRB(
              horizontalPadding,
              AppSpacing.space24,
              horizontalPadding,
              bottomPadding,
            ),
            child: ConstrainedBox(
              constraints: BoxConstraints(
                minHeight: math.max(
                  320,
                  constraints.maxHeight - AppSpacing.space24 - bottomPadding,
                ),
              ),
              child: Center(
                child: ConstrainedBox(
                  constraints: BoxConstraints(
                    maxWidth:
                        metrics.artifactWidth * metrics.columns +
                        (metrics.horizontalCellExtent - metrics.artifactWidth) *
                            math.max(0, metrics.columns - 1),
                  ),
                  child: Wrap(
                    alignment: WrapAlignment.center,
                    runAlignment: WrapAlignment.center,
                    spacing: math.max(
                      0,
                      metrics.horizontalCellExtent - metrics.artifactWidth,
                    ),
                    runSpacing: AppSpacing.space24,
                    children: [
                      for (final document in documents)
                        SizedBox(
                          width: metrics.artifactWidth,
                          height: metrics.cellExtent,
                          child: Center(
                            child: _artifact(document, metrics.artifactWidth),
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
          );
        }
        return CustomScrollView(
          controller: controller,
          slivers: [
            SliverPadding(
              padding: EdgeInsets.fromLTRB(
                horizontalPadding,
                AppSpacing.space24,
                horizontalPadding,
                180,
              ),
              sliver: SliverGrid.builder(
                itemCount: documents.length,
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: metrics.columns,
                  mainAxisExtent: metrics.cellExtent,
                  mainAxisSpacing: AppSpacing.space16,
                  crossAxisSpacing: AppSpacing.space16,
                ),
                itemBuilder: (context, index) {
                  final document = documents[index];
                  return Center(
                    child: _artifact(document, metrics.artifactWidth),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _artifact(LibraryDocument document, double width) {
    Widget buildArtifact(bool selected) => ArtifactDocument(
      key: ValueKey('artifact-${document.id}'),
      document: document,
      width: width,
      selected: selected,
      selectionMode: selectionMode,
      reduceMotion: reduceMotion,
      onSelect: () => onSelect(document),
      onOpen: () => onOpen(document),
      onHover: (hovered) => onHover(hovered ? document.id : null),
      onMenu: (position) => onMenu(document, position),
      onVisible: () => onDocumentVisible?.call(document),
      onHidden: () => onDocumentHidden?.call(document),
    );
    if (selectionMode) {
      return buildArtifact(selectedDocumentIds.contains(document.id));
    }
    return ValueListenableBuilder<bool>(
      valueListenable: selectionListenableFor(document.id),
      builder: (context, selected, _) => buildArtifact(selected),
    );
  }

  Map<String, List<LibraryDocument>> _groups(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final result = <String, List<LibraryDocument>>{};
    for (final document in documents) {
      final label = switch (grouping) {
        LibraryGrouping.none => '',
        LibraryGrouping.author =>
          document.effectiveAuthor?.trim().isNotEmpty == true
              ? document.effectiveAuthor!.trim()
              : strings.get('library.unknownAuthor'),
        LibraryGrouping.collection =>
          collections
                  .where(
                    (collection) =>
                        collectionMemberships[collection.id]?.contains(
                          document.id,
                        ) ==
                        true,
                  )
                  .firstOrNull
                  ?.name ??
              strings.get('library.ungroupedCollection'),
        LibraryGrouping.tag =>
          tags
                  .where(
                    (tag) =>
                        tagMemberships[tag.id]?.contains(document.id) == true,
                  )
                  .firstOrNull
                  ?.name ??
              strings.get('library.untagged'),
        LibraryGrouping.readingState => strings.get(
          'library.readingState.${readingStateOf(document).name}',
        ),
      };
      result.putIfAbsent(label, () => <LibraryDocument>[]).add(document);
    }
    return result;
  }
}

final class _ArtifactGroupHeader extends StatelessWidget {
  const _ArtifactGroupHeader({
    required this.label,
    required this.count,
    required this.collapsed,
    required this.onToggle,
  });

  final String label;
  final int count;
  final bool collapsed;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    expanded: !collapsed,
    label: '$label, $count',
    child: InkWell(
      onTap: onToggle,
      child: ConstrainedBox(
        constraints: const BoxConstraints(
          minHeight: AppSizes.touchControlExtent,
        ),
        child: Row(
          children: [
            Icon(
              collapsed
                  ? Icons.chevron_right_rounded
                  : Icons.expand_more_rounded,
              size: 20,
            ),
            const SizedBox(width: AppSpacing.space6),
            Expanded(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Theme.of(context).textTheme.titleSmall,
              ),
            ),
            Text('$count', style: Theme.of(context).textTheme.labelSmall),
          ],
        ),
      ),
    ),
  );
}

final class _ArtifactList extends StatelessWidget {
  const _ArtifactList({
    required this.controller,
    required this.documents,
    required this.horizontalPadding,
    required this.selectionListenableFor,
    required this.selectedDocumentIds,
    required this.selectionMode,
    required this.reserveInfoSpace,
    required this.grouping,
    required this.groups,
    required this.collapsedGroups,
    required this.onToggleGroup,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
    required this.onDocumentVisible,
    required this.onDocumentHidden,
  });

  final ScrollController controller;
  final List<LibraryDocument> documents;
  final double horizontalPadding;
  final ValueListenable<bool> Function(String documentId)
  selectionListenableFor;
  final Set<String> selectedDocumentIds;
  final bool selectionMode;
  final bool reserveInfoSpace;
  final LibraryGrouping grouping;
  final Map<String, List<LibraryDocument>> groups;
  final Set<String> collapsedGroups;
  final ValueChanged<String> onToggleGroup;
  final ValueChanged<LibraryDocument> onSelect;
  final ValueChanged<LibraryDocument> onOpen;
  final ValueChanged<String?> onHover;
  final void Function(LibraryDocument document, Offset position) onMenu;
  final ValueChanged<LibraryDocument>? onDocumentVisible;
  final ValueChanged<LibraryDocument>? onDocumentHidden;

  @override
  Widget build(BuildContext context) {
    final items = <Object>[];
    if (grouping != LibraryGrouping.none) {
      for (final entry in groups.entries) {
        items.add(
          _ArtifactGroupMarker(label: entry.key, count: entry.value.length),
        );
        if (!collapsedGroups.contains(entry.key)) items.addAll(entry.value);
      }
    } else {
      items.addAll(documents);
    }
    return LayoutBuilder(
      builder: (context, constraints) {
        final textScale = MediaQuery.textScalerOf(context).scale(1);
        final compact =
            constraints.maxWidth < 760 ||
            (textScale > 1.3 && constraints.maxWidth < 1400);
        return CustomScrollView(
          controller: controller,
          slivers: [
            SliverPadding(
              padding: EdgeInsets.fromLTRB(
                horizontalPadding,
                AppSpacing.space12,
                horizontalPadding,
                0,
              ),
              sliver: SliverToBoxAdapter(
                child: _ArtifactListHeader(compact: compact),
              ),
            ),
            SliverPadding(
              padding: EdgeInsets.fromLTRB(
                horizontalPadding,
                0,
                horizontalPadding,
                reserveInfoSpace ? 156 : AppSpacing.space32,
              ),
              sliver: SliverList.builder(
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final item = items[index];
                  if (item is _ArtifactGroupMarker) {
                    return Padding(
                      padding: const EdgeInsets.fromLTRB(
                        AppSpacing.space8,
                        AppSpacing.space20,
                        AppSpacing.space8,
                        AppSpacing.space8,
                      ),
                      child: _ArtifactGroupHeader(
                        label: item.label,
                        count: item.count,
                        collapsed: collapsedGroups.contains(item.label),
                        onToggle: () => onToggleGroup(item.label),
                      ),
                    );
                  }
                  final document = item as LibraryDocument;
                  Widget row(bool selected) => _ArtifactListRow(
                    key: ValueKey('artifact-list-${document.id}'),
                    document: document,
                    selected: selected,
                    selectionMode: selectionMode,
                    compact: compact,
                    onSelect: () => onSelect(document),
                    onOpen: () => onOpen(document),
                    onHover: (hovered) => onHover(hovered ? document.id : null),
                    onMenu: (position) => onMenu(document, position),
                    onVisible: () => onDocumentVisible?.call(document),
                    onHidden: () => onDocumentHidden?.call(document),
                  );
                  if (selectionMode) {
                    return row(selectedDocumentIds.contains(document.id));
                  }
                  return ValueListenableBuilder<bool>(
                    valueListenable: selectionListenableFor(document.id),
                    builder: (context, selected, _) => row(selected),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }
}

final class _ArtifactGroupMarker {
  const _ArtifactGroupMarker({required this.label, required this.count});

  final String label;
  final int count;
}

final class _ArtifactListHeader extends StatelessWidget {
  const _ArtifactListHeader({required this.compact});

  final bool compact;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final style = Theme.of(context).textTheme.labelSmall?.copyWith(
      color: context.appColors.textMuted,
      fontWeight: FontWeight.w600,
    );
    return Container(
      height: 32,
      padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space12),
      decoration: BoxDecoration(
        color: context.appColors.surfaceSecondary,
        border: Border(
          top: BorderSide(color: context.appColors.borderSubtle),
          bottom: BorderSide(color: context.appColors.borderStrong),
        ),
      ),
      child: Row(
        children: [
          const SizedBox(width: 44),
          Expanded(
            flex: 5,
            child: Text(strings.get('library.column.document'), style: style),
          ),
          if (!compact) ...[
            Expanded(
              flex: 2,
              child: Text(strings.get('library.column.state'), style: style),
            ),
            SizedBox(
              width: 92,
              child: Text(strings.get('library.column.progress'), style: style),
            ),
            SizedBox(
              width: 96,
              child: Text(strings.get('library.column.opened'), style: style),
            ),
          ],
        ],
      ),
    );
  }
}

final class _ArtifactListRow extends StatefulWidget {
  const _ArtifactListRow({
    super.key,
    required this.document,
    required this.selected,
    required this.selectionMode,
    required this.compact,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
    this.onVisible,
    this.onHidden,
  });

  final LibraryDocument document;
  final bool selected;
  final bool selectionMode;
  final bool compact;
  final VoidCallback onSelect;
  final VoidCallback onOpen;
  final ValueChanged<bool> onHover;
  final ValueChanged<Offset> onMenu;
  final VoidCallback? onVisible;
  final VoidCallback? onHidden;

  @override
  State<_ArtifactListRow> createState() => _ArtifactListRowState();
}

final class _ArtifactListRowState extends State<_ArtifactListRow> {
  bool _hovered = false;
  bool _focused = false;
  bool _reportedVisible = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _reportedVisible = true;
      widget.onVisible?.call();
    });
  }

  @override
  void dispose() {
    if (_reportedVisible) widget.onHidden?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final colors = context.appColors;
    final document = widget.document;
    final progress = (document.readingProgress.clamp(0.0, 1.0) * 100).round();
    final status = document.isMissing
        ? (
            Icons.link_off_rounded,
            strings.get('library.fileNotFound'),
            AppStatusTone.error,
          )
        : document.hasFileChanged
        ? (
            Icons.sync_problem_rounded,
            strings.get('library.fileChanged'),
            AppStatusTone.warning,
          )
        : document.enrichment?.status == EnrichmentStatus.review
        ? (
            Icons.fact_check_outlined,
            strings.get('library.metadataNeedsReview'),
            AppStatusTone.warning,
          )
        : document.readingProgress >= 1
        ? (
            Icons.check_circle_outline_rounded,
            strings.get('library.finished'),
            AppStatusTone.success,
          )
        : (
            Icons.menu_book_outlined,
            document.readingProgress > 0
                ? strings.get(
                    'library.inProgress',
                    parameters: {'value': progress},
                  )
                : strings.get('library.notStarted'),
            AppStatusTone.neutral,
          );
    final background = widget.selected
        ? colors.surfaceActive
        : _hovered
        ? colors.surfaceHover
        : Colors.transparent;
    return Semantics(
      button: true,
      selected: widget.selected,
      label: _artifactSemanticLabel(document, strings),
      child: Focus(
        onFocusChange: (focused) => setState(() => _focused = focused),
        onKeyEvent: (_, event) {
          if (event is! KeyDownEvent) return KeyEventResult.ignored;
          if (event.logicalKey == LogicalKeyboardKey.enter) {
            widget.onOpen();
            return KeyEventResult.handled;
          }
          if (widget.selectionMode &&
              event.logicalKey == LogicalKeyboardKey.space) {
            widget.onSelect();
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: MouseRegion(
          cursor: SystemMouseCursors.basic,
          onEnter: (_) {
            setState(() => _hovered = true);
            widget.onHover(true);
          },
          onExit: (_) {
            setState(() => _hovered = false);
            widget.onHover(false);
          },
          child: Listener(
            onPointerDown: (_) => widget.onSelect(),
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onDoubleTap: widget.onOpen,
              onSecondaryTapDown: (details) =>
                  widget.onMenu(details.globalPosition),
              child: AnimatedContainer(
                duration: AppMotion.whenAllowed(
                  MediaQuery.disableAnimationsOf(context),
                  AppMotion.fast,
                ),
                constraints: BoxConstraints(
                  minHeight: MediaQuery.textScalerOf(context).scale(1) > 1.3
                      ? 64
                      : 52,
                ),
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space12,
                  vertical: AppSpacing.space6,
                ),
                decoration: BoxDecoration(
                  color: background,
                  border: Border(
                    left: BorderSide(
                      color: _focused
                          ? colors.focusRing
                          : widget.selected
                          ? colors.librarySelectionAccent
                          : Colors.transparent,
                      width: 2,
                    ),
                    bottom: BorderSide(color: colors.borderSubtle),
                  ),
                ),
                child: Row(
                  children: [
                    _ArtifactListCover(document: document),
                    const SizedBox(width: AppSpacing.space12),
                    Expanded(
                      flex: 5,
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Row(
                            children: [
                              Flexible(
                                child: Tooltip(
                                  message: document.effectiveTitle,
                                  child: Text(
                                    document.effectiveTitle,
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: Theme.of(context)
                                        .textTheme
                                        .bodyMedium
                                        ?.copyWith(
                                          fontWeight: widget.selected
                                              ? FontWeight.w600
                                              : FontWeight.w500,
                                        ),
                                  ),
                                ),
                              ),
                              if (document.isFavorite) ...[
                                const SizedBox(width: AppSpacing.space6),
                                Icon(
                                  Icons.bookmark_rounded,
                                  size: 14,
                                  color: colors.textSecondary,
                                ),
                              ],
                            ],
                          ),
                          const SizedBox(height: AppSpacing.space2),
                          Text(
                            document.effectiveAuthor ??
                                strings.get('library.unknownAuthor'),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.labelSmall
                                ?.copyWith(color: colors.textSecondary),
                          ),
                          if (widget.compact) ...[
                            const SizedBox(height: AppSpacing.space2),
                            Row(
                              children: [
                                Icon(
                                  status.$1,
                                  size: 13,
                                  color: colors.textMuted,
                                ),
                                const SizedBox(width: AppSpacing.space4),
                                Expanded(
                                  child: Text(
                                    document.readingProgress > 0 &&
                                            document.readingProgress < 1
                                        ? '${status.$2} · ${document.pageCount} p.'
                                        : '${status.$2} · $progress % · ${document.pageCount} p.',
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: Theme.of(context)
                                        .textTheme
                                        .labelSmall
                                        ?.copyWith(color: colors.textMuted),
                                  ),
                                ),
                              ],
                            ),
                          ],
                        ],
                      ),
                    ),
                    if (!widget.compact) ...[
                      Expanded(
                        flex: 2,
                        child: Align(
                          alignment: Alignment.centerLeft,
                          child: AppStatusLabel(
                            icon: status.$1,
                            label: status.$2,
                            tone: status.$3,
                          ),
                        ),
                      ),
                      SizedBox(
                        width: 92,
                        child: Text(
                          '$progress % · ${document.pageCount} p.',
                          style: Theme.of(context).textTheme.labelSmall
                              ?.copyWith(
                                fontFeatures: const [
                                  FontFeature.tabularFigures(),
                                ],
                              ),
                        ),
                      ),
                      SizedBox(
                        width: 96,
                        child: Text(
                          _relativeArtifactDate(document.lastOpenedAt, strings),
                          style: Theme.of(context).textTheme.labelSmall
                              ?.copyWith(color: colors.textSecondary),
                        ),
                      ),
                    ],
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

final class _ArtifactListCover extends StatelessWidget {
  const _ArtifactListCover({required this.document});

  final LibraryDocument document;

  @override
  Widget build(BuildContext context) {
    final path = artifactCoverPathFor(document, selected: false);
    return Container(
      width: 32,
      height: 42,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: context.appColors.surfaceSunken,
        border: Border.all(color: context.appColors.borderSubtle),
        borderRadius: BorderRadius.circular(AppRadii.indicator),
      ),
      child: path == null
          ? Center(
              child: Text(
                'PDF',
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: context.appColors.textMuted,
                  fontWeight: FontWeight.w600,
                ),
              ),
            )
          : Image.file(
              File(path),
              fit: BoxFit.contain,
              filterQuality: FilterQuality.low,
              errorBuilder: (_, _, _) => Center(
                child: Icon(
                  Icons.broken_image_outlined,
                  size: 15,
                  color: context.appColors.textMuted,
                ),
              ),
            ),
    );
  }
}

final class ArtifactDocument extends StatefulWidget {
  const ArtifactDocument({
    super.key,
    required this.document,
    required this.width,
    required this.selected,
    required this.selectionMode,
    required this.reduceMotion,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
    this.onVisible,
    this.onHidden,
  });

  final LibraryDocument document;
  final double width;
  final bool selected;
  final bool selectionMode;
  final bool reduceMotion;
  final VoidCallback onSelect;
  final VoidCallback onOpen;
  final ValueChanged<bool> onHover;
  final ValueChanged<Offset> onMenu;
  final VoidCallback? onVisible;
  final VoidCallback? onHidden;

  @override
  State<ArtifactDocument> createState() => _ArtifactDocumentState();
}

final class _ArtifactDocumentState extends State<ArtifactDocument> {
  bool _hovered = false;
  bool _focused = false;
  bool _reportedVisible = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _reportedVisible = true;
      widget.onVisible?.call();
    });
  }

  @override
  void dispose() {
    if (_reportedVisible) widget.onHidden?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final jitter = LibraryArtifactLayout.jitterFor(widget.document.id);
    final highContrast = MediaQuery.highContrastOf(context);
    final applyJitter =
        !widget.reduceMotion &&
        !highContrast &&
        !widget.selectionMode &&
        !widget.selected;
    final scale = widget.reduceMotion
        ? 1.0
        : widget.selected
        ? AppSizes.artifactSelectedScale
        : _hovered
        ? AppSizes.artifactHoverScale
        : jitter.scale;
    final elevation = widget.selected
        ? AppSizes.artifactSelectedLift
        : _hovered
        ? AppSizes.artifactHoverLift
        : 0.0;
    final aspectRatio =
        widget.document.coverMetadata?.pageAspectRatio.clamp(0.25, 3.2) ??
        0.704;
    final height = widget.width / aspectRatio;
    final maxHeight = widget.width * 1.42;
    final fittedHeight = math.min(height, maxHeight);
    final duration = widget.reduceMotion ? Duration.zero : AppMotion.standard;
    final coverPath = artifactCoverPathFor(
      widget.document,
      selected: widget.selected,
    );
    final enrichmentStatus = widget.document.enrichment?.status;

    return Semantics(
      button: true,
      selected: widget.selected,
      label: _artifactSemanticLabel(widget.document, strings),
      hint: widget.selectionMode
          ? strings.get('library.artifactSelectionHint')
          : strings.get('library.artifactOpenHint'),
      child: Focus(
        onFocusChange: (focused) {
          setState(() => _focused = focused);
          if (focused && !widget.selectionMode) widget.onSelect();
          widget.onHover(focused);
        },
        onKeyEvent: (_, event) {
          if (widget.selectionMode &&
              event is KeyDownEvent &&
              event.logicalKey == LogicalKeyboardKey.space) {
            widget.onSelect();
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: MouseRegion(
          onEnter: (_) {
            setState(() => _hovered = true);
            widget.onHover(true);
          },
          onExit: (_) {
            setState(() => _hovered = false);
            widget.onHover(false);
          },
          child: Listener(
            onPointerDown: (_) => widget.onSelect(),
            child: GestureDetector(
              onDoubleTap: widget.onOpen,
              onSecondaryTapDown: (details) =>
                  widget.onMenu(details.globalPosition),
              child: Transform.translate(
                offset: applyJitter
                    ? Offset(jitter.offsetX, jitter.offsetY)
                    : Offset.zero,
                child: Transform.rotate(
                  angle: applyJitter
                      ? jitter.rotationDegrees * math.pi / 180
                      : 0,
                  child: AnimatedScale(
                    duration: duration,
                    curve: Curves.easeOutCubic,
                    scale: scale,
                    child: SizedBox(
                      width: widget.width,
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Stack(
                            clipBehavior: Clip.none,
                            children: [
                              if (_focused)
                                Positioned.fill(
                                  left: -3,
                                  top: -3,
                                  right: -3,
                                  bottom: -3,
                                  child: IgnorePointer(
                                    child: DecoratedBox(
                                      decoration: BoxDecoration(
                                        border: Border.all(
                                          color: context.appColors.focusRing,
                                          width: 2,
                                        ),
                                        borderRadius: BorderRadius.circular(
                                          AppRadii.page + 3,
                                        ),
                                      ),
                                    ),
                                  ),
                                ),
                              AnimatedContainer(
                                // Grid reflow changes the cover dimensions. A
                                // layout tween can temporarily exceed the new
                                // cell when filters alter the column count.
                                duration: Duration.zero,
                                curve: Curves.easeOutCubic,
                                transform: Matrix4.translationValues(
                                  0,
                                  -elevation,
                                  0,
                                ),
                                width: widget.width,
                                height: fittedHeight,
                                decoration: BoxDecoration(
                                  color: context.appColors.surfaceSunken,
                                  borderRadius: BorderRadius.circular(
                                    AppRadii.page,
                                  ),
                                  border: Border.all(
                                    color: widget.selected
                                        ? context
                                              .appColors
                                              .librarySelectionAccent
                                        : context.appColors.borderSubtle,
                                    width: widget.selected ? 1.5 : 1,
                                  ),
                                  boxShadow: widget.reduceMotion || highContrast
                                      ? null
                                      : [
                                          if (widget.selected)
                                            BoxShadow(
                                              color: context
                                                  .appColors
                                                  .librarySelectionAccent
                                                  .withValues(alpha: 0.24),
                                              blurRadius:
                                                  AppShadow.artifactGlowBlur,
                                              spreadRadius:
                                                  AppShadow.artifactGlowSpread,
                                              offset: const Offset(0, 12),
                                            ),
                                          BoxShadow(
                                            color: context
                                                .appColors
                                                .readerPageShadow
                                                .withValues(
                                                  alpha: widget.selected
                                                      ? 0.72
                                                      : 0.38,
                                                ),
                                            blurRadius: widget.selected
                                                ? AppShadow.artifactSelectedBlur
                                                : AppShadow.artifactRestBlur,
                                            spreadRadius: widget.selected
                                                ? 1
                                                : 0,
                                            offset: Offset(
                                              0,
                                              widget.selected
                                                  ? AppShadow
                                                        .artifactSelectedOffset
                                                  : AppShadow
                                                        .artifactRestOffset,
                                            ),
                                          ),
                                        ],
                                ),
                                clipBehavior: Clip.antiAlias,
                                child: Stack(
                                  fit: StackFit.expand,
                                  children: [
                                    AnimatedOpacity(
                                      duration: duration,
                                      opacity: widget.document.isMissing
                                          ? 0.56
                                          : 1,
                                      child: coverPath == null
                                          ? _ArtifactPlaceholder(
                                              document: widget.document,
                                            )
                                          : ColorFiltered(
                                              colorFilter: ColorFilter.matrix(
                                                _saturationMatrix(1),
                                              ),
                                              child: Image.file(
                                                File(coverPath),
                                                fit: BoxFit.contain,
                                                cacheWidth:
                                                    (widget.width *
                                                            MediaQuery.devicePixelRatioOf(
                                                              context,
                                                            ))
                                                        .ceil(),
                                                cacheHeight:
                                                    (fittedHeight *
                                                            MediaQuery.devicePixelRatioOf(
                                                              context,
                                                            ))
                                                        .ceil(),
                                                gaplessPlayback: true,
                                                filterQuality:
                                                    FilterQuality.low,
                                                errorBuilder: (_, _, _) =>
                                                    _ArtifactPlaceholder(
                                                      document: widget.document,
                                                      failed: true,
                                                    ),
                                              ),
                                            ),
                                    ),
                                    if (widget.document.readingProgress > 0 &&
                                        widget.document.readingProgress < 1)
                                      Align(
                                        alignment: Alignment.bottomCenter,
                                        child: ColoredBox(
                                          color: context.appColors.textMuted
                                              .withValues(alpha: 0.28),
                                          child: const SizedBox(
                                            width: double.infinity,
                                            height: 4,
                                          ),
                                        ),
                                      ),
                                    if (widget.document.readingProgress > 0 &&
                                        widget.document.readingProgress < 1)
                                      Align(
                                        alignment: Alignment.bottomLeft,
                                        child: FractionallySizedBox(
                                          widthFactor:
                                              widget.document.readingProgress,
                                          child: ColoredBox(
                                            color: widget.selected
                                                ? context
                                                      .appColors
                                                      .librarySelectionAccent
                                                : context
                                                      .appColors
                                                      .textSecondary,
                                            child: const SizedBox(height: 4),
                                          ),
                                        ),
                                      ),
                                    if (widget.document.readingProgress >= 1)
                                      Positioned(
                                        right: AppSpacing.space8,
                                        bottom: AppSpacing.space8,
                                        child: Tooltip(
                                          message: strings.get(
                                            'library.finished',
                                          ),
                                          child: Container(
                                            width: 12,
                                            height: 12,
                                            decoration: BoxDecoration(
                                              shape: BoxShape.circle,
                                              border: Border.all(
                                                color: context
                                                    .appColors
                                                    .textPrimary,
                                              ),
                                            ),
                                          ),
                                        ),
                                      ),
                                    if (widget.document.isFavorite)
                                      Positioned(
                                        right: AppSpacing.space6,
                                        top: AppSpacing.space6,
                                        child: Tooltip(
                                          message: strings.get(
                                            'library.favorite',
                                          ),
                                          child: Icon(
                                            Icons.bookmark_rounded,
                                            size: 17,
                                            color:
                                                context.appColors.textPrimary,
                                          ),
                                        ),
                                      ),
                                    if (!widget.selectionMode &&
                                        widget.document.hasFileChanged)
                                      Positioned(
                                        left: AppSpacing.space6,
                                        top: AppSpacing.space6,
                                        child: _ArtifactStateMarker(
                                          icon: Icons.sync_problem_rounded,
                                          label: strings.get(
                                            'library.fileChanged',
                                          ),
                                          tone: AppStatusTone.warning,
                                        ),
                                      )
                                    else if (!widget.selectionMode &&
                                        enrichmentStatus != null &&
                                        enrichmentStatus !=
                                            EnrichmentStatus.completed &&
                                        enrichmentStatus !=
                                            EnrichmentStatus.noText)
                                      Positioned(
                                        left: AppSpacing.space6,
                                        top: AppSpacing.space6,
                                        child: _ArtifactStateMarker(
                                          icon: switch (enrichmentStatus) {
                                            EnrichmentStatus.pending =>
                                              Icons.hourglass_empty_rounded,
                                            EnrichmentStatus.analyzing =>
                                              Icons.sync_rounded,
                                            EnrichmentStatus.review =>
                                              Icons.fact_check_outlined,
                                            EnrichmentStatus.failed =>
                                              Icons.error_outline_rounded,
                                            _ => Icons.info_outline_rounded,
                                          },
                                          label: switch (enrichmentStatus) {
                                            EnrichmentStatus.pending =>
                                              strings.get(
                                                'library.analysisPending',
                                              ),
                                            EnrichmentStatus.analyzing =>
                                              strings.get(
                                                'library.analysisInProgress',
                                              ),
                                            EnrichmentStatus.review =>
                                              strings.get(
                                                'library.metadataNeedsReview',
                                              ),
                                            EnrichmentStatus.failed =>
                                              strings.get(
                                                'library.analysisFailed',
                                              ),
                                            _ => strings.get(
                                              'library.analyzeMetadata',
                                            ),
                                          },
                                          tone:
                                              enrichmentStatus ==
                                                  EnrichmentStatus.failed
                                              ? AppStatusTone.error
                                              : enrichmentStatus ==
                                                    EnrichmentStatus.review
                                              ? AppStatusTone.warning
                                              : AppStatusTone.neutral,
                                        ),
                                      ),
                                    if (widget.selectionMode && widget.selected)
                                      Positioned(
                                        left: AppSpacing.space8,
                                        top: AppSpacing.space8,
                                        child: DecoratedBox(
                                          decoration: BoxDecoration(
                                            color: context
                                                .appColors
                                                .surfacePrimary,
                                            shape: BoxShape.circle,
                                            border: Border.all(
                                              color: context
                                                  .appColors
                                                  .librarySelectionAccent,
                                            ),
                                          ),
                                          child: Padding(
                                            padding: const EdgeInsets.all(2),
                                            child: Icon(
                                              Icons.check_rounded,
                                              size: 16,
                                              color: context
                                                  .appColors
                                                  .librarySelectionAccent,
                                            ),
                                          ),
                                        ),
                                      ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: AppSpacing.space6),
                          Tooltip(
                            message: widget.document.effectiveTitle,
                            child: Text(
                              widget.document.effectiveTitle,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              textAlign: TextAlign.center,
                              style: Theme.of(context).textTheme.labelSmall
                                  ?.copyWith(
                                    color: widget.selected
                                        ? context
                                              .appColors
                                              .librarySelectionAccent
                                        : context.appColors.textSecondary,
                                    fontWeight: widget.selected
                                        ? FontWeight.w600
                                        : FontWeight.w400,
                                  ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _ArtifactStateMarker extends StatelessWidget {
  const _ArtifactStateMarker({
    required this.icon,
    required this.label,
    required this.tone,
  });

  final IconData icon;
  final String label;
  final AppStatusTone tone;

  @override
  Widget build(BuildContext context) {
    final color = switch (tone) {
      AppStatusTone.neutral => context.appColors.textSecondary,
      AppStatusTone.success => context.appColors.success,
      AppStatusTone.warning => context.appColors.warning,
      AppStatusTone.error => context.appColors.danger,
    };
    return Tooltip(
      message: label,
      child: Semantics(
        label: label,
        child: Container(
          width: 26,
          height: 26,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: context.appColors.surfaceFloating,
            border: Border.all(color: color),
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
          child: Icon(icon, size: 15, color: color),
        ),
      ),
    );
  }
}

final class _ArtifactPlaceholder extends StatelessWidget {
  const _ArtifactPlaceholder({required this.document, this.failed = false});

  final LibraryDocument document;
  final bool failed;

  @override
  Widget build(BuildContext context) {
    final label = document.effectiveTitle.trim();
    final initial = label.isEmpty
        ? 'PDF'
        : label.characters.first.toUpperCase();
    return ColoredBox(
      color: context.appColors.surfaceSunken,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              failed || document.isMissing ? 'PDF' : initial,
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                color: context.appColors.textMuted,
                fontWeight: FontWeight.w400,
              ),
            ),
            if (failed || document.isMissing) ...[
              const SizedBox(height: AppSpacing.space8),
              Icon(
                document.isMissing
                    ? Icons.link_off_rounded
                    : Icons.broken_image_outlined,
                size: 18,
                color: context.appColors.textMuted,
              ),
            ],
          ],
        ),
      ),
    );
  }
}

final class DocumentInfoOverlay extends StatelessWidget {
  const DocumentInfoOverlay({
    super.key,
    required this.document,
    this.windowClass = AppWindowClass.expanded,
    this.interactive = false,
    this.onPrimary,
    this.onToggleFavorite,
    this.onShowInfo,
    this.onDismiss,
  });

  final LibraryDocument document;
  final AppWindowClass windowClass;
  final bool interactive;
  final VoidCallback? onPrimary;
  final VoidCallback? onToggleFavorite;
  final VoidCallback? onShowInfo;
  final VoidCallback? onDismiss;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final completed = document.readingProgress >= 1;
    final notStarted = document.readingProgress <= 0;
    final progressLine = completed
        ? strings.get(
            'library.finishedPages',
            parameters: {'count': document.pageCount},
          )
        : notStarted
        ? strings.get(
            'library.notStartedPages',
            parameters: {'count': document.pageCount},
          )
        : strings.get(
            'library.pageProgress',
            parameters: {
              'page': document.lastPageIndex + 1,
              'count': document.pageCount,
              'progress': (document.readingProgress * 100).round(),
            },
          );
    final compact = windowClass == AppWindowClass.compact;
    final medium = windowClass == AppWindowClass.medium;
    final content = ConstrainedBox(
      constraints: BoxConstraints(
        maxWidth: compact
            ? 620
            : medium
            ? 880
            : 1040,
      ),
      child: Material(
        color: Colors.transparent,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: context.appColors.surfaceFloating,
            border: Border.all(color: context.appColors.borderSubtle),
            borderRadius: BorderRadius.circular(
              compact ? AppRadii.panel : AppRadii.floating,
            ),
            boxShadow: MediaQuery.highContrastOf(context)
                ? null
                : [
                    BoxShadow(
                      color: context.appColors.ambientShadow,
                      blurRadius: AppShadow.floatingControlBlur,
                      offset: const Offset(0, AppShadow.floatingControlOffset),
                    ),
                  ],
          ),
          child: Padding(
            padding: EdgeInsets.fromLTRB(
              AppSpacing.space20,
              AppSpacing.space12,
              interactive ? AppSpacing.space12 : AppSpacing.space20,
              AppSpacing.space12,
            ),
            child: compact
                ? Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      _DocumentContextSummary(document: document),
                      const SizedBox(height: AppSpacing.space12),
                      _DocumentProgressColumn(
                        document: document,
                        progressLine: progressLine,
                      ),
                      if (interactive) ...[
                        const SizedBox(height: AppSpacing.space12),
                        _DocumentContextActions(
                          document: document,
                          onPrimary: onPrimary,
                          onToggleFavorite: onToggleFavorite,
                          onShowInfo: onShowInfo,
                          onDismiss: onDismiss,
                        ),
                      ],
                    ],
                  )
                : Row(
                    children: [
                      Expanded(
                        child: _DocumentContextSummary(document: document),
                      ),
                      const SizedBox(width: AppSpacing.space20),
                      SizedBox(
                        width: medium ? 156 : 190,
                        child: _DocumentProgressColumn(
                          document: document,
                          progressLine: progressLine,
                        ),
                      ),
                      if (interactive) ...[
                        const SizedBox(width: AppSpacing.space20),
                        SizedBox(
                          height: 48,
                          child: VerticalDivider(
                            width: 1,
                            color: context.appColors.borderSubtle,
                          ),
                        ),
                        const SizedBox(width: AppSpacing.space12),
                        _DocumentContextActions(
                          document: document,
                          onPrimary: onPrimary,
                          onToggleFavorite: onToggleFavorite,
                          onShowInfo: onShowInfo,
                          onDismiss: onDismiss,
                        ),
                      ],
                    ],
                  ),
          ),
        ),
      ),
    );
    if (!compact || !interactive || onDismiss == null) return content;
    return Dismissible(
      key: ValueKey('library-context-dismiss-${document.id}'),
      direction: DismissDirection.down,
      resizeDuration: AppMotion.whenAllowed(
        MediaQuery.disableAnimationsOf(context),
        AppMotion.standard,
      ),
      onDismissed: (_) => onDismiss!(),
      child: content,
    );
  }
}

final class _DocumentContextSummary extends StatelessWidget {
  const _DocumentContextSummary({required this.document});

  final LibraryDocument document;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final enrichmentStatus = document.enrichment?.status;
    final status = document.isMissing
        ? (
            icon: Icons.link_off_rounded,
            label: strings.get('library.fileNotFound'),
            tone: AppStatusTone.error,
          )
        : document.hasFileChanged
        ? (
            icon: Icons.sync_problem_rounded,
            label: strings.get('library.fileChanged'),
            tone: AppStatusTone.warning,
          )
        : enrichmentStatus == EnrichmentStatus.review
        ? (
            icon: Icons.fact_check_outlined,
            label: strings.get('library.metadataNeedsReview'),
            tone: AppStatusTone.warning,
          )
        : document.readingProgress >= 1
        ? (
            icon: Icons.check_circle_outline_rounded,
            label: strings.get('library.finished'),
            tone: AppStatusTone.success,
          )
        : document.readingProgress > 0
        ? (
            icon: Icons.menu_book_rounded,
            label: strings.get(
              'library.inProgress',
              parameters: {'value': (document.readingProgress * 100).round()},
            ),
            tone: AppStatusTone.neutral,
          )
        : (
            icon: Icons.fiber_new_outlined,
            label: strings.get('library.notStarted'),
            tone: AppStatusTone.neutral,
          );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Tooltip(
          message: document.effectiveTitle,
          child: Text(
            document.effectiveTitle,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        const SizedBox(height: AppSpacing.space2),
        Text(
          document.effectiveAuthor ?? strings.get('library.unknownAuthor'),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        const SizedBox(height: AppSpacing.space8),
        Wrap(
          spacing: AppSpacing.space8,
          runSpacing: AppSpacing.space6,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            AppStatusLabel(
              icon: status.icon,
              label: status.label,
              tone: status.tone,
            ),
            Text(
              _relativeArtifactDate(document.lastOpenedAt, strings),
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: context.appColors.textMuted,
              ),
            ),
            if (document.isFavorite)
              Icon(
                Icons.bookmark_rounded,
                size: 15,
                color: context.appColors.textSecondary,
              ),
          ],
        ),
        if (document.libraryNote case final note?) ...[
          const SizedBox(height: AppSpacing.space2),
          Text(
            note,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.labelSmall,
          ),
        ],
      ],
    );
  }
}

final class _DocumentProgressColumn extends StatelessWidget {
  const _DocumentProgressColumn({
    required this.document,
    required this.progressLine,
  });

  final LibraryDocument document;
  final String progressLine;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final progress = document.readingProgress.clamp(0.0, 1.0).toDouble();
    return Semantics(
      label: progressLine,
      value: '${(progress * 100).round()} %',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  strings.get('reader.progress'),
                  style: Theme.of(context).textTheme.labelSmall,
                ),
              ),
              Text(
                '${(progress * 100).round()} %',
                style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: context.appColors.textPrimary,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
          const SizedBox(height: AppSpacing.space6),
          ClipRRect(
            borderRadius: BorderRadius.circular(AppRadii.indicator),
            child: LinearProgressIndicator(
              value: progress,
              minHeight: 3,
              backgroundColor: context.appColors.borderSubtle,
              color: document.isMissing
                  ? context.appColors.warning
                  : context.appColors.librarySelectionAccent,
            ),
          ),
          const SizedBox(height: AppSpacing.space6),
          Text(
            progressLine,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.labelSmall,
          ),
        ],
      ),
    );
  }
}

final class _DocumentContextActions extends StatelessWidget {
  const _DocumentContextActions({
    required this.document,
    required this.onPrimary,
    required this.onToggleFavorite,
    required this.onShowInfo,
    required this.onDismiss,
  });

  final LibraryDocument document;
  final VoidCallback? onPrimary;
  final VoidCallback? onToggleFavorite;
  final VoidCallback? onShowInfo;
  final VoidCallback? onDismiss;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Wrap(
      alignment: WrapAlignment.end,
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: AppSpacing.space4,
      runSpacing: AppSpacing.space4,
      children: [
        if (onToggleFavorite != null)
          IconButton(
            key: const Key('library-context-favorite'),
            tooltip: strings.get(
              document.isFavorite
                  ? 'library.unmarkFavorite'
                  : 'library.markFavorite',
            ),
            onPressed: onToggleFavorite,
            icon: Icon(
              document.isFavorite
                  ? Icons.bookmark_rounded
                  : Icons.bookmark_border_rounded,
            ),
          ),
        if (onShowInfo != null)
          IconButton(
            key: const Key('library-context-info'),
            tooltip: strings.get('library.showInformation'),
            onPressed: onShowInfo,
            icon: const Icon(Icons.info_outline_rounded),
          ),
        FilledButton.icon(
          key: const Key('library-context-primary'),
          onPressed: onPrimary,
          icon: Icon(
            document.isMissing
                ? Icons.link_rounded
                : Icons.arrow_forward_rounded,
            size: 18,
          ),
          label: Text(
            document.isMissing
                ? strings.get('library.locate')
                : document.readingProgress >= 1
                ? strings.get('action.reread')
                : document.readingProgress > 0
                ? strings.get('action.continue')
                : strings.get('action.open'),
          ),
        ),
        if (onDismiss != null)
          IconButton(
            key: const Key('library-context-close'),
            tooltip: strings.get('library.closeContext'),
            onPressed: onDismiss,
            icon: const Icon(Icons.close_rounded),
          ),
      ],
    );
  }
}

final class _FilteredSelectionNotice extends StatelessWidget {
  const _FilteredSelectionNotice({
    required this.onReset,
    required this.onDismiss,
  });

  final VoidCallback onReset;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Material(
      color: context.appColors.surfaceFloating,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadii.floating),
        side: BorderSide(color: context.appColors.borderSubtle),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space12,
          vertical: AppSpacing.space8,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.filter_alt_off_outlined, size: 18),
            const SizedBox(width: AppSpacing.space8),
            Flexible(child: Text(strings.get('library.selectionFiltered'))),
            const SizedBox(width: AppSpacing.space8),
            TextButton(
              key: const Key('library-selection-show-again'),
              onPressed: onReset,
              child: Text(strings.get('library.showSelectionAgain')),
            ),
            IconButton(
              tooltip: strings.get('library.closeContext'),
              onPressed: onDismiss,
              icon: const Icon(Icons.close_rounded),
            ),
          ],
        ),
      ),
    );
  }
}

final class _SearchOverlay extends StatelessWidget {
  const _SearchOverlay({
    required this.controller,
    required this.focusNode,
    required this.resultCount,
    required this.onChanged,
    required this.onClose,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final int resultCount;
  final ValueChanged<String> onChanged;
  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Material(
      color: context.appColors.surfaceElevated,
      elevation: AppElevation.popup,
      shadowColor: context.appColors.readerPageShadow,
      borderRadius: BorderRadius.circular(AppRadii.field),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 500),
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.space8),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  key: const Key('library-filter'),
                  controller: controller,
                  focusNode: focusNode,
                  textInputAction: TextInputAction.search,
                  decoration: InputDecoration(
                    hintText: strings.librarySearch,
                    prefixIcon: const Icon(Icons.search_rounded),
                  ),
                  onChanged: onChanged,
                ),
              ),
              const SizedBox(width: AppSpacing.space8),
              Semantics(
                liveRegion: true,
                label: strings.count('count.results', resultCount),
                child: Text(
                  '$resultCount',
                  style: Theme.of(context).textTheme.labelMedium,
                ),
              ),
              IconButton(
                key: const Key('library-filter-clear'),
                tooltip: strings.get('library.closeSearch'),
                onPressed: onClose,
                icon: const Icon(Icons.close_rounded),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

final class _NoArtifactResults extends StatelessWidget {
  const _NoArtifactResults({required this.query, required this.onReset});

  final String query;
  final VoidCallback onReset;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Center(
      child: Semantics(
        liveRegion: true,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.search_off_rounded, color: context.appColors.textMuted),
            const SizedBox(height: AppSpacing.space12),
            Text(
              query.isEmpty
                  ? strings.libraryNoResults
                  : strings.get(
                      'library.noQueryResults',
                      parameters: {'query': query},
                    ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: AppSpacing.space8),
            TextButton(
              onPressed: onReset,
              child: Text(strings.get('action.reset')),
            ),
          ],
        ),
      ),
    );
  }
}

enum _ArtifactMenuAction {
  open,
  favorite,
  cover,
  reanalyze,
  info,
  reveal,
  remove,
}

@visibleForTesting
String? artifactCoverPathFor(
  LibraryDocument document, {
  required bool selected,
}) => selected
    ? document.coverMetadata?.largeThumbnailPath ?? document.thumbnailPath
    : document.coverMetadata?.smallThumbnailPath ?? document.thumbnailPath;

enum _FilterCommand {
  all,
  inProgress,
  notStarted,
  completed,
  favorites,
  lastOpened,
  dateAdded,
  title,
  progress,
  reset,
}

LibraryDocument? _documentById(
  Iterable<LibraryDocument> documents,
  String? id,
) {
  if (id == null) return null;
  for (final document in documents) {
    if (document.id == id) return document;
  }
  return null;
}

String _artifactSemanticLabel(
  LibraryDocument document,
  YamantakaLocalizations strings,
) {
  final title = document.effectiveTitle;
  if (document.isMissing) {
    return strings.get('library.artifactMissing', parameters: {'title': title});
  }
  if (document.readingProgress >= 1) {
    return strings.get(
      'library.artifactFinished',
      parameters: {'title': title},
    );
  }
  if (document.readingProgress > 0) {
    return strings.get(
      'library.artifactProgress',
      parameters: {
        'title': title,
        'progress': (document.readingProgress * 100).round(),
      },
    );
  }
  return strings.get(
    'library.artifactNotStarted',
    parameters: {'title': title},
  );
}

List<double> _saturationMatrix(double saturation) {
  final inverse = 1 - saturation;
  final red = 0.213 * inverse;
  final green = 0.715 * inverse;
  final blue = 0.072 * inverse;
  return <double>[
    red + saturation,
    green,
    blue,
    0,
    0,
    red,
    green + saturation,
    blue,
    0,
    0,
    red,
    green,
    blue + saturation,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
  ];
}

String _relativeArtifactDate(DateTime value, YamantakaLocalizations strings) {
  final difference = DateTime.now().difference(value.toLocal());
  if (difference.inMinutes < 1) return strings.get('time.openedNow');
  if (difference.inDays < 1) return strings.get('time.openedToday');
  if (difference.inDays == 1) return strings.get('time.openedYesterday');
  if (difference.inDays < 7) {
    return strings.get(
      'time.openedDaysAgo',
      parameters: {'count': difference.inDays},
    );
  }
  final local = value.toLocal();
  return strings.get(
    'time.openedDate',
    parameters: {
      'day': local.day.toString().padLeft(2, '0'),
      'month': local.month.toString().padLeft(2, '0'),
    },
  );
}
