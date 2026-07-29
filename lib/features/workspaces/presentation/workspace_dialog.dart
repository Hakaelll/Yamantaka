import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/workspace_controller.dart';
import '../domain/workspace_models.dart';

final class WorkspaceDialog extends StatelessWidget {
  const WorkspaceDialog({
    super.key,
    required this.controller,
    required this.onRestore,
    required this.panelSection,
    required this.panelWidth,
  });

  final WorkspaceController controller;
  final ValueChanged<WorkspaceSnapshot> onRestore;
  final String panelSection;
  final double panelWidth;

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 620, maxHeight: 620),
        child: ListenableBuilder(
          listenable: controller,
          builder: (context, _) => Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(AppSpacing.space16),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        'Espacios de trabajo',
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                    ),
                    FilledButton.tonalIcon(
                      onPressed: () => _save(context),
                      icon: const Icon(Icons.save_outlined, size: 18),
                      label: const Text('Guardar actual'),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    IconButton(
                      tooltip: 'Cerrar',
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(Icons.close_rounded),
                    ),
                  ],
                ),
              ),
              const Divider(height: 1),
              Expanded(
                child: controller.workspaces.isEmpty
                    ? const AppStatusView(
                        icon: Icons.space_dashboard_outlined,
                        title: 'Sin espacios guardados',
                        message:
                            'Guarda las pestañas, posiciones y panel actuales '
                            'para recuperar esta sesión de lectura.',
                      )
                    : ListView.separated(
                        padding: const EdgeInsets.symmetric(
                          vertical: AppSpacing.space8,
                        ),
                        itemCount: controller.workspaces.length,
                        separatorBuilder: (_, _) => const Divider(
                          indent: AppSpacing.space16,
                          endIndent: AppSpacing.space16,
                        ),
                        itemBuilder: (context, index) {
                          final workspace = controller.workspaces[index];
                          final missing = workspace.tabs
                              .where((tab) => tab.isMissing)
                              .length;
                          return ListTile(
                            leading: Icon(
                              workspace.isDefault
                                  ? Icons.home_work_rounded
                                  : Icons.space_dashboard_outlined,
                            ),
                            title: Text(workspace.name),
                            subtitle: Text(
                              '${workspace.tabs.length} pestañas'
                              '${missing == 0 ? '' : ' · $missing ausentes'}',
                            ),
                            onTap: () {
                              Navigator.pop(context);
                              onRestore(workspace);
                            },
                            trailing: PopupMenuButton<_WorkspaceAction>(
                              tooltip: 'Acciones del espacio',
                              onSelected: (action) =>
                                  _runAction(context, workspace, action),
                              itemBuilder: (context) => const [
                                PopupMenuItem(
                                  value: _WorkspaceAction.duplicate,
                                  child: Text('Duplicar'),
                                ),
                                PopupMenuItem(
                                  value: _WorkspaceAction.delete,
                                  child: Text('Eliminar'),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _save(BuildContext context) async {
    final name = await _askName(context, 'Guardar espacio');
    if (name == null) return;
    controller.saveCurrent(
      name: name,
      panelSection: panelSection,
      panelWidth: panelWidth,
      isDefault: controller.workspaces.isEmpty,
    );
  }

  Future<void> _runAction(
    BuildContext context,
    WorkspaceSnapshot workspace,
    _WorkspaceAction action,
  ) async {
    switch (action) {
      case _WorkspaceAction.duplicate:
        final name = await _askName(
          context,
          'Duplicar espacio',
          initialValue: '${workspace.name} — copia',
        );
        if (name != null) controller.duplicate(workspace.id, name);
      case _WorkspaceAction.delete:
        final confirmed = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Eliminar espacio'),
            content: const Text(
              'Se elimina la sesión guardada, no los documentos ni sus '
              'anotaciones.',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: const Text('Cancelar'),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: const Text('Eliminar'),
              ),
            ],
          ),
        );
        if (confirmed == true) controller.delete(workspace.id);
    }
  }

  Future<String?> _askName(
    BuildContext context,
    String title, {
    String initialValue = '',
  }) async {
    final textController = TextEditingController(text: initialValue);
    final value = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: TextField(
          controller: textController,
          autofocus: true,
          maxLength: 80,
          decoration: const InputDecoration(labelText: 'Nombre'),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, textController.text),
            child: const Text('Guardar'),
          ),
        ],
      ),
    );
    textController.dispose();
    final clean = value?.trim();
    return clean == null || clean.isEmpty ? null : clean;
  }
}

enum _WorkspaceAction { duplicate, delete }
