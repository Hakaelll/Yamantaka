import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/app_config.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/library_controller.dart';
import '../domain/library_document.dart';

enum LibrarySortOrder { lastOpened, title, progress }

enum LibraryReadingState { all, inProgress, notStarted, completed }

extension LibraryReadingStateLabel on LibraryReadingState {
  String get label => switch (this) {
    LibraryReadingState.all => 'Todos',
    LibraryReadingState.inProgress => 'En curso',
    LibraryReadingState.notStarted => 'Sin empezar',
    LibraryReadingState.completed => 'Terminados',
  };
}

final class LibraryFilterResult {
  const LibraryFilterResult({required this.documents, required this.counts});

  final List<LibraryDocument> documents;
  final Map<LibraryReadingState, int> counts;
}

LibraryFilterResult evaluateLibraryDocuments(
  Iterable<LibraryDocument> documents, {
  String query = '',
  LibrarySortOrder sortOrder = LibrarySortOrder.lastOpened,
  LibraryReadingState readingState = LibraryReadingState.all,
}) {
  final normalizedQuery = query.trim().toLowerCase();
  final visible = <LibraryDocument>[];
  final counts = <LibraryReadingState, int>{
    for (final state in LibraryReadingState.values) state: 0,
  };
  for (final document in documents) {
    final matchesText =
        normalizedQuery.isEmpty ||
        document.title?.toLowerCase().contains(normalizedQuery) == true ||
        document.author?.toLowerCase().contains(normalizedQuery) == true ||
        document.fileName.toLowerCase().contains(normalizedQuery);
    if (!matchesText) continue;
    final documentState = _readingStateOf(document);
    counts[LibraryReadingState.all] = counts[LibraryReadingState.all]! + 1;
    counts[documentState] = counts[documentState]! + 1;
    if (readingState == LibraryReadingState.all ||
        readingState == documentState) {
      visible.add(document);
    }
  }
  visible.sort((left, right) {
    final comparison = switch (sortOrder) {
      LibrarySortOrder.lastOpened => right.lastOpenedAt.compareTo(
        left.lastOpenedAt,
      ),
      LibrarySortOrder.title => _documentLabel(
        left,
      ).compareTo(_documentLabel(right)),
      LibrarySortOrder.progress => right.readingProgress.compareTo(
        left.readingProgress,
      ),
    };
    if (comparison != 0) return comparison;
    return right.lastOpenedAt.compareTo(left.lastOpenedAt);
  });
  return LibraryFilterResult(documents: visible, counts: counts);
}

List<LibraryDocument> filterAndSortLibraryDocuments(
  Iterable<LibraryDocument> documents, {
  String query = '',
  LibrarySortOrder sortOrder = LibrarySortOrder.lastOpened,
  LibraryReadingState readingState = LibraryReadingState.all,
}) => evaluateLibraryDocuments(
  documents,
  query: query,
  sortOrder: sortOrder,
  readingState: readingState,
).documents;

LibraryReadingState _readingStateOf(LibraryDocument document) {
  if (document.readingProgress >= 1) return LibraryReadingState.completed;
  if (document.readingProgress > 0) return LibraryReadingState.inProgress;
  return LibraryReadingState.notStarted;
}

