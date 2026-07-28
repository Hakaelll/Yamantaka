import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';

import '../../../app/app_config.dart';
import '../application/library_controller.dart';
import '../domain/library_document.dart';

final class LibraryScreen extends StatefulWidget {
  const LibraryScreen({
    super.key,
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.onOpenSettings,
    required this.showRecents,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final VoidCallback onOpenSettings;
  final bool showRecents;

  @override
  State<LibraryScreen> createState() => _LibraryScreenState();
}

final class _LibraryScreenState extends State<LibraryScreen> {
  bool _dragging = false;

  @override
  Widget build(BuildContext context) {
    return DropTarget(
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
            IconButton(
              tooltip: 'Configuración',
              onPressed: widget.onOpenSettings,
              icon: const Icon(Icons.tune_rounded),
            ),
            const SizedBox(width: 8),
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
              ),
            ),
            if (_dragging)
              Positioned.fill(
                child: ColoredBox(
                  color: Theme.of(
                    context,
                  ).colorScheme.primary.withValues(alpha: 0.12),
                  child: Center(
                    child: Semantics(
                      label: 'Suelta el PDF para abrirlo',
                      child: const _DropMessage(),
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

final class _LibraryBody extends StatelessWidget {
  const _LibraryBody({
    required this.controller,
    required this.onOpenPath,
    required this.onPickDocument,
    required this.showRecents,
  });

  final LibraryController controller;
  final Future<void> Function(String path) onOpenPath;
  final Future<void> Function() onPickDocument;
  final bool showRecents;

  @override
  Widget build(BuildContext context) {
    if (controller.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (controller.error case final error?) {
      return _LibraryError(
        message: error.userMessage,
        onRetry: controller.load,
      );
    }
    if (controller.documents.isEmpty) {
      return EmptyLibraryState(onOpen: onPickDocument);
    }
    final continueReading = controller.continueReading;
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 920),
        child: CustomScrollView(
          slivers: [
            SliverPadding(
              padding: const EdgeInsets.fromLTRB(32, 32, 32, 12),
              sliver: SliverToBoxAdapter(
                child: Text(
                  'Tu lectura',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
            ),
            if (continueReading != null)
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 32),
                sliver: SliverToBoxAdapter(
                  child: ContinueReadingCard(
                    document: continueReading,
                    onOpen: () => onOpenPath(continueReading.path),
                  ),
                ),
              ),
            if (showRecents) ...[
              SliverPadding(
                padding: const EdgeInsets.fromLTRB(32, 28, 32, 8),
                sliver: SliverToBoxAdapter(
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          'Recientes',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                      ),
                      TextButton.icon(
                        onPressed: onPickDocument,
                        icon: const Icon(Icons.file_open_outlined, size: 18),
                        label: const Text('Abrir PDF'),
                      ),
                    ],
                  ),
                ),
              ),
              SliverPadding(
                padding: const EdgeInsets.fromLTRB(32, 0, 32, 40),
                sliver: SliverList.separated(
                  itemCount: controller.documents.length,
                  separatorBuilder: (_, _) => const Divider(height: 1),
                  itemBuilder: (context, index) {
                    final document = controller.documents[index];
                    return RecentDocumentItem(
                      document: document,
                      onOpen: () => document.isMissing
                          ? onPickDocument()
                          : onOpenPath(document.path),
                    );
                  },
                ),
              ),
            ] else
              SliverPadding(
                padding: const EdgeInsets.all(32),
                sliver: SliverToBoxAdapter(
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton.icon(
                      onPressed: onPickDocument,
                      icon: const Icon(Icons.file_open_outlined),
                      label: const Text('Abrir otro PDF'),
                    ),
                  ),
                ),
              ),
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
    return LayoutBuilder(
      builder: (context, constraints) => SingleChildScrollView(
        child: ConstrainedBox(
          constraints: BoxConstraints(minHeight: constraints.maxHeight),
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: Padding(
                padding: const EdgeInsets.all(32),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.menu_book_outlined,
                      size: 48,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    const SizedBox(height: 20),
                    Text(
                      'Una biblioteca tranquila',
                      style: Theme.of(context).textTheme.headlineSmall,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Abre un PDF o arrástralo a esta ventana. El archivo se '
                      'queda en su carpeta original.',
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 24),
                    FilledButton.icon(
                      key: const Key('open-pdf-button'),
                      onPressed: onOpen,
                      icon: const Icon(Icons.file_open_outlined),
                      label: const Text('Abrir PDF'),
                    ),
                    const SizedBox(height: 12),
                    Text(
                      'También puedes usar Ctrl+O',
                      style: Theme.of(context).textTheme.bodySmall,
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

final class ContinueReadingCard extends StatelessWidget {
  const ContinueReadingCard({
    super.key,
    required this.document,
    required this.onOpen,
  });

  final LibraryDocument document;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label:
          'Continuar leyendo ${document.fileName}, '
          '${(document.readingProgress * 100).round()} por ciento',
      child: InkWell(
        onTap: onOpen,
        borderRadius: BorderRadius.circular(4),
        child: Ink(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surface,
            border: Border.all(color: Theme.of(context).dividerColor),
            borderRadius: BorderRadius.circular(4),
          ),
          padding: const EdgeInsets.all(20),
          child: Row(
            children: [
              _DocumentCover(document: document, width: 68, height: 92),
              const SizedBox(width: 20),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Continuar leyendo',
                      style: Theme.of(context).textTheme.labelLarge?.copyWith(
                        color: Theme.of(context).colorScheme.primary,
                      ),
                    ),
                    const SizedBox(height: 6),
                    Text(
                      document.title ?? document.fileName,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                    const SizedBox(height: 12),
                    LinearProgressIndicator(
                      value: document.readingProgress,
                      minHeight: 3,
                    ),
                    const SizedBox(height: 6),
                    Text(
                      'Página ${document.lastPageIndex + 1} de '
                      '${document.pageCount}',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 16),
              const Icon(Icons.arrow_forward_rounded),
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
  });

  final LibraryDocument document;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      key: ValueKey('recent-${document.id}'),
      onTap: onOpen,
      contentPadding: const EdgeInsets.symmetric(vertical: 8),
      leading: _DocumentCover(document: document, width: 40, height: 54),
      title: Text(
        document.fileName,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        document.isMissing
            ? 'Archivo no encontrado · Localizar'
            : '${(document.readingProgress * 100).round()} % · '
                  '${_relativeDate(document.lastOpenedAt)}',
        style: document.isMissing
            ? TextStyle(color: Theme.of(context).colorScheme.error)
            : null,
      ),
      trailing: document.isMissing
          ? const Icon(Icons.link_off_rounded)
          : Text('${document.lastPageIndex + 1}/${document.pageCount}'),
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
    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border.all(color: Theme.of(context).dividerColor),
      ),
      clipBehavior: Clip.antiAlias,
      child: thumbnailPath != null && File(thumbnailPath).existsSync()
          ? Image.file(File(thumbnailPath), fit: BoxFit.cover)
          : const Icon(Icons.picture_as_pdf_outlined),
    );
  }
}

final class _DropMessage extends StatelessWidget {
  const _DropMessage();

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border.all(
          color: Theme.of(context).colorScheme.primary,
          width: 2,
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: const Padding(
        padding: EdgeInsets.symmetric(horizontal: 28, vertical: 20),
        child: Text('Suelta el PDF para abrirlo'),
      ),
    );
  }
}

final class _LibraryError extends StatelessWidget {
  const _LibraryError({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline_rounded, size: 40),
          const SizedBox(height: 12),
          Text(message),
          const SizedBox(height: 16),
          OutlinedButton(onPressed: onRetry, child: const Text('Reintentar')),
        ],
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
