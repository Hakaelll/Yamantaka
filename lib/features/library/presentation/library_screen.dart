import 'dart:async';
import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_theme.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../organization/application/organization_controller.dart';
import '../../organization/domain/organization_models.dart';
import '../application/library_controller.dart';
import '../application/library_file_validation_controller.dart';
import '../application/document_enrichment_controller.dart';
import '../application/library_filter.dart';
import '../domain/library_document.dart';
import '../domain/library_selection.dart';
import 'artifact_library.dart';

export '../application/library_filter.dart';

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
    this.onBulkOperation,
    this.onDocumentInfo,
    this.onRelocateDocument,
    this.onToggleFavorite,
    this.onChangeCover,
    this.onRevealDocument,
    this.onRemoveDocument,
    this.onDocumentVisible,
    this.organization,
    this.onGlobalSearch,
    this.onOpenPaths,
    this.onWorkspaces,
    this.onImportFolder,
    this.onRecoverFolder,
    this.safeMode = false,
    this.enrichment,
    this.fileValidation,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final VoidCallback onOpenSettings;
  final bool showRecents;
  final LibraryReadingState initialReadingFilter;
  final ValueChanged<LibraryReadingState>? onReadingFilterChanged;
  final BulkOperationHandler? onBulkOperation;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;
  final ValueChanged<LibraryDocument>? onDocumentVisible;
  final OrganizationController? organization;
  final VoidCallback? onGlobalSearch;
  final Future<void> Function(List<String> paths)? onOpenPaths;
  final VoidCallback? onWorkspaces;
  final VoidCallback? onImportFolder;
  final VoidCallback? onRecoverFolder;
  final bool safeMode;
  final DocumentEnrichmentController? enrichment;
  final LibraryFileValidationController? fileValidation;

  @override
  State<LibraryScreen> createState() => _LibraryScreenState();
}

final class _LibraryScreenState extends State<LibraryScreen> {
  bool _dragging = false;
  late LibraryReadingState _readingState;

