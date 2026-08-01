import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_tokens.dart';
import '../application/library_controller.dart';
import '../application/library_filter.dart';
import '../domain/library_artifact_layout.dart';
import '../domain/library_document.dart';

final class ArtifactLibraryView extends StatefulWidget {
  const ArtifactLibraryView({
    super.key,
    required this.controller,
    required this.documents,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.initialReadingState,
    required this.onReadingStateChanged,
    this.onDocumentInfo,
    this.onRelocateDocument,
    this.onToggleFavorite,
    this.onChangeCover,
    this.onRevealDocument,
    this.onRemoveDocument,
  });

  final LibraryController controller;
  final List<LibraryDocument> documents;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final LibraryReadingState initialReadingState;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;

  @override
  State<ArtifactLibraryView> createState() => _ArtifactLibraryViewState();
}

final class _ArtifactLibraryViewState extends State<ArtifactLibraryView> {
  late final ScrollController _scrollController = ScrollController(
    initialScrollOffset: widget.controller.scrollOffset,
  )..addListener(_rememberScroll);
  late final TextEditingController _searchController = TextEditingController(
    text: widget.controller.query,
  );
  final FocusNode _libraryFocusNode = FocusNode(debugLabel: 'artifact-library');
  final FocusNode _searchFocusNode = FocusNode(debugLabel: 'library-search');
  String? _hoveredDocumentId;
  late LibraryReadingState _readingState;
  late LibrarySortOrder _sortOrder;
  late bool _favoritesOnly;
  late bool _searchOpen;

