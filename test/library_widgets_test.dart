import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

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

    expect(find.text('Archivo no encontrado'), findsOneWidget);
    expect(find.byIcon(Icons.link_off_rounded), findsOneWidget);
    expect(find.text('Localizar'), findsOneWidget);
  });

  testWidgets('recent rows adapt at the exact compact boundary', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final now = DateTime.utc(2026, 7, 28);
    final document = LibraryDocument(
      id: 'responsive',
      path: 'responsive.pdf',
      normalizedPath: 'responsive.pdf',
      fileName: 'responsive.pdf',
      pageCount: 240,
      fileSize: 2048,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'responsive',
      isMissing: true,
    );

    for (final width in const [420.0, 679.0, 680.0, 1099.0, 1100.0]) {
      tester.view.physicalSize = Size(width, 300);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: RecentDocumentItem(document: document, onOpen: () {}),
          ),
        ),
      );
      await tester.pump();

      expect(tester.takeException(), isNull, reason: 'viewport width $width');
      expect(find.text('Archivo no encontrado'), findsOneWidget);
    }
  });

  testWidgets(
    'document information is independently reachable from a recent row',
    (tester) async {
      var opened = false;
      var inspected = false;
      final document = _document(
        id: 'changed',
        fileName: 'changed.pdf',
        openedAt: DateTime.utc(2026, 7, 29),
      ).copyWith(hasFileChanged: true);

      await tester.pumpWidget(
        MaterialApp(
          theme: AppTheme.light(),
          home: Scaffold(
            body: RecentDocumentItem(
              document: document,
              onOpen: () => opened = true,
              onInfo: () => inspected = true,
            ),
          ),
        ),
      );

      expect(find.text('El archivo ha cambiado'), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('document-info-changed')));
      expect(inspected, isTrue);
      expect(opened, isFalse);
    },
  );

  test('library filtering and sorting are local and deterministic', () {
    final older = _document(
      id: 'older',
      fileName: 'zeta.pdf',
      title: 'Zeta',
      author: 'Ana',
      openedAt: DateTime.utc(2026, 7, 27),
      progress: 0.9,
    );
    final newer = _document(
      id: 'newer',
      fileName: 'alpha.pdf',
      title: 'Alpha',
      author: 'Luis',
      openedAt: DateTime.utc(2026, 7, 29),
      progress: 0.2,
    );

    expect(
      filterAndSortLibraryDocuments([older, newer]).map((item) => item.id),
      ['newer', 'older'],
    );
    expect(
      filterAndSortLibraryDocuments([older, newer], query: 'ana').single.id,
      'older',
    );
    expect(
      filterAndSortLibraryDocuments([
        older,
        newer,
      ], sortOrder: LibrarySortOrder.title).map((item) => item.id),
      ['newer', 'older'],
    );
    expect(
      filterAndSortLibraryDocuments([
        older,
        newer,
      ], sortOrder: LibrarySortOrder.progress).map((item) => item.id),
      ['older', 'newer'],
    );
  });

  test('reading-state filtering derives counts in one combined evaluation', () {
    final documents = [
      _document(
        id: 'new',
        fileName: 'new.pdf',
        title: 'Alfa',
        openedAt: DateTime.utc(2026, 7, 27),
      ),
      _document(
        id: 'active',
        fileName: 'active.pdf',
        title: 'Beta',
        openedAt: DateTime.utc(2026, 7, 28),
        progress: 0.45,
      ),
      _document(
        id: 'done',
        fileName: 'done.pdf',
        title: 'Gamma',
        openedAt: DateTime.utc(2026, 7, 29),
        progress: 1,
      ),
    ];

    final result = evaluateLibraryDocuments(
      documents,
      readingState: LibraryReadingState.inProgress,
      sortOrder: LibrarySortOrder.title,
    );
    expect(result.documents.map((item) => item.id), ['active']);
    expect(result.counts, {
      LibraryReadingState.all: 3,
      LibraryReadingState.inProgress: 1,
      LibraryReadingState.notStarted: 1,
      LibraryReadingState.completed: 1,
    });

    final textFiltered = evaluateLibraryDocuments(
      documents,
      query: 'gamma',
      readingState: LibraryReadingState.completed,
    );
    expect(textFiltered.documents.single.id, 'done');
    expect(textFiltered.counts[LibraryReadingState.all], 1);
    expect(textFiltered.counts[LibraryReadingState.completed], 1);
  });

  testWidgets(
    'Ctrl+F focuses the library filter and filtering is recoverable',
    (tester) async {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final repository = DocumentRepository(database);
      repository.save(
        _document(
          id: 'alpha',
          fileName: 'alpha.pdf',
          title: 'Alpha',
          author: 'Ana',
          openedAt: DateTime.utc(2026, 7, 29),
        ),
      );
      repository.save(
        _document(
          id: 'beta',
          fileName: 'beta.pdf',
          title: 'Beta',
          author: 'Luis',
          openedAt: DateTime.utc(2026, 7, 28),
        ),
      );
      final controller = LibraryController(
        repository,
        AppLogger(minimumLevel: LogLevel.error),
      );
      await controller.load();
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      for (final width in const [420.0, 680.0, 1100.0]) {
        tester.view.physicalSize = Size(width, 760);
        await tester.pumpWidget(
          MaterialApp(
            theme: AppTheme.light(),
            home: LibraryScreen(
              controller: controller,
              onOpenPath: (_) async {},
              onPickDocument: () async {},
              onOpenSettings: () {},
              showRecents: true,
            ),
          ),
        );
        await tester.pump();
        expect(tester.takeException(), isNull, reason: 'viewport width $width');
        expect(find.byKey(const Key('library-filter')), findsOneWidget);
      }

      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyF);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      final field = tester.widget<EditableText>(find.byType(EditableText));
      expect(field.focusNode.hasFocus, isTrue);

      await tester.enterText(find.byKey(const Key('library-filter')), 'ana');
      await tester.pump();
      expect(find.byKey(const ValueKey('recent-alpha')), findsOneWidget);
      expect(find.byKey(const ValueKey('recent-beta')), findsNothing);

      await tester.enterText(
        find.byKey(const Key('library-filter')),
        'sin coincidencias',
      );
      await tester.pump();
      expect(find.textContaining('No hay documentos'), findsOneWidget);
      await tester.tap(find.text('Limpiar filtro'));
      await tester.pump();
      expect(find.byKey(const ValueKey('recent-alpha')), findsOneWidget);
      expect(find.byKey(const ValueKey('recent-beta')), findsOneWidget);
    },
  );

  testWidgets(
    'reading-state selector is responsive, semantic and recoverable',
    (tester) async {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final repository = DocumentRepository(database)
        ..save(
          _document(
            id: 'new',
            fileName: 'new.pdf',
            openedAt: DateTime.utc(2026, 7, 27),
          ),
        )
        ..save(
          _document(
            id: 'active',
            fileName: 'active.pdf',
            openedAt: DateTime.utc(2026, 7, 28),
            progress: 0.5,
          ),
        )
        ..save(
          _document(
            id: 'done',
            fileName: 'done.pdf',
            openedAt: DateTime.utc(2026, 7, 29),
            progress: 1,
          ),
        );
      final controller = LibraryController(
        repository,
        AppLogger(minimumLevel: LogLevel.error),
      );
      await controller.load();
      controller.documents = repository
          .recent()
          .map((document) => document.copyWith(isMissing: false))
          .toList(growable: false);
      expect(controller.continueReading?.id, 'active');
      var selected = LibraryReadingState.all;
      tester.view.physicalSize = const Size(420, 760);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        MaterialApp(
          theme: AppTheme.light(),
          home: LibraryScreen(
            controller: controller,
            onOpenPath: (_) async {},
            onPickDocument: () async {},
            onOpenSettings: () {},
            showRecents: true,
            onReadingFilterChanged: (value) => selected = value,
          ),
        ),
      );
      await tester.pump();
      expect(tester.takeException(), isNull);
      expect(find.text('Todos 3'), findsOneWidget);
      expect(find.text('En curso 1'), findsOneWidget);

      await tester.tap(find.text('En curso 1'));
      await tester.pump();
      expect(selected, LibraryReadingState.inProgress);
      expect(find.byKey(const ValueKey('recent-active')), findsOneWidget);
      expect(find.byKey(const ValueKey('recent-new')), findsNothing);
      expect(find.byKey(const ValueKey('recent-done')), findsNothing);

      await tester.tap(find.text('Todos 3'));
      await tester.pump();
      expect(find.byKey(const ValueKey('recent-new')), findsOneWidget);
      expect(find.byKey(const ValueKey('recent-done')), findsOneWidget);
    },
  );

  testWidgets('thumbnail decoding is bounded by its physical display size', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetDevicePixelRatio);
    final document = _document(
      id: 'cover',
      fileName: 'cover.pdf',
      openedAt: DateTime.utc(2026, 7, 29),
      thumbnailPath: 'missing-cover.png',
    );

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: Scaffold(
          body: RecentDocumentItem(document: document, onOpen: () {}),
        ),
      ),
    );
    final image = tester.widget<Image>(find.byType(Image));
    final provider = image.image as ResizeImage;

    expect(provider.width, 88);
    expect(provider.height, 120);
    expect(image.filterQuality, FilterQuality.low);
    expect(find.byType(RepaintBoundary), findsWidgets);
  });
}

LibraryDocument _document({
  required String id,
  required String fileName,
  required DateTime openedAt,
  String? title,
  String? author,
  double progress = 0,
  String? thumbnailPath,
}) => LibraryDocument(
  id: id,
  path: fileName,
  normalizedPath: fileName.toLowerCase(),
  fileName: fileName,
  title: title,
  author: author,
  pageCount: 20,
  fileSize: 1024,
  fileModifiedAt: openedAt,
  addedAt: openedAt,
  lastOpenedAt: openedAt,
  readingProgress: progress,
  fingerprint: id,
  thumbnailPath: thumbnailPath,
);
