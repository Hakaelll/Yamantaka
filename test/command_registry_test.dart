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
}