  @override
  void initState() {
    super.initState();
    _readingState = LibraryReadingState.values.firstWhere(
      (state) => state.name == widget.controller.readingState,
      orElse: () => widget.initialReadingState,
    );
    _sortOrder = LibrarySortOrder.values.firstWhere(
      (order) => order.name == widget.controller.sortOrder,
      orElse: () => LibrarySortOrder.lastOpened,
    );
    _favoritesOnly = widget.controller.favoritesOnly;
    _searchOpen = widget.controller.query.isNotEmpty;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_searchOpen) {
        _searchFocusNode.requestFocus();
      } else {
        _libraryFocusNode.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    _scrollController
      ..removeListener(_rememberScroll)
      ..dispose();
    _searchController.dispose();
    _libraryFocusNode.dispose();
    _searchFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final reduceMotion = MediaQuery.disableAnimationsOf(context);
    final result = evaluateLibraryDocuments(
      widget.documents,
      query: widget.controller.query,
      sortOrder: _sortOrder,
      readingState: _readingState,
      favoritesOnly: _favoritesOnly,
    );
    final selected = _documentById(
      result.documents,
      widget.controller.selectedDocumentId,
    );
    final hovered = _documentById(result.documents, _hoveredDocumentId);
    final describedDocument = hovered ?? selected;
    final activeFilter =
        _readingState != LibraryReadingState.all ||
        _sortOrder != LibrarySortOrder.lastOpened ||
        _favoritesOnly;

    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyF, control: true):
            _openSearch,
        const SingleActivator(LogicalKeyboardKey.escape): _handleEscape,
        const SingleActivator(LogicalKeyboardKey.enter): () =>
            _openSelected(result.documents),
        const SingleActivator(LogicalKeyboardKey.delete): () =>
            _deleteSelected(result.documents),
        const SingleActivator(LogicalKeyboardKey.arrowLeft): () =>
            _navigate(result.documents, ArtifactNavigationDirection.left),
        const SingleActivator(LogicalKeyboardKey.arrowRight): () =>
            _navigate(result.documents, ArtifactNavigationDirection.right),
        const SingleActivator(LogicalKeyboardKey.arrowUp): () =>
            _navigate(result.documents, ArtifactNavigationDirection.up),
        const SingleActivator(LogicalKeyboardKey.arrowDown): () =>
            _navigate(result.documents, ArtifactNavigationDirection.down),
        const SingleActivator(LogicalKeyboardKey.home): () =>
            _navigate(result.documents, ArtifactNavigationDirection.first),
        const SingleActivator(LogicalKeyboardKey.end): () =>
            _navigate(result.documents, ArtifactNavigationDirection.last),
      },
      child: Focus(
        focusNode: _libraryFocusNode,
        autofocus: true,
        child: Stack(
          children: [
            Column(
              children: [
                _LibraryCommandBar(
                  resultCount: result.documents.length,
                  totalCount: widget.documents.length,
                  query: widget.controller.query,
                  counts: result.counts,
                  readingState: _readingState,
                  sortOrder: _sortOrder,
                  favoritesOnly: _favoritesOnly,
                  activeFilter: activeFilter,
                  onSearch: _openSearch,
                  onFilter: _applyFilterCommand,
                ),
                Expanded(
                  child: result.documents.isEmpty
                      ? _NoArtifactResults(
                          query: widget.controller.query,
                          onReset: _resetFilters,
                        )
                      : _ArtifactCanvas(
                          controller: _scrollController,
                          documents: result.documents,
                          selectedDocumentId:
                              widget.controller.selectedDocumentId,
                          reserveInfoSpace: describedDocument != null,
                          reduceMotion: reduceMotion,
                          onSelect: _select,
                          onOpen: _open,
                          onHover: (documentId) =>
                              setState(() => _hoveredDocumentId = documentId),
                          onMenu: _showDocumentMenu,
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
                    resultCount: result.documents.length,
                    onChanged: _updateQuery,
                    onClose: _closeSearch,
                  ),
                ),
              ),
            if (describedDocument != null)
              Positioned(
                left: AppSpacing.space20,
                right: AppSpacing.space20,
                bottom: AppSpacing.space20,
                child: IgnorePointer(
                  child: Align(
                    alignment: Alignment.bottomCenter,
                    child: DocumentInfoOverlay(document: describedDocument),
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
    _libraryFocusNode.requestFocus();
  }

  void _updateQuery(String value) {
    widget.controller.query = value;
    setState(() {});
  }

  void _handleEscape() {
    if (_searchOpen) {
      _closeSearch();
      return;
    }
    if (widget.controller.selectedDocumentId != null) {
      widget.controller.selectDocument(null);
      return;
    }
    _libraryFocusNode.requestFocus();
  }

  void _select(LibraryDocument document) {
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

  void _deleteSelected(List<LibraryDocument> documents) {
    final selected = _documentById(
      documents,
      widget.controller.selectedDocumentId,
    );
    if (selected != null) widget.onRemoveDocument?.call(selected);
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
    final nextIndex = LibraryArtifactLayout.indexAfterNavigation(
      currentIndex: currentIndex,
      itemCount: documents.length,
      columns: metrics.columns,
      direction: direction,
    );
    if (nextIndex < 0) return;
    widget.controller.selectDocument(documents[nextIndex].id);
    if (_scrollController.hasClients && documents.length > 6) {
      final row = nextIndex ~/ metrics.columns;
      final offset = row * metrics.cellExtent;
      _scrollController.animateTo(
        offset.clamp(0, _scrollController.position.maxScrollExtent),
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
    _select(document);
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
          child: Text(
            document.readingProgress > 0 ? 'Continuar leyendo' : 'Abrir',
          ),
        ),
        PopupMenuItem(
          value: _ArtifactMenuAction.favorite,
          child: Text(
            document.isFavorite
                ? 'Quitar de favoritos'
                : 'Marcar como favorito',
          ),
        ),
        const PopupMenuItem(
          value: _ArtifactMenuAction.cover,
          child: Text('Cambiar portada'),
        ),
        const PopupMenuItem(
          value: _ArtifactMenuAction.info,
          child: Text('Mostrar información'),
        ),
        if (!document.isMissing)
          const PopupMenuItem(
            value: _ArtifactMenuAction.reveal,
            child: Text('Mostrar en el explorador'),
          ),
        const PopupMenuDivider(),
        const PopupMenuItem(
          value: _ArtifactMenuAction.remove,
          child: Text('Eliminar de la biblioteca'),
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
    setState(() {
      switch (command) {
        case _FilterCommand.all:
          _readingState = LibraryReadingState.all;
        case _FilterCommand.inProgress:
          _readingState = LibraryReadingState.inProgress;
        case _FilterCommand.notStarted:
          _readingState = LibraryReadingState.notStarted;
        case _FilterCommand.completed:
          _readingState = LibraryReadingState.completed;
        case _FilterCommand.favorites:
          _favoritesOnly = !_favoritesOnly;
        case _FilterCommand.lastOpened:
          _sortOrder = LibrarySortOrder.lastOpened;
        case _FilterCommand.dateAdded:
          _sortOrder = LibrarySortOrder.dateAdded;
        case _FilterCommand.title:
          _sortOrder = LibrarySortOrder.title;
        case _FilterCommand.progress:
          _sortOrder = LibrarySortOrder.progress;
        case _FilterCommand.reset:
          _readingState = LibraryReadingState.all;
          _sortOrder = LibrarySortOrder.lastOpened;
          _favoritesOnly = false;
      }
      widget.controller.readingState = _readingState.name;
      widget.controller.sortOrder = _sortOrder.name;
      widget.controller.favoritesOnly = _favoritesOnly;
    });
    widget.onReadingStateChanged(_readingState);
  }

  void _resetFilters() {
    _searchController.clear();
    widget.controller.query = '';
    _applyFilterCommand(_FilterCommand.reset);
  }
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
    required this.onSearch,
    required this.onFilter,
  });

  final int resultCount;
  final int totalCount;
  final String query;
  final Map<LibraryReadingState, int> counts;
  final LibraryReadingState readingState;
  final LibrarySortOrder sortOrder;
  final bool favoritesOnly;
  final bool activeFilter;
  final VoidCallback onSearch;
  final ValueChanged<_FilterCommand> onFilter;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        AppSpacing.space20,
        AppSpacing.space12,
        AppSpacing.space20,
        AppSpacing.space8,
      ),
      child: Row(
        children: [
          Text(
            query.isEmpty && !activeFilter
                ? '$totalCount ${totalCount == 1 ? 'documento' : 'documentos'}'
                : '$resultCount de $totalCount',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const Spacer(),
          IconButton(
            key: const Key('library-search-button'),
            tooltip: 'Buscar en la biblioteca (Ctrl+F)',
            onPressed: onSearch,
            icon: const Icon(Icons.search_rounded),
          ),
          PopupMenuButton<_FilterCommand>(
            key: const Key('library-filter-button'),
            tooltip: 'Filtrar y ordenar',
            onSelected: onFilter,
            icon: Badge(
              isLabelVisible: activeFilter,
              smallSize: 6,
              backgroundColor: context.appColors.textSecondary,
              child: const Icon(Icons.tune_rounded),
            ),
            itemBuilder: (context) => [
              const PopupMenuItem(
                enabled: false,
                height: 32,
                child: Text('Estado'),
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
                  child: Text('${state.label} · ${counts[state] ?? 0}'),
                ),
              const PopupMenuDivider(),
              CheckedPopupMenuItem(
                value: _FilterCommand.favorites,
                checked: favoritesOnly,
                child: const Text('Solo favoritos'),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(
                enabled: false,
                height: 32,
                child: Text('Orden'),
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
                  child: Text(switch (order) {
                    LibrarySortOrder.lastOpened => 'Última apertura',
                    LibrarySortOrder.dateAdded => 'Fecha de incorporación',
                    LibrarySortOrder.title => 'Título',
                    LibrarySortOrder.progress => 'Progreso',
                  }),
                ),
              if (activeFilter) ...[
                const PopupMenuDivider(),
                const PopupMenuItem(
                  value: _FilterCommand.reset,
                  child: Text('Restablecer filtros'),
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }
}

final class _ArtifactCanvas extends StatelessWidget {
  const _ArtifactCanvas({
    required this.controller,
    required this.documents,
    required this.selectedDocumentId,
    required this.reserveInfoSpace,
    required this.reduceMotion,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
  });

  final ScrollController controller;
  final List<LibraryDocument> documents;
  final String? selectedDocumentId;
  final bool reserveInfoSpace;
  final bool reduceMotion;
  final ValueChanged<LibraryDocument> onSelect;
  final ValueChanged<LibraryDocument> onOpen;
  final ValueChanged<String?> onHover;
  final void Function(LibraryDocument document, Offset position) onMenu;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final horizontalPadding = AppSpacing.pagePadding(constraints.maxWidth);
        final metrics = LibraryArtifactLayout.metricsFor(
          viewportWidth: constraints.maxWidth - horizontalPadding * 2,
          documentCount: documents.length,
        );
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
                            child: ArtifactDocument(
                              key: ValueKey('artifact-${document.id}'),
                              document: document,
                              width: metrics.artifactWidth,
                              selected: selectedDocumentId == document.id,
                              reduceMotion: reduceMotion,
                              onSelect: () => onSelect(document),
                              onOpen: () => onOpen(document),
                              onHover: (hovered) =>
                                  onHover(hovered ? document.id : null),
                              onMenu: (position) => onMenu(document, position),
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
                  mainAxisExtent: metrics.cellExtent,
                  mainAxisSpacing: AppSpacing.space16,
                  crossAxisSpacing: AppSpacing.space16,
                ),
                itemBuilder: (context, index) {
                  final document = documents[index];
                  return Center(
                    child: ArtifactDocument(
                      key: ValueKey('artifact-${document.id}'),
                      document: document,
                      width: metrics.artifactWidth,
                      selected: selectedDocumentId == document.id,
                      reduceMotion: reduceMotion,
                      onSelect: () => onSelect(document),
                      onOpen: () => onOpen(document),
                      onHover: (hovered) =>
                          onHover(hovered ? document.id : null),
                      onMenu: (position) => onMenu(document, position),
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
}

final class ArtifactDocument extends StatefulWidget {
  const ArtifactDocument({
    super.key,
    required this.document,
    required this.width,
    required this.selected,
    required this.reduceMotion,
    required this.onSelect,
    required this.onOpen,
    required this.onHover,
    required this.onMenu,
  });

  final LibraryDocument document;
  final double width;
  final bool selected;
  final bool reduceMotion;
  final VoidCallback onSelect;
  final VoidCallback onOpen;
  final ValueChanged<bool> onHover;
  final ValueChanged<Offset> onMenu;

  @override
  State<ArtifactDocument> createState() => _ArtifactDocumentState();
}

final class _ArtifactDocumentState extends State<ArtifactDocument> {
  bool _hovered = false;
  bool _focused = false;

  @override
  Widget build(BuildContext context) {
    final jitter = LibraryArtifactLayout.jitterFor(widget.document.id);
    final emphasized = widget.selected || _hovered || _focused;
    final scale = widget.reduceMotion
        ? 1.0
        : widget.selected
        ? 1.055
        : _hovered
        ? 1.02
        : jitter.scale;
    final elevation = widget.selected
        ? 8.0
        : _hovered
        ? 3.0
        : 0.0;
    final aspectRatio =
        widget.document.coverMetadata?.pageAspectRatio.clamp(0.25, 3.2) ??
        0.704;
    final height = widget.width / aspectRatio;
    final maxHeight = widget.width * 1.62;
    final fittedHeight = math.min(height, maxHeight);
    final duration = widget.reduceMotion ? Duration.zero : AppMotion.standard;
    final coverPath = widget.selected
        ? widget.document.coverMetadata?.largeThumbnailPath ??
              widget.document.thumbnailPath
        : widget.document.coverMetadata?.smallThumbnailPath ??
              widget.document.thumbnailPath;

    return Semantics(
      button: true,
      selected: widget.selected,
      label: _artifactSemanticLabel(widget.document),
      hint: 'Un clic selecciona; doble clic o Enter abre; menú contextual',
      child: Focus(
        onFocusChange: (focused) {
          setState(() => _focused = focused);
          if (focused) widget.onSelect();
          widget.onHover(focused);
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
                offset: Offset(jitter.offsetX, jitter.offsetY),
                child: Transform.rotate(
                  angle: widget.reduceMotion
                      ? 0
                      : jitter.rotationDegrees * math.pi / 180,
                  child: AnimatedScale(
                    duration: duration,
                    curve: Curves.easeOutCubic,
                    scale: scale,
                    child: AnimatedContainer(
                      duration: duration,
                      curve: Curves.easeOutCubic,
                      transform: Matrix4.translationValues(0, -elevation, 0),
                      width: widget.width,
                      height: fittedHeight,
                      decoration: BoxDecoration(
                        color: context.appColors.surfaceSunken,
                        borderRadius: BorderRadius.circular(AppRadii.indicator),
                        border: Border.all(
                          color: _focused
                              ? context.appColors.focusRing
                              : widget.selected
                              ? context.appColors.accentPrimary
                              : context.appColors.borderSubtle,
                          width: _focused
                              ? 2
                              : widget.selected
                              ? 1.5
                              : 1,
                        ),
                        boxShadow: [
                          BoxShadow(
                            color: context.appColors.readerPageShadow
                                .withValues(
                                  alpha: widget.selected ? 0.72 : 0.38,
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
                                ? 0.48
                                : emphasized
                                ? 1
                                : 0.84,
                            child: coverPath == null
                                ? _ArtifactPlaceholder(
                                    document: widget.document,
                                  )
                                : ColorFiltered(
                                    colorFilter: ColorFilter.matrix(
                                      _saturationMatrix(emphasized ? 1 : 0.42),
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
                                      filterQuality: FilterQuality.high,
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
                              alignment: Alignment.bottomLeft,
                              child: FractionallySizedBox(
                                widthFactor: widget.document.readingProgress,
                                child: ColoredBox(
                                  color: context.appColors.textPrimary,
                                  child: const SizedBox(height: 2),
                                ),
                              ),
                            ),
                          if (widget.document.readingProgress >= 1)
                            Positioned(
                              right: AppSpacing.space8,
                              bottom: AppSpacing.space8,
                              child: Tooltip(
                                message: 'Terminado',
                                child: Container(
                                  width: 9,
                                  height: 9,
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
                                message: 'Favorito',
                                child: Icon(
                                  Icons.bookmark_rounded,
                                  size: 15,
                                  color: context.appColors.textPrimary,
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

final class _ArtifactPlaceholder extends StatelessWidget {
  const _ArtifactPlaceholder({required this.document, this.failed = false});

  final LibraryDocument document;
  final bool failed;

  @override
  Widget build(BuildContext context) {
    final label = (document.title ?? document.fileName).trim();
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
  const DocumentInfoOverlay({super.key, required this.document});

  final LibraryDocument document;

  @override
  Widget build(BuildContext context) {
    final completed = document.readingProgress >= 1;
    final notStarted = document.readingProgress <= 0;
    final progressLine = completed
        ? 'Terminado · ${document.pageCount} páginas'
        : notStarted
        ? 'Sin empezar · ${document.pageCount} páginas'
        : 'Página ${document.lastPageIndex + 1} de ${document.pageCount}'
              ' · ${(document.readingProgress * 100).round()} %';
    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 560),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: context.appColors.appBackground.withValues(alpha: 0.94),
          border: Border(
            top: BorderSide(color: context.appColors.borderSubtle),
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(
            AppSpacing.space20,
            AppSpacing.space12,
            AppSpacing.space20,
            AppSpacing.space8,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Tooltip(
                message: (document.title?.trim().isNotEmpty ?? false)
                    ? document.title!
                    : 'Documento sin título',
                child: Text(
                  (document.title?.trim().isNotEmpty ?? false)
                      ? document.title!
                      : 'Documento sin título',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              if (document.author case final author?)
                Text(
                  author,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              const SizedBox(height: AppSpacing.space4),
              Text(
                document.isMissing
                    ? 'Documento no disponible · localizar para continuar'
                    : '$progressLine · ${_relativeArtifactDate(document.lastOpenedAt)}',
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
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
                  decoration: const InputDecoration(
                    hintText: 'Buscar en la biblioteca…',
                    prefixIcon: Icon(Icons.search_rounded),
                  ),
                  onChanged: onChanged,
                ),
              ),
              const SizedBox(width: AppSpacing.space8),
              Semantics(
                liveRegion: true,
                label:
                    '$resultCount ${resultCount == 1 ? 'resultado' : 'resultados'}',
                child: Text(
                  '$resultCount',
                  style: Theme.of(context).textTheme.labelMedium,
                ),
              ),
              IconButton(
                key: const Key('library-filter-clear'),
                tooltip: 'Cerrar búsqueda (Escape)',
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
                  ? 'No hay documentos con estos filtros.'
                  : 'No hay documentos que coincidan con “$query”.',
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: AppSpacing.space8),
            TextButton(onPressed: onReset, child: const Text('Restablecer')),
          ],
        ),
      ),
    );
  }
}

enum _ArtifactMenuAction { open, favorite, cover, info, reveal, remove }

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

String _artifactSemanticLabel(LibraryDocument document) {
  final title = document.title ?? document.fileName;
  if (document.isMissing) return '$title, archivo no encontrado';
  if (document.readingProgress >= 1) return '$title, terminado';
  if (document.readingProgress > 0) {
    return '$title, ${(document.readingProgress * 100).round()} por ciento';
  }
  return '$title, sin empezar';
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

String _relativeArtifactDate(DateTime value) {
  final difference = DateTime.now().difference(value.toLocal());
  if (difference.inMinutes < 1) return 'abierto ahora';
  if (difference.inDays < 1) return 'abierto hoy';
  if (difference.inDays == 1) return 'abierto ayer';
  if (difference.inDays < 7) return 'abierto hace ${difference.inDays} días';
  final local = value.toLocal();
  return 'abierto ${local.day.toString().padLeft(2, '0')}/'
      '${local.month.toString().padLeft(2, '0')}';
}
