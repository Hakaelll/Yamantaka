import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../library/domain/library_document.dart';

final class CommandPalette extends StatefulWidget {
  const CommandPalette({
    super.key,
    required this.actions,
    this.reduceMotion = false,
    this.shortcutOverrides = '{}',
    this.recentDocuments = const [],
    this.onOpenRecent,
  });

  final Map<AppCommandId, VoidCallback> actions;
  final bool reduceMotion;
  final String shortcutOverrides;
  final List<LibraryDocument> recentDocuments;
  final ValueChanged<LibraryDocument>? onOpenRecent;

  @override
  State<CommandPalette> createState() => _CommandPaletteState();
}

final class _CommandPaletteState extends State<CommandPalette> {
  final ScrollController _scrollController = ScrollController();
  String _query = '';
  int _selectedIndex = 0;

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final entries = _filteredEntries(strings);
    final selectedIndex = entries.isEmpty
        ? 0
        : _selectedIndex.clamp(0, entries.length - 1);
    return Focus(
      autofocus: true,
      onKeyEvent: (_, event) => _handleKey(event, entries, selectedIndex),
      child: Dialog(
        alignment: const Alignment(0, -0.52),
        insetPadding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space24,
          vertical: AppSpacing.space24,
        ),
        child: ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: AppSizes.dialogMaxWidth,
            maxHeight: AppSizes.commandPaletteMaxHeight,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  AppSpacing.space16,
                  AppSpacing.space12,
                  AppSpacing.space12,
                  AppSpacing.space12,
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: TextField(
                        key: const Key('command-palette-search'),
                        autofocus: true,
                        decoration: InputDecoration(
                          hintText: strings.commandSearch,
                          prefixIcon: const Icon(Icons.search_rounded),
                        ),
                        onChanged: (value) => setState(() {
                          _query = value;
                          _selectedIndex = 0;
                        }),
                      ),
                    ),
                    const SizedBox(width: AppSpacing.space12),
                    const AppShortcutHint('Esc'),
                  ],
                ),
              ),
              const Divider(height: 1),
              Flexible(
                child: entries.isEmpty
                    ? _EmptyPalette(message: strings.commandEmpty)
                    : ListView(
                        controller: _scrollController,
                        padding: const EdgeInsets.only(
                          bottom: AppSpacing.space8,
                        ),
                        shrinkWrap: true,
                        children: _entryWidgets(
                          entries,
                          selectedIndex,
                          strings,
                        ),
                      ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space16,
                  vertical: AppSpacing.space8,
                ),
                decoration: BoxDecoration(
                  color: context.appColors.surfaceSecondary,
                  border: Border(
                    top: BorderSide(color: context.appColors.borderSubtle),
                  ),
                ),
                child: Row(
                  children: [
                    const AppShortcutHint('↑↓'),
                    const SizedBox(width: AppSpacing.space6),
                    Text(strings.commandMove),
                    const SizedBox(width: AppSpacing.space16),
                    const AppShortcutHint('Enter'),
                    const SizedBox(width: AppSpacing.space6),
                    Text(strings.commandRun),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<_PaletteEntry> _filteredEntries(YamantakaLocalizations strings) {
    final normalized = _query.trim().toLowerCase();
    final entries = <_PaletteEntry>[];
    for (final definition in AppCommandRegistry.definitionsFor(
      widget.shortcutOverrides,
      strings: strings,
    )) {
      if (!widget.actions.containsKey(definition.id)) continue;
      final localizedLabel = strings.get('command.${definition.id.name}');
      if (normalized.isNotEmpty &&
          !localizedLabel.toLowerCase().contains(normalized) &&
          !definition.keywords.any(
            (keyword) => keyword.toLowerCase().contains(normalized),
          )) {
        continue;
      }
      entries.add(_PaletteEntry.command(definition, localizedLabel));
    }
    if (widget.onOpenRecent != null) {
      for (final document in widget.recentDocuments.take(8)) {
        final label = document.effectiveTitle;
        final searchable =
            '$label ${document.effectiveAuthor ?? ''} ${document.fileName} ${document.libraryNote ?? ''}'
                .toLowerCase();
        if (normalized.isEmpty || searchable.contains(normalized)) {
          entries.add(
            _PaletteEntry.document(
              document,
              strings.get('command.recentDocument'),
            ),
          );
        }
      }
    }
    return entries;
  }

  KeyEventResult _handleKey(
    KeyEvent event,
    List<_PaletteEntry> entries,
    int selectedIndex,
  ) {
    if (event is! KeyDownEvent) {
      return KeyEventResult.ignored;
    }
    if (event.logicalKey == LogicalKeyboardKey.escape) {
      Navigator.maybePop(context);
      return KeyEventResult.handled;
    }
    if (entries.isEmpty) return KeyEventResult.ignored;
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      _select((selectedIndex + 1).clamp(0, entries.length - 1));
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      _select((selectedIndex - 1).clamp(0, entries.length - 1));
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.numpadEnter) {
      _execute(entries[selectedIndex]);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _select(int index) {
    setState(() => _selectedIndex = index);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;
      final precedingSectionHeight =
          AppSizes.commandPaletteRowHeight + AppSpacing.space12;
      final hasRecentSectionBefore = _filteredEntries(
        YamantakaLocalizations.of(context),
      ).take(index + 1).any((entry) => entry.document != null);
      final target =
          (index * AppSizes.commandPaletteRowHeight +
                  precedingSectionHeight +
                  (hasRecentSectionBefore ? precedingSectionHeight : 0))
              .clamp(0, _scrollController.position.maxScrollExtent)
              .toDouble();
      if (widget.reduceMotion || MediaQuery.disableAnimationsOf(context)) {
        _scrollController.jumpTo(target);
        return;
      }
      _scrollController.animateTo(
        target,
        duration: AppMotion.fast,
        curve: Curves.easeOutCubic,
      );
    });
  }

  void _execute(_PaletteEntry entry) {
    Navigator.pop(context);
    if (entry.command case final command?) {
      widget.actions[command.id]?.call();
    } else if (entry.document case final document?) {
      widget.onOpenRecent?.call(document);
    }
  }

  List<Widget> _entryWidgets(
    List<_PaletteEntry> entries,
    int selectedIndex,
    YamantakaLocalizations strings,
  ) {
    final widgets = <Widget>[];
    var currentGroup = '';
    for (var index = 0; index < entries.length; index++) {
      final entry = entries[index];
      final group = entry.document == null
          ? strings.commandSection
          : strings.get('command.recentSection');
      if (group != currentGroup) {
        currentGroup = group;
        widgets.add(_PaletteSectionLabel(group));
      }
      widgets.add(
        _PaletteEntryTile(
          entry: entry,
          selected: index == selectedIndex,
          onHover: () => setState(() => _selectedIndex = index),
          onTap: () => _execute(entry),
        ),
      );
    }
    return widgets;
  }
}

final class _PaletteSectionLabel extends StatelessWidget {
  const _PaletteSectionLabel(this.label);

  final String label;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(
      AppSpacing.space16,
      AppSpacing.space12,
      AppSpacing.space16,
      AppSpacing.space6,
    ),
    child: Text(label, style: Theme.of(context).textTheme.labelSmall),
  );
}

