import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../application/document_tabs_controller.dart';
import '../domain/workspace_models.dart';

final class DocumentTabBar extends StatelessWidget {
  const DocumentTabBar({
    super.key,
    required this.controller,
    required this.onActivate,
    required this.onClose,
    required this.onOpenDocument,
  });

  final DocumentTabsController controller;
  final ValueChanged<DocumentTab> onActivate;
  final ValueChanged<DocumentTab> onClose;
  final VoidCallback onOpenDocument;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        if (controller.tabs.isEmpty) return const SizedBox.shrink();
        return Material(
          color: context.appColors.surfaceSunken,
          child: SizedBox(
            height: 42,
            child: Row(
              children: [
                Expanded(
                  child: ReorderableListView.builder(
                    scrollDirection: Axis.horizontal,
                    buildDefaultDragHandles: false,
                    padding: const EdgeInsets.only(left: AppSpacing.space8),
                    itemCount: controller.tabs.length,
                    onReorderItem: controller.reorder,
                    itemBuilder: (context, index) {
                      final tab = controller.tabs[index];
                      return ReorderableDragStartListener(
                        key: ValueKey(tab.id),
                        index: index,
                        child: _DocumentTabItem(
                          tab: tab,
                          selected: tab.id == controller.activeTabId,
                          onActivate: () => onActivate(tab),
                          onClose: () => onClose(tab),
                          onPin: () => controller.togglePinned(tab.id),
                          onCloseOthers: () => controller.closeOthers(tab.id),
                          onCloseRight: () => controller.closeToRight(tab.id),
                        ),
                      );
                    },
                  ),
                ),
                PopupMenuButton<String>(
                  tooltip: 'Lista de pestañas',
                  onSelected: (id) {
                    final tab = controller.tabs
                        .where((item) => item.id == id)
                        .firstOrNull;
                    if (tab != null) onActivate(tab);
                  },
                  itemBuilder: (context) => [
                    for (final tab in controller.tabs)
                      PopupMenuItem(
                        value: tab.id,
                        child: Row(
                          children: [
                            if (tab.isPinned)
                              const Padding(
                                padding: EdgeInsets.only(
                                  right: AppSpacing.space8,
                                ),
                                child: Icon(Icons.push_pin_outlined, size: 16),
                              ),
                            Expanded(
                              child: Text(
                                tab.title,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
                  icon: const Icon(Icons.keyboard_arrow_down_rounded),
                ),
                IconButton(
                  tooltip: 'Abrir PDF en pestaña nueva',
                  onPressed: onOpenDocument,
                  icon: const Icon(Icons.add_rounded),
                ),
                const SizedBox(width: AppSpacing.space4),
              ],
            ),
          ),
        );
      },
    );
  }
}

final class _DocumentTabItem extends StatelessWidget {
  const _DocumentTabItem({
    required this.tab,
    required this.selected,
    required this.onActivate,
    required this.onClose,
    required this.onPin,
    required this.onCloseOthers,
    required this.onCloseRight,
  });

  final DocumentTab tab;
  final bool selected;
  final VoidCallback onActivate;
  final VoidCallback onClose;
  final VoidCallback onPin;
  final VoidCallback onCloseOthers;
  final VoidCallback onCloseRight;

  @override
  Widget build(BuildContext context) {
    final warning = tab.isMissing || tab.isExternallyModified;
    return Semantics(
      selected: selected,
      button: true,
      label:
          'Pestaña ${tab.title}, '
          '${tab.isPinned ? 'fijada, ' : ''}'
          '${tab.resourceState.name}',
      child: SizedBox(
        width: 190,
        child: InkWell(
          onTap: onActivate,
          onSecondaryTapDown: (details) =>
              _showMenu(context, details.globalPosition),
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: selected
                  ? context.appColors.surfacePrimary
                  : Colors.transparent,
              border: Border(
                bottom: BorderSide(
                  color: selected
                      ? context.appColors.accentPrimary
                      : context.appColors.borderSubtle,
                  width: selected ? 2 : 1,
                ),
                right: BorderSide(color: context.appColors.borderSubtle),
              ),
            ),
            child: Column(
              children: [
                Expanded(
                  child: Row(
                    children: [
                      const SizedBox(width: AppSpacing.space8),
                      if (tab.isPinned)
                        const Padding(
                          padding: EdgeInsets.only(right: AppSpacing.space6),
                          child: Icon(Icons.push_pin_outlined, size: 14),
                        ),
                      if (warning)
                        Padding(
                          padding: const EdgeInsets.only(
                            right: AppSpacing.space6,
                          ),
                          child: Icon(
                            tab.isMissing
                                ? Icons.link_off_rounded
                                : Icons.sync_problem_rounded,
                            size: 15,
                            color: context.appColors.danger,
                          ),
                        ),
                      Expanded(
                        child: Tooltip(
                          message: tab.title,
                          child: Text(
                            tab.title,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.labelMedium,
                          ),
                        ),
                      ),
                      IconButton(
                        tooltip: 'Cerrar ${tab.title}',
                        onPressed: onClose,
                        visualDensity: VisualDensity.compact,
                        iconSize: 16,
                        icon: const Icon(Icons.close_rounded),
                      ),
                    ],
                  ),
                ),
                LinearProgressIndicator(
                  value: tab.readingProgress.clamp(0, 1),
                  minHeight: 1,
                  backgroundColor: Colors.transparent,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _showMenu(BuildContext context, Offset position) async {
    final action = await showMenu<_TabAction>(
      context: context,
      position: RelativeRect.fromLTRB(
        position.dx,
        position.dy,
        position.dx,
        position.dy,
      ),
      items: [
        PopupMenuItem(
          value: _TabAction.pin,
          child: Text(tab.isPinned ? 'Desfijar' : 'Fijar pestaña'),
        ),
        const PopupMenuItem(
          value: _TabAction.closeOthers,
          child: Text('Cerrar otras'),
        ),
        const PopupMenuItem(
          value: _TabAction.closeRight,
          child: Text('Cerrar las de la derecha'),
        ),
        const PopupMenuItem(value: _TabAction.close, child: Text('Cerrar')),
      ],
    );
    switch (action) {
      case _TabAction.pin:
        onPin();
      case _TabAction.closeOthers:
        onCloseOthers();
      case _TabAction.closeRight:
        onCloseRight();
      case _TabAction.close:
        onClose();
      case null:
        break;
    }
  }
}

enum _TabAction { pin, closeOthers, closeRight, close }
