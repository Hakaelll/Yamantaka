import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../app/ui/yama_components.dart';
import '../../../core/i18n/app_localizations.dart';
import '../application/library_controller.dart';
import '../application/library_filter.dart';
import '../domain/document_enrichment.dart';
import '../domain/library_artifact_layout.dart';
import '../domain/library_document.dart';
import '../domain/library_query.dart';
import '../domain/library_selection.dart';
import '../../organization/domain/organization_models.dart';

part 'artifact_library_components.dart';

final class ArtifactLibraryView extends StatefulWidget {
  const ArtifactLibraryView({
    super.key,
    required this.controller,
    required this.documents,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.initialReadingState,
    required this.onReadingStateChanged,
    this.onBulkOperation,
    this.collections = const [],
    this.tags = const [],
    this.collectionMemberships = const {},
    this.tagMemberships = const {},
    this.onDocumentInfo,
    this.onRelocateDocument,
    this.onToggleFavorite,
    this.onChangeCover,
    this.onRevealDocument,
    this.onRemoveDocument,
    this.onReanalyzeDocument,
    this.onDocumentVisible,
    this.onDocumentHidden,
  });

  final LibraryController controller;
  final List<LibraryDocument> documents;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final LibraryReadingState initialReadingState;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final BulkOperationHandler? onBulkOperation;
  final List<DocumentCollection> collections;
  final List<DocumentTag> tags;
  final Map<String, Set<String>> collectionMemberships;
  final Map<String, Set<String>> tagMemberships;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;
  final Future<void> Function(LibraryDocument document)? onReanalyzeDocument;
  final ValueChanged<LibraryDocument>? onDocumentVisible;
  final ValueChanged<LibraryDocument>? onDocumentHidden;

  @override
  State<ArtifactLibraryView> createState() => _ArtifactLibraryViewState();
}

final class _ArtifactLibraryViewState extends State<ArtifactLibraryView> {
  late final ScrollController _scrollController =
      ScrollController(initialScrollOffset: widget.controller.scrollOffset)
        ..addListener(_rememberScroll)
        ..addListener(_loadMoreIfNeeded);
  late final TextEditingController _searchController = TextEditingController(
    text: widget.controller.query,
  );
  final FocusNode _libraryFocusNode = FocusNode(debugLabel: 'artifact-library');
  final FocusNode _searchFocusNode = FocusNode(debugLabel: 'library-search');
  final ValueNotifier<String?> _hoveredDocumentId = ValueNotifier(null);
  late bool _searchOpen;
  bool _selectionMode = false;
  _LibraryViewMode _viewMode = _LibraryViewMode.grid;
  final LibrarySelection _selection = LibrarySelection();
  String? _selectionAnchorId;
  Timer? _queryDebounce;
  final Set<String> _collapsedGroups = <String>{};

