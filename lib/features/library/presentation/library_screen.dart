import 'dart:async';
import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../organization/application/organization_controller.dart';
import '../../organization/domain/organization_models.dart';
import '../../reader/domain/reading_session.dart';
import '../application/library_controller.dart';
import '../application/library_file_validation_controller.dart';
import '../application/document_enrichment_controller.dart';
import '../application/library_filter.dart';
import '../domain/library_document.dart';
import '../domain/library_selection.dart';
import 'artifact_library.dart';

export '../application/library_filter.dart';

@visibleForTesting
List<String> acceptedDroppedPdfPaths(Iterable<String> paths) {
  final accepted = <String>[];
  final seen = <String>{};
  for (final path in paths) {
    if (!path.toLowerCase().endsWith('.pdf')) continue;
    if (seen.add(path.toLowerCase())) accepted.add(path);
  }
  return List.unmodifiable(accepted);
}

final class ContinuitySummary {
  const ContinuitySummary({
    this.openDocuments = 0,
    this.pinnedDocuments = 0,
    this.pendingReadings = 0,
    this.workspaceCount = 0,
    this.problemCount = 0,
  });

  final int openDocuments;
  final int pinnedDocuments;
  final int pendingReadings;
  final int workspaceCount;
  final int problemCount;

  bool get isEmpty =>
      openDocuments == 0 &&
      pinnedDocuments == 0 &&
      pendingReadings == 0 &&
      workspaceCount == 0 &&
      problemCount == 0;
}

