part of 'reader_side_panel.dart';

final class _PanelSectionBar extends StatelessWidget {
  const _PanelSectionBar({required this.selected, required this.onSelected});

  final ReaderPanelSection selected;
  final ValueChanged<ReaderPanelSection> onSelected;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      container: true,
      label: strings.get('reader.panelSections'),
      child: SizedBox(
        height: AppSizes.bottomControlHeight,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            for (final section in ReaderPanelSection.values)
              YamaIconButton(
                semanticLabel: _label(section, strings),
                tooltip: _label(section, strings),
                icon: _icon(section),
                selected: selected == section,
                onPressed: () => onSelected(section),
              ),
          ],
        ),
      ),
    );
  }

  String _label(ReaderPanelSection section, YamantakaLocalizations strings) =>
      switch (section) {
        ReaderPanelSection.thumbnails => strings.get('reader.thumbnails'),
        ReaderPanelSection.outline => strings.get('reader.outline'),
        ReaderPanelSection.bookmarks => strings.get('reader.bookmarks'),
        ReaderPanelSection.annotations => strings.get('reader.annotations'),
        ReaderPanelSection.search => strings.get('reader.search'),
      };

  IconData _icon(ReaderPanelSection section) => switch (section) {
    ReaderPanelSection.thumbnails => Icons.grid_view_outlined,
    ReaderPanelSection.outline => Icons.format_list_bulleted_rounded,
    ReaderPanelSection.bookmarks => Icons.bookmarks_outlined,
    ReaderPanelSection.annotations => Icons.rate_review_outlined,
    ReaderPanelSection.search => Icons.search_rounded,
  };
}

final class _PanelSectionRail extends StatelessWidget {
  const _PanelSectionRail({required this.selected, required this.onSelected});