final class _PaletteEntryTile extends StatelessWidget {
  const _PaletteEntryTile({
    required this.entry,
    required this.selected,
    required this.onHover,
    required this.onTap,
  });

  final _PaletteEntry entry;
  final bool selected;
  final VoidCallback onHover;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    return Semantics(
      selected: selected,
      button: true,
      child: MouseRegion(
        onEnter: (_) => onHover(),
        child: Material(
          color: selected ? colors.selectionBackground : Colors.transparent,
          child: InkWell(
            key: ValueKey(entry.key),
            onTap: onTap,
            child: Container(
              height: AppSizes.commandPaletteRowHeight,
              decoration: BoxDecoration(
                border: Border(
                  left: BorderSide(
                    color: selected ? colors.accentPrimary : Colors.transparent,
                    width: 3,
                  ),
                ),
              ),
              padding: const EdgeInsets.symmetric(
                horizontal: AppSpacing.space16,
              ),
              child: Row(
                children: [
                  Icon(entry.icon, size: 18, color: colors.textMuted),
                  const SizedBox(width: AppSpacing.space12),
                  Expanded(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          entry.label,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: Theme.of(context).textTheme.bodyMedium,
                        ),
                        if (entry.subtitle case final subtitle?)
                          Text(
                            subtitle,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.labelSmall,
                          ),
                      ],
                    ),
                  ),
                  if (entry.shortcutLabel case final shortcut?)
                    AppShortcutHint(shortcut),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _PaletteEntry {
  const _PaletteEntry._({
    required this.key,
    required this.label,
    required this.icon,
    this.subtitle,
    this.shortcutLabel,
    this.command,
    this.document,
  });

  factory _PaletteEntry.command(
    AppCommandDefinition command,
    String localizedLabel,
  ) => _PaletteEntry._(
    key: 'command-${command.id.name}',
    label: localizedLabel,
    icon: Icons.bolt_rounded,
    shortcutLabel: command.shortcutLabel,
    command: command,
  );

  factory _PaletteEntry.document(
    LibraryDocument document,
    String recentDocumentLabel,
  ) => _PaletteEntry._(
    key: 'recent-${document.id}',
    label: document.effectiveTitle,
    icon: Icons.history_rounded,
    subtitle: document.effectiveAuthor ?? recentDocumentLabel,
    document: document,
  );

  final String key;
  final String label;
  final IconData icon;
  final String? subtitle;
  final String? shortcutLabel;
  final AppCommandDefinition? command;
  final LibraryDocument? document;
}

final class _EmptyPalette extends StatelessWidget {
  const _EmptyPalette({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(AppSpacing.space32),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.search_off_rounded, color: context.appColors.textMuted),
          const SizedBox(height: AppSpacing.space8),
          Text(message),
        ],
      ),
    );
  }
}
