import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/folder_import_controller.dart';

final class FolderImportDialog extends StatefulWidget {
  const FolderImportDialog({
    super.key,
    required this.controller,
    required this.onImport,
    required this.directoryPath,
  });

  final FolderImportController controller;
  final Future<void> Function(bool watchRoot) onImport;
  final String directoryPath;

  @override
  State<FolderImportDialog> createState() => _FolderImportDialogState();
}

final class _FolderImportDialogState extends State<FolderImportDialog> {
  bool _watchRoot = false;

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680, maxHeight: 700),
        child: ListenableBuilder(
          listenable: widget.controller,
          builder: (context, _) {
            final newCount = widget.controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.newDocument,
                )
                .length;
            final duplicates = widget.controller.candidates
                .where((item) => item.status == FolderCandidateStatus.duplicate)
                .length;
            final inaccessible = widget.controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.inaccessible,
                )
                .length;
            final invalid = widget.controller.candidates
                .where((item) => item.status == FolderCandidateStatus.invalid)
                .length;
            return Column(
              children: [
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          'Importar carpeta',
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                      IconButton(
                        tooltip: 'Cerrar',
                        onPressed: widget.controller.isImporting
                            ? null
                            : () => Navigator.pop(context),
                        icon: const Icon(Icons.close_rounded),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    0,
                    AppSpacing.space16,
                    AppSpacing.space8,
                  ),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      widget.directoryPath,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                ),
                if (widget.controller.isScanning ||
                    widget.controller.isImporting)
                  LinearProgressIndicator(
                    value: widget.controller.total == 0
                        ? null
                        : widget.controller.processed / widget.controller.total,
                  ),
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppSpacing.space16,
                    vertical: AppSpacing.space8,
                  ),
                  child: Wrap(
                    spacing: AppSpacing.space8,
                    runSpacing: AppSpacing.space8,
                    children: [
                      _CountChip(
                        label: 'Encontrados',
                        value: widget.controller.candidates.length,
                      ),
                      _CountChip(label: 'Nuevos', value: newCount),
                      _CountChip(label: 'Duplicados', value: duplicates),
                      _CountChip(label: 'Inaccesibles', value: inaccessible),
                      _CountChip(label: 'No válidos', value: invalid),
                    ],
                  ),
                ),
                const Divider(height: 1),
                Expanded(
                  child: widget.controller.candidates.isEmpty
                      ? AppStatusView(
                          icon: Icons.folder_off_outlined,
                          title: widget.controller.isScanning
                              ? 'Buscando PDF'
                              : 'No se encontraron PDF',
                          message:
                              'Ningún archivo se añadirá sin confirmación.',
                          loading: widget.controller.isScanning,
                        )
                      : ListView.builder(
                          itemCount: widget.controller.candidates.length,
                          itemBuilder: (context, index) {
                            final candidate =
                                widget.controller.candidates[index];
                            return ListTile(
                              leading: Icon(_icon(candidate.status)),
                              title: Text(
                                candidate.fileName,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              subtitle: Text(_label(candidate.status)),
                            );
                          },
                        ),
                ),
                const Divider(height: 1),
                CheckboxListTile(
                  value: _watchRoot,
                  onChanged: widget.controller.isImporting
                      ? null
                      : (value) => setState(() => _watchRoot = value ?? false),
                  title: const Text(
                    'Observar esta carpeta después de importar',
                  ),
                  subtitle: const Text(
                    'Solo revalidará cambios. No importará, moverá ni borrará PDF automáticamente.',
                  ),
                  controlAffinity: ListTileControlAffinity.leading,
                  dense: true,
                ),
                const Divider(height: 1),
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      if (widget.controller.isScanning ||
                          widget.controller.isImporting)
                        TextButton(
                          onPressed: widget.controller.cancel,
                          child: const Text('Cancelar proceso'),
                        )
                      else
                        TextButton(
                          onPressed: () => Navigator.pop(context),
                          child: const Text('Cerrar'),
                        ),
                      const SizedBox(width: AppSpacing.space8),
                      FilledButton.icon(
                        onPressed:
                            newCount == 0 ||
                                widget.controller.isScanning ||
                                widget.controller.isImporting
                            ? null
                            : () => widget.onImport(_watchRoot),
                        icon: const Icon(Icons.library_add_outlined, size: 18),
                        label: Text('Añadir $newCount'),
                      ),
                    ],
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  static IconData _icon(FolderCandidateStatus status) => switch (status) {
    FolderCandidateStatus.newDocument => Icons.add_circle_outline_rounded,
    FolderCandidateStatus.duplicate => Icons.content_copy_rounded,
    FolderCandidateStatus.inaccessible => Icons.lock_outline_rounded,
    FolderCandidateStatus.invalid => Icons.error_outline_rounded,
  };

  static String _label(FolderCandidateStatus status) => switch (status) {
    FolderCandidateStatus.newDocument => 'Nuevo',
    FolderCandidateStatus.duplicate => 'Ya está en la biblioteca',
    FolderCandidateStatus.inaccessible => 'Sin permiso de lectura',
    FolderCandidateStatus.invalid => 'PDF no válido',
  };
}

final class _CountChip extends StatelessWidget {
  const _CountChip({required this.label, required this.value});

  final String label;
  final int value;

  @override
  Widget build(BuildContext context) =>
      Chip(label: Text('$label: $value'), visualDensity: VisualDensity.compact);
}
