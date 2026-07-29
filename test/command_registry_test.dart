import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/shortcuts/app_commands.dart';

void main() {
  test('central registry exposes every required core shortcut', () {
    var calls = 0;
    final actions = {for (final id in AppCommandId.values) id: () => calls++};
    final shortcuts = AppCommandRegistry.shortcuts(actions);

    expect(
      shortcuts,
      contains(const SingleActivator(LogicalKeyboardKey.keyO, control: true)),
    );
    expect(
      shortcuts,
      contains(const SingleActivator(LogicalKeyboardKey.keyK, control: true)),
    );
    expect(shortcuts, contains(const SingleActivator(LogicalKeyboardKey.f11)));
    expect(
      shortcuts,
      contains(const SingleActivator(LogicalKeyboardKey.keyH, control: true)),
    );
    expect(
      shortcuts,
      contains(const SingleActivator(LogicalKeyboardKey.pageDown)),
    );
    expect(
      shortcuts,
      contains(const SingleActivator(LogicalKeyboardKey.space, shift: true)),
    );

    shortcuts[const SingleActivator(LogicalKeyboardKey.keyO, control: true)]
        ?.call();
    expect(calls, 1);
  });

  test('custom shortcuts are validated, conflict-free and reversible', () {
    final current = AppCommandRegistry.resolvedTokens('{}');
    expect(
      AppCommandRegistry.assignmentError(
        AppCommandId.search,
        current[AppCommandId.commandPalette]!,
        current,
      ),
      contains('asignada'),
    );
    expect(
      AppCommandRegistry.assignmentError(
        AppCommandId.escape,
        'ctrl-g',
        current,
      ),
      contains('reservada'),
    );

    current[AppCommandId.openDocument] = 'ctrl-shift-o';
    current[AppCommandId.search] = 'ctrl-shift-f';
    final encoded = AppCommandRegistry.encodeOverrides(current);
    final resolved = AppCommandRegistry.resolvedTokens(encoded);
    expect(resolved[AppCommandId.search], 'ctrl-shift-f');
    expect(resolved[AppCommandId.openDocument], 'ctrl-shift-o');
    expect(
      AppCommandRegistry.shortcutLabel(AppCommandId.search, encoded),
      'Ctrl+Mayús+F',
    );
    expect(
      AppCommandRegistry.resolvedTokens('{}'),
      AppCommandRegistry.defaultTokens,
    );
    expect(
      AppCommandRegistry.configurableIds,
      containsAll({
        AppCommandId.openDocument,
        AppCommandId.togglePanel,
        AppCommandId.nextPage,
        AppCommandId.previousPage,
        AppCommandId.zoomIn,
        AppCommandId.zoomOut,
        AppCommandId.toggleChrome,
      }),
    );
  });
}
