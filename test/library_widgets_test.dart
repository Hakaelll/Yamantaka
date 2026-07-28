import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';

void main() {
  testWidgets('empty library presents one clear opening action', (
    tester,
  ) async {
    var opened = false;
    await tester.pumpWidget(
      MaterialApp(home: EmptyLibraryState(onOpen: () => opened = true)),
    );

    expect(find.text('Una biblioteca tranquila'), findsOneWidget);
    expect(find.byKey(const Key('open-pdf-button')), findsOneWidget);
    await tester.ensureVisible(find.byKey(const Key('open-pdf-button')));
    await tester.tap(find.byKey(const Key('open-pdf-button')));
    expect(opened, isTrue);
  });

  testWidgets('recent item exposes missing-file state without color alone', (
    tester,
  ) async {
    final now = DateTime.utc(2026, 7, 28);
    final document = LibraryDocument(
      id: 'missing',
      path: 'missing.pdf',
      normalizedPath: 'missing.pdf',
      fileName: 'missing.pdf',
      pageCount: 5,
      fileSize: 100,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'x',
      isMissing: true,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: RecentDocumentItem(document: document, onOpen: () {}),
        ),
      ),
    );

    expect(find.text('Archivo no encontrado · Localizar'), findsOneWidget);
    expect(find.byIcon(Icons.link_off_rounded), findsOneWidget);
  });
}