final class LibraryScreen extends StatefulWidget {
  const LibraryScreen({
    super.key,
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.onOpenSettings,
    this.onToggleFullScreen,
    this.isFullScreen = false,
    required this.showRecents,
    this.reduceMotion = false,
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
    this.onDocumentHidden,
    this.organization,
    this.onGlobalSearch,
    this.onOpenPaths,
    this.onWorkspaces,
    this.onImportFolder,
    this.onRecoverFolder,
    this.safeMode = false,
    this.enrichment,
    this.fileValidation,
    this.recentlyClosed = const [],
    this.onRestoreClosed,
    this.onDismissClosed,
    this.onReread,
    this.onMarkFinished,
    this.readingGoal,
    this.onSetReadingGoal,
    this.continuity = const ContinuitySummary(),
    this.onOpenNext,
    this.onResumeSession,
    this.onDiscardContext,
    this.onProblems,
    this.onAnnotations,
    this.onAutomations,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final VoidCallback onOpenSettings;
  final VoidCallback? onToggleFullScreen;
  final bool isFullScreen;
  final bool showRecents;
  final bool reduceMotion;
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
  final ValueChanged<LibraryDocument>? onDocumentHidden;
  final OrganizationController? organization;
  final VoidCallback? onGlobalSearch;
  final Future<void> Function(List<String> paths)? onOpenPaths;
  final VoidCallback? onWorkspaces;
  final VoidCallback? onImportFolder;
  final VoidCallback? onRecoverFolder;
  final bool safeMode;
  final DocumentEnrichmentController? enrichment;
  final LibraryFileValidationController? fileValidation;
  final List<RecentlyClosedDocument> recentlyClosed;
  final ValueChanged<RecentlyClosedDocument>? onRestoreClosed;
  final ValueChanged<RecentlyClosedDocument>? onDismissClosed;
  final Future<void> Function(LibraryDocument document)? onReread;
  final Future<void> Function(LibraryDocument document)? onMarkFinished;
  final ReadingGoal? readingGoal;
  final ValueChanged<LibraryDocument>? onSetReadingGoal;
  final ContinuitySummary continuity;
  final VoidCallback? onOpenNext;
  final VoidCallback? onResumeSession;
  final VoidCallback? onDiscardContext;
  final VoidCallback? onProblems;
  final VoidCallback? onAnnotations;
  final VoidCallback? onAutomations;

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
    // Consume the application theme directly; no duplicate inherited theme.
    return CallbackShortcuts(
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
            final pdfs = acceptedDroppedPdfPaths(
              details.files.map((file) => file.path),
            );
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
              titleSpacing: 0,
              title: Semantics(
                header: true,
                label: strings.libraryTitle,
                child: const SizedBox.shrink(),
              ),
              actions: [
                if (widget.onToggleFullScreen != null)
                  IconButton(
                    key: const Key('library-fullscreen-button'),
                    tooltip: strings.get(
                      widget.isFullScreen
                          ? 'reader.exitFullScreen'
                          : 'command.fullScreen',
                    ),
                    onPressed: widget.onToggleFullScreen,
                    icon: Icon(
                      widget.isFullScreen
                          ? Icons.fullscreen_exit
                          : Icons.fullscreen,
                    ),
                  ),
                if (widget.showRecents && !widget.continuity.isEmpty)
                  ContinuityCenter(
                    summary: widget.continuity,
                    hasActiveReading: widget.controller.continueReading != null,
                    hasRecentlyClosed: widget.recentlyClosed.isNotEmpty,
                    onContinue: widget.controller.continueReading == null
                        ? null
                        : () => widget.onOpenPath(
                            widget.controller.continueReading!.path,
                          ),
                    onOpenNext: widget.onOpenNext,
                    onResumeSession: widget.onResumeSession,
                    onDiscardContext: widget.onDiscardContext,
                    onProblems: widget.onProblems,
                  ),
                if (widget.showRecents && widget.recentlyClosed.isNotEmpty)
                  RecentlyClosedStrip(
                    entries: widget.recentlyClosed
                        .take(8)
                        .toList(growable: false),
                    onRestore: widget.onRestoreClosed,
                    onDismiss: widget.onDismissClosed,
                  ),
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
                      case _LibraryAppMenuAction.annotations:
                        widget.onAnnotations?.call();
                      case _LibraryAppMenuAction.automations:
                        widget.onAutomations?.call();
                      case _LibraryAppMenuAction.problems:
                        widget.onProblems?.call();
                      case _LibraryAppMenuAction.settings:
                        widget.onOpenSettings();
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
                    if (widget.onAnnotations != null)
                      PopupMenuItem(
                        value: _LibraryAppMenuAction.annotations,
                        child: Text(strings.get('annotations.globalTitle')),
                      ),
                    if (widget.onAutomations != null)
                      PopupMenuItem(
                        value: _LibraryAppMenuAction.automations,
                        child: Text(strings.get('automation.title')),
                      ),
                    if (widget.onProblems != null)
                      PopupMenuItem(
                        value: _LibraryAppMenuAction.problems,
                        child: Text(strings.get('problems.title')),
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
                    const PopupMenuDivider(),
                    PopupMenuItem(
                      value: _LibraryAppMenuAction.settings,
                      child: Text(strings.settingsTitle),
                    ),
                  ],
                  icon: const Icon(Icons.more_horiz_rounded),
                ),
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
                        padding: const EdgeInsets.symmetric(horizontal: 12),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const Icon(Icons.add, size: 18),
                          if (MediaQuery.sizeOf(context).width >= 900 &&
                              MediaQuery.textScalerOf(context).scale(14) <=
                                  21) ...[
                            const SizedBox(width: 8),
                            Text(strings.libraryOpenPdf),
                          ],
                        ],
                      ),
                    ),
                  ),
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
                    reduceMotion: widget.reduceMotion,
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
                    onDocumentHidden: widget.onDocumentHidden,
                    collections: widget.organization?.collections ?? const [],
                    tags: widget.organization?.tags ?? const [],
                    collectionMemberships:
                        widget.organization?.collectionMemberships ?? const {},
                    tagMemberships:
                        widget.organization?.tagMemberships ?? const {},
                    recentlyClosed: widget.recentlyClosed,
                    onRestoreClosed: widget.onRestoreClosed,
                    onDismissClosed: widget.onDismissClosed,
                    onReread: widget.onReread,
                    onMarkFinished: widget.onMarkFinished,
                    readingGoal: widget.readingGoal,
                    onSetReadingGoal: widget.onSetReadingGoal,
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
    required this.reduceMotion,
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
    this.onDocumentHidden,
    this.collections = const [],
    this.tags = const [],
    this.collectionMemberships = const {},
    this.tagMemberships = const {},
    this.recentlyClosed = const [],
    this.onRestoreClosed,
    this.onDismissClosed,
    this.onReread,
    this.onMarkFinished,
    this.readingGoal,
    this.onSetReadingGoal,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final bool showRecents;
  final bool reduceMotion;
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
  final ValueChanged<LibraryDocument>? onDocumentHidden;
  final List<DocumentCollection> collections;
  final List<DocumentTag> tags;
  final Map<String, Set<String>> collectionMemberships;
  final Map<String, Set<String>> tagMemberships;
  final List<RecentlyClosedDocument> recentlyClosed;
  final ValueChanged<RecentlyClosedDocument>? onRestoreClosed;
  final ValueChanged<RecentlyClosedDocument>? onDismissClosed;
  final Future<void> Function(LibraryDocument document)? onReread;
  final Future<void> Function(LibraryDocument document)? onMarkFinished;
  final ReadingGoal? readingGoal;
  final ValueChanged<LibraryDocument>? onSetReadingGoal;

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
    final documents = controller.documents;
    final continueReading = controller.continueReading;
    return Column(
      children: [
        if (showRecents && continueReading != null)
          ContinueReadingStrip(
            document: continueReading,
            onOpen: () => onOpenPath(continueReading.path),
            onInfo: onDocumentInfo == null
                ? null
                : () => onDocumentInfo!(continueReading),
            onReread: onReread == null
                ? null
                : () => onReread!(continueReading),
            onMarkFinished: onMarkFinished == null
                ? null
                : () => onMarkFinished!(continueReading),
            goal: readingGoal,
            onSetGoal: onSetReadingGoal == null
                ? null
                : () => onSetReadingGoal!(continueReading),
          ),
        Expanded(
          child: controller.libraryDocumentCount == 0
              ? EmptyLibraryState(onOpen: onPickDocument)
              : ArtifactLibraryView(
                  controller: controller,
                  documents: documents,
                  onOpenPath: onOpenPath,
                  onPickDocument: onPickDocument,
                  initialReadingState: readingState,
                  onReadingStateChanged: onReadingStateChanged,
                  reduceMotion: reduceMotion,
                  onBulkOperation: onBulkOperation,
                  collections: collections,
                  tags: tags,
                  collectionMemberships: collectionMemberships,
                  tagMemberships: tagMemberships,
                  onDocumentInfo: onDocumentInfo,
                  onRelocateDocument: onRelocateDocument,
                  onToggleFavorite: onToggleFavorite,
                  onChangeCover: onChangeCover,
                  onRevealDocument: onRevealDocument,
                  onRemoveDocument: onRemoveDocument,
                  onReanalyzeDocument: onReanalyzeDocument,
                  onDocumentVisible: onDocumentVisible,
                  onDocumentHidden: onDocumentHidden,
                ),
        ),
      ],
    );
  }
}

