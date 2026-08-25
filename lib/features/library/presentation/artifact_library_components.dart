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
    required this.selectionMode,
    required this.selectedCount,
    required this.onSearch,
    required this.onFilter,
    required this.onFacets,
    required this.onToggleSelectionMode,
  });

  final int resultCount;
  final int totalCount;
  final String query;
  final Map<LibraryReadingState, int> counts;
  final LibraryReadingState readingState;
  final LibrarySortOrder sortOrder;
  final bool favoritesOnly;
  final bool activeFilter;
  final bool selectionMode;
  final int selectedCount;
  final VoidCallback onSearch;
  final ValueChanged<_FilterCommand> onFilter;
  final VoidCallback onFacets;
  final VoidCallback onToggleSelectionMode;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        AppSpacing.space20,
        AppSpacing.space12,
        AppSpacing.space20,
        AppSpacing.space8,
      ),
      child: Row(
        children: [
          if (query.isNotEmpty || activeFilter)
            Text(
              strings.get(
                'library.resultOfTotal',
                parameters: {'result': resultCount, 'total': totalCount},
              ),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          const Spacer(),
          if (selectedCount > 0)
            Semantics(
              liveRegion: true,
              label: strings.count('count.documentsSelected', selectedCount),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space8,
                ),
                child: Text(
                  strings.count('count.selected', selectedCount),
                  style: Theme.of(context).textTheme.labelMedium,
                ),
              ),
            ),
          YamaToolbarGroup(
            label: strings.get('library.commandBar'),
            flat: MediaQuery.highContrastOf(context),
            children: [
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
                selected: activeFilter,
                icon: Icons.account_tree_outlined,
              ),
              if (activeFilter)
                YamaIconButton(
                  key: const Key('library-reset-filters-button'),
                  semanticLabel: strings.get('library.resetFilters'),
                  tooltip: strings.get('library.resetFilters'),
                  onPressed: () => onFilter(_FilterCommand.reset),
                  icon: Icons.filter_alt_off_outlined,
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
                        LibraryReadingState.inProgress =>
                          _FilterCommand.inProgress,
                        LibraryReadingState.notStarted =>
                          _FilterCommand.notStarted,
                        LibraryReadingState.completed =>
                          _FilterCommand.completed,
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
                        LibrarySortOrder.lastOpened =>
                          _FilterCommand.lastOpened,
                        LibrarySortOrder.dateAdded => _FilterCommand.dateAdded,
                        LibrarySortOrder.title => _FilterCommand.title,
                        LibrarySortOrder.progress => _FilterCommand.progress,
                      },
                      checked: sortOrder == order,
                      child: Text(
                        strings.get('library.sortOrder.${order.name}'),
                      ),
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
          ),
        ],
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
    required this.groupByAuthor,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
    this.onDocumentVisible,
  });

  final ScrollController controller;
  final List<LibraryDocument> documents;
  final ValueListenable<bool> Function(String documentId)
  selectionListenableFor;
  final Set<String> selectedDocumentIds;
  final bool selectionMode;
  final bool reserveInfoSpace;
  final bool reduceMotion;
  final bool groupByAuthor;
  final ValueChanged<LibraryDocument> onSelect;
  final ValueChanged<LibraryDocument> onOpen;
  final ValueChanged<String?> onHover;
  final void Function(LibraryDocument document, Offset position) onMenu;
  final ValueChanged<LibraryDocument>? onDocumentVisible;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final horizontalPadding = AppSpacing.pagePadding(constraints.maxWidth);
        final metrics = LibraryArtifactLayout.metricsFor(
          viewportWidth: constraints.maxWidth - horizontalPadding * 2,
          documentCount: documents.length,
          compact:
              AppBreakpoints.classify(constraints.maxWidth) ==
              AppWindowClass.compact,
        );
        if (groupByAuthor) {
          final groups = groupLibraryDocumentsByAuthor(documents);
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
                    child: Row(
                      children: [
                        Text(
                          entry.key,
                          style: Theme.of(context).textTheme.titleSmall,
                        ),
                        const SizedBox(width: AppSpacing.space8),
                        Text(
                          '${entry.value.length}',
                          style: Theme.of(context).textTheme.labelSmall,
                        ),
                      ],
                    ),
                  ),
                ),
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
                          height: metrics.artifactHeight + AppSpacing.space24,
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
    );
    if (selectionMode) {
      return buildArtifact(selectedDocumentIds.contains(document.id));
    }
    return ValueListenableBuilder<bool>(
      valueListenable: selectionListenableFor(document.id),
      builder: (context, selected, _) => buildArtifact(selected),
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

  @override
  State<ArtifactDocument> createState() => _ArtifactDocumentState();
}

