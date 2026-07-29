import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/folder_import_controller.dart';

final class FolderImportDialog extends StatelessWidget {
  const FolderImportDialog({
    super.key,
    required this.controller,
    required this.onImport,
  });

  final FolderImportController controller;
  final Future<void> Function() onImport;

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680, maxHeight: 640),
        child: ListenableBuilder(
          listenable: controller,
          builder: (context, _) {
            final newCount = controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.newDocument,
                )
                .length;
            final duplicates = controller.candidates
                .where((item) => item.status == FolderCandidateStatus.duplicate)
                .length;
            final inaccessible = controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.inaccessible,
                )
                .length;
            final invalid = controller.candidates
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
                        onPressed: controller.isImporting
                            ? null
                            : () => Navigator.pop(context),
                        icon: const Icon(Icons.close_rounded),
                      ),
                    ],
                  ),
                ),
                if (controller.isScanning || controller.isImporting)
                  LinearProgressIndicator(
                    value: controller.total == 0
                        ? null
                        : controller.processed / controller.total,
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
                        value: controller.candidates.length,
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
                  child: controller.candidates.isEmpty
                      ? AppStatusView(
                          icon: Icons.folder_off_outlined,
                          title: controller.isScanning
                              ? 'Buscando PDF'
                              : 'No se encontraron PDF',
                          message:
                              'Ningún archivo se añadirá sin confirmación.',
                          loading: controller.isScanning,
                        )
                      : ListView.builder(
                          itemCount: controller.candidates.length,
                          itemBuilder: (context, index) {
                            final candidate = controller.candidates[index];
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
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      if (controller.isScanning || controller.isImporting)
                        TextButton(
                          onPressed: controller.cancel,
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
                                controller.isScanning ||
                                controller.isImporting
                            ? null
                            : onImport,
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