final class LibraryScreen extends StatefulWidget {
  const LibraryScreen({
    super.key,
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.onOpenSettings,
    required this.showRecents,
    this.initialReadingFilter = LibraryReadingState.all,
    this.onReadingFilterChanged,
    this.onDocumentInfo,
    this.onRelocateDocument,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final VoidCallback onOpenSettings;
  final bool showRecents;
  final LibraryReadingState initialReadingFilter;
  final ValueChanged<LibraryReadingState>? onReadingFilterChanged;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;

  @override
  State<LibraryScreen> createState() => _LibraryScreenState();
}

final class _LibraryScreenState extends State<LibraryScreen> {
  final TextEditingController _filterController = TextEditingController();
  final FocusNode _filterFocusNode = FocusNode(debugLabel: 'library-filter');
  bool _dragging = false;
  String _query = '';
  LibrarySortOrder _sortOrder = LibrarySortOrder.lastOpened;
  late LibraryReadingState _readingState;

  @override
  void initState() {
    super.initState();
    _readingState = widget.initialReadingFilter;
  }

  @override
  void didUpdateWidget(covariant LibraryScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialReadingFilter != widget.initialReadingFilter &&
        widget.initialReadingFilter != _readingState) {
      _readingState = widget.initialReadingFilter;
    }
  }

  @override
  void dispose() {
    _filterController.dispose();
    _filterFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final compact = context.windowClass == AppWindowClass.compact;
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyF, control: true):
            _focusFilter,
      },
      child: Focus(
        autofocus: true,
        child: DropTarget(
          onDragEntered: (_) => setState(() => _dragging = true),
          onDragExited: (_) => setState(() => _dragging = false),
          onDragDone: (details) {
            setState(() => _dragging = false);
            final pdf = details.files
                .where((file) => file.path.toLowerCase().endsWith('.pdf'))
                .firstOrNull;
            if (pdf != null) widget.onOpenPath(pdf.path);
          },
          child: Scaffold(
            appBar: AppBar(
              title: const Text(AppConfig.productName),
              actions: [
                if (compact)
                  IconButton(
                    key: const Key('library-open-button'),
                    tooltip: 'Abrir PDF (Ctrl+O)',
                    onPressed: widget.onPickDocument,
                    icon: const Icon(Icons.file_open_outlined),
                  )
                else
                  TextButton.icon(
                    key: const Key('library-open-button'),
                    onPressed: widget.onPickDocument,
                    icon: const Icon(Icons.file_open_outlined, size: 18),
                    label: const Text('Abrir PDF'),
                  ),
                const SizedBox(width: AppSpacing.space4),
                IconButton(
                  tooltip: 'Ajustes',
                  onPressed: widget.onOpenSettings,
                  icon: const Icon(Icons.tune_rounded),
                ),
                const SizedBox(width: AppSpacing.space8),
              ],
            ),
            body: Stack(
              children: [
                ListenableBuilder(
                  listenable: widget.controller,
                  builder: (context, _) => _LibraryBody(
                    controller: widget.controller,
                    onOpenPath: widget.onOpenPath,
                    onPickDocument: widget.onPickDocument,
                    showRecents: widget.showRecents,
                    filterController: _filterController,
                    filterFocusNode: _filterFocusNode,
                    query: _query,
                    sortOrder: _sortOrder,
                    readingState: _readingState,
                    onQueryChanged: (value) => setState(() => _query = value),
                    onClearQuery: _clearFilter,
                    onSortChanged: (value) =>
                        setState(() => _sortOrder = value),
                    onReadingStateChanged: (value) {
                      setState(() => _readingState = value);
                      widget.onReadingFilterChanged?.call(value);
                    },
                    onDocumentInfo: widget.onDocumentInfo,
                    onRelocateDocument: widget.onRelocateDocument,
                  ),
                ),
                if (_dragging)
                  Positioned.fill(
                    child: ColoredBox(
                      color: context.appColors.accentSoft.withValues(
                        alpha: 0.92,
                      ),
                      child: Center(
                        child: Semantics(
                          liveRegion: true,
                          label: 'Suelta el PDF para abrirlo',
                          child: const _DropMessage(),
                        ),
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

  void _focusFilter() {
    if (!widget.showRecents) return;
    _filterFocusNode.requestFocus();
    _filterController.selection = TextSelection(
      baseOffset: 0,
      extentOffset: _filterController.text.length,
    );
  }

  void _clearFilter() {
    _filterController.clear();
    setState(() => _query = '');
    _filterFocusNode.requestFocus();
  }
}

final class _LibraryBody extends StatelessWidget {
  const _LibraryBody({
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.showRecents,
    required this.filterController,
    required this.filterFocusNode,
    required this.query,
    required this.sortOrder,
    required this.readingState,
    required this.onQueryChanged,
    required this.onClearQuery,
    required this.onSortChanged,
    required this.onReadingStateChanged,
    required this.onDocumentInfo,
    required this.onRelocateDocument,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final bool showRecents;
  final TextEditingController filterController;
  final FocusNode filterFocusNode;
  final String query;
  final LibrarySortOrder sortOrder;
  final LibraryReadingState readingState;
  final ValueChanged<String> onQueryChanged;
  final VoidCallback onClearQuery;
  final ValueChanged<LibrarySortOrder> onSortChanged;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;

  @override
  Widget build(BuildContext context) {
    if (controller.isLoading) {
      return const AppStatusView(
        icon: Icons.library_books_outlined,
        title: 'Preparando tu biblioteca',
        message: 'Revisando documentos y portadas guardadas en este equipo.',
        loading: true,
      );
    }
    if (controller.error case final error?) {
      return AppStatusView(
        icon: Icons.error_outline_rounded,
        title: 'No pudimos cargar la biblioteca',
        message: error.userMessage,
        action: controller.load,
        actionLabel: 'Reintentar',
        tone: AppStatusTone.error,
      );
    }
    if (controller.documents.isEmpty) {
      return EmptyLibraryState(onOpen: onPickDocument);
    }
    final filterResult = evaluateLibraryDocuments(
      controller.documents,
      query: query,
      sortOrder: sortOrder,
      readingState: readingState,
    );
    final visibleDocuments = filterResult.documents;

    return LayoutBuilder(
      builder: (context, constraints) {
        final padding = AppSpacing.pagePadding(constraints.maxWidth);
        final continueReading = controller.continueReading;
        final compact =
            AppBreakpoints.classify(constraints.maxWidth) ==
            AppWindowClass.compact;
        final filterBar = _LibraryFilterBar(
          controller: filterController,
          focusNode: filterFocusNode,
          query: query,
          sortOrder: sortOrder,
          resultCount: visibleDocuments.length,
          counts: filterResult.counts,
          readingState: readingState,
          onQueryChanged: onQueryChanged,
          onClear: onClearQuery,
          onSortChanged: onSortChanged,
          onReadingStateChanged: onReadingStateChanged,
        );
        return Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(
              maxWidth: AppSizes.contentMaxWidth,
            ),
            child: CustomScrollView(
              slivers: [
                SliverPadding(
                  padding: EdgeInsets.fromLTRB(
                    padding,
                    AppSpacing.space32,
                    padding,
                    AppSpacing.space12,
                  ),
                  sliver: SliverToBoxAdapter(
                    child: Text(
                      'Biblioteca',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                  ),
                ),
                if (continueReading != null) ...[
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      padding,
                      AppSpacing.space16,
                      padding,
                      AppSpacing.space12,
                    ),
                    sliver: const SliverToBoxAdapter(
                      child: AppSectionHeader(
                        title: 'Continuar leyendo',
                        description: 'Retoma exactamente donde lo dejaste.',
                      ),
                    ),
                  ),
                  SliverPadding(
                    padding: EdgeInsets.symmetric(horizontal: padding),
                    sliver: SliverToBoxAdapter(
                      child: ContinueReadingCard(
                        document: continueReading,
                        onOpen: () => onOpenPath(continueReading.path),
                        onInfo: onDocumentInfo == null
                            ? null
                            : () => onDocumentInfo!(continueReading),
                      ),
                    ),
                  ),
                ],
                if (showRecents) ...[
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      padding,
                      context.appDensity.sectionSpacing,
                      padding,
                      AppSpacing.space12,
                    ),
                    sliver: SliverToBoxAdapter(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          AppSectionHeader(
                            title: 'Recientes',
                            description: _libraryResultDescription(
                              visibleDocuments.length,
                              controller.documents.length,
                              query,
                              readingState,
                            ),
                          ),
                          SizedBox(
                            height: compact
                                ? AppSpacing.space12
                                : AppSpacing.space16,
                          ),
                          filterBar,
                        ],
                      ),
                    ),
                  ),
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      padding,
                      0,
                      padding,
                      AppSpacing.space48,
                    ),
                    sliver: visibleDocuments.isEmpty
                        ? SliverToBoxAdapter(
                            child: _NoLibraryResults(
                              query: query,
                              onClear: onClearQuery,
                            ),
                          )
                        : SliverList.separated(
                            itemCount: visibleDocuments.length,
                            separatorBuilder: (_, _) =>
                                const Divider(height: 1),
                            itemBuilder: (context, index) {
                              final document = visibleDocuments[index];
                              return RecentDocumentItem(
                                document: document,
                                onOpen: () => document.isMissing
                                    ? (onRelocateDocument?.call(document) ??
                                          onPickDocument())
                                    : onOpenPath(document.path),
                                onInfo: onDocumentInfo == null
                                    ? null
                                    : () => onDocumentInfo!(document),
                              );
                            },
                          ),
                  ),
                ] else
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      padding,
                      AppSpacing.space32,
                      padding,
                      AppSpacing.space48,
                    ),
                    sliver: SliverToBoxAdapter(
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: OutlinedButton.icon(
                          onPressed: onPickDocument,
                          icon: const Icon(Icons.file_open_outlined, size: 18),
                          label: const Text('Abrir otro PDF'),
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}

final class _LibraryFilterBar extends StatelessWidget {
  const _LibraryFilterBar({
    required this.controller,
    required this.focusNode,
    required this.query,
    required this.sortOrder,
    required this.resultCount,
    required this.counts,
    required this.readingState,
    required this.onQueryChanged,
    required this.onClear,
    required this.onSortChanged,
    required this.onReadingStateChanged,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final String query;
  final LibrarySortOrder sortOrder;
  final int resultCount;
  final Map<LibraryReadingState, int> counts;
  final LibraryReadingState readingState;
  final ValueChanged<String> onQueryChanged;
  final VoidCallback onClear;
  final ValueChanged<LibrarySortOrder> onSortChanged;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: TextField(
                  key: const Key('library-filter'),
                  controller: controller,
                  focusNode: focusNode,
                  textInputAction: TextInputAction.search,
                  decoration: InputDecoration(
                    hintText: 'Filtrar biblioteca',
                    prefixIcon: const Icon(Icons.search_rounded),
                    suffixIcon: query.isEmpty
                        ? null
                        : IconButton(
                            key: const Key('library-filter-clear'),
                            tooltip: 'Limpiar filtro',
                            onPressed: onClear,
                            icon: const Icon(Icons.close_rounded),
                          ),
                  ),
                  onChanged: onQueryChanged,
                ),
              ),
              const SizedBox(width: AppSpacing.space8),
              PopupMenuButton<LibrarySortOrder>(
                key: const Key('library-sort'),
                initialValue: sortOrder,
                tooltip: 'Ordenar biblioteca',
                onSelected: onSortChanged,
                itemBuilder: (context) => const [
                  PopupMenuItem(
                    value: LibrarySortOrder.lastOpened,
                    child: Text('Última apertura'),
                  ),
                  PopupMenuItem(
                    value: LibrarySortOrder.title,
                    child: Text('Título'),
                  ),
                  PopupMenuItem(
                    value: LibrarySortOrder.progress,
                    child: Text('Progreso'),
                  ),
                ],
                icon: const Icon(Icons.sort_rounded),
              ),
              Semantics(
                liveRegion: true,
                label:
                    '$resultCount ${resultCount == 1 ? 'resultado' : 'resultados'}',
                child: const SizedBox.shrink(),
              ),
            ],
          ),
          const SizedBox(height: AppSpacing.space8),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: SegmentedButton<LibraryReadingState>(
              key: const Key('library-reading-state'),
              showSelectedIcon: false,
              selected: {readingState},
              onSelectionChanged: (selection) =>
                  onReadingStateChanged(selection.first),
              segments: [
                for (final state in LibraryReadingState.values)
                  ButtonSegment(
                    value: state,
                    label: Text('${state.label} ${counts[state] ?? 0}'),
                    tooltip: '${state.label}: ${counts[state] ?? 0} documentos',
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

final class _NoLibraryResults extends StatelessWidget {
  const _NoLibraryResults({required this.query, required this.onClear});

  final String query;
  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      liveRegion: true,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: AppSpacing.space40),
        child: Column(
          children: [
            Icon(Icons.search_off_rounded, color: context.appColors.textMuted),
            const SizedBox(height: AppSpacing.space8),
            Text(
              'No hay documentos que coincidan con “$query”.',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: AppSpacing.space12),
            TextButton(onPressed: onClear, child: const Text('Limpiar filtro')),
          ],
        ),
      ),
    );
  }
}

final class EmptyLibraryState extends StatelessWidget {
  const EmptyLibraryState({super.key, required this.onOpen});

  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    return AppStatusView(
      icon: Icons.menu_book_outlined,
      title: 'Una biblioteca tranquila',
      message:
          'Abre un PDF o arrástralo a esta ventana. El archivo permanece '
          'en su carpeta original y tu progreso se guarda solo aquí.',
      action: onOpen,
      actionLabel: 'Abrir PDF',
      actionKey: const Key('open-pdf-button'),
    );
  }
}

final class ContinueReadingCard extends StatelessWidget {
  const ContinueReadingCard({
    super.key,
    required this.document,
    required this.onOpen,
    this.onInfo,
  });

  final LibraryDocument document;
  final VoidCallback onOpen;
  final VoidCallback? onInfo;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label:
          'Continuar leyendo ${document.title ?? document.fileName}, '
          '${(document.readingProgress * 100).round()} por ciento',
      child: Material(
        color: context.appColors.surfacePrimary,
        child: InkWell(
          key: const Key('continue-reading'),
          onTap: onOpen,
          focusColor: context.appColors.accentSoft,
          child: Container(
            decoration: BoxDecoration(
              border: Border.symmetric(
                horizontal: BorderSide(color: context.appColors.borderSubtle),
              ),
            ),
            padding: const EdgeInsets.symmetric(
              horizontal: AppSpacing.space20,
              vertical: AppSpacing.space20,
            ),
            child: LayoutBuilder(
              builder: (context, constraints) {
                final compact = constraints.maxWidth < AppBreakpoints.medium;
                return Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    _DocumentCover(
                      document: document,
                      width: compact ? 72 : 86,
                      height: compact ? 100 : 118,
                    ),
                    SizedBox(
                      width: compact ? AppSpacing.space16 : AppSpacing.space24,
                    ),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            document.title ?? document.fileName,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.headlineSmall,
                          ),
                          if (document.author case final author?) ...[
                            const SizedBox(height: AppSpacing.space4),
                            Text(
                              author,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                          ],
                          const SizedBox(height: AppSpacing.space16),
                          ClipRRect(
                            borderRadius: BorderRadius.circular(
                              AppRadii.indicator,
                            ),
                            child: LinearProgressIndicator(
                              value: document.readingProgress,
                              minHeight: 3,
                            ),
                          ),
                          const SizedBox(height: AppSpacing.space8),
                          Wrap(
                            spacing: AppSpacing.space12,
                            runSpacing: AppSpacing.space4,
                            children: [
                              Text(
                                'Página ${document.lastPageIndex + 1} '
                                'de ${document.pageCount}',
                                style: Theme.of(context).textTheme.labelMedium,
                              ),
                              Text(
                                '${(document.readingProgress * 100).round()} %',
                                style: Theme.of(context).textTheme.labelMedium,
                              ),
                              Text(
                                _relativeDate(document.lastOpenedAt),
                                style: Theme.of(context).textTheme.labelMedium,
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                    if (!compact) ...[
                      const SizedBox(width: AppSpacing.space24),
                      FilledButton.icon(
                        onPressed: onOpen,
                        icon: const Icon(Icons.arrow_forward_rounded, size: 18),
                        label: const Text('Continuar'),
                      ),
                    ] else if (onInfo == null) ...[
                      const SizedBox(width: AppSpacing.space8),
                      Icon(
                        Icons.arrow_forward_rounded,
                        color: context.appColors.accentPrimary,
                      ),
                    ],
                    if (onInfo != null) ...[
                      const SizedBox(width: AppSpacing.space8),
                      IconButton(
                        key: ValueKey('continue-info-${document.id}'),
                        tooltip: 'Información del documento',
                        onPressed: onInfo,
                        icon: const Icon(Icons.info_outline_rounded),
                      ),
                    ],
                  ],
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}

final class RecentDocumentItem extends StatelessWidget {
  const RecentDocumentItem({
    super.key,
    required this.document,
    required this.onOpen,
    this.onInfo,
  });

  final LibraryDocument document;
  final VoidCallback onOpen;
  final VoidCallback? onInfo;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: document.isMissing
          ? '${document.fileName}, archivo no encontrado, localizar'
          : '${document.fileName}, '
                '${(document.readingProgress * 100).round()} por ciento',
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          key: ValueKey('recent-${document.id}'),
          onTap: onOpen,
          child: Padding(
            padding: EdgeInsets.symmetric(
              vertical: context.appDensity.listRowVerticalPadding,
              horizontal: AppSpacing.space4,
            ),
            child: LayoutBuilder(
              builder: (context, constraints) {
                final compact = constraints.maxWidth < AppBreakpoints.medium;
                return Row(
                  children: [
                    Opacity(
                      opacity: document.isMissing ? 0.55 : 1,
                      child: _DocumentCover(
                        document: document,
                        width: context.appDensity.recentCoverWidth,
                        height: context.appDensity.recentCoverHeight,
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            document.title ?? document.fileName,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          const SizedBox(height: AppSpacing.space6),
                          if (document.isMissing)
                            const AppStatusLabel(
                              icon: Icons.link_off_rounded,
                              label: 'Archivo no encontrado',
                              tone: AppStatusTone.error,
                            )
                          else if (document.hasFileChanged)
                            const AppStatusLabel(
                              icon: Icons.sync_problem_rounded,
                              label: 'El archivo ha cambiado',
                              tone: AppStatusTone.warning,
                            )
                          else
                            Text(
                              '${_readingStateDescription(document)}'
                              '  ·  ${_relativeDate(document.lastOpenedAt)}',
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                        ],
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space12),
                    if (!compact && document.isMissing)
                      TextButton.icon(
                        onPressed: onOpen,
                        icon: const Icon(
                          Icons.drive_file_move_outline,
                          size: 17,
                        ),
                        label: const Text('Localizar'),
                      )
                    else if (!compact)
                      SizedBox(
                        width: 88,
                        child: Text(
                          'Página ${document.lastPageIndex + 1}',
                          textAlign: TextAlign.right,
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                      ),
                    const SizedBox(width: AppSpacing.space8),
                    if (onInfo != null)
                      IconButton(
                        key: ValueKey('document-info-${document.id}'),
                        tooltip: 'Información del documento',
                        onPressed: onInfo,
                        icon: const Icon(Icons.info_outline_rounded, size: 20),
                      )
                    else
                      Icon(
                        Icons.chevron_right_rounded,
                        color: context.appColors.textMuted,
                        size: 20,
                      ),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}

final class _DocumentCover extends StatelessWidget {
  const _DocumentCover({
    required this.document,
    required this.width,
    required this.height,
  });

  final LibraryDocument document;
  final double width;
  final double height;

  @override
  Widget build(BuildContext context) {
    final thumbnailPath = document.thumbnailPath;
    final devicePixelRatio = MediaQuery.devicePixelRatioOf(context);
    final cacheWidth = (width * devicePixelRatio).ceil();
    final cacheHeight = (height * devicePixelRatio).ceil();
    final fallback = ColoredBox(
      color: context.appColors.surfaceSunken,
      child: Center(
        child: Icon(
          Icons.picture_as_pdf_outlined,
          color: context.appColors.textMuted,
          size: 20,
        ),
      ),
    );
    return RepaintBoundary(
      child: Container(
        width: width,
        height: height,
        decoration: BoxDecoration(
          color: context.appColors.surfaceSunken,
          border: Border.all(color: context.appColors.borderSubtle),
          boxShadow: [
            BoxShadow(
              color: context.appColors.readerPageShadow,
              blurRadius: 3,
              offset: const Offset(0, 1),
            ),
          ],
        ),
        clipBehavior: Clip.antiAlias,
        child: thumbnailPath == null
            ? fallback
            : Image.file(
                File(thumbnailPath),
                fit: BoxFit.cover,
                cacheWidth: cacheWidth,
                cacheHeight: cacheHeight,
                filterQuality: FilterQuality.low,
                errorBuilder: (_, _, _) => ColoredBox(
                  color: context.appColors.surfaceSunken,
                  child: Center(
                    child: Icon(
                      Icons.broken_image_outlined,
                      color: context.appColors.warning,
                      size: 20,
                    ),
                  ),
                ),
              ),
      ),
    );
  }
}

final class _DropMessage extends StatelessWidget {
  const _DropMessage();

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.appColors.surfaceElevated,
        border: Border.all(color: context.appColors.focusRing, width: 2),
        borderRadius: BorderRadius.circular(AppRadii.dialog),
      ),
      child: const Padding(
        padding: EdgeInsets.symmetric(
          horizontal: AppSpacing.space32,
          vertical: AppSpacing.space24,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.file_download_outlined),
            SizedBox(width: AppSpacing.space12),
            Text('Suelta el PDF para abrirlo'),
          ],
        ),
      ),
    );
  }
}

String _relativeDate(DateTime value) {
  final difference = DateTime.now().difference(value.toLocal());
  if (difference.inMinutes < 1) return 'ahora';
  if (difference.inHours < 1) return 'hace ${difference.inMinutes} min';
  if (difference.inDays < 1) return 'hace ${difference.inHours} h';
  if (difference.inDays < 7) return 'hace ${difference.inDays} d';
  final local = value.toLocal();
  return '${local.day.toString().padLeft(2, '0')}/'
      '${local.month.toString().padLeft(2, '0')}/${local.year}';
}

String _documentLabel(LibraryDocument document) =>
    (document.title ?? document.fileName).toLowerCase();

String _readingStateDescription(LibraryDocument document) {
  if (document.readingProgress >= 1) return 'Terminado';
  if (document.readingProgress > 0) {
    return 'En curso · ${(document.readingProgress * 100).round()} %';
  }
  return 'Sin empezar';
}

String _libraryResultDescription(
  int visible,
  int total,
  String query,
  LibraryReadingState readingState,
) {
  if (query.trim().isEmpty && readingState == LibraryReadingState.all) {
    return '$total ${total == 1 ? 'documento' : 'documentos'}';
  }
  return '$visible de $total ${total == 1 ? 'documento' : 'documentos'}';
}
