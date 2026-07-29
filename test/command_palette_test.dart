import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/shortcuts/app_commands.dart';
import 'package:yamantaka/features/command_palette/presentation/command_palette.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';

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

  testWidgets('arrow keys select a command and Enter executes it', (
    tester,
  ) async {
    var executed = '';
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CommandPalette(
            actions: {
              AppCommandId.openDocument: () => executed = 'open',
              AppCommandId.search: () => executed = 'search',
            },
          ),
        ),
      ),
    );
    await tester.pump();

    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();

    expect(executed, 'search');
  });

  testWidgets('an unmatched query presents an explicit empty state', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CommandPalette(actions: {AppCommandId.openDocument: () {}}),
        ),
      ),
    );

    await tester.enterText(
      find.byKey(const Key('command-palette-search')),
      'acción inexistente',
    );
    await tester.pump();

    expect(find.text('No hay acciones que coincidan.'), findsOneWidget);
  });

  testWidgets('Escape closes the palette dialog while search has focus', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => FilledButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) =>
                  CommandPalette(actions: {AppCommandId.openDocument: () {}}),
            ),
            child: const Text('Mostrar paleta'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Mostrar paleta'));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('command-palette-search')), findsOneWidget);

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('command-palette-search')), findsNothing);
  });

  testWidgets('reduced motion uses immediate keyboard selection scrolling', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CommandPalette(
            reduceMotion: true,
            actions: {
              for (final definition in AppCommandRegistry.definitions)
                definition.id: () {},
            },
          ),
        ),
      ),
    );

    for (var index = 0; index < 9; index++) {
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();
    }

    final list = tester.widget<ListView>(find.byType(ListView));
    expect(list.controller!.position.isScrollingNotifier.value, isFalse);
  });

  testWidgets('recent documents share search and keyboard execution', (
    tester,
  ) async {
    LibraryDocument? opened;
    final now = DateTime.utc(2026, 7, 29);
    final document = LibraryDocument(
      id: 'recent-1',
      path: 'notes.pdf',
      normalizedPath: 'notes.pdf',
      fileName: 'notes.pdf',
      title: 'Notas del volcán',
      author: 'Ana',
      pageCount: 12,
      fileSize: 1024,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'recent-1',
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CommandPalette(
            actions: {AppCommandId.openDocument: () {}},
            recentDocuments: [document],
            onOpenRecent: (value) => opened = value,
          ),
        ),
      ),
    );

    await tester.enterText(
      find.byKey(const Key('command-palette-search')),
      'volcán',
    );
    await tester.pump();
    expect(find.text('Notas del volcán'), findsOneWidget);
    expect(find.text('Documento reciente'), findsNothing);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    expect(opened?.id, 'recent-1');
  });
}
