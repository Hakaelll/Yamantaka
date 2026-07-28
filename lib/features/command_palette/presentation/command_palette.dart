import 'package:flutter/material.dart';

import '../../../app/shortcuts/app_commands.dart';

final class CommandPalette extends StatefulWidget {
  const CommandPalette({super.key, required this.actions});

  final Map<AppCommandId, VoidCallback> actions;

  @override
  State<CommandPalette> createState() => _CommandPaletteState();
}

final class _CommandPaletteState extends State<CommandPalette> {
  String _query = '';

  @override
  Widget build(BuildContext context) {
    final normalized = _query.trim().toLowerCase();
    final commands = AppCommandRegistry.definitions
        .where((definition) => widget.actions.containsKey(definition.id))
        .where(
          (definition) =>
              normalized.isEmpty ||
              definition.label.toLowerCase().contains(normalized) ||
              definition.keywords.any(
                (keyword) => keyword.toLowerCase().contains(normalized),
              ),
        )
        .toList(growable: false);
    return Dialog(
      alignment: const Alignment(0, -0.55),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560, maxHeight: 480),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              key: const Key('command-palette-search'),
              autofocus: true,
              decoration: const InputDecoration(
                hintText: 'Escribe una acción…',
                prefixIcon: Icon(Icons.search_rounded),
                border: InputBorder.none,
              ),
              onChanged: (value) => setState(() => _query = value),
            ),
            const Divider(height: 1),
            Flexible(
              child: commands.isEmpty
                  ? const Padding(
                      padding: EdgeInsets.all(28),
                      child: Text('No hay acciones que coincidan.'),
                    )
                  : ListView.builder(
                      shrinkWrap: true,
                      itemCount: commands.length,
                      itemBuilder: (context, index) {
                        final command = commands[index];
                        return ListTile(
                          autofocus: index == 0,
                          title: Text(command.label),
                          trailing: command.shortcutLabel == null
                              ? null
                              : Text(
                                  command.shortcutLabel!,
                                  style: Theme.of(context).textTheme.bodySmall,
                                ),
                          onTap: () {
                            Navigator.pop(context);
                            widget.actions[command.id]?.call();
                          },
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }
}
