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
  collection,
  query,
  readingState,
  favorites,
  sort,
  authors,
  topics,
  grouping,
}

enum _LibraryDisplayCommand { grid, list, selection, facets }

final class _LibraryCommandBar extends StatelessWidget {
  const _LibraryCommandBar({
    required this.resultCount,
    required this.totalCount,
    required this.query,
    this.collectionLabel,
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
  final String? collectionLabel;
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
    final search = IconButton(
      key: const Key('library-search-button'),
      tooltip: query.isEmpty ? strings.librarySearch : query,
      onPressed: onSearch,
      icon: const Icon(Icons.search_rounded, size: 18),
    );
    final tools = YamaToolbarGroup(
      label: strings.get('library.commandBar'),
      flat: MediaQuery.highContrastOf(context),
      bare: true,
      padding: EdgeInsets.zero,
      children: [
        search,
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
        PopupMenuButton<_LibraryDisplayCommand>(
          key: const Key('library-display-menu-button'),
          tooltip: strings.get('library.commandBar'),
          onSelected: (command) {
            switch (command) {
              case _LibraryDisplayCommand.grid:
                onViewModeChanged(_LibraryViewMode.grid);
              case _LibraryDisplayCommand.list:
                onViewModeChanged(_LibraryViewMode.list);
              case _LibraryDisplayCommand.selection:
                onToggleSelectionMode();
              case _LibraryDisplayCommand.facets:
                onFacets();
            }
          },
          icon: Badge(
            isLabelVisible: selectedCount > 0,
            label: Text('$selectedCount'),
            child: const Icon(Icons.more_vert_rounded),
          ),
          itemBuilder: (context) => [
            CheckedPopupMenuItem(
              key: const Key('library-display-grid'),
              value: _LibraryDisplayCommand.grid,
              checked: viewMode == _LibraryViewMode.grid,
              child: Text(strings.get('library.view.grid')),
            ),
            CheckedPopupMenuItem(
              key: const Key('library-display-list'),
              value: _LibraryDisplayCommand.list,
              checked: viewMode == _LibraryViewMode.list,
              child: Text(strings.get('library.view.list')),
            ),
            const PopupMenuDivider(),
            CheckedPopupMenuItem(
              key: const Key('library-display-selection'),
              value: _LibraryDisplayCommand.selection,
              checked: selectionMode,
              child: Text(
                selectionMode
                    ? strings.get('library.exitSelectionMode')
                    : strings.get('library.selectMultiple'),
              ),
            ),
            PopupMenuItem(
              key: const Key('library-display-facets'),
              value: _LibraryDisplayCommand.facets,
              child: Text(strings.get('library.authorsTopicsGrouping')),
            ),
          ],
        ),
      ],
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        return Padding(
          padding: const EdgeInsets.fromLTRB(
            AppSpacing.space20,
            AppSpacing.space6,
            AppSpacing.space20,
            0,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Semantics(
                    liveRegion: query.isNotEmpty || activeFilter,
                    label: strings.count(
                      resultCount == totalCount
                          ? 'count.documents'
                          : 'count.results',
                      resultCount,
                    ),
                    child: const SizedBox.shrink(),
                  ),
                  if (activeFilter)
                    IconButton(
                      key: const Key('library-reset-filters-button'),
                      tooltip: strings.get('library.resetFilters'),
                      onPressed: () => onFilter(_FilterCommand.reset),
                      icon: Badge(
                        label: Text('$activeFilterCount'),
                        child: const Icon(Icons.filter_alt_off_outlined),
                      ),
                    ),
                  tools,
                ],
              ),
              if (activeFilter) ...[
                const SizedBox(height: AppSpacing.space8),
                _ActiveFilterStrip(
                  query: query,
                  collectionLabel: collectionLabel,
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
    this.collectionLabel,
    required this.readingState,
    required this.sortOrder,
    required this.favoritesOnly,
    required this.selectedAuthorCount,
    required this.selectedTopicCount,
    required this.grouping,
    required this.onClear,
  });

  final String query;
  final String? collectionLabel;
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
            if (collectionLabel != null)
              _ActiveFilterChip(
                label: collectionLabel!,
                onDeleted: () => onClear(_ActiveFilterKind.collection),
              ),
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
                        mainAxisExtent:
                            metrics.cellExtent +
                            38 *
                                (MediaQuery.textScalerOf(context).scale(1) - 1),
                        mainAxisSpacing: AppSpacing.space16,
                        crossAxisSpacing: AppSpacing.space16,
                      ),
                      itemBuilder: (context, index) {
                        final document = entry.value[index];
                        return Center(
                          child: _artifact(
                            document,
                            metrics.artifactWidth,
                            entranceIndex: index,
                          ),
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
                      for (final (index, document) in documents.indexed)
                        SizedBox(
                          width: metrics.artifactWidth,
                          height:
                              metrics.cellExtent +
                              38 *
                                  (MediaQuery.textScalerOf(context).scale(1) -
                                      1),
                          child: Center(
                            child: _artifact(
                              document,
                              metrics.artifactWidth,
                              entranceIndex: index,
                            ),
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
                  mainAxisExtent:
                      metrics.cellExtent +
                      38 * (MediaQuery.textScalerOf(context).scale(1) - 1),
                  mainAxisSpacing: AppSpacing.space16,
                  crossAxisSpacing: AppSpacing.space16,
                ),
                itemBuilder: (context, index) {
                  final document = documents[index];
                  return Center(
                    child: _artifact(
                      document,
                      metrics.artifactWidth,
                      entranceIndex: index,
                    ),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _artifact(
    LibraryDocument document,
    double width, {
    int entranceIndex = 0,
  }) {
    Widget buildArtifact(bool selected) => ArtifactDocument(
      key: ValueKey('artifact-${document.id}'),
      document: document,
      width: width,
      selected: selected,
      selectionMode: selectionMode,
      reduceMotion: reduceMotion,
      entranceIndex: entranceIndex,
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
                  border: _focused
                      ? Border.all(color: colors.focusRing, width: 2)
                      : Border(
                          left: BorderSide(
                            color: widget.selected
                                ? colors.librarySelectionAccent
                                : Colors.transparent,
                            width: 2,
                          ),
                          bottom: BorderSide(color: colors.borderSubtle),
                        ),
                  borderRadius: _focused
                      ? BorderRadius.circular(AppRadii.control)
                      : null,
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
              child: Icon(
                Icons.description_outlined,
                size: 18,
                color: context.appColors.textMuted,
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
    this.entranceIndex = 0,
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
  final int entranceIndex;
  final VoidCallback? onVisible;
  final VoidCallback? onHidden;

  @override
  State<ArtifactDocument> createState() => _ArtifactDocumentState();
}

final class _ArtifactDocumentState extends State<ArtifactDocument> {
  bool _hovered = false;
  bool _focused = false;
  bool _entered = false;
  bool _reportedVisible = false;
  Offset _pointerTilt = Offset.zero;
  Timer? _entranceTimer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _reportedVisible = true;
      widget.onVisible?.call();
      if (widget.reduceMotion) {
        setState(() => _entered = true);
        return;
      }
      final delay = Duration(
        milliseconds: math.min(widget.entranceIndex, 7) * 18,
      );
      _entranceTimer = Timer(delay, () {
        if (mounted) setState(() => _entered = true);
      });
    });
  }

  @override
  void dispose() {
    _entranceTimer?.cancel();
    if (_reportedVisible) widget.onHidden?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final highContrast = MediaQuery.highContrastOf(context);
    final fittedHeight = widget.width * 1.42;
    final coverPath = artifactCoverPathFor(
      widget.document,
      selected: widget.selected,
    );
    final enrichmentStatus = widget.document.enrichment?.status;
    final motionDisabled = widget.reduceMotion || highContrast;
    final emphasized = widget.selected || _hovered || _focused;
    final scale = motionDisabled
        ? 1.0
        : widget.selected
        ? AppSizes.artifactSelectedScale
        : emphasized
        ? AppSizes.artifactHoverScale
        : 1.0;
    final lift = motionDisabled
        ? 0.0
        : widget.selected
        ? AppSizes.artifactSelectedLift
        : emphasized
        ? AppSizes.artifactHoverLift
        : 0.0;
    final animatedScale = _entered || motionDisabled ? scale : 0.985;
    // Keep the cover pixel-aligned at rest. A permanent Y rotation makes the
    // long top and bottom edges land on fractional pixels, which is visibly
    // stair-stepped by the Windows rasterizer. Depth remains in the spine,
    // page block and shadow; perspective is revealed during pointer movement.
    final bookAngle = highContrast || motionDisabled
        ? 0.0
        : !_entered
        ? -0.035
        : 0.0;
    final transform = Matrix4.identity()
      ..setEntry(3, 2, 0.0012)
      ..translateByDouble(0, _entered || motionDisabled ? -lift : 8, 0, 1)
      ..rotateX(motionDisabled ? 0 : -_pointerTilt.dy * 0.025)
      ..rotateY(bookAngle + (motionDisabled ? 0 : _pointerTilt.dx * 0.038))
      ..scaleByDouble(animatedScale, animatedScale, 1, 1);
    final transformDuration = motionDisabled
        ? AppMotion.instant
        : _hovered
        ? AppMotion.fast
        : AppMotion.standard;

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
          onHover: (event) {
            if (motionDisabled) return;
            if (event.localPosition.dy > fittedHeight) {
              if (_pointerTilt != Offset.zero) {
                setState(() => _pointerTilt = Offset.zero);
              }
              return;
            }
            final next = Offset(
              ((event.localPosition.dx / widget.width) - 0.5).clamp(-0.5, 0.5) *
                  2,
              ((event.localPosition.dy / fittedHeight) - 0.5).clamp(-0.5, 0.5) *
                  2,
            );
            if ((next - _pointerTilt).distance < 0.08) return;
            setState(() => _pointerTilt = next);
          },
          onExit: (_) {
            setState(() {
              _hovered = false;
              _pointerTilt = Offset.zero;
            });
            widget.onHover(false);
          },
          child: Listener(
            onPointerDown: (_) => widget.onSelect(),
            child: GestureDetector(
              onDoubleTap: widget.onOpen,
              onSecondaryTapDown: (details) =>
                  widget.onMenu(details.globalPosition),
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
                                    AppRadii.artifactCover + 3,
                                  ),
                                ),
                              ),
                            ),
                          ),
                        if (!highContrast)
                          Positioned.fill(
                            left: 7,
                            top: 8,
                            right: -5,
                            bottom: -5,
                            child: IgnorePointer(
                              child: _ArtifactBookPageBlock(
                                duration: transformDuration,
                                transform: Matrix4.copy(transform),
                              ),
                            ),
                          ),
                        _ArtifactTransform(
                          transformKey: ValueKey(
                            'artifact-book-transform-${widget.document.id}',
                          ),
                          duration: transformDuration,
                          transform: transform,
                          child: AnimatedContainer(
                            key: ValueKey(
                              'artifact-book-face-${widget.document.id}',
                            ),
                            // Grid reflow changes the cover dimensions. A
                            // layout tween can temporarily exceed the new
                            // cell when filters alter the column count.
                            duration: motionDisabled
                                ? AppMotion.instant
                                : AppMotion.standard,
                            curve: Curves.easeOutCubic,
                            width: widget.width,
                            height: fittedHeight,
                            decoration: BoxDecoration(
                              color: context.appColors.surfaceSunken,
                              borderRadius: BorderRadius.circular(
                                AppRadii.artifactCover,
                              ),
                              border: Border.all(
                                color: widget.selected
                                    ? context.appColors.librarySelectionAccent
                                    : _hovered
                                    ? context.appColors.borderStrong
                                    : context.appColors.borderSubtle,
                                width: widget.selected ? 1.5 : 1,
                              ),
                              boxShadow: motionDisabled
                                  ? null
                                  : [
                                      if (widget.selected)
                                        BoxShadow(
                                          color: context
                                              .appColors
                                              .librarySelectionAccent
                                              .withValues(alpha: 0.10),
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
                                                  ? 0.16
                                                  : 0.08,
                                            ),
                                        blurRadius: widget.selected
                                            ? AppShadow.artifactSelectedBlur
                                            : AppShadow.artifactRestBlur,
                                        spreadRadius: widget.selected ? 1 : 0,
                                        offset: Offset(
                                          0,
                                          widget.selected
                                              ? AppShadow.artifactSelectedOffset
                                              : AppShadow.artifactRestOffset,
                                        ),
                                      ),
                                    ],
                            ),
                            clipBehavior: Clip.antiAliasWithSaveLayer,
                            child: Stack(
                              fit: StackFit.expand,
                              children: [
                                coverPath == null
                                    ? _ArtifactPlaceholder(
                                        document: widget.document,
                                        active: emphasized,
                                        reduceMotion: motionDisabled,
                                      )
                                    : _ArtifactCoverImage(
                                        path: coverPath,
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
                                        error: _ArtifactPlaceholder(
                                          document: widget.document,
                                          failed: true,
                                          active: emphasized,
                                          reduceMotion: motionDisabled,
                                        ),
                                      ),
                                if (!highContrast)
                                  _ArtifactFrontFinish(
                                    active: emphasized,
                                    reduceMotion: motionDisabled,
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
                                            : context.appColors.textSecondary,
                                        child: const SizedBox(height: 4),
                                      ),
                                    ),
                                  ),
                                if (widget.document.readingProgress >= 1)
                                  Positioned(
                                    right: AppSpacing.space8,
                                    bottom: AppSpacing.space8,
                                    child: Tooltip(
                                      message: strings.get('library.finished'),
                                      child: Container(
                                        width: 12,
                                        height: 12,
                                        decoration: BoxDecoration(
                                          shape: BoxShape.circle,
                                          border: Border.all(
                                            color:
                                                context.appColors.textPrimary,
                                          ),
                                        ),
                                      ),
                                    ),
                                  ),
                                if (widget.document.isFavorite)
                                  Positioned(
                                    right: AppSpacing.space6,
                                    top: widget.selected
                                        ? 40
                                        : AppSpacing.space6,
                                    child: Tooltip(
                                      message: strings.get('library.favorite'),
                                      child: Icon(
                                        Icons.bookmark_rounded,
                                        size: 17,
                                        color: context.appColors.textPrimary,
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
                                      label: strings.get('library.fileChanged'),
                                      tone: AppStatusTone.warning,
                                    ),
                                  )
                                else if (!widget.selectionMode &&
                                    enrichmentStatus != null &&
                                    enrichmentStatus !=
                                        EnrichmentStatus.completed &&
                                    enrichmentStatus != EnrichmentStatus.noText)
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
                                        EnrichmentStatus.pending => strings.get(
                                          'library.analysisPending',
                                        ),
                                        EnrichmentStatus.analyzing =>
                                          strings.get(
                                            'library.analysisInProgress',
                                          ),
                                        EnrichmentStatus.review => strings.get(
                                          'library.metadataNeedsReview',
                                        ),
                                        EnrichmentStatus.failed => strings.get(
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
                                if (widget.selected)
                                  Positioned(
                                    right: AppSpacing.space8,
                                    top: AppSpacing.space8,
                                    child: DecoratedBox(
                                      decoration: BoxDecoration(
                                        color: context.appColors.surfacePrimary,
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
                        ),
                      ],
                    ),
                    const SizedBox(height: AppSpacing.space6),
                    if (coverPath == null)
                      SizedBox(
                        height: 38 * MediaQuery.textScalerOf(context).scale(1),
                      )
                    else
                      SizedBox(
                        height: 38 * MediaQuery.textScalerOf(context).scale(1),
                        child: Tooltip(
                          message: widget.document.effectiveTitle,
                          child: Text(
                            widget.document.effectiveTitle,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            textAlign: TextAlign.start,
                            style: Theme.of(context).textTheme.labelLarge
                                ?.copyWith(
                                  color: context.appColors.textPrimary,
                                  fontWeight: widget.selected
                                      ? FontWeight.w600
                                      : FontWeight.w500,
                                ),
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

final class _ArtifactTransform extends StatelessWidget {
  const _ArtifactTransform({
    required this.duration,
    required this.transform,
    required this.child,
    this.transformKey,
  });

  final Duration duration;
  final Matrix4 transform;
  final Widget child;
  final Key? transformKey;

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<Matrix4>(
      duration: duration,
      curve: Curves.easeOutCubic,
      tween: Matrix4Tween(end: transform),
      builder: (context, value, child) => Transform(
        key: transformKey,
        transform: value,
        alignment: Alignment.center,
        filterQuality: FilterQuality.high,
        child: child,
      ),
      child: child,
    );
  }
}

final class _ArtifactBookPageBlock extends StatelessWidget {
  const _ArtifactBookPageBlock({
    required this.duration,
    required this.transform,
  });

  final Duration duration;
  final Matrix4 transform;

  @override
  Widget build(BuildContext context) {
    final dark = Theme.of(context).brightness == Brightness.dark;
    final paper = dark ? const Color(0xFF9B998F) : const Color(0xFFE7E3D6);
    final paperShadow = dark
        ? const Color(0xFF55554F)
        : const Color(0xFFB5B1A5);
    return _ArtifactTransform(
      duration: duration,
      transform: transform,
      child: Container(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [paper, Color.lerp(paper, paperShadow, 0.72)!],
          ),
          borderRadius: BorderRadius.circular(AppRadii.artifactCover),
          border: Border.all(color: paperShadow.withValues(alpha: 0.86)),
        ),
        clipBehavior: Clip.antiAliasWithSaveLayer,
        child: CustomPaint(
          painter: _ArtifactPageEdgesPainter(
            lineColor: paperShadow.withValues(alpha: dark ? 0.58 : 0.42),
          ),
        ),
      ),
    );
  }
}

final class _ArtifactPageEdgesPainter extends CustomPainter {
  const _ArtifactPageEdgesPainter({required this.lineColor});

  final Color lineColor;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = lineColor
      ..strokeWidth = 0.65;
    for (var y = 4.0; y < size.height; y += 4.0) {
      final inset = (y.round() % 12 == 0) ? 0.0 : 1.5;
      canvas.drawLine(Offset(inset, y), Offset(size.width - inset, y), paint);
    }
    final edge = Paint()
      ..color = lineColor.withValues(alpha: 0.85)
      ..strokeWidth = 1.2;
    canvas.drawLine(
      Offset(size.width - 2, 3),
      Offset(size.width - 2, size.height - 3),
      edge,
    );
  }

  @override
  bool shouldRepaint(_ArtifactPageEdgesPainter oldDelegate) =>
      lineColor != oldDelegate.lineColor;
}

final class _ArtifactCoverImage extends StatelessWidget {
  const _ArtifactCoverImage({
    required this.path,
    required this.cacheWidth,
    required this.cacheHeight,
    required this.error,
  });

  final String path;
  final int cacheWidth;
  final int cacheHeight;
  final Widget error;

  @override
  Widget build(BuildContext context) {
    return Stack(
      fit: StackFit.expand,
      children: [
        Image.file(
          File(path),
          fit: BoxFit.cover,
          alignment: Alignment.topCenter,
          cacheWidth: cacheWidth,
          cacheHeight: cacheHeight,
          gaplessPlayback: true,
          filterQuality: FilterQuality.medium,
          errorBuilder: (_, _, _) => error,
        ),
        IgnorePointer(
          child: DecoratedBox(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.centerLeft,
                end: Alignment.centerRight,
                stops: const [0, 0.035, 0.10, 1],
                colors: [
                  Colors.black.withValues(alpha: 0.32),
                  Colors.black.withValues(alpha: 0.10),
                  Colors.transparent,
                  Colors.black.withValues(alpha: 0.04),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}

final class _ArtifactFrontFinish extends StatelessWidget {
  const _ArtifactFrontFinish({
    required this.active,
    required this.reduceMotion,
  });

  final bool active;
  final bool reduceMotion;

  @override
  Widget build(BuildContext context) {
    return IgnorePointer(
      child: Stack(
        fit: StackFit.expand,
        children: [
          Align(
            alignment: Alignment.centerLeft,
            child: Container(
              width: 9,
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  colors: [
                    Colors.black.withValues(alpha: 0.28),
                    Colors.white.withValues(alpha: 0.08),
                    Colors.transparent,
                  ],
                ),
              ),
            ),
          ),
          AnimatedOpacity(
            duration: reduceMotion ? AppMotion.instant : AppMotion.panel,
            curve: Curves.easeOutCubic,
            opacity: active ? 0.16 : 0.07,
            child: DecoratedBox(
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                  stops: const [0, 0.30, 0.48, 1],
                  colors: [
                    Colors.white,
                    Colors.transparent,
                    Colors.transparent,
                    Colors.black,
                  ],
                ),
              ),
            ),
          ),
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              height: 1,
              margin: const EdgeInsets.only(left: 8),
              color: Colors.white.withValues(alpha: 0.20),
            ),
          ),
        ],
      ),
    );
  }
}

final class _ArtifactPlaceholder extends StatefulWidget {
  const _ArtifactPlaceholder({
    required this.document,
    required this.active,
    required this.reduceMotion,
    this.failed = false,
  });

  final LibraryDocument document;
  final bool active;
  final bool reduceMotion;
  final bool failed;

  @override
  State<_ArtifactPlaceholder> createState() => _ArtifactPlaceholderState();
}

final class _ArtifactPlaceholderState extends State<_ArtifactPlaceholder> {
  Object? _appearance;
  Widget? _content;

  @override
  Widget build(BuildContext context) {
    final document = widget.document;
    final colors = context.appColors;
    final strings = YamantakaLocalizations.of(context);
    final textTheme = Theme.of(context).textTheme;
    final textScaler = MediaQuery.textScalerOf(context);
    final highContrast = MediaQuery.highContrastOf(context);
    final formatLabel = document.isMissing
        ? strings.get('library.locate')
        : 'PDF';
    final seed = _artifactVisualSeed(document.id);
    final palette = highContrast
        ? _ArtifactCoverPalette.highContrast(colors)
        : _ArtifactCoverPalette.forSeed(
            seed,
            darkMode: Theme.of(context).brightness == Brightness.dark,
          );
    final appearance = (
      document.id,
      document.effectiveTitle,
      document.effectiveAuthor,
      document.pageCount,
      document.isMissing,
      widget.failed,
      colors,
      textTheme,
      textScaler,
      highContrast,
      formatLabel,
      widget.active,
      widget.reduceMotion,
    );
    // Density changes outside the cover do not change its editorial content.
    // Keep one subtree per mounted cover, not a document/global cache.
    if (_appearance == appearance && _content != null) return _content!;
    _appearance = appearance;
    final largeText = textScaler.scale(1) > 1.4;
    final fallbackIcon = document.isMissing
        ? Icons.link_off_rounded
        : widget.failed
        ? Icons.broken_image_outlined
        : null;
    return _content = RepaintBoundary(
      child: DecoratedBox(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [palette.background, palette.backgroundEnd],
          ),
          border: BorderDirectional(
            start: BorderSide(color: palette.accent, width: 6),
          ),
        ),
        child: Stack(
          fit: StackFit.expand,
          children: [
            ExcludeSemantics(
              child: CustomPaint(
                painter: _ArtifactCoverPatternPainter(
                  seed: seed,
                  color: palette.pattern,
                ),
              ),
            ),
            if (!highContrast)
              _ArtifactCoverSheen(
                active: widget.active,
                reduceMotion: widget.reduceMotion,
                color: palette.accent,
              ),
            Padding(
              padding: const EdgeInsets.fromLTRB(
                AppSpacing.space16,
                AppSpacing.space12,
                AppSpacing.space12,
                AppSpacing.space16,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Text(
                        formatLabel,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: textTheme.labelSmall?.copyWith(
                          color: palette.inkMuted,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 1.2,
                        ),
                      ),
                      const Spacer(),
                      Icon(
                        Icons.layers_outlined,
                        size: AppIconSizes.compact,
                        color: palette.inkMuted,
                      ),
                      const SizedBox(width: AppSpacing.space4),
                      Text(
                        '${document.pageCount}',
                        style: textTheme.labelSmall?.copyWith(
                          color: palette.inkMuted,
                          fontFeatures: const [FontFeature.tabularFigures()],
                        ),
                      ),
                      const SizedBox(width: AppSpacing.space24),
                    ],
                  ),
                  Expanded(
                    child: Center(
                      child: ExcludeSemantics(
                        child: _ArtifactCoverObject(
                          variant: seed % 6,
                          palette: palette,
                          active: widget.active,
                          reduceMotion: widget.reduceMotion,
                          highContrast: highContrast,
                          compact: largeText,
                          fallbackIcon: fallbackIcon,
                        ),
                      ),
                    ),
                  ),
                  Text(
                    document.effectiveTitle,
                    maxLines: largeText ? 2 : 3,
                    overflow: TextOverflow.ellipsis,
                    style: textTheme.titleMedium?.copyWith(
                      color: palette.ink,
                      fontWeight: FontWeight.w700,
                      height: 1.08,
                      letterSpacing: -0.2,
                    ),
                  ),
                  if (!largeText &&
                      document.effectiveAuthor?.isNotEmpty == true) ...[
                    const SizedBox(height: AppSpacing.space8),
                    Text(
                      document.effectiveAuthor!,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: textTheme.bodySmall?.copyWith(
                        color: palette.inkMuted,
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _ArtifactCoverSheen extends StatelessWidget {
  const _ArtifactCoverSheen({
    required this.active,
    required this.reduceMotion,
    required this.color,
  });

  final bool active;
  final bool reduceMotion;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return IgnorePointer(
      child: AnimatedAlign(
        duration: reduceMotion ? AppMotion.instant : AppMotion.panel,
        curve: Curves.easeOutCubic,
        alignment: active
            ? const Alignment(1.7, -0.8)
            : const Alignment(-1.7, 0.8),
        child: Transform.rotate(
          angle: -0.34,
          child: Container(
            width: 28,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Colors.transparent,
                  color.withValues(alpha: active ? 0.16 : 0.05),
                  Colors.white.withValues(alpha: active ? 0.13 : 0.03),
                  Colors.transparent,
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _ArtifactCoverObject extends StatelessWidget {
  const _ArtifactCoverObject({
    required this.variant,
    required this.palette,
    required this.active,
    required this.reduceMotion,
    required this.highContrast,
    required this.compact,
    this.fallbackIcon,
  });

  final int variant;
  final _ArtifactCoverPalette palette;
  final bool active;
  final bool reduceMotion;
  final bool highContrast;
  final bool compact;
  final IconData? fallbackIcon;

  @override
  Widget build(BuildContext context) {
    final duration = reduceMotion ? AppMotion.instant : AppMotion.panel;
    final objectSize = compact ? 66.0 : 96.0;
    final turns = ((variant % 3) - 1) * 0.006;
    return AnimatedSlide(
      duration: duration,
      curve: Curves.easeOutCubic,
      offset: active && !reduceMotion ? const Offset(0, -0.055) : Offset.zero,
      child: AnimatedScale(
        duration: duration,
        curve: Curves.easeOutBack,
        scale: active && !reduceMotion ? 1.075 : 1,
        child: AnimatedRotation(
          duration: duration,
          curve: Curves.easeOutCubic,
          turns: active && !reduceMotion ? -turns : turns,
          child: SizedBox.square(
            dimension: objectSize,
            child: Stack(
              fit: StackFit.expand,
              children: [
                CustomPaint(
                  painter: fallbackIcon == null
                      ? _ArtifactCoverObjectPainter(
                          variant: variant,
                          palette: palette,
                          highContrast: highContrast,
                        )
                      : _ArtifactUnavailableObjectPainter(
                          palette: palette,
                          highContrast: highContrast,
                        ),
                ),
                if (fallbackIcon != null)
                  Center(
                    child: Icon(
                      fallbackIcon,
                      size: compact ? 26 : 34,
                      color: highContrast ? palette.background : palette.ink,
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

final class _ArtifactUnavailableObjectPainter extends CustomPainter {
  const _ArtifactUnavailableObjectPainter({
    required this.palette,
    required this.highContrast,
  });

  final _ArtifactCoverPalette palette;
  final bool highContrast;

  @override
  void paint(Canvas canvas, Size size) {
    final ink = highContrast ? palette.ink : palette.emblem;
    final shadow = highContrast
        ? palette.ink
        : Color.lerp(palette.emblem, Colors.black, 0.52)!;
    final rear = RRect.fromRectAndRadius(
      Rect.fromLTWH(
        size.width * 0.24,
        size.height * 0.18,
        size.width * 0.58,
        size.height * 0.65,
      ),
      const Radius.circular(8),
    );
    final front = RRect.fromRectAndRadius(
      Rect.fromLTWH(
        size.width * 0.16,
        size.height * 0.10,
        size.width * 0.58,
        size.height * 0.65,
      ),
      const Radius.circular(8),
    );
    canvas.drawShadow(Path()..addRRect(rear), Colors.black, 8, false);
    canvas.drawRRect(rear, Paint()..color = shadow);
    canvas.drawRRect(front, Paint()..color = ink);
    final edge = Paint()
      ..color = palette.accent
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawRRect(front, edge);
    final tear = Path()
      ..moveTo(size.width * 0.50, size.height * 0.10)
      ..lineTo(size.width * 0.58, size.height * 0.22)
      ..lineTo(size.width * 0.51, size.height * 0.31)
      ..lineTo(size.width * 0.58, size.height * 0.42);
    canvas.drawPath(
      tear,
      Paint()
        ..color = palette.background
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.2
        ..strokeCap = StrokeCap.round,
    );
  }

  @override
  bool shouldRepaint(_ArtifactUnavailableObjectPainter oldDelegate) =>
      palette != oldDelegate.palette ||
      highContrast != oldDelegate.highContrast;
}

final class _ArtifactCoverObjectPainter extends CustomPainter {
  const _ArtifactCoverObjectPainter({
    required this.variant,
    required this.palette,
    required this.highContrast,
  });

  final int variant;
  final _ArtifactCoverPalette palette;
  final bool highContrast;

  Color get _light => Color.lerp(palette.emblem, Colors.white, 0.42)!;
  Color get _mid => palette.emblem;
  Color get _dark => Color.lerp(palette.emblem, Colors.black, 0.46)!;

  Paint _paint(
    Color color, {
    PaintingStyle style = PaintingStyle.fill,
    double width = 1,
  }) => Paint()
    ..color = highContrast ? palette.ink : color
    ..style = style
    ..strokeWidth = width
    ..strokeCap = StrokeCap.round
    ..strokeJoin = StrokeJoin.round;

  @override
  void paint(Canvas canvas, Size size) {
    final bounds = Offset.zero & size;
    if (!highContrast) {
      final shadowPath = Path()
        ..addOval(
          Rect.fromCenter(
            center: Offset(size.width * 0.5, size.height * 0.76),
            width: size.width * 0.68,
            height: size.height * 0.18,
          ),
        );
      canvas.drawShadow(shadowPath, Colors.black, 10, false);
    }
    switch (variant % 6) {
      case 0:
        _paintOrbital(canvas, bounds);
      case 1:
        _paintCrystal(canvas, bounds);
      case 2:
        _paintCube(canvas, bounds);
      case 3:
        _paintMonolith(canvas, bounds);
      case 4:
        _paintRibbon(canvas, bounds);
      default:
        _paintStar(canvas, bounds);
    }
  }

  void _paintOrbital(Canvas canvas, Rect r) {
    final c = r.center;
    canvas.save();
    canvas.translate(c.dx, c.dy);
    canvas.rotate(-0.22);
    canvas.drawOval(
      Rect.fromCenter(
        center: Offset.zero,
        width: r.width * 0.94,
        height: r.height * 0.38,
      ),
      _paint(palette.accent, style: PaintingStyle.stroke, width: 3.2),
    );
    canvas.drawOval(
      Rect.fromCenter(
        center: Offset.zero,
        width: r.width * 0.54,
        height: r.height * 0.92,
      ),
      _paint(_light, style: PaintingStyle.stroke, width: 1.5),
    );
    final sphere = Rect.fromCircle(center: Offset.zero, radius: r.width * 0.24);
    canvas.drawOval(
      sphere,
      Paint()
        ..shader = RadialGradient(
          center: const Alignment(-0.4, -0.45),
          colors: [_light, _mid, _dark],
        ).createShader(sphere),
    );
    canvas.drawCircle(
      Offset(r.width * 0.42, 0),
      r.width * 0.055,
      _paint(_light),
    );
    canvas.restore();
  }

  void _paintCrystal(Canvas canvas, Rect r) {
    final top = Offset(r.width * 0.52, r.height * 0.05);
    final left = Offset(r.width * 0.16, r.height * 0.50);
    final bottom = Offset(r.width * 0.48, r.height * 0.94);
    final right = Offset(r.width * 0.84, r.height * 0.43);
    final center = Offset(r.width * 0.52, r.height * 0.47);
    final outline = Path()..addPolygon([top, right, bottom, left], true);
    canvas.drawShadow(outline, Colors.black, 8, false);
    canvas.drawPath(
      Path()..addPolygon([top, left, center], true),
      _paint(_mid),
    );
    canvas.drawPath(
      Path()..addPolygon([top, center, right], true),
      _paint(_light),
    );
    canvas.drawPath(
      Path()..addPolygon([left, bottom, center], true),
      _paint(_dark),
    );
    canvas.drawPath(
      Path()..addPolygon([center, bottom, right], true),
      _paint(palette.accent),
    );
    canvas.drawPath(
      outline,
      _paint(
        palette.ink.withValues(alpha: 0.5),
        style: PaintingStyle.stroke,
        width: 1.4,
      ),
    );
  }

  void _paintCube(Canvas canvas, Rect r) {
    final top = Offset(r.width * 0.5, r.height * 0.12);
    final leftTop = Offset(r.width * 0.14, r.height * 0.33);
    final rightTop = Offset(r.width * 0.86, r.height * 0.33);
    final center = Offset(r.width * 0.5, r.height * 0.55);
    final leftBottom = Offset(r.width * 0.14, r.height * 0.72);
    final rightBottom = Offset(r.width * 0.86, r.height * 0.72);
    final bottom = Offset(r.width * 0.5, r.height * 0.94);
    final outline = Path()
      ..addPolygon([
        top,
        rightTop,
        rightBottom,
        bottom,
        leftBottom,
        leftTop,
      ], true);
    canvas.drawShadow(outline, Colors.black, 9, false);
    canvas.drawPath(
      Path()..addPolygon([top, rightTop, center, leftTop], true),
      _paint(_light),
    );
    canvas.drawPath(
      Path()..addPolygon([leftTop, center, bottom, leftBottom], true),
      _paint(_mid),
    );
    canvas.drawPath(
      Path()..addPolygon([center, rightTop, rightBottom, bottom], true),
      _paint(_dark),
    );
    canvas.drawPath(
      outline,
      _paint(palette.accent, style: PaintingStyle.stroke, width: 1.2),
    );
  }

  void _paintMonolith(Canvas canvas, Rect r) {
    final plates = <({Rect rect, Color color})>[
      (
        rect: Rect.fromLTWH(
          r.width * 0.10,
          r.height * 0.66,
          r.width * 0.80,
          r.height * 0.18,
        ),
        color: _dark,
      ),
      (
        rect: Rect.fromLTWH(
          r.width * 0.18,
          r.height * 0.47,
          r.width * 0.68,
          r.height * 0.20,
        ),
        color: _mid,
      ),
      (
        rect: Rect.fromLTWH(
          r.width * 0.27,
          r.height * 0.24,
          r.width * 0.54,
          r.height * 0.24,
        ),
        color: _light,
      ),
      (
        rect: Rect.fromLTWH(
          r.width * 0.39,
          r.height * 0.08,
          r.width * 0.34,
          r.height * 0.17,
        ),
        color: palette.accent,
      ),
    ];
    for (final plate in plates) {
      final rr = RRect.fromRectAndRadius(
        plate.rect,
        Radius.circular(r.width * 0.055),
      );
      canvas.drawShadow(Path()..addRRect(rr), Colors.black, 6, false);
      canvas.drawRRect(rr, _paint(plate.color));
      canvas.drawLine(
        plate.rect.topLeft + Offset(r.width * 0.05, r.height * 0.035),
        plate.rect.topRight - Offset(r.width * 0.05, -r.height * 0.035),
        _paint(
          Colors.white.withValues(alpha: 0.38),
          style: PaintingStyle.stroke,
          width: 1,
        ),
      );
    }
  }

  void _paintRibbon(Canvas canvas, Rect r) {
    final shadow = Path()
      ..moveTo(r.width * 0.19, r.height * 0.72)
      ..cubicTo(
        r.width * 0.16,
        r.height * 0.18,
        r.width * 0.78,
        r.height * 0.08,
        r.width * 0.79,
        r.height * 0.56,
      )
      ..cubicTo(
        r.width * 0.80,
        r.height * 0.90,
        r.width * 0.35,
        r.height * 0.94,
        r.width * 0.31,
        r.height * 0.55,
      );
    canvas.drawPath(
      shadow,
      _paint(
        Colors.black.withValues(alpha: 0.42),
        style: PaintingStyle.stroke,
        width: r.width * 0.26,
      ),
    );
    final ribbonPaint = Paint()
      ..shader = LinearGradient(colors: [_light, _mid, _dark]).createShader(r)
      ..style = PaintingStyle.stroke
      ..strokeWidth = r.width * 0.20
      ..strokeCap = StrokeCap.round;
    canvas.drawPath(
      shadow,
      highContrast
          ? _paint(
              palette.ink,
              style: PaintingStyle.stroke,
              width: r.width * 0.20,
            )
          : ribbonPaint,
    );
    canvas.drawPath(
      shadow,
      _paint(palette.accent, style: PaintingStyle.stroke, width: 1.4),
    );
  }

  void _paintStar(Canvas canvas, Rect r) {
    final c = r.center;
    final points = <Offset>[];
    for (var i = 0; i < 16; i++) {
      final radius = (i.isEven ? 0.43 : 0.19) * r.width;
      final angle = -math.pi / 2 + i * math.pi / 8;
      points.add(c + Offset(math.cos(angle), math.sin(angle)) * radius);
    }
    final star = Path()..addPolygon(points, true);
    canvas.drawShadow(star, Colors.black, 10, false);
    canvas.drawPath(star, _paint(_mid));
    for (var i = 0; i < 16; i += 2) {
      final triangle = Path()
        ..addPolygon([c, points[i], points[(i + 2) % 16]], true);
      canvas.drawPath(triangle, _paint(i % 4 == 0 ? _light : _dark));
    }
    canvas.drawCircle(c, r.width * 0.10, _paint(palette.accent));
    canvas.drawPath(
      star,
      _paint(
        palette.ink.withValues(alpha: 0.45),
        style: PaintingStyle.stroke,
        width: 1.2,
      ),
    );
  }

  @override
  bool shouldRepaint(_ArtifactCoverObjectPainter oldDelegate) =>
      variant != oldDelegate.variant ||
      palette != oldDelegate.palette ||
      highContrast != oldDelegate.highContrast;
}

int _artifactVisualSeed(String value) {
  var hash = 0x811c9dc5;
  for (final codeUnit in value.codeUnits) {
    hash ^= codeUnit;
    hash = (hash * 0x01000193) & 0x7fffffff;
  }
  return hash;
}

final class _ArtifactCoverPalette {
  const _ArtifactCoverPalette({
    required this.background,
    required this.backgroundEnd,
    required this.accent,
    required this.emblem,
    required this.ink,
    required this.inkMuted,
    required this.pattern,
  });

  final Color background;
  final Color backgroundEnd;
  final Color accent;
  final Color emblem;
  final Color ink;
  final Color inkMuted;
  final Color pattern;

  static _ArtifactCoverPalette forSeed(int seed, {required bool darkMode}) {
    final base = _artifactCoverPalettes[seed % _artifactCoverPalettes.length];
    if (!darkMode) return base;
    return _ArtifactCoverPalette(
      background: Color.lerp(base.background, Colors.black, 0.10)!,
      backgroundEnd: Color.lerp(base.backgroundEnd, Colors.black, 0.05)!,
      accent: base.accent,
      emblem: base.emblem,
      ink: Colors.white,
      inkMuted: Color.lerp(base.inkMuted, Colors.white, 0.10)!,
      pattern: base.pattern,
    );
  }

  factory _ArtifactCoverPalette.highContrast(AppColorTokens colors) =>
      _ArtifactCoverPalette(
        background: colors.surfacePrimary,
        backgroundEnd: colors.surfacePrimary,
        accent: colors.textPrimary,
        emblem: colors.surfacePrimary,
        ink: colors.textPrimary,
        inkMuted: colors.textSecondary,
        pattern: Colors.transparent,
      );
}

const _artifactCoverPalettes = <_ArtifactCoverPalette>[
  _ArtifactCoverPalette(
    background: Color(0xFF211637),
    backgroundEnd: Color(0xFF4E3877),
    accent: Color(0xFFD0BBFF),
    emblem: Color(0xFF8066B4),
    ink: Color(0xFFF8F4FF),
    inkMuted: Color(0xFFD9CEE9),
    pattern: Color(0x244F3976),
  ),
  _ArtifactCoverPalette(
    background: Color(0xFF351B2B),
    backgroundEnd: Color(0xFF74405C),
    accent: Color(0xFFFFC2D8),
    emblem: Color(0xFFA85C7A),
    ink: Color(0xFFFFF5F8),
    inkMuted: Color(0xFFE9CDD7),
    pattern: Color(0x245D2944),
  ),
  _ArtifactCoverPalette(
    background: Color(0xFF103233),
    backgroundEnd: Color(0xFF286967),
    accent: Color(0xFF9DEBE0),
    emblem: Color(0xFF3F8D86),
    ink: Color(0xFFF1FFFC),
    inkMuted: Color(0xFFC6E5E0),
    pattern: Color(0x24336F6B),
  ),
  _ArtifactCoverPalette(
    background: Color(0xFF17243D),
    backgroundEnd: Color(0xFF3D5A8B),
    accent: Color(0xFFBFD3FF),
    emblem: Color(0xFF5E7DB4),
    ink: Color(0xFFF5F8FF),
    inkMuted: Color(0xFFCFD9ED),
    pattern: Color(0x24384F77),
  ),
  _ArtifactCoverPalette(
    background: Color(0xFF3A2812),
    backgroundEnd: Color(0xFF765322),
    accent: Color(0xFFFFD990),
    emblem: Color(0xFFA97931),
    ink: Color(0xFFFFF9EC),
    inkMuted: Color(0xFFE8D8B7),
    pattern: Color(0x245C401C),
  ),
  _ArtifactCoverPalette(
    background: Color(0xFF1B2D29),
    backgroundEnd: Color(0xFF49695B),
    accent: Color(0xFFBDE7CD),
    emblem: Color(0xFF678D7A),
    ink: Color(0xFFF3FFF7),
    inkMuted: Color(0xFFCEE2D5),
    pattern: Color(0x243D594E),
  ),
];

final class _ArtifactCoverPatternPainter extends CustomPainter {
  const _ArtifactCoverPatternPainter({required this.seed, required this.color});

  final int seed;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    if (color.a == 0) return;
    final stroke = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final fill = Paint()
      ..color = color.withValues(alpha: math.min(0.18, color.a * 1.8))
      ..style = PaintingStyle.fill;
    final phase = (seed % 19) / 19;
    switch (seed % 6) {
      case 0:
        final center = Offset(size.width * 0.72, size.height * 0.28);
        for (final radius in [0.13, 0.27, 0.43, 0.61]) {
          canvas.drawCircle(center, size.width * radius, stroke);
        }
        canvas.drawCircle(
          Offset(size.width * 0.20, size.height * 0.59),
          size.width * 0.045,
          fill,
        );
      case 1:
        final gap = math.max(19.0, size.width / 6);
        for (var x = -size.height; x < size.width + size.height; x += gap) {
          canvas.drawLine(
            Offset(x + phase * gap, size.height),
            Offset(x + size.height * 0.58, 0),
            stroke,
          );
        }
        for (var y = 0.0; y < size.height; y += gap) {
          canvas.drawLine(
            Offset.zero.translate(0, y),
            Offset(size.width, y),
            stroke,
          );
        }
      case 2:
        final cell = size.width / 4;
        for (var row = -1; row < 8; row++) {
          for (var col = -1; col < 5; col++) {
            final cx = col * cell + (row.isOdd ? cell / 2 : 0);
            final cy = row * cell * 0.48;
            final diamond = Path()
              ..addPolygon([
                Offset(cx, cy - cell * 0.24),
                Offset(cx + cell * 0.48, cy),
                Offset(cx, cy + cell * 0.24),
                Offset(cx - cell * 0.48, cy),
              ], true);
            canvas.drawPath(diamond, row.isEven && col.isEven ? fill : stroke);
          }
        }
      case 3:
        for (var i = 0; i < 7; i++) {
          final inset = size.width * (0.05 + i * 0.055);
          final top = size.height * (0.08 + i * 0.055);
          canvas.drawRRect(
            RRect.fromRectAndRadius(
              Rect.fromLTWH(
                inset,
                top,
                size.width - inset * 2,
                size.height * 0.62,
              ),
              Radius.circular(5.0 + i),
            ),
            i.isEven ? stroke : fill,
          );
        }
      case 4:
        final wave = Path()..moveTo(-size.width * 0.2, size.height * 0.18);
        for (var i = 0; i < 5; i++) {
          final y = size.height * (0.18 + i * 0.16);
          wave
            ..moveTo(-size.width * 0.2, y)
            ..cubicTo(
              size.width * 0.20,
              y - size.height * 0.16,
              size.width * 0.76,
              y + size.height * 0.16,
              size.width * 1.2,
              y - size.height * 0.02,
            );
        }
        canvas.drawPath(wave, stroke..strokeWidth = 2.2);
      default:
        final center = Offset(size.width * 0.52, size.height * 0.34);
        for (var i = 0; i < 18; i++) {
          final angle = i * math.pi * 2 / 18 + phase;
          final inner = size.width * (0.10 + (i % 3) * 0.035);
          final outer = size.width * (0.48 + (i % 4) * 0.045);
          canvas.drawLine(
            center + Offset(math.cos(angle), math.sin(angle)) * inner,
            center + Offset(math.cos(angle), math.sin(angle)) * outer,
            stroke,
          );
        }
        canvas.drawCircle(center, size.width * 0.085, fill);
    }
  }

  @override
  bool shouldRepaint(_ArtifactCoverPatternPainter oldDelegate) =>
      seed != oldDelegate.seed || color != oldDelegate.color;
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
            ? 560
            : medium
            ? 720
            : 880,
      ),
      child: Material(
        color: Colors.transparent,
        child: Stack(
          children: [
            DecoratedBox(
              decoration: BoxDecoration(
                color: context.appColors.surfaceFloating,
                border: Border.all(color: context.appColors.borderSubtle),
                borderRadius: BorderRadius.circular(AppRadii.floating),
                boxShadow: MediaQuery.highContrastOf(context)
                    ? null
                    : [
                        BoxShadow(
                          color: context.appColors.ambientShadow,
                          blurRadius: AppShadow.floatingControlBlur,
                          offset: const Offset(
                            0,
                            AppShadow.floatingControlOffset,
                          ),
                        ),
                      ],
              ),
              child: Padding(
                padding: EdgeInsets.fromLTRB(
                  AppSpacing.space16,
                  AppSpacing.space12,
                  interactive ? AppSpacing.space12 : AppSpacing.space16,
                  AppSpacing.space12,
                ),
                child: compact
                    ? Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.center,
                            children: [
                              _DocumentContextCover(document: document),
                              const SizedBox(width: AppSpacing.space12),
                              Expanded(
                                child: _DocumentContextSummary(
                                  document: document,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: AppSpacing.space12),
                          _DocumentProgressColumn(
                            document: document,
                            progressLine: progressLine,
                          ),
                          if (interactive) ...[
                            const SizedBox(height: AppSpacing.space8),
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
                          _DocumentContextCover(document: document),
                          const SizedBox(width: AppSpacing.space12),
                          Expanded(
                            child: _DocumentContextSummary(document: document),
                          ),
                          const SizedBox(width: AppSpacing.space16),
                          SizedBox(
                            width: medium ? 142 : 164,
                            child: _DocumentProgressColumn(
                              document: document,
                              progressLine: progressLine,
                            ),
                          ),
                          if (interactive) ...[
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
            PositionedDirectional(
              start: 0,
              top: AppSpacing.space16,
              bottom: AppSpacing.space16,
              child: IgnorePointer(
                child: Container(
                  width: 3,
                  decoration: BoxDecoration(
                    color: context.appColors.librarySelectionAccent,
                    borderRadius: BorderRadius.circular(AppRadii.indicator),
                  ),
                ),
              ),
            ),
          ],
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

final class _DocumentContextCover extends StatelessWidget {
  const _DocumentContextCover({required this.document});

  final LibraryDocument document;

  @override
  Widget build(BuildContext context) {
    final path = artifactCoverPathFor(document, selected: true);
    final fallback = _DocumentContextCoverFallback(document: document);
    return ExcludeSemantics(
      child: SizedBox(
        key: ValueKey('library-context-cover-${document.id}'),
        width: 52,
        height: 68,
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            Positioned(
              left: 4,
              top: 3,
              right: -3,
              bottom: -3,
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: context.appColors.surfaceSecondary,
                  border: Border.all(color: context.appColors.borderStrong),
                  borderRadius: BorderRadius.circular(AppRadii.artifactCover),
                  boxShadow: MediaQuery.highContrastOf(context)
                      ? null
                      : [
                          BoxShadow(
                            color: context.appColors.ambientShadow,
                            blurRadius: 4,
                            offset: const Offset(1, 2),
                          ),
                        ],
                ),
              ),
            ),
            Positioned.fill(
              child: ClipRRect(
                borderRadius: BorderRadius.circular(AppRadii.artifactCover),
                clipBehavior: Clip.antiAliasWithSaveLayer,
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: context.appColors.surfaceSunken,
                    border: Border.all(
                      color: context.appColors.librarySelectionAccent,
                    ),
                    borderRadius: BorderRadius.circular(AppRadii.artifactCover),
                  ),
                  child: path == null
                      ? fallback
                      : Image.file(
                          File(path),
                          fit: BoxFit.cover,
                          filterQuality: FilterQuality.high,
                          errorBuilder: (_, _, _) => fallback,
                        ),
                ),
              ),
            ),
            Positioned(
              left: 4,
              top: 2,
              bottom: 2,
              child: Container(
                width: 2,
                color: Colors.black.withValues(alpha: 0.14),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _DocumentContextCoverFallback extends StatelessWidget {
  const _DocumentContextCoverFallback({required this.document});

  final LibraryDocument document;

  @override
  Widget build(BuildContext context) {
    final accent = context.appColors.librarySelectionAccent;
    return DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            context.appColors.surfaceElevated,
            Color.alphaBlend(
              accent.withValues(alpha: 0.22),
              context.appColors.surfaceSunken,
            ),
          ],
        ),
      ),
      child: CustomPaint(
        painter: _ArtifactCoverPatternPainter(
          seed: document.id.hashCode,
          color: accent.withValues(alpha: 0.58),
        ),
      ),
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
