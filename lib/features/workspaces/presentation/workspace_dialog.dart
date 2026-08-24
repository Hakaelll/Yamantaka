import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
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
    final strings = YamantakaLocalizations.of(context);
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
                        strings.get('workspace.title'),
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                    ),
                    FilledButton.tonalIcon(
                      onPressed: () => _save(context),
                      icon: const Icon(Icons.save_outlined, size: 18),
                      label: Text(strings.get('workspace.saveCurrent')),
                    ),
                    const SizedBox(width: AppSpacing.space8),
                    IconButton(
                      tooltip: strings.get('action.close'),
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(Icons.close_rounded),
                    ),
                  ],
                ),
              ),
              const Divider(height: 1),
              Expanded(
                child: controller.workspaces.isEmpty
                    ? AppStatusView(
                        icon: Icons.space_dashboard_outlined,
                        title: strings.get('workspace.emptyTitle'),
                        message: strings.get('workspace.emptyBody'),
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
                              strings.get(
                                'workspace.summary',
                                parameters: {
                                  'documents': strings.count(
                                    'workspace.documents',
                                    workspace.tabs.length,
                                  ),
                                  'missing': missing == 0
                                      ? strings.get('workspace.missing.none')
                                      : strings.count(
                                          'workspace.missing',
                                          missing,
                                        ),
                                },
                              ),
                            ),
                            onTap: () {
                              Navigator.pop(context);
                              onRestore(workspace);
                            },
                            trailing: PopupMenuButton<_WorkspaceAction>(
                              tooltip: strings.get('workspace.actions'),
                              onSelected: (action) =>
                                  _runAction(context, workspace, action),
                              itemBuilder: (context) => [
                                PopupMenuItem(
                                  value: _WorkspaceAction.duplicate,
                                  child: Text(
                                    strings.get('workspace.duplicate'),
                                  ),
                                ),
                                PopupMenuItem(
                                  value: _WorkspaceAction.delete,
                                  child: Text(strings.get('action.delete')),
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
    final name = await _askName(
      context,
      YamantakaLocalizations.of(context).get('workspace.saveTitle'),
    );
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
    final strings = YamantakaLocalizations.of(context);
    switch (action) {
      case _WorkspaceAction.duplicate:
        final name = await _askName(
          context,
          strings.get('workspace.duplicateTitle'),
          initialValue: strings.get(
            'workspace.copySuffix',
            parameters: {'name': workspace.name},
          ),
        );
        if (name != null) controller.duplicate(workspace.id, name);
      case _WorkspaceAction.delete:
        final confirmed = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(strings.get('workspace.deleteTitle')),
            content: Text(strings.get('workspace.deleteBody')),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(strings.get('action.cancel')),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(strings.get('action.delete')),
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
          decoration: InputDecoration(
            labelText: YamantakaLocalizations.of(context).get('workspace.name'),
          ),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(
              YamantakaLocalizations.of(context).get('action.cancel'),
            ),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, textController.text),
            child: Text(YamantakaLocalizations.of(context).get('action.save')),
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
