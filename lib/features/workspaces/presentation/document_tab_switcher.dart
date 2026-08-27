import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../domain/workspace_models.dart';

final class DocumentTabSwitcherDialog extends StatefulWidget {
  const DocumentTabSwitcherDialog({
    super.key,
    required this.tabs,
    required this.activeTabId,
    required this.reverse,
  });

  final List<DocumentTab> tabs;
  final String? activeTabId;
  final bool reverse;

  @override
  State<DocumentTabSwitcherDialog> createState() =>
      _DocumentTabSwitcherDialogState();
}

final class _DocumentTabSwitcherDialogState
    extends State<DocumentTabSwitcherDialog> {
  late int _selectedIndex;

  @override
  void initState() {
    super.initState();
    final active = widget.tabs.indexWhere(
      (tab) => tab.id == widget.activeTabId,
    );
    final start = active < 0 ? 0 : active;
    _selectedIndex = widget.tabs.length < 2
        ? start
        : (start + (widget.reverse ? -1 : 1)) % widget.tabs.length;
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.arrowDown): () => _move(1),
        const SingleActivator(LogicalKeyboardKey.arrowUp): () => _move(-1),
        const SingleActivator(LogicalKeyboardKey.tab, control: true): () =>
            _move(1),
        const SingleActivator(
          LogicalKeyboardKey.tab,
          control: true,
          shift: true,
        ): () =>
            _move(-1),
        const SingleActivator(LogicalKeyboardKey.enter): _accept,
        const SingleActivator(LogicalKeyboardKey.numpadEnter): _accept,
      },
      child: Focus(
        autofocus: true,
        child: Dialog(
          alignment: Alignment.topCenter,
          insetPadding: const EdgeInsets.only(top: 72, left: 16, right: 16),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 520, maxHeight: 420),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: AppSpacing.space12),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: AppSpacing.space16,
                    ),
                    child: Text(
                      strings.get('tabs.switcherTitle'),
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  Flexible(
                    child: ListView.builder(
                      shrinkWrap: true,
                      itemCount: widget.tabs.length,
                      itemBuilder: (context, index) {
                        final tab = widget.tabs[index];
                        final selected = index == _selectedIndex;
                        return ListTile(
                          key: ValueKey('tab-switcher-${tab.id}'),
                          selected: selected,
                          minTileHeight: 48,
                          leading: Icon(
                            tab.isMissing
                                ? Icons.link_off_rounded
                                : tab.isPinned
                                ? Icons.push_pin_rounded
                                : Icons.picture_as_pdf_outlined,
                            size: 19,
                          ),
                          title: Text(
                            tab.title,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          subtitle: Text(
                            strings.get(
                              'tabs.position',
                              parameters: {
                                'page': tab.pageIndex + 1,
                                'progress': (tab.readingProgress * 100).round(),
                              },
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          trailing: tab.isExternallyModified
                              ? Tooltip(
                                  message: strings.get('search.resultStale'),
                                  child: const Icon(
                                    Icons.sync_problem_rounded,
                                    size: 18,
                                  ),
                                )
                              : tab.resourceState == TabResourceState.suspended
                              ? Tooltip(
                                  message: strings.get('tabs.suspended'),
                                  child: const Icon(
                                    Icons.pause_circle_outline_rounded,
                                    size: 18,
                                  ),
                                )
                              : null,
                          onTap: () => Navigator.pop(context, tab.id),
                          onFocusChange: (focused) {
                            if (focused) setState(() => _selectedIndex = index);
                          },
                        );
                      },
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(
                      AppSpacing.space16,
                      AppSpacing.space8,
                      AppSpacing.space16,
                      0,
                    ),
                    child: Text(
                      strings.get('tabs.switcherHint'),
                      style: Theme.of(context).textTheme.labelSmall,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _move(int delta) {
    if (widget.tabs.isEmpty) return;
    setState(() {
      _selectedIndex = (_selectedIndex + delta) % widget.tabs.length;
    });
  }

  void _accept() {
    if (widget.tabs.isEmpty) return;
    Navigator.pop(context, widget.tabs[_selectedIndex].id);
  }
}