final class ContinuityCenter extends StatelessWidget {
  const ContinuityCenter({
    super.key,
    required this.summary,
    required this.hasActiveReading,
    required this.hasRecentlyClosed,
    this.onContinue,
    this.onOpenNext,
    this.onResumeSession,
    this.onDiscardContext,
    this.onProblems,
  });

  final ContinuitySummary summary;
  final bool hasActiveReading;
  final bool hasRecentlyClosed;
  final VoidCallback? onContinue;
  final VoidCallback? onOpenNext;
  final VoidCallback? onResumeSession;
  final VoidCallback? onDiscardContext;
  final VoidCallback? onProblems;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final resumeDetails = <String>[
      if (summary.openDocuments > 0)
        '${strings.get('continuity.open')}: ${summary.openDocuments}',
      if (summary.pinnedDocuments > 0)
        '${strings.get('continuity.pinned')}: ${summary.pinnedDocuments}',
      if (summary.workspaceCount > 0)
        '${strings.get('continuity.workspaces')}: ${summary.workspaceCount}',
    ].join(' · ');
    final activityCount =
        summary.openDocuments +
        summary.pendingReadings +
        summary.workspaceCount +
        summary.problemCount;
    return PopupMenuButton<_ContinuityAction>(
      key: const Key('continuity-menu-button'),
      tooltip: strings.get('continuity.title'),
      constraints: const BoxConstraints(minWidth: 260, maxWidth: 360),
      onSelected: (action) {
        switch (action) {
          case _ContinuityAction.continueReading:
            onContinue?.call();
          case _ContinuityAction.openNext:
            onOpenNext?.call();
          case _ContinuityAction.resume:
            onResumeSession?.call();
          case _ContinuityAction.problems:
            onProblems?.call();
          case _ContinuityAction.discard:
            onDiscardContext?.call();
        }
      },
      itemBuilder: (context) => [
        if (hasActiveReading && onContinue != null)
          PopupMenuItem(
            key: const Key('continuity-continue'),
            value: _ContinuityAction.continueReading,
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.play_arrow_rounded),
              title: Text(strings.get('continuity.continue')),
            ),
          ),
        if (onResumeSession != null)
          PopupMenuItem(
            key: const Key('continuity-resume'),
            value: _ContinuityAction.resume,
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.restore_rounded),
              title: Text(strings.get('continuity.resume')),
              subtitle: resumeDetails.isEmpty
                  ? null
                  : Text(
                      resumeDetails,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
            ),
          ),
        if (onOpenNext != null)
          PopupMenuItem(
            key: const Key('continuity-open-next'),
            value: _ContinuityAction.openNext,
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.skip_next_rounded),
              title: Text(strings.get('continuity.openNext')),
              trailing: summary.pendingReadings > 0
                  ? Badge(label: Text('${summary.pendingReadings}'))
                  : null,
            ),
          ),
        if (summary.problemCount > 0 && onProblems != null)
          PopupMenuItem(
            key: const Key('continuity-problems'),
            value: _ContinuityAction.problems,
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.report_problem_outlined),
              title: Text(strings.get('continuity.problems')),
              trailing: Badge(label: Text('${summary.problemCount}')),
            ),
          ),
        if ((hasActiveReading || hasRecentlyClosed) &&
            onDiscardContext != null) ...[
          const PopupMenuDivider(),
          PopupMenuItem(
            key: const Key('continuity-discard'),
            value: _ContinuityAction.discard,
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.layers_clear_outlined),
              title: Text(strings.get('continuity.discard')),
            ),
          ),
        ],
      ],
      icon: Badge(
        isLabelVisible: activityCount > 0,
        label: Text(activityCount > 99 ? '99+' : '$activityCount'),
        child: const Icon(Icons.layers_outlined),
      ),
    );
  }
}

