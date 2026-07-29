import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_tokens.dart';
import '../../organization/application/organization_controller.dart';
import '../../organization/domain/organization_models.dart';
import '../application/library_controller.dart';
import '../application/library_filter.dart';
import '../domain/library_document.dart';
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
    this.onDocumentInfo,
    this.onRelocateDocument,
    this.onToggleFavorite,
    this.onChangeCover,
    this.onRevealDocument,
    this.onRemoveDocument,
    this.organization,
    this.onGlobalSearch,
    this.onOpenPaths,
    this.onWorkspaces,
    this.onImportFolder,
    this.onRecoverFolder,
    this.safeMode = false,
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
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;
  final OrganizationController? organization;
  final VoidCallback? onGlobalSearch;
  final Future<void> Function(List<String> paths)? onOpenPaths;
  final VoidCallback? onWorkspaces;
  final VoidCallback? onImportFolder;
  final VoidCallback? onRecoverFolder;
  final bool safeMode;

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
    final compact = context.windowClass == AppWindowClass.compact;
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
            appBar: AppBar(
              title: const Text('Biblioteca'),
              actions: [
                if (widget.safeMode)
                  const Tooltip(
                    message:
                        'Modo seguro: la sesión y el índice no se restauraron',
                    child: Padding(
                      padding: EdgeInsets.all(AppSpacing.space12),
                      child: Icon(Icons.health_and_safety_outlined),
                    ),
                  ),
                PopupMenuButton<_LibraryAppMenuAction>(
                  tooltip: 'Organizar y más acciones',
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
                    }
                  },
                  itemBuilder: (context) => [
                    if (widget.organization != null)
                      const PopupMenuItem(
                        value: _LibraryAppMenuAction.collections,
                        child: Text('Colecciones'),
                      ),
                    if (widget.onWorkspaces != null)
                      const PopupMenuItem(
                        value: _LibraryAppMenuAction.workspaces,
                        child: Text('Espacios de trabajo'),
                      ),
                    if (widget.onImportFolder != null)
                      const PopupMenuItem(
                        value: _LibraryAppMenuAction.importFolder,
                        child: Text('Importar carpeta'),
                      ),
                    if (widget.onRecoverFolder != null &&
                        widget.controller.documents.any(
                          (document) => document.isMissing,
                        ))
                      const PopupMenuItem(
                        value: _LibraryAppMenuAction.recover,
                        child: Text('Recuperar documentos movidos'),
                      ),
                    if (widget.onGlobalSearch != null)
                      const PopupMenuItem(
                        value: _LibraryAppMenuAction.globalSearch,
                        child: Text('Búsqueda global'),
                      ),
                  ],
                  icon: const Icon(Icons.more_horiz_rounded),
                ),
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
                    onDocumentInfo: widget.onDocumentInfo,
                    onRelocateDocument: widget.onRelocateDocument,
                    onToggleFavorite: widget.onToggleFavorite,
                    onChangeCover: widget.onChangeCover,
                    onRevealDocument: widget.onRevealDocument,
                    onRemoveDocument: widget.onRemoveDocument,
                    documentPredicate: widget.organization?.includes,
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

  Future<void> _showCollections() async {
    final organization = widget.organization;
    if (organization == null) return;
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: ListView(
          shrinkWrap: true,
          children: [
            ListTile(
              leading: const Icon(Icons.library_books_outlined),
              title: const Text('Toda la biblioteca'),
              selected:
                  organization.selectedCollectionId == null &&
                  organization.selectedSmartCollection == null,
              onTap: () {
                organization.clearSelection();
                Navigator.pop(context);
              },
            ),
            for (final smart in SmartCollection.values)
              ListTile(
                leading: const Icon(Icons.auto_awesome_outlined),
                title: Text(smart.label),
                selected: organization.selectedSmartCollection == smart,
                onTap: () {
                  organization.selectSmartCollection(smart);
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
                  Navigator.pop(context);
                },
              ),
            ListTile(
              leading: const Icon(Icons.create_new_folder_outlined),
              title: const Text('Nueva colección'),
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
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Nueva colección'),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: 60,
          decoration: const InputDecoration(labelText: 'Nombre'),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, controller.text),
            child: const Text('Crear'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (name != null && name.trim().isNotEmpty) {
      final created = organization.createCollection(name);
      organization.selectCollection(created.id);
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
    required this.onDocumentInfo,
    required this.onRelocateDocument,
    required this.onToggleFavorite,
    required this.onChangeCover,
    required this.onRevealDocument,
    required this.onRemoveDocument,
    this.documentPredicate,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final bool showRecents;
  final LibraryReadingState readingState;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final Future<void> Function(LibraryDocument document)? onDocumentInfo;
  final Future<void> Function(LibraryDocument document)? onRelocateDocument;
  final Future<void> Function(LibraryDocument document)? onToggleFavorite;
  final Future<void> Function(LibraryDocument document)? onChangeCover;
  final Future<void> Function(LibraryDocument document)? onRevealDocument;
  final Future<void> Function(LibraryDocument document)? onRemoveDocument;
  final bool Function(String documentId)? documentPredicate;

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
    final documents = controller.documents
        .where((document) => documentPredicate?.call(document.id) ?? true)
        .toList(growable: false);
    if (!showRecents) {
      return Center(
        child: OutlinedButton.icon(
          onPressed: onPickDocument,
          icon: const Icon(Icons.file_open_outlined, size: 18),
          label: const Text('Abrir otro PDF'),
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
      onDocumentInfo: onDocumentInfo,
      onRelocateDocument: onRelocateDocument,
      onToggleFavorite: onToggleFavorite,
      onChangeCover: onChangeCover,
      onRevealDocument: onRevealDocument,
      onRemoveDocument: onRemoveDocument,
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

String _readingStateDescription(LibraryDocument document) {
  if (document.readingProgress >= 1) return 'Terminado';
  if (document.readingProgress > 0) {
    return 'En curso · ${(document.readingProgress * 100).round()} %';
  }
  return 'Sin empezar';
}

enum _LibraryAppMenuAction {
  collections,
  workspaces,
  importFolder,
  recover,
  globalSearch,
}
