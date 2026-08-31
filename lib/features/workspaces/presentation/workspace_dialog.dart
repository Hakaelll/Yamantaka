import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

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
    required this.focusMode,
    this.onOpenQueued,
  });

  final WorkspaceController controller;
  final ValueChanged<WorkspaceSnapshot> onRestore;
  final String panelSection;
  final double panelWidth;
  final bool focusMode;
  final ValueChanged<DocumentTab>? onOpenQueued;

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
                    if (controller.activeWorkspaceId != null) ...[
                      const SizedBox(width: AppSpacing.space8),
                      OutlinedButton.icon(
                        onPressed: () {
                          final next = controller.openNext();
                          if (next == null) return;
                          Navigator.pop(context);
                          onOpenQueued?.call(next);
                        },
                        icon: const Icon(Icons.skip_next_rounded, size: 18),
                        label: Text(strings.get('workspace.openNext')),
                      ),
                    ],
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
                child: WorkspaceOverview(
                  controller: controller,
                  onRestore: (workspace) {
                    Navigator.pop(context);
                    onRestore(workspace);
                  },
                  onOpenQueued: (tab) {
                    Navigator.pop(context);
                    onOpenQueued?.call(tab);
                  },
                  onAction: (workspace, action) =>
                      _runAction(context, workspace, action),
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
      focusMode: focusMode,
    );
  }

  Future<void> _runAction(
    BuildContext context,
    WorkspaceSnapshot workspace,
    WorkspaceAction action,
  ) async {
    final strings = YamantakaLocalizations.of(context);
    switch (action) {
      case WorkspaceAction.restore:
        Navigator.pop(context);
        onRestore(workspace);
      case WorkspaceAction.rename:
        final name = await _askName(
          context,
          strings.get('workspace.renameTitle'),
          initialValue: workspace.name,
        );
        if (name != null) controller.rename(workspace.id, name);
      case WorkspaceAction.duplicate:
        final name = await _askName(
          context,
          strings.get('workspace.duplicateTitle'),
          initialValue: strings.get(
            'workspace.copySuffix',
            parameters: {'name': workspace.name},
          ),
        );
        if (name != null) controller.duplicate(workspace.id, name);
      case WorkspaceAction.delete:
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
        if (confirmed == true) {
          controller.delete(workspace.id);
          if (!context.mounted) return;
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(strings.get('workspace.deleted')),
              action: SnackBarAction(
                label: strings.get('action.undo'),
                onPressed: controller.undoDelete,
              ),
            ),
          );
        }
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

enum WorkspaceAction { restore, rename, duplicate, delete }

final class WorkspaceOverview extends StatelessWidget {
  const WorkspaceOverview({
    super.key,
    required this.controller,
    required this.onRestore,
    required this.onOpenQueued,
    required this.onAction,
    this.expandActiveWorkspace = false,
  });