enum _ContinuityAction { continueReading, openNext, resume, problems, discard }

final class ContinueReadingStrip extends StatefulWidget {
  const ContinueReadingStrip({
    super.key,
    required this.document,
    required this.onOpen,
    this.onInfo,
    this.onReread,
    this.onMarkFinished,
    this.goal,
    this.onSetGoal,
  });

  final LibraryDocument document;
  final VoidCallback onOpen;
  final VoidCallback? onInfo;
  final VoidCallback? onReread;
  final VoidCallback? onMarkFinished;
  final ReadingGoal? goal;
  final VoidCallback? onSetGoal;

  @override
  State<ContinueReadingStrip> createState() => _ContinueReadingStripState();
}

final class _ContinueReadingStripState extends State<ContinueReadingStrip> {
  final FocusNode _focusNode = FocusNode(debugLabel: 'continue-reading');

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final document = widget.document;
    final page = (document.lastPageIndex + 1).clamp(1, document.pageCount);
    final percent = (document.readingProgress * 100).round().clamp(1, 99);
    final progress = strings.get(
      'library.pageProgress',
      parameters: {
        'page': page,
        'count': document.pageCount,
        'progress': percent,
      },
    );
    return Semantics(
      container: true,
      button: true,
      label:
          '${strings.get('library.continueReading')}. '
          '${document.effectiveTitle}. $progress.',
      hint: 'Enter',
      child: Focus(
        focusNode: _focusNode,
        onKeyEvent: (_, event) {
          if (event is KeyDownEvent &&
              (event.logicalKey == LogicalKeyboardKey.enter ||
                  event.logicalKey == LogicalKeyboardKey.numpadEnter)) {
            widget.onOpen();
            return KeyEventResult.handled;
          }
          if (event is KeyDownEvent &&
              (event.logicalKey == LogicalKeyboardKey.contextMenu ||
                  (event.logicalKey == LogicalKeyboardKey.f10 &&
                      HardwareKeyboard.instance.isShiftPressed))) {
            _showContextMenu(context, const Offset(48, 96));
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 4),
          child: GestureDetector(
            onSecondaryTapDown: (details) =>
                _showContextMenu(context, details.globalPosition),
            child: Material(
              color: context.appColors.surfacePrimary,
              borderRadius: BorderRadius.circular(AppRadii.panel),
              child: InkWell(
                borderRadius: BorderRadius.circular(AppRadii.panel),
                key: const Key('continue-reading-strip'),
                onTap: widget.onOpen,
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    AppSpacing.space6,
                    AppSpacing.space8,
                    AppSpacing.space6,
                  ),
                  child: LayoutBuilder(
                    builder: (context, _) {
                      return Row(
                        children: [
                          Container(
                            width: 4,
                            height: 44,
                            decoration: BoxDecoration(
                              color: context.appColors.focusRing,
                              borderRadius: BorderRadius.circular(
                                AppRadii.indicator,
                              ),
                            ),
                          ),
                          const SizedBox(width: AppSpacing.space12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  document.effectiveTitle,
                                  style: Theme.of(
                                    context,
                                  ).textTheme.titleMedium,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                ),
                                const SizedBox(height: AppSpacing.space4),
                                Row(
                                  children: [
                                    Flexible(
                                      child: Text(
                                        progress,
                                        maxLines: 1,
                                        overflow: TextOverflow.ellipsis,
                                        style: Theme.of(
                                          context,
                                        ).textTheme.bodySmall,
                                      ),
                                    ),
                                    const SizedBox(width: AppSpacing.space12),
                                    Expanded(
                                      child: LinearProgressIndicator(
                                        value: document.readingProgress,
                                        minHeight: 3,
                                        borderRadius: BorderRadius.circular(2),
                                      ),
                                    ),
                                  ],
                                ),
                                if (widget.goal case final goal?)
                                  Padding(
                                    padding: const EdgeInsets.only(
                                      top: AppSpacing.space4,
                                    ),
                                    child: Text(
                                      _goalLabel(strings, goal),
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: Theme.of(context)
                                          .textTheme
                                          .labelSmall
                                          ?.copyWith(
                                            color: context.appColors.textMuted,
                                            fontFeatures: const [
                                              FontFeature.tabularFigures(),
                                            ],
                                          ),
                                    ),
                                  ),
                              ],
                            ),
                          ),
                          const SizedBox(width: AppSpacing.space12),
                          Tooltip(
                            message: strings.get('action.resumeReading'),
                            child: FilledButton(
                              onPressed: widget.onOpen,
                              style: FilledButton.styleFrom(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 12,
                                ),
                              ),
                              child: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  if (MediaQuery.sizeOf(context).width >= 900 &&
                                      MediaQuery.textScalerOf(
                                            context,
                                          ).scale(14) <=
                                          21) ...[
                                    Text(strings.get('action.resumeReading')),
                                    const SizedBox(width: 8),
                                  ],
                                  const Icon(Icons.arrow_forward, size: 18),
                                ],
                              ),
                            ),
                          ),
                        ],
                      );
                    },
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _showContextMenu(BuildContext context, Offset position) async {
    final strings = YamantakaLocalizations.of(context);
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final selected = await showMenu<_ContinueReadingAction>(
      context: context,
      position: RelativeRect.fromRect(
        Rect.fromPoints(position, position),
        Offset.zero & overlay.size,
      ),
      items: [
        PopupMenuItem(
          value: _ContinueReadingAction.open,
          child: Text(strings.get('action.continue')),
        ),
        if (widget.onInfo != null)
          PopupMenuItem(
            value: _ContinueReadingAction.info,
            child: Text(strings.get('command.documentInfo')),
          ),
        if (widget.onReread != null)
          PopupMenuItem(
            value: _ContinueReadingAction.reread,
            child: Text(strings.get('action.reread')),
          ),
        if (widget.onMarkFinished != null)
          PopupMenuItem(
            value: _ContinueReadingAction.finished,
            child: Text(strings.get('library.finished')),
          ),
        if (widget.onSetGoal != null)
          PopupMenuItem(
            value: _ContinueReadingAction.goal,
            child: Text(strings.get('library.localGoal')),
          ),
      ],
    );
    if (!mounted) return;
    switch (selected) {
      case _ContinueReadingAction.open:
        widget.onOpen();
      case _ContinueReadingAction.info:
        widget.onInfo?.call();
      case _ContinueReadingAction.reread:
        widget.onReread?.call();
      case _ContinueReadingAction.finished:
        widget.onMarkFinished?.call();
      case _ContinueReadingAction.goal:
        widget.onSetGoal?.call();
      case null:
        _focusNode.requestFocus();
    }
  }

