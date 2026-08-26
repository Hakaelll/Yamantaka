import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main() {
  testWidgets('empty library opens in Spanish and English', (tester) async {
    for (final language in AppLanguage.values) {
      await tester.pumpWidget(
        MaterialApp(
          locale: Locale(language.code),
          supportedLocales: const [Locale('es'), Locale('en')],
          localizationsDelegates: const [
            YamantakaLocalizations.delegate,
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          home: EmptyLibraryState(onOpen: () {}),
        ),
      );
      await tester.pump();

      expect(
        find.text(
          language == AppLanguage.spanish
              ? 'Tu archivo privado de lectura.'
              : 'Your private reading archive.',
        ),
        findsOneWidget,
      );
    }
  });

  testWidgets('empty library presents one clear opening action', (
    tester,
  ) async {
    var opened = false;
    await tester.pumpWidget(
      MaterialApp(home: EmptyLibraryState(onOpen: () => opened = true)),
    );

    expect(find.text('Yamantaka'), findsOneWidget);
    expect(find.text('Tu archivo privado de lectura.'), findsOneWidget);
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
        expect(
          tester.widget(find.byKey(const Key('library-open-button'))),
          isA<FilledButton>(),
        );
        expect(find.byKey(const Key('library-filter')), findsNothing);
        if (width == 1100) {
          final alpha = tester.getCenter(
            find.byKey(const ValueKey('artifact-alpha')),
          );
          final beta = tester.getCenter(
            find.byKey(const ValueKey('artifact-beta')),
          );
          expect((alpha.dx + beta.dx) / 2, closeTo(width / 2, 4));
        }
      }

      controller.selectDocument(null);
      await tester.pump();
      await tester.tap(find.byKey(const ValueKey('artifact-alpha')));
      await tester.pump(const Duration(milliseconds: 400));
      expect(controller.selectedDocumentId, 'alpha');
      expect(find.text('Alpha'), findsWidgets);
      expect(find.byKey(const Key('library-context-primary')), findsOneWidget);
      expect(find.text('Abrir'), findsOneWidget);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyF);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      expect(find.byKey(const Key('library-filter')), findsOneWidget);
      final field = tester.widget<EditableText>(find.byType(EditableText));
      expect(field.focusNode.hasFocus, isTrue);

      await tester.enterText(find.byKey(const Key('library-filter')), 'ana');
      await tester.pump(const Duration(milliseconds: 220));
      await tester.pump();
      expect(find.byKey(const ValueKey('artifact-alpha')), findsOneWidget);
      expect(find.byKey(const ValueKey('artifact-beta')), findsNothing);

      await tester.enterText(
        find.byKey(const Key('library-filter')),
        'sin coincidencias',
      );
      await tester.pump(const Duration(milliseconds: 220));
      await tester.pump();
      expect(find.textContaining('No hay documentos'), findsOneWidget);
      expect(
        find.text(
          'El documento seleccionado ya no pertenece a este resultado.',
        ),
        findsOneWidget,
      );
      expect(controller.selectedDocumentId, 'alpha');
      await tester.tap(find.text('Restablecer'));
      await tester.pump();
      await tester.pump();
      expect(find.byKey(const ValueKey('artifact-alpha')), findsOneWidget);
      expect(find.byKey(const ValueKey('artifact-beta')), findsOneWidget);
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
      await tester.tap(find.byKey(const Key('library-filter-button')));
      await tester.pumpAndSettle();
      expect(find.text('Todos · 3'), findsOneWidget);
      expect(find.text('En curso · 1'), findsOneWidget);

      await tester.tap(find.text('En curso · 1'), warnIfMissed: false);
      await tester.pumpAndSettle();
      expect(selected, LibraryReadingState.inProgress);
      expect(find.byKey(const ValueKey('artifact-active')), findsOneWidget);
      expect(find.byKey(const ValueKey('artifact-new')), findsNothing);
      expect(find.byKey(const ValueKey('artifact-done')), findsNothing);
      expect(
        find.byKey(const Key('library-reset-filters-button')),
        findsOneWidget,
      );

      await tester.tap(find.byKey(const Key('library-reset-filters-button')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('artifact-new')), findsOneWidget);
      expect(find.byKey(const ValueKey('artifact-done')), findsOneWidget);
    },
  );

  testWidgets('grid and dense list are distinct, keyboard-safe archive views', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = DocumentRepository(database)
      ..save(
        _document(
          id: 'alpha',
          fileName: 'alpha.pdf',
          title: 'Alpha',
          author: 'Ana',
          openedAt: DateTime.utc(2026, 7, 29),
          progress: 0.35,
        ),
      )
      ..save(
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
    tester.view.physicalSize = const Size(1280, 760);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(
            context,
          ).copyWith(textScaler: const TextScaler.linear(1.5)),
          child: child!,
        ),
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
    expect(find.byKey(const ValueKey('artifact-alpha')), findsOneWidget);
    controller.scrollOffset = 42;

    await tester.tap(find.byKey(const Key('library-list-view-button')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('artifact-alpha')), findsNothing);
    expect(find.byKey(const ValueKey('artifact-list-alpha')), findsOneWidget);
    expect(find.text('Documento'), findsOneWidget);
    expect(find.text('Última apertura'), findsNothing);
    expect(find.textContaining('En curso · 35 %'), findsOneWidget);
    expect(controller.scrollOffset, 42);
    expect(tester.takeException(), isNull);

    await tester.tap(find.text('Alpha').first, warnIfMissed: false);
    await tester.pump();
    expect(controller.selectedDocumentId, 'alpha');

    await tester.tap(find.byKey(const Key('library-grid-view-button')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('artifact-alpha')), findsOneWidget);
    expect(find.byKey(const ValueKey('artifact-list-alpha')), findsNothing);
  });

  testWidgets('active library filters are visible and removable by group', (
    tester,
  ) async {
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
      );
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    await controller.load();
    tester.view.physicalSize = const Size(1100, 720);
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
        ),
      ),
    );
    await tester.tap(find.byKey(const Key('library-filter-button')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('En curso · 1'), warnIfMissed: false);
    await tester.pumpAndSettle();

    final activeChip = find.widgetWithText(InputChip, 'En curso');
    expect(find.text('Filtros activos'), findsOneWidget);
    expect(activeChip, findsOneWidget);
    tester.widget<InputChip>(activeChip).onDeleted!();
    await tester.pumpAndSettle();

    expect(find.byKey(const ValueKey('artifact-active')), findsOneWidget);
    expect(find.byKey(const ValueKey('artifact-new')), findsOneWidget);
    expect(find.text('Filtros activos'), findsNothing);
  });

  testWidgets('library selection mode supports multiple accessible items', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = DocumentRepository(database)
      ..save(
        _document(
          id: 'alpha',
          fileName: 'alpha.pdf',
          openedAt: DateTime.utc(2026, 7, 29),
        ),
      )
      ..save(
        _document(
          id: 'beta',
          fileName: 'beta.pdf',
          openedAt: DateTime.utc(2026, 7, 28),
        ),
      );
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    await controller.load();
    tester.view.physicalSize = const Size(1100, 760);
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
        ),
      ),
    );
    await tester.pump();
    await tester.tap(find.byKey(const Key('library-selection-mode-button')));
    await tester.tap(find.byKey(const ValueKey('artifact-alpha')));
    await tester.tap(find.byKey(const ValueKey('artifact-beta')));
    await tester.pump(const Duration(milliseconds: 50));

    expect(find.text('2 seleccionados'), findsWidgets);
    expect(find.byIcon(Icons.check_rounded), findsNWidgets(2));

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(find.text('2 seleccionados'), findsNothing);
  });

  testWidgets(
    'artifact context actions stay keyboard reachable and adapt to missing PDFs',
    (tester) async {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final repository = DocumentRepository(database);
      final document = _document(
        id: 'actions',
        fileName: 'actions.pdf',
        openedAt: DateTime.utc(2026, 7, 29),
        progress: 0.4,
      );
      repository.save(document);
      final controller = LibraryController(
        repository,
        AppLogger(minimumLevel: LogLevel.error),
      );
      await controller.load();
      controller.documents = [document];
      var coverChanged = false;
      var relocated = false;
      tester.view.physicalSize = const Size(1100, 760);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      Widget app() => MaterialApp(
        theme: AppTheme.light(),
        home: LibraryScreen(
          controller: controller,
          onOpenPath: (_) async {},
          onPickDocument: () async {},
          onOpenSettings: () {},
          showRecents: true,
          onRelocateDocument: (_) async => relocated = true,
          onChangeCover: (_) async => coverChanged = true,
          onDocumentInfo: (_) async {},
          onToggleFavorite: (_) async {},
          onRevealDocument: (_) async {},
          onRemoveDocument: (_) async {},
        ),
      );

      await tester.pumpWidget(app());
      await tester.pump();
      expect(find.byKey(const ValueKey('artifact-actions')), findsOneWidget);
      expect(find.text('Tu espacio de lectura'), findsNothing);
      expect(find.text('Abrir documento'), findsNothing);
      expect(
        tester.getCenter(find.byKey(const ValueKey('artifact-actions'))).dx,
        closeTo(550, 2),
      );
      await tester.tap(find.byKey(const ValueKey('artifact-actions')));
      await tester.pump();
      expect(find.byKey(const Key('library-context-primary')), findsOneWidget);
      expect(find.byKey(const Key('library-context-favorite')), findsOneWidget);
      expect(find.text('Continuar'), findsOneWidget);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.f10);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));

      final visibleLabels = tester
          .widgetList<Text>(find.byType(Text))
          .map((text) => text.data)
          .whereType<String>()
          .toSet();
      expect(
        visibleLabels,
        containsAll(const {
          'Continuar leyendo',
          'Cambiar portada',
          'Mostrar información',
          'Mostrar en el explorador',
          'Retirar de la biblioteca',
        }),
      );
      expect(find.text('1 seleccionados'), findsNothing);
      await tester.tap(
        find.ancestor(
          of: find.text('Cambiar portada'),
          matching: find.byWidgetPredicate((widget) => widget is PopupMenuItem),
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(coverChanged, isTrue);

      controller.documents = [document.copyWith(isMissing: true)];
      await tester.pumpWidget(app());
      await tester.pump();
      await tester.tap(find.byKey(const ValueKey('artifact-actions')));
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.contextMenu);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));

      final locateAction = find.ancestor(
        of: find.text('Localizar archivo'),
        matching: find.byWidgetPredicate((widget) => widget is PopupMenuItem),
      );
      expect(locateAction, findsOneWidget);
      expect(find.text('Cambiar portada'), findsNothing);
      expect(find.text('Mostrar en el explorador'), findsNothing);
      await tester.tap(locateAction);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(relocated, isTrue);
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