  @override
  void initState() {
    super.initState();
    _searchOpen = widget.controller.query.isNotEmpty;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_searchOpen) {
        _searchFocusNode.requestFocus();
      } else {
        _libraryFocusNode.requestFocus();
      }
    });
  }

  LibraryReadingState get _readingState =>
      LibraryReadingState.values
          .where((state) => state.name == widget.controller.readingState)
          .firstOrNull ??
      widget.initialReadingState;

  LibrarySortOrder get _sortOrder => switch (widget.controller.sortOrder) {
    'added' => LibrarySortOrder.dateAdded,
    'title' => LibrarySortOrder.title,
    'progress' => LibrarySortOrder.progress,
    _ => LibrarySortOrder.lastOpened,
  };

  bool get _favoritesOnly => widget.controller.favoritesOnly;

  @override
  void dispose() {
    _scrollController
      ..removeListener(_rememberScroll)
      ..removeListener(_loadMoreIfNeeded)
      ..dispose();
    _queryDebounce?.cancel();
    _searchController.dispose();
    _hoveredDocumentId.dispose();
    _libraryFocusNode.dispose();
    _searchFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final reduceMotion =
        MediaQuery.disableAnimationsOf(context) ||
        MediaQuery.highContrastOf(context);
    // Documents are already searched, filtered, sorted and paged by SQLite.
    // Presentation must not reinterpret a partial materialized page.
    final materializedCounts = _materializedReadingStateCounts(
      widget.documents,
    );
    final fullCounts = <LibraryReadingState, int>{
      LibraryReadingState.all:
          widget.controller.readingStateCounts[LibraryQueryReadingState.all] ??
          materializedCounts[LibraryReadingState.all]!,
      LibraryReadingState.inProgress:
          widget.controller.readingStateCounts[LibraryQueryReadingState
              .inProgress] ??
          materializedCounts[LibraryReadingState.inProgress]!,
      LibraryReadingState.notStarted:
          widget.controller.readingStateCounts[LibraryQueryReadingState
              .notStarted] ??
          materializedCounts[LibraryReadingState.notStarted]!,
      LibraryReadingState.completed:
          widget.controller.readingStateCounts[LibraryQueryReadingState
              .completed] ??
          materializedCounts[LibraryReadingState.completed]!,
    };
    _selection.retain(widget.documents.map((document) => document.id));
    final activeFilter =
        widget.controller.query.isNotEmpty ||
        _readingState != LibraryReadingState.all ||
        _sortOrder != LibrarySortOrder.lastOpened ||
        _favoritesOnly ||
        widget.controller.selectedAuthors.isNotEmpty ||
        widget.controller.selectedTopics.isNotEmpty ||
        widget.controller.grouping != LibraryGrouping.none;
    final activeFilterCount = <bool>[
      widget.controller.query.isNotEmpty,
      _readingState != LibraryReadingState.all,
      _sortOrder != LibrarySortOrder.lastOpened,
      _favoritesOnly,
      widget.controller.selectedAuthors.isNotEmpty,
      widget.controller.selectedTopics.isNotEmpty,
      widget.controller.grouping != LibraryGrouping.none,
    ].where((active) => active).length;

    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyF, control: true):
            _openSearch,
        const SingleActivator(LogicalKeyboardKey.escape): _handleEscape,
        const SingleActivator(LogicalKeyboardKey.enter): () =>
            _openSelected(widget.documents),
        const SingleActivator(LogicalKeyboardKey.f10, shift: true): () =>
            _showSelectedDocumentMenu(widget.documents),
        const SingleActivator(LogicalKeyboardKey.contextMenu): () =>
            _showSelectedDocumentMenu(widget.documents),
        const SingleActivator(LogicalKeyboardKey.delete): () =>
            _deleteSelected(widget.documents),
        const SingleActivator(LogicalKeyboardKey.arrowLeft): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.left),
        const SingleActivator(LogicalKeyboardKey.arrowRight): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.right),
        const SingleActivator(LogicalKeyboardKey.arrowUp): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.up),
        const SingleActivator(LogicalKeyboardKey.arrowDown): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.down),
        const SingleActivator(LogicalKeyboardKey.home): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.first),
        const SingleActivator(LogicalKeyboardKey.end): () =>
            _navigate(widget.documents, ArtifactNavigationDirection.last),
      },
      child: Focus(
        focusNode: _libraryFocusNode,
        autofocus: true,
        child: Stack(
          children: [
            Column(
              children: [
                _LibraryCommandBar(
                  resultCount: widget.documents.length,
                  totalCount: widget.controller.totalDocuments,
                  query: widget.controller.query,
                  counts: fullCounts,
                  readingState: _readingState,
                  sortOrder: _sortOrder,
                  favoritesOnly: _favoritesOnly,
                  activeFilter: activeFilter,
                  activeFilterCount: activeFilterCount,
                  selectionMode: _selectionMode,
                  selectedCount: _selection.length,
                  viewMode: _viewMode,
                  selectedAuthorCount: widget.controller.selectedAuthors.length,
                  selectedTopicCount: widget.controller.selectedTopics.length,
                  grouping: widget.controller.grouping,
                  onSearch: _openSearch,
                  onFilter: _applyFilterCommand,
                  onFacets: _showFacetFilters,
                  onToggleSelectionMode: _toggleSelectionMode,
                  onViewModeChanged: (mode) => setState(() => _viewMode = mode),
                  onClearActiveFilter: _clearActiveFilter,
                ),
                if (_selectionMode && _selection.isNotEmpty)
                  _BulkActionBar(
                    selectedCount: _selection.length,
                    onClear: _clearSelection,
                    onAction: _runBulkAction,
                    canRun: widget.onBulkOperation != null,
                  ),
                Expanded(
                  child: widget.documents.isEmpty
                      ? _NoArtifactResults(
                          query: widget.controller.query,
                          onReset: _resetFilters,
                        )
                      : TweenAnimationBuilder<double>(
                          key: ValueKey(_viewMode),
                          duration: AppMotion.whenAllowed(
                            reduceMotion,
                            AppMotion.fast,
                          ),
                          curve: Curves.easeOutCubic,
                          tween: Tween(begin: 0.88, end: 1),
                          builder: (context, opacity, child) =>
                              Opacity(opacity: opacity, child: child),
                          child: _ArtifactCanvas(
                            controller: _scrollController,
                            documents: widget.documents,
                            selectionListenableFor:
                                widget.controller.selectionListenableFor,
                            selectedDocumentIds: _selection.ids.toSet(),
                            selectionMode: _selectionMode,
                            reserveInfoSpace: widget.documents.isNotEmpty,
                            reduceMotion: reduceMotion,
                            grouping: widget.controller.grouping,
                            collections: widget.collections,
                            tags: widget.tags,
                            collectionMemberships: widget.collectionMemberships,
                            tagMemberships: widget.tagMemberships,
                            collapsedGroups: _collapsedGroups,
                            onToggleGroup: (group) => setState(() {
                              _collapsedGroups.contains(group)
                                  ? _collapsedGroups.remove(group)
                                  : _collapsedGroups.add(group);
                            }),
                            viewMode: _viewMode,
                            onSelect: _select,
                            onOpen: _open,
                            onHover: (documentId) =>
                                _hoveredDocumentId.value = documentId,
                            onMenu: _showDocumentMenu,
                            onDocumentVisible: widget.onDocumentVisible,
                            onDocumentHidden: widget.onDocumentHidden,
                          ),
                        ),
                ),
              ],
            ),
            if (_searchOpen)
              Positioned(
                top: AppSpacing.space12,
                left: AppSpacing.space20,
                right: AppSpacing.space20,
                child: Align(
                  alignment: Alignment.topCenter,
                  child: _SearchOverlay(
                    controller: _searchController,
                    focusNode: _searchFocusNode,
                    resultCount: widget.controller.totalDocuments,
                    onChanged: _updateQuery,
                    onClose: _closeSearch,
                  ),
                ),
              ),
            Positioned(
              left: AppSpacing.space20,
              right: AppSpacing.space20,
              bottom: AppSpacing.space20,
              child: ValueListenableBuilder<String?>(
                valueListenable: widget.controller.selectedDocumentListenable,
                builder: (context, selectedId, _) =>
                    ValueListenableBuilder<String?>(
                      valueListenable: _hoveredDocumentId,
                      builder: (context, hoveredId, _) {
                        final selectedDocument = _documentById(
                          widget.documents,
                          selectedId,
                        );
                        final describedDocument =
                            selectedDocument ??
                            _documentById(widget.documents, hoveredId);
                        if (describedDocument == null) {
                          if (selectedId != null && activeFilter) {
                            return Align(
                              alignment: Alignment.bottomCenter,
                              child: _FilteredSelectionNotice(
                                onReset: _resetFilters,
                                onDismiss: () =>
                                    widget.controller.selectDocument(null),
                              ),
                            );
                          }
                          return const SizedBox.shrink();
                        }
                        return Align(
                          alignment: Alignment.bottomCenter,
                          child: AnimatedSwitcher(
                            duration: AppMotion.whenAllowed(
                              reduceMotion,
                              AppMotion.inspector,
                            ),
                            child: DocumentInfoOverlay(
                              key: ValueKey(
                                'library-context-${describedDocument.id}',
                              ),
                              document: describedDocument,
                              windowClass: context.windowClass,
                              interactive: selectedDocument != null,
                              onPrimary: () => _open(describedDocument),
                              onToggleFavorite: widget.onToggleFavorite == null
                                  ? null
                                  : () => unawaited(
                                      widget.onToggleFavorite!(
                                        describedDocument,
                                      ),
                                    ),
                              onShowInfo: widget.onDocumentInfo == null
                                  ? null
                                  : () => unawaited(
                                      widget.onDocumentInfo!(describedDocument),
                                    ),
                              onDismiss: () =>
                                  widget.controller.selectDocument(null),
                            ),
                          ),
                        );
                      },
                    ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _rememberScroll() {
    widget.controller.scrollOffset = _scrollController.offset;
  }

  void _loadMoreIfNeeded() {
    if (!_scrollController.hasClients || !widget.controller.hasMore) return;
    if (_scrollController.position.extentAfter > 720) return;
    unawaited(widget.controller.loadNextPage());
  }

  void _openSearch() {
    if (!_searchOpen) setState(() => _searchOpen = true);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _searchFocusNode.requestFocus();
      _searchController.selection = TextSelection(
        baseOffset: 0,
        extentOffset: _searchController.text.length,
      );
    });
  }

  void _closeSearch() {
    _searchController.clear();
    widget.controller.query = '';
    setState(() => _searchOpen = false);
    unawaited(widget.controller.reload());
    _libraryFocusNode.requestFocus();
  }

  void _updateQuery(String value) {
    widget.controller.query = value;
    setState(() {});
    _queryDebounce?.cancel();
    _queryDebounce = Timer(AppMotion.searchDebounce, () {
      if (mounted) unawaited(widget.controller.reload());
    });
  }

  void _handleEscape() {
    if (_searchOpen) {
      _closeSearch();
      return;
    }
    if (widget.controller.selectedDocumentId != null) {
      widget.controller.selectDocument(null);
      _clearSelection();
      return;
    }
    if (_selection.isNotEmpty || _selectionMode) {
      _clearSelection();
      return;
    }
    _libraryFocusNode.requestFocus();
  }

  void _select(LibraryDocument document) {
    final keyboard = HardwareKeyboard.instance;
    final range =
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.shiftLeft) ||
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.shiftRight);
    final additive =
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.controlLeft) ||
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.controlRight) ||
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.metaLeft) ||
        keyboard.logicalKeysPressed.contains(LogicalKeyboardKey.metaRight);
    if (_selectionMode || range || additive) {
      final visible = widget.documents;
      if (range && _selectionAnchorId != null) {
        _selection.selectRange(
          visible.map((item) => item.id).toList(growable: false),
          _selectionAnchorId!,
          document.id,
          additive: additive,
        );
      } else {
        _selection.toggle(
          document.id,
          selected: _selectionMode
              ? null
              : additive
              ? !_selection.contains(document.id)
              : true,
        );
      }
      _selectionAnchorId = document.id;
      setState(() {});
    }
    widget.controller.selectDocument(document.id);
    _libraryFocusNode.requestFocus();
  }

  void _selectForContextMenu(LibraryDocument document) {
    if (_selectionMode && !_selection.contains(document.id)) {
      setState(() {
        _selection
          ..clear()
          ..toggle(document.id, selected: true);
        _selectionAnchorId = document.id;
      });
    }
    widget.controller.selectDocument(document.id);
    _libraryFocusNode.requestFocus();
  }

  void _open(LibraryDocument document) {
    _select(document);
    if (document.isMissing) {
      widget.onRelocateDocument?.call(document);
    } else {
      widget.onOpenPath(document.path);
    }
  }

  void _openSelected(List<LibraryDocument> documents) {
    final selected = _documentById(
      documents,
      widget.controller.selectedDocumentId,
    );
    if (selected != null) _open(selected);
  }

  void _showSelectedDocumentMenu(List<LibraryDocument> documents) {
    final selected = _documentById(
      documents,
      widget.controller.selectedDocumentId,
    );
    if (selected == null) return;
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final position = overlay.localToGlobal(
      Offset(overlay.size.width / 2, overlay.size.height / 2),
    );
    unawaited(_showDocumentMenu(selected, position));
  }

  void _deleteSelected(List<LibraryDocument> documents) {
    if (_selection.isNotEmpty) {
      unawaited(_runBulkAction(BulkLibraryAction.removeFromLibrary));
      return;
    }
    final selected = _documentById(
      documents,
      widget.controller.selectedDocumentId,
    );
    if (selected != null) widget.onRemoveDocument?.call(selected);
  }

  void _toggleSelectionMode() {
    setState(() {
      _selectionMode = !_selectionMode;
      if (!_selectionMode) _selection.clear();
    });
    if (_selectionMode) {
      unawaited(
        SemanticsService.sendAnnouncement(
          View.of(context),
          YamantakaLocalizations.of(context).get(
            'library.selectionActivated',
            parameters: {'count': _selection.length},
          ),
          Directionality.of(context),
        ),
      );
    }
    _libraryFocusNode.requestFocus();
  }

  void _clearSelection() {
    if (_selection.isEmpty && !_selectionMode) return;
    setState(() {
      _selection.clear();
      _selectionAnchorId = null;
      _selectionMode = false;
    });
    widget.controller.selectDocument(null);
    _libraryFocusNode.requestFocus();
  }

  Future<void> _runBulkAction(BulkLibraryAction action) async {
    final strings = YamantakaLocalizations.of(context);
    final handler = widget.onBulkOperation;
    if (handler == null || _selection.isEmpty) return;
    final targetId = switch (action) {
      BulkLibraryAction.assignTag => await _chooseTag(),
      BulkLibraryAction.addToCollection => await _chooseCollection(
        title: strings.get('library.addToCollection'),
      ),
      BulkLibraryAction.removeFromCollection => await _chooseCollection(
        title: strings.get('library.removeFromCollection'),
      ),
      _ => null,
    };
    if ((action == BulkLibraryAction.assignTag ||
            action == BulkLibraryAction.addToCollection ||
            action == BulkLibraryAction.removeFromCollection) &&
        targetId == null) {
      return;
    }
    if (BulkOperationPolicy.requiresConfirmation(action)) {
      final confirmed = await _confirmBulkRemoval();
      if (!confirmed) return;
    }
    final ids = _selection.ids;
    late final BulkOperationSummary summary;
    try {
      summary = await handler(action, ids, targetId: targetId);
    } on Object {
      _showLocalNotice(strings.get('library.bulkFailed'));
      return;
    }
    if (!mounted) return;
    final failedIds = summary.results
        .where((result) => !result.success)
        .map((result) => result.id)
        .toList(growable: false);
    setState(() {
      _selection.clear();
      for (final id in failedIds) {
        _selection.toggle(id, selected: true);
      }
      _selectionAnchorId = null;
      _selectionMode = failedIds.isNotEmpty;
    });
    widget.controller.selectDocument(null);
    await _showBulkSummary(summary);
    _libraryFocusNode.requestFocus();
  }

  Future<String?> _chooseCollection({required String title}) async {
    final strings = YamantakaLocalizations.of(context);
    if (widget.collections.isEmpty) {
      _showLocalNotice(strings.get('library.noCollections'));
      return null;
    }
    return showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: Text(title),
        children: [
          for (final collection in widget.collections)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(context, collection.id),
              child: Semantics(
                button: true,
                label: strings.get(
                  'library.collectionSemantics',
                  parameters: {
                    'name': collection.name,
                    'count': collection.documentCount,
                  },
                ),
                child: Text(collection.name),
              ),
            ),
        ],
      ),
    );
  }

  Future<String?> _chooseTag() async {
    final strings = YamantakaLocalizations.of(context);
    if (widget.tags.isEmpty) {
      _showLocalNotice(strings.get('library.noTags'));
      return null;
    }
    return showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: Text(strings.get('library.assignTag')),
        children: [
          for (final tag in widget.tags)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(context, tag.id),
              child: Semantics(
                button: true,
                label: strings.get(
                  'library.tagSemantics',
                  parameters: {'name': tag.name, 'count': tag.usageCount},
                ),
                child: Text(tag.name),
              ),
            ),
        ],
      ),
    );
  }

  Future<bool> _confirmBulkRemoval() async {
    final strings = YamantakaLocalizations.of(context);
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(strings.get('library.removeDocuments')),
            content: Text(
              strings.get(
                'library.removeDocuments.description',
                parameters: {'count': _selection.length},
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(strings.get('action.cancel')),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(strings.get('library.remove')),
              ),
            ],
          ),
        ) ??
        false;
  }

  Future<void> _showBulkSummary(BulkOperationSummary summary) async {
    final strings = YamantakaLocalizations.of(context);
    final labels = <String, String>{
      for (final document in widget.documents)
        document.id: document.effectiveTitle,
    };
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('library.actionCompleted')),
        content: SizedBox(
          width: 460,
          child: ListView(
            shrinkWrap: true,
            children: [
              Text(
                strings.get(
                  'library.bulkSummary',
                  parameters: {
                    'succeeded': summary.succeeded,
                    'failed': summary.failed,
                  },
                ),
              ),
              const SizedBox(height: AppSpacing.space12),
              for (final result in summary.results)
                ListTile(
                  dense: true,
                  leading: Icon(
                    result.success
                        ? Icons.check_circle_outline
                        : Icons.error_outline,
                  ),
                  title: Text(
                    labels[result.id] ??
                        strings.get('library.selectedDocument'),
                  ),
                  subtitle: result.success
                      ? null
                      : Text(strings.get('library.operationFailed')),
                ),
            ],
          ),
        ),
        actions: [
          FilledButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.close')),
          ),
        ],
      ),
    );
  }

  void _showLocalNotice(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  void _navigate(
    List<LibraryDocument> documents,
    ArtifactNavigationDirection direction,
  ) {
    if (documents.isEmpty) return;
    final width = MediaQuery.sizeOf(context).width;
    final metrics = LibraryArtifactLayout.metricsFor(
      viewportWidth: width - AppSpacing.space40,
      documentCount: documents.length,
    );
    final currentIndex = math.max(
      0,
      documents.indexWhere(
        (document) => document.id == widget.controller.selectedDocumentId,
      ),
    );
    final columns = _viewMode == _LibraryViewMode.list ? 1 : metrics.columns;
    final nextIndex = LibraryArtifactLayout.indexAfterNavigation(
      currentIndex: currentIndex,
      itemCount: documents.length,
      columns: columns,
      direction: direction,
    );
    if (nextIndex < 0) return;
    widget.controller.selectDocument(documents[nextIndex].id);
    if (_scrollController.hasClients && documents.length > 6) {
      final row = nextIndex ~/ columns;
      final offset =
          row * (_viewMode == _LibraryViewMode.list ? 60 : metrics.cellExtent);
      _scrollController.animateTo(
        offset.clamp(0, _scrollController.position.maxScrollExtent).toDouble(),
        duration: MediaQuery.disableAnimationsOf(context)
            ? Duration.zero
            : AppMotion.standard,
        curve: Curves.easeOutCubic,
      );
    }
  }

  Future<void> _showDocumentMenu(
    LibraryDocument document,
    Offset position,
  ) async {
    final strings = YamantakaLocalizations.of(context);
    _selectForContextMenu(document);
    final overlay =
        Overlay.of(context).context.findRenderObject()! as RenderBox;
    final action = await showMenu<_ArtifactMenuAction>(
      context: context,
      position: RelativeRect.fromRect(
        Rect.fromPoints(position, position),
        Offset.zero & overlay.size,
      ),
      items: [
        PopupMenuItem(
          value: _ArtifactMenuAction.open,
          child: AppMenuItemContent(
            icon: document.isMissing
                ? Icons.link_rounded
                : Icons.menu_book_outlined,
            label: document.isMissing
                ? strings.get('library.locateFile')
                : document.readingProgress > 0
                ? strings.get('library.continueReading')
                : strings.get('action.open'),
          ),
        ),
        PopupMenuItem(
          value: _ArtifactMenuAction.favorite,
          child: AppMenuItemContent(
            icon: document.isFavorite
                ? Icons.bookmark_remove_outlined
                : Icons.bookmark_add_outlined,
            label: document.isFavorite
                ? strings.get('library.unmarkFavorite')
                : strings.get('library.markFavorite'),
          ),
        ),
        if (!document.isMissing)
          PopupMenuItem(
            value: _ArtifactMenuAction.cover,
            child: AppMenuItemContent(
              icon: Icons.image_outlined,
              label: strings.get('library.changeCover'),
            ),
          ),
        if (!document.isMissing && widget.onReanalyzeDocument != null)
          PopupMenuItem(
            value: _ArtifactMenuAction.reanalyze,
            child: AppMenuItemContent(
              icon: Icons.manage_search_rounded,
              label: strings.get('library.reanalyzeMetadata'),
            ),
          ),
        PopupMenuItem(
          value: _ArtifactMenuAction.info,
          child: AppMenuItemContent(
            icon: Icons.info_outline_rounded,
            label: strings.get('library.showInformation'),
          ),
        ),
        if (!document.isMissing)
          PopupMenuItem(
            value: _ArtifactMenuAction.reveal,
            child: AppMenuItemContent(
              icon: Icons.folder_open_outlined,
              label: strings.get('library.showInExplorer'),
            ),
          ),
        const PopupMenuDivider(),
        PopupMenuItem(
          value: _ArtifactMenuAction.remove,
          child: AppMenuItemContent(
            icon: Icons.remove_circle_outline,
            label: strings.get('library.removeFromLibrary'),
            destructive: true,
          ),
        ),
      ],
    );
    if (!mounted || action == null) return;
    switch (action) {
      case _ArtifactMenuAction.open:
        _open(document);
      case _ArtifactMenuAction.favorite:
        await widget.onToggleFavorite?.call(document);
      case _ArtifactMenuAction.cover:
        await widget.onChangeCover?.call(document);
      case _ArtifactMenuAction.reanalyze:
        await widget.onReanalyzeDocument?.call(document);
      case _ArtifactMenuAction.info:
        await widget.onDocumentInfo?.call(document);
      case _ArtifactMenuAction.reveal:
        await widget.onRevealDocument?.call(document);
      case _ArtifactMenuAction.remove:
        await widget.onRemoveDocument?.call(document);
    }
    _libraryFocusNode.requestFocus();
  }

  void _applyFilterCommand(_FilterCommand command) {
    switch (command) {
      case _FilterCommand.all:
        widget.controller.readingState = 'all';
      case _FilterCommand.inProgress:
        widget.controller.readingState = 'inProgress';
      case _FilterCommand.notStarted:
        widget.controller.readingState = 'notStarted';
      case _FilterCommand.completed:
        widget.controller.readingState = 'completed';
      case _FilterCommand.favorites:
        widget.controller.favoritesOnly = !widget.controller.favoritesOnly;
      case _FilterCommand.lastOpened:
        widget.controller.sortOrder = 'lastOpened';
      case _FilterCommand.dateAdded:
        widget.controller.sortOrder = 'added';
      case _FilterCommand.title:
        widget.controller.sortOrder = 'title';
      case _FilterCommand.progress:
        widget.controller.sortOrder = 'progress';
      case _FilterCommand.reset:
        widget.controller
          ..readingState = 'all'
          ..sortOrder = 'lastOpened'
          ..favoritesOnly = false
          ..selectedAuthors = const {}
          ..selectedTopics = const {}
          ..groupByAuthor = false;
    }
    widget.onReadingStateChanged(_readingState);
    unawaited(widget.controller.reload());
  }

  void _clearActiveFilter(_ActiveFilterKind filter) {
    switch (filter) {
      case _ActiveFilterKind.query:
        _searchController.clear();
        widget.controller.query = '';
      case _ActiveFilterKind.readingState:
        widget.controller.readingState = 'all';
      case _ActiveFilterKind.favorites:
        widget.controller.favoritesOnly = false;
      case _ActiveFilterKind.sort:
        widget.controller.sortOrder = 'lastOpened';
      case _ActiveFilterKind.authors:
        widget.controller.selectedAuthors = const {};
      case _ActiveFilterKind.topics:
        widget.controller.selectedTopics = const {};
      case _ActiveFilterKind.grouping:
        widget.controller.groupByAuthor = false;
    }
    widget.onReadingStateChanged(_readingState);
    unawaited(widget.controller.reload());
  }

  Future<void> _showFacetFilters() async {
    final strings = YamantakaLocalizations.of(context);
    var authors = Set<String>.from(widget.controller.selectedAuthors);
    var topics = Set<LibraryTopic>.from(widget.controller.selectedTopics);
    var grouping = widget.controller.grouping;
    final applied = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          icon: const Icon(Icons.account_tree_outlined),
          title: Text(strings.get('library.authorsAndTopics')),
          content: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 620, maxHeight: 560),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  DropdownButtonFormField<LibraryGrouping>(
                    initialValue: grouping,
                    decoration: InputDecoration(
                      labelText: strings.get('library.grouping'),
                      helperText: strings.get('library.grouping.description'),
                    ),
                    items: [
                      for (final value in LibraryGrouping.values)
                        DropdownMenuItem(
                          value: value,
                          child: Text(
                            strings.get('library.grouping.${value.name}'),
                          ),
                        ),
                    ],
                    onChanged: (value) => setDialogState(
                      () => grouping = value ?? LibraryGrouping.none,
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  Text(
                    strings.get('library.authors'),
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  if (widget.controller.authorFacets.isEmpty)
                    Text(strings.get('library.noAuthors'))
                  else
                    Wrap(
                      spacing: AppSpacing.space8,
                      runSpacing: AppSpacing.space8,
                      children: [
                        for (final facet in widget.controller.authorFacets)
                          FilterChip(
                            label: Text('${facet.value} · ${facet.count}'),
                            selected: authors.contains(
                              facet.value.trim().toLowerCase(),
                            ),
                            onSelected: (selected) => setDialogState(() {
                              final key = facet.value.trim().toLowerCase();
                              selected ? authors.add(key) : authors.remove(key);
                            }),
                          ),
                      ],
                    ),
                  const SizedBox(height: AppSpacing.space20),
                  Text(
                    strings.get('library.topics'),
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  Wrap(
                    spacing: AppSpacing.space8,
                    runSpacing: AppSpacing.space8,
                    children: [
                      for (final topic in LibraryTopic.values)
                        FilterChip(
                          label: Text(
                            '${strings.get('topic.${topic.name}')} · '
                            '${widget.controller.topicFacets[topic] ?? 0}',
                          ),
                          selected: topics.contains(topic),
                          onSelected: (selected) => setDialogState(() {
                            selected ? topics.add(topic) : topics.remove(topic);
                          }),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => setDialogState(() {
                authors.clear();
                topics.clear();
                grouping = LibraryGrouping.none;
              }),
              child: Text(strings.get('action.reset')),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(strings.get('action.cancel')),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(strings.get('action.apply')),
            ),
          ],
        ),
      ),
    );
    if (applied != true || !mounted) return;
    await widget.controller.setFacets(
      authors: authors,
      topics: topics,
      groupAuthors: false,
    );
    widget.controller.setGrouping(grouping);
    if (mounted) setState(() {});
  }

  void _resetFilters() {
    _searchController.clear();
    widget.controller.query = '';
    _applyFilterCommand(_FilterCommand.reset);
  }
}
