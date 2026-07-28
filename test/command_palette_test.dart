import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/shortcuts/app_commands.dart';
import 'package:yamantaka/features/command_palette/presentation/command_palette.dart';

void main() {
  testWidgets('command palette filters and executes a registered action', (
    tester,
  ) async {
    var executed = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CommandPalette(
            actions: {
              AppCommandId.openDocument: () => executed = true,
              AppCommandId.search: () {},
            },
          ),
        ),
      ),
    );

    await tester.enterText(
      find.byKey(const Key('command-palette-search')),
      'abrir',
    );
    await tester.pump();
    expect(find.text('Abrir documento'), findsOneWidget);
    expect(find.text('Buscar en el documento'), findsNothing);
    await tester.tap(find.text('Abrir documento'));
    expect(executed, isTrue);
  });
}