  final WorkspaceController controller;
  final ValueChanged<WorkspaceSnapshot> onRestore;
  final ValueChanged<DocumentTab> onOpenQueued;
  final void Function(WorkspaceSnapshot workspace, WorkspaceAction action)
  onAction;
  final bool expandActiveWorkspace;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    if (controller.workspaces.isEmpty) {
      return AppStatusView(
        icon: Icons.space_dashboard_outlined,
        title: strings.get('workspace.emptyTitle'),
        message: strings.get('workspace.emptyBody'),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: AppSpacing.space8),
      itemCount: controller.workspaces.length,
      separatorBuilder: (_, _) => const Divider(
        indent: AppSpacing.space16,
        endIndent: AppSpacing.space16,
      ),
      itemBuilder: (context, index) {
        final workspace = controller.workspaces[index];
        final missing = workspace.tabs.where((tab) => tab.isMissing).length;
        final isActive = controller.activeWorkspaceId == workspace.id;
        return ExpansionTile(
          initiallyExpanded: expandActiveWorkspace && isActive,
          leading: Icon(
            workspace.isDefault
                ? Icons.home_work_rounded
                : Icons.space_dashboard_outlined,
          ),
          title: Row(
            children: [
              Flexible(child: Text(workspace.name)),
              if (isActive) ...[
                const SizedBox(width: AppSpacing.space8),
                Chip(
                  visualDensity: VisualDensity.compact,
                  label: Text(strings.get('workspace.active')),
                ),
              ],
            ],
          ),
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
                    : strings.count('workspace.missing', missing),
              },
            ),
          ),
          trailing: PopupMenuButton<WorkspaceAction>(
            tooltip: strings.get('workspace.actions'),
            onSelected: (action) => onAction(workspace, action),
            itemBuilder: (context) => [
              PopupMenuItem(
                value: WorkspaceAction.restore,
                child: Text(strings.get('workspace.restore')),
              ),
              PopupMenuItem(
                value: WorkspaceAction.rename,
                child: Text(strings.get('action.rename')),
              ),
              PopupMenuItem(
                value: WorkspaceAction.duplicate,
                child: Text(strings.get('workspace.duplicate')),
              ),
              PopupMenuItem(
                value: WorkspaceAction.delete,
                child: Text(strings.get('action.delete')),
              ),
            ],
          ),
          children: [
            if (!isActive)
              ListTile(
                leading: const Icon(Icons.restore_outlined),
                title: Text(strings.get('workspace.restoreToEdit')),
                onTap: () => onRestore(workspace),
              )
            else if (workspace.tabs.isEmpty)
              ListTile(title: Text(strings.get('workspace.queueEmpty')))
            else
              for (
                var tabIndex = 0;
                tabIndex < workspace.tabs.length;
                tabIndex++
              )
                _QueueEntryTile(
                  key: ValueKey(workspace.tabs[tabIndex].id),
                  tab: workspace.tabs[tabIndex],
                  index: tabIndex,
                  count: workspace.tabs.length,
                  onMove: (target) => controller.reorderQueue(tabIndex, target),
                  onStatus: (status) => controller.updateQueueEntry(
                    workspace.tabs[tabIndex].id,
                    status: status,
                  ),
                  onOpen: () => onOpenQueued(workspace.tabs[tabIndex]),
                ),
          ],
        );
      },
    );
  }
}

final class _QueueEntryTile extends StatelessWidget {
  const _QueueEntryTile({
    super.key,
    required this.tab,
    required this.index,
    required this.count,
    required this.onMove,
    required this.onStatus,
    required this.onOpen,
  });

  final DocumentTab tab;
  final int index;
  final int count;
  final ValueChanged<int> onMove;
  final ValueChanged<WorkspaceQueueStatus> onStatus;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Focus(
      onKeyEvent: (_, event) {
        if (event is! KeyDownEvent ||
            !HardwareKeyboard.instance.isControlPressed) {
          return KeyEventResult.ignored;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowUp && index > 0) {
          onMove(index - 1);
          return KeyEventResult.handled;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowDown &&
            index < count - 1) {
          onMove(index + 1);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: ListTile(
        contentPadding: const EdgeInsets.only(
          left: AppSpacing.space32,
          right: AppSpacing.space12,
        ),
        leading: SizedBox(
          width: 88,
          child: Row(
            children: [
              IconButton(
                tooltip: strings.get('workspace.moveUp'),
                onPressed: index == 0 ? null : () => onMove(index - 1),
                constraints: const BoxConstraints.tightFor(
                  width: 44,
                  height: 44,
                ),
                icon: const Icon(Icons.arrow_upward_rounded, size: 18),
              ),
              IconButton(
                tooltip: strings.get('workspace.moveDown'),
                onPressed: index == count - 1 ? null : () => onMove(index + 1),
                constraints: const BoxConstraints.tightFor(
                  width: 44,
                  height: 44,
                ),
                icon: const Icon(Icons.arrow_downward_rounded, size: 18),
              ),
            ],
          ),
        ),
        title: Text(tab.title, maxLines: 1, overflow: TextOverflow.ellipsis),
        subtitle: tab.blockingReason == null && tab.nextAction == null
            ? null
            : Text(
                [
                  tab.blockingReason,
                  tab.nextAction,
                ].whereType<String>().join(' · '),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
        onTap: onOpen,
        trailing: Tooltip(
          message: strings.get('workspace.queueStatus'),
          child: DropdownButton<WorkspaceQueueStatus>(
            value: tab.queueStatus,
            onChanged: (value) {
              if (value != null) onStatus(value);
            },
            items: [
              for (final status in WorkspaceQueueStatus.values)
                DropdownMenuItem(
                  value: status,
                  child: Text(strings.get('workspace.status.${status.name}')),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