  @override
  void initState() {
    super.initState();
    _readingState = widget.initialReadingFilter;
    if (widget.controller.readingState == LibraryReadingState.all.name) {
      widget.controller.readingState = _readingState.name;
    }
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
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final compact = context.windowClass == AppWindowClass.compact;
    final libraryTheme = AppTheme.library(
      density: context.appDensity.interfaceDensity,
      highContrast: MediaQuery.highContrastOf(context),
    );
    return Theme(
      data: libraryTheme,
      child: Builder(
        builder: (context) => CallbackShortcuts(
          bindings: {
            if (widget.onGlobalSearch != null)
              const SingleActivator(
                LogicalKeyboardKey.keyF,
                control: true,
                shift: true,
              ): widget.onGlobalSearch!,
          },
          child: Focus(
            autofocus: true,
            child: DropTarget(
              onDragEntered: (_) => setState(() => _dragging = true),
              onDragExited: (_) => setState(() => _dragging = false),
              onDragDone: (details) {
                setState(() => _dragging = false);
                final pdfs = details.files
                    .where((file) => file.path.toLowerCase().endsWith('.pdf'))
                    .map((file) => file.path)
                    .toSet()
                    .toList(growable: false);
                if (pdfs.isEmpty) return;
                if (widget.onOpenPaths != null) {
                  widget.onOpenPaths!(pdfs);
                } else {
                  widget.onOpenPath(pdfs.first);
                }
              },
              child: Scaffold(
                backgroundColor: context.appColors.libraryBackground,
                appBar: AppBar(
                  backgroundColor: context.appColors.libraryBackground,
                  surfaceTintColor: Colors.transparent,
                  shape: const Border(),
                  toolbarHeight: AppSizes.libraryHeaderHeight,
                  title: ListenableBuilder(
                    listenable: Listenable.merge([
                      widget.controller,
                      if (widget.enrichment != null) widget.enrichment!,
                      if (widget.fileValidation != null) widget.fileValidation!,
                    ]),
                    builder: (context, _) => Semantics(
                      header: true,
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            strings.libraryTitle,
                            style: Theme.of(context).textTheme.headlineSmall,
                          ),
                          const SizedBox(height: AppSpacing.space2),
                          Text(
                            widget.enrichment?.progress.isRunning == true
                                ? strings.get(
                                    'library.analyzing',
                                    parameters: {
                                      'current':
                                          widget
                                              .enrichment!
                                              .progress
                                              .completed +
                                          1,
                                      'total':
                                          widget.enrichment!.progress.total,
                                    },
                                  )
                                : strings.count(
                                    'count.documents',
                                    widget.controller.libraryDocumentCount,
                                  ),
                            style: Theme.of(context).textTheme.labelSmall,
                          ),
                        ],
                      ),
                    ),
                  ),
                  actions: [
                    if (widget.enrichment?.progress.isRunning == true)
                      IconButton(
                        tooltip: strings.get('library.cancelAnalysis'),
                        onPressed: widget.enrichment!.cancel,
                        icon: const Icon(Icons.stop_circle_outlined),
                      ),
                    if (widget.fileValidation?.progress.isRunning == true)
                      IconButton(
                        tooltip: strings.get('action.cancel'),
                        onPressed: widget.fileValidation!.cancel,
                        icon: const Icon(Icons.stop_circle_outlined),
                      ),
                    if (widget.safeMode)
                      Tooltip(
                        message: strings.get('library.safeMode'),
                        child: const Padding(
                          padding: EdgeInsets.all(AppSpacing.space12),
                          child: Icon(Icons.health_and_safety_outlined),
                        ),
                      ),
                    PopupMenuButton<_LibraryAppMenuAction>(
                      tooltip: strings.get('library.organizeMore'),
                      onSelected: (action) {
                        switch (action) {
                          case _LibraryAppMenuAction.collections:
                            _showCollections();
                          case _LibraryAppMenuAction.workspaces:
                            widget.onWorkspaces?.call();
                          case _LibraryAppMenuAction.importFolder:
                            widget.onImportFolder?.call();
                          case _LibraryAppMenuAction.recover:
                            widget.onRecoverFolder?.call();
                          case _LibraryAppMenuAction.globalSearch:
                            widget.onGlobalSearch?.call();
                          case _LibraryAppMenuAction.enrich:
                            widget.enrichment?.analyzePending();
                          case _LibraryAppMenuAction.reanalyze:
                            unawaited(_reanalyzeLibrary());
                          case _LibraryAppMenuAction.checkFiles:
                            unawaited(_checkFiles());
                        }
                      },
                      itemBuilder: (context) => [
                        if (widget.organization != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.collections,
                            child: Text(strings.get('library.collections')),
                          ),
                        if (widget.onWorkspaces != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.workspaces,
                            child: Text(strings.get('library.workspaces')),
                          ),
                        if (widget.onImportFolder != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.importFolder,
                            child: Text(strings.get('library.importFolder')),
                          ),
                        if (widget.onRecoverFolder != null &&
                            widget.controller.documents.any(
                              (document) => document.isMissing,
                            ))
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.recover,
                            child: Text(strings.get('library.recoverMoved')),
                          ),
                        if (widget.onGlobalSearch != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.globalSearch,
                            child: Text(strings.get('library.globalSearch')),
                          ),
                        if (widget.enrichment != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.enrich,
                            child: Text(strings.get('library.analyzeMetadata')),
                          ),
                        if (widget.enrichment != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.reanalyze,
                            child: Text(strings.get('library.reanalyze')),
                          ),
                        if (widget.fileValidation != null)
                          PopupMenuItem(
                            value: _LibraryAppMenuAction.checkFiles,
                            child: Text(strings.libraryCheckFiles),
                          ),
                      ],
                      icon: const Icon(Icons.more_horiz_rounded),
                    ),
                    if (compact)
                      Tooltip(
                        message: '${strings.libraryOpenPdf} (Ctrl+O)',
                        child: Semantics(
                          button: true,
                          label: strings.libraryOpenPdf,
                          child: FilledButton(
                            key: const Key('library-open-button'),
                            onPressed: widget.onPickDocument,
                            style: FilledButton.styleFrom(
                              minimumSize: const Size.square(
                                AppSizes.desktopControlExtent,
                              ),
                              padding: EdgeInsets.zero,
                            ),
                            child: const Icon(
                              Icons.file_open_outlined,
                              size: 18,
                            ),
                          ),
                        ),
                      )
                    else
                      FilledButton.icon(
                        key: const Key('library-open-button'),
                        onPressed: widget.onPickDocument,
                        icon: const Icon(Icons.file_open_outlined, size: 18),
                        label: Text(strings.libraryOpenPdf),
                      ),
                    const SizedBox(width: AppSpacing.space4),
                    IconButton(
                      tooltip: strings.settingsTitle,
                      onPressed: widget.onOpenSettings,
                      icon: const Icon(Icons.tune_rounded),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                  ],
                ),
                body: Stack(
                  children: [
                    ListenableBuilder(
                      listenable: Listenable.merge([
                        widget.controller,
                        if (widget.organization != null) widget.organization!,
                      ]),
                      builder: (context, _) => _LibraryBody(
                        controller: widget.controller,
                        onOpenPath: widget.onOpenPath,
                        onPickDocument: widget.onPickDocument,
                        showRecents: widget.showRecents,
                        readingState: _readingState,
                        onReadingStateChanged: (value) {
                          setState(() => _readingState = value);
                          widget.onReadingFilterChanged?.call(value);
                        },
                        onBulkOperation: widget.onBulkOperation,
                        onDocumentInfo: widget.onDocumentInfo,
                        onRelocateDocument: widget.onRelocateDocument,
                        onToggleFavorite: widget.onToggleFavorite,
                        onChangeCover: widget.onChangeCover,
                        onRevealDocument: widget.onRevealDocument,
                        onRemoveDocument: widget.onRemoveDocument,
                        onReanalyzeDocument: widget.enrichment == null
                            ? null
                            : (document) async {
                                await widget.enrichment!.reanalyzeDocuments([
                                  document,
                                ]);
                                await widget.controller.reload();
                              },
                        onDocumentVisible: widget.onDocumentVisible,
                        collections:
                            widget.organization?.collections ?? const [],
                        tags: widget.organization?.tags ?? const [],
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
                              label: strings.get('library.dropPdf'),
                              child: _DropMessage(
                                message: strings.get('library.dropPdf'),
                              ),
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

  Future<void> _reanalyzeLibrary() async {
    final strings = YamantakaLocalizations.of(context);
    final enrichment = widget.enrichment;
    if (enrichment == null || enrichment.progress.isRunning) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('library.reanalyze')),
        content: Text(strings.get('library.reanalyze.description')),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.get('library.analyze')),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    await enrichment.reanalyzeAll();
    await widget.controller.reload();
  }

  Future<void> _checkFiles() async {
    final validation = widget.fileValidation;
    if (validation == null || validation.progress.isRunning) return;
    final summary = await validation.checkAll();
    if (!mounted) return;
    final strings = YamantakaLocalizations.of(context);
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.fact_check_outlined),
        title: Text(strings.libraryCheckTitle),
        content: Semantics(
          liveRegion: true,
          child: Text(
            strings.fileCheckSummary(
              available: summary.available,
              missing: summary.missing,
              modified: summary.modified,
              failed: summary.failed,
              cancelled: summary.cancelled,
            ),
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

  Future<void> _showCollections() async {
    final organization = widget.organization;
    if (organization == null) return;
    final strings = YamantakaLocalizations.of(context);
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: ListView(
          shrinkWrap: true,
          children: [
            ListTile(
              leading: const Icon(Icons.library_books_outlined),
              title: Text(strings.get('library.allLibrary')),
              selected:
                  organization.selectedCollectionId == null &&
                  organization.selectedSmartCollection == null,
              onTap: () {
                organization.clearSelection();
                unawaited(widget.controller.setOrganizationScope());
                Navigator.pop(context);
              },
            ),
            for (final smart in SmartCollection.values)
              ListTile(
                leading: const Icon(Icons.auto_awesome_outlined),
                title: Text(
                  strings.get('library.smartCollection.${smart.name}'),
                ),
                selected: organization.selectedSmartCollection == smart,
                onTap: () {
                  organization.selectSmartCollection(smart);
                  unawaited(
                    widget.controller.setOrganizationScope(
                      smartCollection: smart,
                    ),
                  );
                  Navigator.pop(context);
                },
              ),
            if (organization.collections.isNotEmpty) const Divider(),
            for (final collection in organization.collections)
              ListTile(
                leading: Icon(
                  collection.isPinned
                      ? Icons.folder_special_outlined
                      : Icons.folder_outlined,
                ),
                title: Text(collection.name),
                trailing: Text('${collection.documentCount}'),
                selected: organization.selectedCollectionId == collection.id,
                onTap: () {
                  organization.selectCollection(collection.id);
                  unawaited(
                    widget.controller.setOrganizationScope(
                      collectionId: collection.id,
                    ),
                  );
                  Navigator.pop(context);
                },
              ),
            ListTile(
              leading: const Icon(Icons.create_new_folder_outlined),
              title: Text(strings.get('library.newCollection')),
              onTap: () async {
                Navigator.pop(context);
                await _createCollection(organization);
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _createCollection(OrganizationController organization) async {
    final strings = YamantakaLocalizations.of(context);
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('library.newCollection')),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: 60,
          decoration: InputDecoration(labelText: strings.get('library.name')),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, controller.text),
            child: Text(strings.get('action.create')),
          ),
        ],
      ),
    );
    controller.dispose();
    if (name != null && name.trim().isNotEmpty) {
      final created = organization.createCollection(name);
      organization.selectCollection(created.id);
      await widget.controller.setOrganizationScope(collectionId: created.id);
    }
  }
}

final class _LibraryBody extends StatelessWidget {
  const _LibraryBody({
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.showRecents,
    required this.readingState,
    required this.onReadingStateChanged,
    this.onBulkOperation,
    required this.onDocumentInfo,
    required this.onRelocateDocument,
    required this.onToggleFavorite,
    required this.onChangeCover,
    required this.onRevealDocument,
    required this.onRemoveDocument,
    this.onReanalyzeDocument,
    this.onDocumentVisible,
    this.collections = const [],
    this.tags = const [],
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final bool showRecents;
  final LibraryReadingState readingState;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final BulkOperationHandler? onBulkOperation;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;
  final Future<void> Function(LibraryDocument document)? onReanalyzeDocument;
  final ValueChanged<LibraryDocument>? onDocumentVisible;
  final List<DocumentCollection> collections;
  final List<DocumentTag> tags;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    if (controller.isLoading && controller.documents.isEmpty) {
      return AppStatusView(
        icon: Icons.library_books_outlined,
        title: strings.get('library.preparing'),
        message: strings.get('library.preparing.description'),
        loading: true,
      );
    }
    if (controller.error case final error?) {
      final presentation = FailurePresentations.forFailure(
        error,
        strings: strings,
      );
      return AppStatusView(
        icon: Icons.error_outline_rounded,
        title: presentation.title,
        message: presentation.message,
        action: controller.load,
        actionLabel: strings.get('action.retry'),
        tone: AppStatusTone.error,
      );
    }
    if (controller.libraryDocumentCount == 0) {
      return EmptyLibraryState(onOpen: onPickDocument);
    }
    final documents = controller.documents;
    if (!showRecents) {
      return Center(
        child: OutlinedButton.icon(
          onPressed: onPickDocument,
          icon: const Icon(Icons.file_open_outlined, size: 18),
          label: Text(strings.get('reader.openAnotherPdf')),
        ),
      );
    }
    return ArtifactLibraryView(
      controller: controller,
      documents: documents,
      onOpenPath: onOpenPath,
      onPickDocument: onPickDocument,
      initialReadingState: readingState,
      onReadingStateChanged: onReadingStateChanged,
      onBulkOperation: onBulkOperation,
      collections: collections,
      tags: tags,
      onDocumentInfo: onDocumentInfo,
      onRelocateDocument: onRelocateDocument,
      onToggleFavorite: onToggleFavorite,
      onChangeCover: onChangeCover,
      onRevealDocument: onRevealDocument,
      onRemoveDocument: onRemoveDocument,
      onReanalyzeDocument: onReanalyzeDocument,
      onDocumentVisible: onDocumentVisible,
    );
  }
}

final class EmptyLibraryState extends StatelessWidget {
  const EmptyLibraryState({super.key, required this.onOpen});

  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AppStatusView(
      icon: Icons.menu_book_outlined,
      title: strings.libraryEmptyTitle,
      message: strings.libraryEmptyBody,
      action: onOpen,
      actionLabel: strings.libraryOpenPdf,
      actionKey: const Key('open-pdf-button'),
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
    final strings = YamantakaLocalizations.of(context);
    return Semantics(
      button: true,
      label: document.isMissing
          ? strings.get(
              'library.missingDocumentSemantics',
              parameters: {'file': document.fileName},
            )
          : '${document.fileName}, '
                '${strings.get('library.percent', parameters: {'value': (document.readingProgress * 100).round()})}',
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
                            document.effectiveTitle,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          const SizedBox(height: AppSpacing.space6),
                          if (document.isMissing)
                            AppStatusLabel(
                              icon: Icons.link_off_rounded,
                              label: strings.get('library.fileNotFound'),
                              tone: AppStatusTone.error,
                            )
                          else if (document.hasFileChanged)
                            AppStatusLabel(
                              icon: Icons.sync_problem_rounded,
                              label: strings.get('library.fileChanged'),
                              tone: AppStatusTone.warning,
                            )
                          else
                            Text(
                              '${_readingStateDescription(document, strings)}'
                              '  ·  ${_relativeDate(document.lastOpenedAt, strings)}',
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
                        label: Text(strings.get('library.locate')),
                      )
                    else if (!compact)
                      SizedBox(
                        width: 88,
                        child: Text(
                          strings.get(
                            'reader.page',
                            parameters: {'page': document.lastPageIndex + 1},
                          ),
                          textAlign: TextAlign.right,
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                      ),
                    const SizedBox(width: AppSpacing.space8),
                    if (onInfo != null)
                      IconButton(
                        key: ValueKey('document-info-${document.id}'),
                        tooltip: strings.get('command.documentInfo'),
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
              blurRadius: AppShadow.readerPageBlur,
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
                gaplessPlayback: true,
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
  const _DropMessage({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.appColors.surfaceElevated,
        border: Border.all(color: context.appColors.focusRing, width: 2),
        borderRadius: BorderRadius.circular(AppRadii.dialog),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space32,
          vertical: AppSpacing.space24,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.file_download_outlined),
            const SizedBox(width: AppSpacing.space12),
            Text(message),
          ],
        ),
      ),
    );
  }
}

String _relativeDate(DateTime value, YamantakaLocalizations strings) {
  final difference = DateTime.now().difference(value.toLocal());
  if (difference.inMinutes < 1) return strings.get('time.now');
  if (difference.inHours < 1) {
    return strings.get(
      'time.minutesAgo',
      parameters: {'count': difference.inMinutes},
    );
  }
  if (difference.inDays < 1) {
    return strings.get(
      'time.hoursAgo',
      parameters: {'count': difference.inHours},
    );
  }
  if (difference.inDays < 7) {
    return strings.get(
      'time.daysAgo',
      parameters: {'count': difference.inDays},
    );
  }
  final local = value.toLocal();
  return '${local.day.toString().padLeft(2, '0')}/'
      '${local.month.toString().padLeft(2, '0')}/${local.year}';
}

String _readingStateDescription(
  LibraryDocument document,
  YamantakaLocalizations strings,
) {
  if (document.readingProgress >= 1) return strings.get('library.finished');
  if (document.readingProgress > 0) {
    return strings.get(
      'library.inProgress',
      parameters: {'value': (document.readingProgress * 100).round()},
    );
  }
  return strings.get('library.notStarted');
}

enum _LibraryAppMenuAction {
  collections,
  workspaces,
  importFolder,
  recover,
  globalSearch,
  enrich,
  reanalyze,
  checkFiles,
}