final class _ArtifactDocumentState extends State<ArtifactDocument> {
  bool _hovered = false;
  bool _focused = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) widget.onVisible?.call();
    });
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final jitter = LibraryArtifactLayout.jitterFor(widget.document.id);
    final emphasized = widget.selected || _hovered || _focused;
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
    final maxHeight = widget.width * 1.62;
    final fittedHeight = math.min(height, maxHeight);
    final duration = widget.reduceMotion ? Duration.zero : AppMotion.standard;
    final coverPath = artifactCoverPathFor(
      widget.document,
      selected: widget.selected,
    );

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
                    child: Stack(
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
                          duration: duration,
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
                            borderRadius: BorderRadius.circular(AppRadii.page),
                            border: Border.all(
                              color: widget.selected
                                  ? context.appColors.librarySelectionAccent
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
                                        blurRadius: AppShadow.artifactGlowBlur,
                                        spreadRadius:
                                            AppShadow.artifactGlowSpread,
                                        offset: const Offset(0, 12),
                                      ),
                                    BoxShadow(
                                      color: context.appColors.readerPageShadow
                                          .withValues(
                                            alpha: widget.selected
                                                ? 0.72
                                                : 0.38,
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
                          clipBehavior: Clip.antiAlias,
                          child: Stack(
                            fit: StackFit.expand,
                            children: [
                              AnimatedOpacity(
                                duration: duration,
                                opacity: widget.document.isMissing
                                    ? 0.56
                                    : emphasized
                                    ? 1
                                    : 0.84,
                                child: coverPath == null
                                    ? _ArtifactPlaceholder(
                                        document: widget.document,
                                      )
                                    : ColorFiltered(
                                        colorFilter: ColorFilter.matrix(
                                          _saturationMatrix(
                                            emphasized ? 1 : 0.72,
                                          ),
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
                                          filterQuality: FilterQuality.low,
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
                                          color: context.appColors.textPrimary,
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
                                    message: strings.get('library.favorite'),
                                    child: Icon(
                                      Icons.bookmark_rounded,
                                      size: 17,
                                      color: context.appColors.textPrimary,
                                    ),
                                  ),
                                ),
                              if (widget.selectionMode && widget.selected)
                                Positioned(
                                  left: AppSpacing.space8,
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
                      ],
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
    final content = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: compact ? 620 : 920),
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
                      _DocumentContextSummary(
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
                        child: _DocumentContextSummary(
                          document: document,
                          progressLine: progressLine,
                        ),
                      ),
                      if (interactive) ...[
                        const SizedBox(width: AppSpacing.space20),
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
  const _DocumentContextSummary({
    required this.document,
    required this.progressLine,
  });

  final LibraryDocument document;
  final String progressLine;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
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
        Wrap(
          spacing: AppSpacing.space8,
          runSpacing: AppSpacing.space2,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            if (document.effectiveAuthor case final author?)
              Text(author, style: Theme.of(context).textTheme.bodySmall),
            Text(
              document.isMissing
                  ? strings.get('library.documentUnavailable')
                  : progressLine,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: document.isMissing
                    ? context.appColors.warning
                    : context.appColors.textSecondary,
              ),
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