  String _goalLabel(YamantakaLocalizations strings, ReadingGoal goal) {
    final parts = <String>[];
    if (goal.pages case final pages?) {
      final done = (widget.document.lastPageIndex + 1).clamp(0, pages);
      parts.add('$done/$pages ${strings.get('library.goalPages')}');
    }
    if (goal.minutes case final minutes?) {
      parts.add('$minutes ${strings.get('library.goalMinutes')}');
    }
    return '${strings.get('library.localGoal')}: ${parts.join(' · ')}';
  }
}

enum _ContinueReadingAction { open, info, reread, finished, goal }

final class RecentlyClosedStrip extends StatelessWidget {
  const RecentlyClosedStrip({
    super.key,
    required this.entries,
    this.onRestore,
    this.onDismiss,
  });

  final List<RecentlyClosedDocument> entries;
  final ValueChanged<RecentlyClosedDocument>? onRestore;
  final ValueChanged<RecentlyClosedDocument>? onDismiss;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Align(
      alignment: AlignmentDirectional.centerEnd,
      child: Padding(
        padding: EdgeInsets.zero,
        child: PopupMenuButton<RecentlyClosedDocument>(
          key: const Key('library-recently-closed-menu'),
          tooltip: strings.get('library.recentlyClosed'),
          constraints: const BoxConstraints(minWidth: 280, maxWidth: 440),
          onSelected: onRestore,
          itemBuilder: (context) => [
            for (final entry in entries)
              PopupMenuItem(
                key: ValueKey('recently-closed-${entry.documentId}'),
                value: entry,
                child: Row(
                  children: [
                    const Icon(Icons.restore_page_outlined, size: 20),
                    const SizedBox(width: AppSpacing.space12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            entry.title,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          Text(
                            entry.isMissing
                                ? strings.get('library.fileMissing')
                                : strings.get(
                                    'library.closedAtPage',
                                    parameters: {'page': entry.pageIndex + 1},
                                  ),
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ],
                      ),
                    ),
                    if (onDismiss != null)
                      IconButton(
                        tooltip: strings.get('action.close'),
                        icon: const Icon(Icons.close_rounded, size: 18),
                        onPressed: () {
                          Navigator.of(context).pop();
                          onDismiss!(entry);
                        },
                      ),
                  ],
                ),
              ),
          ],
          icon: const Icon(Icons.history_rounded, size: AppIconSizes.large),
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
    final strings = YamantakaLocalizations.of(context);
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(AppSpacing.space32),
        child: Semantics(
          container: true,
          label: 'Yamantaka. ${strings.get('library.emptyTagline')}',
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                strings.get('library.emptyTagline'),
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: context.appColors.textSecondary,
                ),
              ),
              const SizedBox(height: AppSpacing.space24),
              FilledButton.icon(
                key: const Key('open-pdf-button'),
                onPressed: onOpen,
                icon: const Icon(Icons.file_open_outlined, size: 18),
                label: Text(strings.libraryOpenPdf),
              ),
            ],
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
  annotations,
  automations,
  problems,
  settings,
}
