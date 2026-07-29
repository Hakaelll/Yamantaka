import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
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
    final entries = _filteredEntries();
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
                        decoration: const InputDecoration(
                          hintText: 'Buscar una acción',
                          prefixIcon: Icon(Icons.search_rounded),
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
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  AppSpacing.space16,
                  AppSpacing.space12,
                  AppSpacing.space16,
                  AppSpacing.space6,
                ),
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: Text(
                    'ACCIONES Y DOCUMENTOS RECIENTES',
                    style: Theme.of(context).textTheme.labelSmall,
                  ),
                ),
              ),
              Flexible(
                child: entries.isEmpty
                    ? const _EmptyPalette()
                    : ListView.builder(
                        controller: _scrollController,
                        padding: const EdgeInsets.only(
                          bottom: AppSpacing.space8,
                        ),
                        shrinkWrap: true,
                        itemCount: entries.length,
                        itemExtent: 56,
                        itemBuilder: (context, index) {
                          final entry = entries[index];
                          final selected = index == selectedIndex;
                          return Semantics(
                            selected: selected,
                            button: true,
                            child: MouseRegion(
                              onEnter: (_) =>
                                  setState(() => _selectedIndex = index),
                              child: Material(
                                color: selected
                                    ? context.appColors.selectionBackground
                                    : Colors.transparent,
                                child: InkWell(
                                  key: ValueKey(entry.key),
                                  onTap: () => _execute(entry),
                                  child: Container(
                                    decoration: BoxDecoration(
                                      border: Border(
                                        left: BorderSide(
                                          color: selected
                                              ? context.appColors.accentPrimary
                                              : Colors.transparent,
                                          width: 3,
                                        ),
                                      ),
                                    ),
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: AppSpacing.space16,
                                    ),
                                    child: Row(
                                      children: [
                                        Icon(
                                          entry.icon,
                                          size: 18,
                                          color: context.appColors.textMuted,
                                        ),
                                        const SizedBox(
                                          width: AppSpacing.space12,
                                        ),
                                        Expanded(
                                          child: Column(
                                            mainAxisAlignment:
                                                MainAxisAlignment.center,
                                            crossAxisAlignment:
                                                CrossAxisAlignment.start,
                                            children: [
                                              Text(
                                                entry.label,
                                                maxLines: 1,
                                                overflow: TextOverflow.ellipsis,
                                                style: Theme.of(
                                                  context,
                                                ).textTheme.bodyMedium,
                                              ),
                                              if (entry.subtitle
                                                  case final subtitle?)
                                                Text(
                                                  subtitle,
                                                  maxLines: 1,
                                                  overflow:
                                                      TextOverflow.ellipsis,
                                                  style: Theme.of(
                                                    context,
                                                  ).textTheme.labelSmall,
                                                ),
                                            ],
                                          ),
                                        ),
                                        if (entry.shortcutLabel
                                            case final shortcut?)
                                          AppShortcutHint(shortcut),
                                      ],
                                    ),
                                  ),
                                ),
                              ),
                            ),
                          );
                        },
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
                child: const Row(
                  children: [
                    AppShortcutHint('↑↓'),
                    SizedBox(width: AppSpacing.space6),
                    Text('Mover'),
                    SizedBox(width: AppSpacing.space16),
                    AppShortcutHint('Enter'),
                    SizedBox(width: AppSpacing.space6),
                    Text('Ejecutar'),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<_PaletteEntry> _filteredEntries() {
    final normalized = _query.trim().toLowerCase();
    final entries = <_PaletteEntry>[];
    for (final definition in AppCommandRegistry.definitionsFor(
      widget.shortcutOverrides,
    )) {
      if (!widget.actions.containsKey(definition.id)) continue;
      if (normalized.isNotEmpty &&
          !definition.label.toLowerCase().contains(normalized) &&
          !definition.keywords.any(
            (keyword) => keyword.toLowerCase().contains(normalized),
          )) {
        continue;
      }
      entries.add(_PaletteEntry.command(definition));
    }
    if (widget.onOpenRecent != null) {
      for (final document in widget.recentDocuments) {
        final label = document.title ?? document.fileName;
        final searchable =
            '$label ${document.author ?? ''} ${document.fileName}'
                .toLowerCase();
        if (normalized.isEmpty || searchable.contains(normalized)) {
          entries.add(_PaletteEntry.document(document));
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
      final target = (index * 56.0)
          .clamp(0, _scrollController.position.maxScrollExtent)
          .toDouble();
      if (widget.reduceMotion) {
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

  factory _PaletteEntry.command(AppCommandDefinition command) =>
      _PaletteEntry._(
        key: 'command-${command.id.name}',
        label: command.label,
        icon: Icons.bolt_rounded,
        shortcutLabel: command.shortcutLabel,
        command: command,
      );

  factory _PaletteEntry.document(LibraryDocument document) => _PaletteEntry._(
    key: 'recent-${document.id}',
    label: document.title ?? document.fileName,
    icon: Icons.history_rounded,
    subtitle: document.author ?? 'Documento reciente',
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
  const _EmptyPalette();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(AppSpacing.space32),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.search_off_rounded, color: context.appColors.textMuted),
          const SizedBox(height: AppSpacing.space8),
          const Text('No hay acciones que coincidan.'),
        ],
      ),
    );
  }
}