  final ReaderPanelSection selected;
  final ValueChanged<ReaderPanelSection> onSelected;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      container: true,
      label: strings.get('reader.panelSections'),
      child: SizedBox(
        key: const Key('reader-panel-section-rail'),
        width: 52,
        child: ListView(
          padding: const EdgeInsets.symmetric(vertical: AppSpacing.space8),
          children: [
            for (final section in ReaderPanelSection.values)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space6,
                  vertical: AppSpacing.space2,
                ),
                child: YamaIconButton(
                  semanticLabel: _panelSectionLabel(section, strings),
                  tooltip: _panelSectionLabel(section, strings),
                  icon: _panelSectionIcon(section),
                  selected: selected == section,
                  onPressed: () => onSelected(section),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

String _panelSectionLabel(
  ReaderPanelSection section,
  YamantakaLocalizations strings,
) => switch (section) {
  ReaderPanelSection.thumbnails => strings.get('reader.thumbnails'),
  ReaderPanelSection.outline => strings.get('reader.outline'),
  ReaderPanelSection.bookmarks => strings.get('reader.bookmarks'),
  ReaderPanelSection.annotations => strings.get('reader.annotations'),
  ReaderPanelSection.search => strings.get('reader.search'),
};

IconData _panelSectionIcon(ReaderPanelSection section) => switch (section) {
  ReaderPanelSection.thumbnails => Icons.grid_view_outlined,
  ReaderPanelSection.outline => Icons.format_list_bulleted_rounded,
  ReaderPanelSection.bookmarks => Icons.bookmarks_outlined,
  ReaderPanelSection.annotations => Icons.rate_review_outlined,
  ReaderPanelSection.search => Icons.search_rounded,
};

final class _PanelResizeHandle extends StatefulWidget {
  const _PanelResizeHandle({required this.width, required this.onWidthChanged});

  final double width;
  final ValueChanged<double> onWidthChanged;

  @override
  State<_PanelResizeHandle> createState() => _PanelResizeHandleState();
}

final class _PanelResizeHandleState extends State<_PanelResizeHandle> {
  bool _hovered = false;
  bool _focused = false;
  bool _dragging = false;

  double _clamp(double value) => value
      .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
      .toDouble();

  @override
  Widget build(BuildContext context) {
    final active = _hovered || _focused || _dragging;
    void changeBy(double delta) =>
        widget.onWidthChanged(_clamp(widget.width + delta));
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      label: strings.get('reader.panelWidth'),
      value: strings.get(
        'reader.pixels',
        parameters: {'count': widget.width.round()},
      ),
      increasedValue: strings.get(
        'reader.pixels',
        parameters: {'count': _clamp(widget.width + 16).round()},
      ),
      decreasedValue: strings.get(
        'reader.pixels',
        parameters: {'count': _clamp(widget.width - 16).round()},
      ),
      onIncrease: () => changeBy(16),
      onDecrease: () => changeBy(-16),
      child: Focus(
        onFocusChange: (value) => setState(() => _focused = value),
        onKeyEvent: (_, event) {
          if (event is! KeyDownEvent) return KeyEventResult.ignored;
          if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
            changeBy(16);
            return KeyEventResult.handled;
          }
          if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
            changeBy(-16);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Builder(
          builder: (handleContext) => MouseRegion(
            cursor: SystemMouseCursors.resizeColumn,
            onEnter: (_) => setState(() => _hovered = true),
            onExit: (_) => setState(() => _hovered = false),
            child: GestureDetector(
              key: const Key('reader-panel-resize-handle'),
              behavior: HitTestBehavior.opaque,
              dragStartBehavior: DragStartBehavior.down,
              onTapDown: (_) => Focus.of(handleContext).requestFocus(),
              onHorizontalDragStart: (_) {
                Focus.of(handleContext).requestFocus();
                setState(() => _dragging = true);
              },
              onHorizontalDragUpdate: (details) => changeBy(-details.delta.dx),
              onHorizontalDragEnd: (_) => setState(() => _dragging = false),
              onHorizontalDragCancel: () => setState(() => _dragging = false),
              child: SizedBox(
                width: active ? 12 : 10,
                child: Stack(
                  clipBehavior: Clip.none,
                  alignment: Alignment.center,
                  children: [
                    AnimatedContainer(
                      duration: AppMotion.whenAllowed(
                        MediaQuery.disableAnimationsOf(context),
                        AppMotion.feedback,
                      ),
                      width: active ? 4 : 2,
                      height: active ? 52 : 36,
                      color: active
                          ? context.appColors.focusRing
                          : context.appColors.borderStrong,
                    ),
                    if (active)
                      Positioned(
                        left: 12,
                        child: Material(
                          color: context.appColors.surfaceFloating,
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(
                              AppRadii.control,
                            ),
                            side: BorderSide(
                              color: context.appColors.borderStrong,
                            ),
                          ),
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: AppSpacing.space8,
                              vertical: AppSpacing.space4,
                            ),
                            child: Text(
                              strings.get(
                                'reader.pixels',
                                parameters: {'count': widget.width.round()},
                              ),
                              style: Theme.of(context).textTheme.labelSmall,
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

final class _ThumbnailPanel extends StatefulWidget {
  const _ThumbnailPanel({
    required this.controller,
    required this.engine,
    required this.panelWidth,
  });

  final ReaderController controller;
  final PdfDocumentEngine engine;
  final double panelWidth;

  @override
  State<_ThumbnailPanel> createState() => _ThumbnailPanelState();
}

final class _ThumbnailPanelState extends State<_ThumbnailPanel> {
  static const _maximumRetainedRequests = 18;
  final LinkedHashMap<
    ({int pageIndex, int scaleMilli}),
    Future<RenderedPdfPage>
  >
  _renderRequests = LinkedHashMap();
  final Map<({int pageIndex, int scaleMilli}), PdfOperationToken>
  _renderTokens = {};
  String? _documentId;

  @override
  void initState() {
    super.initState();
    _documentId = widget.controller.openedDocument?.documentId;
  }

  @override
  void dispose() {
    _cancelThumbnailRequests();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ThumbnailPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    final nextDocumentId = widget.controller.openedDocument?.documentId;
    if (_documentId != nextDocumentId) {
      _cancelThumbnailRequests();
      _renderRequests.clear();
      _documentId = nextDocumentId;
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final opened = widget.controller.openedDocument;
    if (opened == null) return const SizedBox.shrink();
    final pixelRatio = MediaQuery.devicePixelRatioOf(context).clamp(1.0, 2.0);
    return ValueListenableBuilder<int>(
      valueListenable: widget.controller.pageIndexListenable,
      builder: (context, currentPageIndex, _) => ListView.builder(
        key: const PageStorageKey('reader-thumbnails'),
        padding: const EdgeInsets.all(AppSpacing.space12),
        itemCount: opened.pageCount,
        itemBuilder: (context, index) {
          final selected = index == currentPageIndex;
          final pageSize = opened.pageSizes[index];
          final renderScale =
              ((widget.panelWidth - AppSpacing.space40) *
                      pixelRatio *
                      1.12 /
                      pageSize.width)
                  .clamp(0.45, 1.2)
                  .toDouble();
          return Semantics(
            button: true,
            selected: selected,
            label: strings.get('reader.page', parameters: {'page': index + 1}),
            child: Material(
              color: selected
                  ? context.appColors.selectionBackground
                  : Colors.transparent,
              child: InkWell(
                onTap: () => widget.controller.navigateTo(
                  index,
                  reason: NavigationReason.thumbnail,
                  label: strings.get(
                    'reader.pageThumbnail',
                    parameters: {'page': index + 1},
                  ),
                ),
                child: Container(
                  margin: const EdgeInsets.only(bottom: AppSpacing.space12),
                  padding: const EdgeInsets.all(AppSpacing.space6),
                  decoration: BoxDecoration(
                    border: Border(
                      left: BorderSide(
                        color: selected
                            ? context.appColors.accentPrimary
                            : context.appColors.borderSubtle,
                        width: selected ? 3 : 1,
                      ),
                      top: BorderSide(color: context.appColors.borderSubtle),
                      right: BorderSide(color: context.appColors.borderSubtle),
                      bottom: BorderSide(color: context.appColors.borderSubtle),
                    ),
                  ),
                  child: Column(
                    children: [
                      AspectRatio(
                        aspectRatio:
                            opened.pageSizes[index].width /
                            opened.pageSizes[index].height,
                        child: FutureBuilder<RenderedPdfPage>(
                          future: _renderRequests.putIfAbsent(
                            (
                              pageIndex: index,
                              scaleMilli: (renderScale * 1000).round(),
                            ),
                            () => _requestThumbnail(
                              documentId: opened.documentId,
                              pageIndex: index,
                              scale: renderScale,
                            ),
                          ),
                          builder: (context, snapshot) {
                            if (snapshot.hasData) {
                              return RawImage(
                                image: snapshot.data!.image,
                                fit: BoxFit.contain,
                                filterQuality: FilterQuality.high,
                              );
                            }
                            if (snapshot.hasError) {
                              return _ThumbnailState(
                                icon: Icons.broken_image_outlined,
                                label: strings.get('reader.unavailable'),
                                color: context.appColors.danger,
                              );
                            }
                            return _ThumbnailState(
                              icon: Icons.hourglass_empty_rounded,
                              label: strings.get('reader.generating'),
                            );
                          },
                        ),
                      ),
                      const SizedBox(height: AppSpacing.space6),
                      Text(
                        '${index + 1}',
                        style: Theme.of(context).textTheme.labelMedium,
                      ),
                      if (widget.controller.annotations?.pageHasAnnotations(
                            index,
                          ) ??
                          false)
                        Padding(
                          padding: const EdgeInsets.only(
                            top: AppSpacing.space4,
                          ),
                          child: Icon(
                            Icons.rate_review_outlined,
                            size: 15,
                            semanticLabel: strings.get(
                              'reader.pageWithAnnotations',
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
          );
        },
      ),
    );
  }

  Future<RenderedPdfPage> _requestThumbnail({
    required String documentId,
    required int pageIndex,
    required double scale,
  }) {
    while (_renderRequests.length >= _maximumRetainedRequests) {
      final oldestKey = _renderRequests.keys.first;
      _renderTokens.remove(oldestKey)?.cancel();
      _renderRequests.remove(oldestKey);
    }
    final key = (pageIndex: pageIndex, scaleMilli: (scale * 1000).round());
    final token = PdfOperationToken();
    _renderTokens[key] = token;
    return widget.engine.renderPage(
      documentId: documentId,
      pageIndex: pageIndex,
      scale: scale,
      cancellationToken: token,
    );
  }

  void _cancelThumbnailRequests() {
    for (final token in _renderTokens.values) {
      token.cancel();
    }
    _renderTokens.clear();
  }
}

final class _ThumbnailState extends StatelessWidget {
  const _ThumbnailState({required this.icon, required this.label, this.color});

  final IconData icon;
  final String label;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final resolvedColor = color ?? context.appColors.textMuted;
    return ColoredBox(
      color: context.appColors.surfaceSunken,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 18, color: resolvedColor),
            const SizedBox(height: AppSpacing.space4),
            Text(
              label,
              style: Theme.of(
                context,
              ).textTheme.labelSmall?.copyWith(color: resolvedColor),
            ),
          ],
        ),
      ),
    );
  }
}

final class _OutlinePanel extends StatelessWidget {
  const _OutlinePanel({required this.controller});

  final ReaderController controller;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    if (controller.outline.isEmpty) {
      return _PanelEmptyState(
        icon: Icons.account_tree_outlined,
        title: strings.get('reader.noOutline'),
        message: strings.get('reader.noOutline.description'),
      );
    }
    final flattened = <({PdfOutlineItem item, int depth})>[];
    void append(List<PdfOutlineItem> items, int depth) {
      for (final item in items) {
        flattened.add((item: item, depth: depth));
        append(item.children, depth + 1);
      }
    }

    append(controller.outline, 0);
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: AppSpacing.space8),
      itemCount: flattened.length,
      itemBuilder: (context, index) {
        final entry = flattened[index];
        return ListTile(
          contentPadding: EdgeInsets.only(
            left: AppSpacing.space12 + entry.depth * AppSpacing.space16,
            right: AppSpacing.space8,
          ),
          title: Text(
            entry.item.title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: entry.item.pageIndex == null
              ? null
              : Text(
                  '${entry.item.pageIndex! + 1}',
                  style: Theme.of(context).textTheme.labelMedium,
                ),
          onTap: entry.item.pageIndex == null
              ? null
              : () => controller.navigateTo(
                  entry.item.pageIndex!,
                  reason: NavigationReason.outline,
                  label: strings.get('reader.outlineSection'),
                ),
        );
      },
    );
  }
}

final class _BookmarksPanel extends StatelessWidget {
  const _BookmarksPanel({required this.controller});

  final ReaderController controller;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return ListenableBuilder(
      listenable: controller.bookmarks,
      builder: (context, _) {
        final bookmarks = controller.bookmarks.bookmarks;
        return Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space12),
              child: SizedBox(
                width: double.infinity,
                child: OutlinedButton.icon(
                  onPressed: () => _addNamedBookmark(context),
                  icon: const Icon(Icons.bookmark_add_outlined, size: 18),
                  label: Text(
                    strings.get(
                      'reader.bookmarkPage',
                      parameters: {'page': controller.currentPageIndex + 1},
                    ),
                  ),
                ),
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: bookmarks.isEmpty
                  ? _PanelEmptyState(
                      icon: Icons.bookmark_border_rounded,
                      title: strings.get('reader.noBookmarks'),
                      message: strings.get('reader.noBookmarks.description'),
                    )
                  : ListView.separated(
                      padding: const EdgeInsets.symmetric(
                        vertical: AppSpacing.space8,
                      ),
                      itemCount: bookmarks.length,
                      separatorBuilder: (_, _) => const Divider(
                        indent: AppSpacing.space16,
                        endIndent: AppSpacing.space16,
                      ),
                      itemBuilder: (context, index) => _BookmarkTile(
                        bookmark: bookmarks[index],
                        onOpen: () => controller.navigateTo(
                          bookmarks[index].pageIndex,
                          reason: NavigationReason.bookmark,
                          label: strings.get(
                            'reader.bookmarkOnPage',
                            parameters: {
                              'page': bookmarks[index].pageIndex + 1,
                            },
                          ),
                        ),
                        onDelete: () =>
                            controller.bookmarks.remove(bookmarks[index]),
                      ),
                    ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _addNamedBookmark(BuildContext context) async {
    final strings = YamantakaLocalizations.of(context);
    final textController = TextEditingController();
    final label = await showDialog<String?>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('reader.addBookmark')),
        content: SizedBox(
          width: 360,
          child: TextField(
            controller: textController,
            autofocus: true,
            maxLength: 60,
            decoration: InputDecoration(
              labelText: strings.get('reader.shortName'),
              hintText: strings.get('reader.optional'),
            ),
            onSubmitted: (value) => Navigator.pop(context, value),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, textController.text),
            child: Text(strings.get('action.save')),
          ),
        ],
      ),
    );
    textController.dispose();
    if (label != null) {
      controller.bookmarks.add(controller.currentPageIndex, label: label);
    }
  }
}

final class _BookmarkTile extends StatelessWidget {
  const _BookmarkTile({
    required this.bookmark,
    required this.onOpen,
    required this.onDelete,
  });

  final Bookmark bookmark;
  final VoidCallback onOpen;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return ListTile(
      leading: Icon(
        Icons.bookmark_rounded,
        color: context.appColors.accentPrimary,
      ),
      title: Text(
        bookmark.label ??
            strings.get(
              'reader.page',
              parameters: {'page': bookmark.pageIndex + 1},
            ),
      ),
      subtitle: bookmark.label == null
          ? null
          : Text(
              strings.get(
                'reader.page',
                parameters: {'page': bookmark.pageIndex + 1},
              ),
            ),
      onTap: onOpen,
      trailing: IconButton(
        tooltip: strings.get('reader.deleteBookmark'),
        onPressed: onDelete,
        icon: const Icon(Icons.delete_outline_rounded),
      ),
    );
  }
}

final class _AnnotationsPanel extends StatefulWidget {
  const _AnnotationsPanel({required this.controller, this.onExport});

  final ReaderController controller;
  final Future<void> Function()? onExport;

  @override
  State<_AnnotationsPanel> createState() => _AnnotationsPanelState();
}

final class _AnnotationsPanelState extends State<_AnnotationsPanel> {
  final TextEditingController _queryController = TextEditingController();

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final annotations = widget.controller.annotations;
    if (annotations == null) {
      return _PanelEmptyState(
        icon: Icons.rate_review_outlined,
        title: strings.get('reader.annotationsUnavailable'),
        message: strings.get('reader.annotationsUnavailable.description'),
      );
    }
    return ListenableBuilder(
      listenable: annotations,
      builder: (context, _) => Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(AppSpacing.space12),
            child: Column(
              children: [
                Row(
                  children: [
                    Expanded(
                      child: FilledButton.tonalIcon(
                        key: const Key('annotation-add-page-note'),
                        onPressed: () => _editAnnotation(),
                        icon: const Icon(Icons.add_comment_outlined, size: 18),
                        label: Text(
                          strings.get(
                            'reader.annotatePage',
                            parameters: {
                              'page': widget.controller.currentPageIndex + 1,
                            },
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    IconButton.outlined(
                      tooltip: strings.get('command.undoAnnotation'),
                      onPressed: annotations.canUndo ? annotations.undo : null,
                      icon: const Icon(Icons.undo_rounded),
                    ),
                    const SizedBox(width: AppSpacing.space4),
                    IconButton.outlined(
                      tooltip: strings.get('command.redoAnnotation'),
                      onPressed: annotations.canRedo ? annotations.redo : null,
                      icon: const Icon(Icons.redo_rounded),
                    ),
                    if (widget.onExport != null) ...[
                      const SizedBox(width: AppSpacing.space8),
                      IconButton.outlined(
                        tooltip: strings.get('reader.exportAnnotations'),
                        onPressed: widget.onExport,
                        icon: const Icon(Icons.ios_share_outlined),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: AppSpacing.space8),
                TextField(
                  controller: _queryController,
                  decoration: InputDecoration(
                    isDense: true,
                    prefixIcon: const Icon(Icons.search_rounded, size: 18),
                    labelText: strings.get('reader.searchNotes'),
                  ),
                  onChanged: (value) => annotations.setFilters(search: value),
                ),
                const SizedBox(height: AppSpacing.space8),
                Row(
                  children: [
                    Expanded(
                      child: DropdownButtonFormField<AnnotationType?>(
                        initialValue: annotations.typeFilter,
                        isExpanded: true,
                        decoration: InputDecoration(
                          isDense: true,
                          labelText: strings.get('reader.type'),
                        ),
                        items: [
                          DropdownMenuItem(
                            value: null,
                            child: Text(strings.get('reader.all')),
                          ),
                          ...AnnotationType.values.map(
                            (type) => DropdownMenuItem(
                              value: type,
                              child: Text(
                                strings.get('annotation.type.${type.name}'),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                        ],
                        onChanged: (value) => annotations.setFilters(
                          type: value,
                          clearType: value == null,
                        ),
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    Expanded(
                      child: DropdownButtonFormField<AnnotationColor?>(
                        initialValue: annotations.colorFilter,
                        isExpanded: true,
                        decoration: InputDecoration(
                          isDense: true,
                          labelText: strings.get('reader.color'),
                        ),
                        items: [
                          DropdownMenuItem(
                            value: null,
                            child: Text(strings.get('reader.all')),
                          ),
                          ...AnnotationColor.values.map(
                            (color) => DropdownMenuItem(
                              value: color,
                              child: Text(
                                strings.get('annotation.color.${color.name}'),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                        ],
                        onChanged: (value) => annotations.setFilters(
                          color: value,
                          clearColor: value == null,
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          if (annotations.reviewRequiredCount > 0)
            MaterialBanner(
              content: Text(
                strings.count(
                  'count.annotationsReview',
                  annotations.reviewRequiredCount,
                ),
              ),
              leading: const Icon(Icons.warning_amber_rounded),
              actions: const [SizedBox.shrink()],
            ),
          const Divider(height: 1),
          Expanded(
            child: annotations.annotations.isEmpty
                ? _PanelEmptyState(
                    icon: Icons.rate_review_outlined,
                    title: strings.get('reader.noAnnotations'),
                    message: strings.get('reader.noAnnotations.description'),
                  )
                : ListView.builder(
                    key: const PageStorageKey('reader-annotations'),
                    padding: const EdgeInsets.symmetric(
                      vertical: AppSpacing.space8,
                    ),
                    itemCount: annotations.annotations.length,
                    itemBuilder: (context, index) {
                      final annotation = annotations.annotations[index];
                      return Semantics(
                        button: true,
                        label: strings.get(
                          'reader.annotationSemantics',
                          parameters: {
                            'type': strings.get(
                              'annotation.type.${annotation.type.name}',
                            ),
                            'page': annotation.pageIndex + 1,
                            'color': strings.get(
                              'annotation.color.${annotation.color.name}',
                            ),
                          },
                        ),
                        child: ListTile(
                          leading: _AnnotationColorMarker(
                            color: Color(annotation.color.argb),
                            type: annotation.type,
                          ),
                          title: Text(
                            annotation.note ??
                                annotation.selectedText ??
                                strings.get(
                                  'annotation.type.${annotation.type.name}',
                                ),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          subtitle: Text(
                            [
                              strings.get(
                                'reader.page',
                                parameters: {'page': annotation.pageIndex + 1},
                              ),
                              if (annotation.tags.isNotEmpty)
                                annotation.tags.take(2).join(', '),
                              if (annotation.status ==
                                  AnnotationAttachmentStatus.reviewRequired)
                                strings.get('reader.reviewPosition'),
                            ].join(' · '),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          onTap: () => widget.controller.navigateTo(
                            annotation.pageIndex,
                            reason: NavigationReason.annotation,
                            label: strings.get(
                              'reader.annotationOnPage',
                              parameters: {'page': annotation.pageIndex + 1},
                            ),
                          ),
                          trailing: PopupMenuButton<_AnnotationAction>(
                            tooltip: strings.get('reader.annotationActions'),
                            onSelected: (action) =>
                                _runAction(action, annotation),
                            itemBuilder: (context) => [
                              PopupMenuItem(
                                value: _AnnotationAction.edit,
                                child: Text(strings.get('action.edit')),
                              ),
                              PopupMenuItem(
                                value: _AnnotationAction.copy,
                                child: Text(strings.get('action.copyContent')),
                              ),
                              PopupMenuItem(
                                value: _AnnotationAction.delete,
                                child: Text(strings.get('action.delete')),
                              ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }

  Future<void> _runAction(
    _AnnotationAction action,
    PdfAnnotation annotation,
  ) async {
    final controller = widget.controller.annotations!;
    switch (action) {
      case _AnnotationAction.edit:
        await _editAnnotation(annotation);
      case _AnnotationAction.copy:
        await Clipboard.setData(
          ClipboardData(text: annotation.note ?? annotation.selectedText ?? ''),
        );
      case _AnnotationAction.delete:
        controller.delete(annotation);
        if (!mounted) return;
        final strings = YamantakaLocalizations.of(context);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(strings.get('reader.annotationDeleted')),
            action: SnackBarAction(
              label: strings.get('action.undo'),
              onPressed: () => controller.undoDelete(),
            ),
          ),
        );
    }
  }

  Future<void> _editAnnotation([PdfAnnotation? annotation]) async {
    final strings = YamantakaLocalizations.of(context);
    final noteController = TextEditingController(text: annotation?.note ?? '');
    final tagsController = TextEditingController(
      text: annotation?.tags.join(', ') ?? '',
    );
    var type = annotation?.type ?? AnnotationType.pageComment;
    var color = annotation?.color ?? AnnotationColor.yellow;
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(
            strings.get(
              annotation == null
                  ? 'reader.newAnnotation'
                  : 'reader.editAnnotation',
            ),
          ),
          content: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 460),
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  DropdownButtonFormField<AnnotationType>(
                    initialValue: type,
                    decoration: InputDecoration(
                      labelText: strings.get('reader.type'),
                    ),
                    items: AnnotationType.values
                        .map(
                          (value) => DropdownMenuItem(
                            value: value,
                            child: Text(
                              strings.get('annotation.type.${value.name}'),
                            ),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => type = value);
                      }
                    },
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  DropdownButtonFormField<AnnotationColor>(
                    initialValue: color,
                    decoration: InputDecoration(
                      labelText: strings.get('reader.color'),
                    ),
                    items: AnnotationColor.values
                        .map(
                          (value) => DropdownMenuItem(
                            value: value,
                            child: Text(
                              strings.get('annotation.color.${value.name}'),
                            ),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => color = value);
                      }
                    },
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  TextField(
                    controller: noteController,
                    autofocus: true,
                    maxLength: AppConfig.maxAnnotationNoteLength,
                    maxLines: 6,
                    decoration: InputDecoration(
                      labelText: strings.get('reader.comment'),
                      alignLabelWithHint: true,
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  TextField(
                    controller: tagsController,
                    maxLength: 160,
                    decoration: InputDecoration(
                      labelText: strings.get('reader.optionalTags'),
                      hintText: strings.get('reader.tagsHint'),
                    ),
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(strings.get('action.cancel')),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(strings.get('action.save')),
            ),
          ],
        ),
      ),
    );
    if (result == true) {
      final tags = tagsController.text
          .split(',')
          .map((tag) => tag.trim())
          .where((tag) => tag.isNotEmpty)
          .toList(growable: false);
      if (annotation == null) {
        widget.controller.annotations!.create(
          pageIndex: widget.controller.currentPageIndex,
          type: type,
          color: color,
          note: noteController.text,
          pageRotation: widget.controller.preferences?.pageRotation ?? 0,
          tags: tags,
        );
      } else {
        widget.controller.annotations!.update(
          annotation.copyWith(
            type: type,
            color: color,
            note: noteController.text,
            tags: tags,
          ),
        );
      }
    }
    noteController.dispose();
    tagsController.dispose();
  }
}

enum _AnnotationAction { edit, copy, delete }

final class _AnnotationColorMarker extends StatelessWidget {
  const _AnnotationColorMarker({required this.color, required this.type});

  final Color color;
  final AnnotationType type;

  @override
  Widget build(BuildContext context) => Tooltip(
    message: YamantakaLocalizations.of(
      context,
    ).get('annotation.type.${type.name}'),
    child: Container(
      width: 28,
      height: 28,
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(AppRadii.control),
        border: Border.all(color: context.appColors.borderStrong),
      ),
      child: Icon(
        type == AnnotationType.pageComment
            ? Icons.chat_bubble_outline_rounded
            : Icons.format_color_fill_rounded,
        size: 16,
      ),
    ),
  );
}

final class _SearchPanel extends StatelessWidget {
  const _SearchPanel({required this.controller, required this.textController});

  final ReaderController controller;
  final TextEditingController textController;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final search = controller.search;
    return ListenableBuilder(
      listenable: search,
      builder: (context, _) => Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(AppSpacing.space12),
            child: TextField(
              key: const Key('pdf-search-field'),
              controller: textController,
              autofocus: true,
              textInputAction: TextInputAction.search,
              decoration: InputDecoration(
                labelText: strings.get('command.search'),
                hintText: strings.get('reader.wordOrPhrase'),
                suffixIcon: search.isSearching
                    ? IconButton(
                        tooltip: strings.get('reader.cancelSearch'),
                        onPressed: search.cancel,
                        icon: const Icon(Icons.close_rounded),
                      )
                    : IconButton(
                        tooltip: strings.get('reader.search'),
                        onPressed: () => search.search(textController.text),
                        icon: const Icon(Icons.search_rounded),
                      ),
              ),
              onSubmitted: search.search,
            ),
          ),
          if (search.isSearching)
            Semantics(
              liveRegion: true,
              label: strings.get('reader.searchingDocument'),
              child: Column(
                children: [
                  const LinearProgressIndicator(minHeight: 2),
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: AppSpacing.space12,
                      vertical: AppSpacing.space8,
                    ),
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: Text(strings.get('reader.searchingCancelable')),
                    ),
                  ),
                ],
              ),
            ),
          if (!search.isSearching && search.matches.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(
                AppSpacing.space12,
                0,
                AppSpacing.space8,
                AppSpacing.space8,
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      strings.get(
                        'reader.searchResultIndicator',
                        parameters: {
                          'current': search.activeIndex + 1,
                          'count': search.matches.length,
                        },
                      ),
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ),
                  IconButton(
                    tooltip: strings.get('reader.previousResult'),
                    onPressed: search.previous,
                    icon: const Icon(Icons.keyboard_arrow_up_rounded),
                  ),
                  IconButton(
                    tooltip: strings.get('reader.nextResult'),
                    onPressed: search.next,
                    icon: const Icon(Icons.keyboard_arrow_down_rounded),
                  ),
                ],
              ),
            ),
          if (search.noExtractableText)
            _PanelEmptyState(
              icon: Icons.image_search_outlined,
              title: strings.get('reader.noExtractableText'),
              message: strings.get('reader.noExtractableText.description'),
            )
          else if (!search.isSearching &&
              search.query.isNotEmpty &&
              search.matches.isEmpty)
            _PanelEmptyState(
              icon: Icons.search_off_rounded,
              title: strings.searchNoResultsTitle,
              message: strings.get('reader.noDocumentResults.description'),
            ),
          if (search.error case final error?)
            Builder(
              builder: (context) {
                final presentation = FailurePresentations.forFailure(
                  error,
                  strings: YamantakaLocalizations.of(context),
                );
                return Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: AppStatusLabel(
                    icon: Icons.error_outline_rounded,
                    label: '${presentation.title}: ${presentation.message}',
                    tone: AppStatusTone.error,
                  ),
                );
              },
            ),
          Expanded(
            child: ListView.separated(
              itemCount: search.matches.length,
              separatorBuilder: (_, _) => const Divider(
                indent: AppSpacing.space12,
                endIndent: AppSpacing.space12,
              ),
              itemBuilder: (context, index) {
                final match = search.matches[index];
                return ListTile(
                  selected: index == search.activeIndex,
                  title: Text(
                    match.snippet.isEmpty
                        ? strings.get('reader.match')
                        : match.snippet,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                  ),
                  subtitle: Text(
                    strings.get(
                      'reader.page',
                      parameters: {'page': match.pageIndex + 1},
                    ),
                  ),
                  onTap: () => search.select(index),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

final class _PanelEmptyState extends StatelessWidget {
  const _PanelEmptyState({
    required this.icon,
    required this.title,
    required this.message,
  });

  final IconData icon;
  final String title;
  final String message;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.space24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, color: context.appColors.textMuted, size: 24),
            const SizedBox(height: AppSpacing.space12),
            Text(title, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: AppSpacing.space6),
            Text(
              message,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}
