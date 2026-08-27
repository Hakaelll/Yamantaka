import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/search/application/global_search_controller.dart';
import 'package:yamantaka/features/search/application/library_index_controller.dart';
import 'package:yamantaka/features/search/domain/global_search_models.dart';
import 'package:yamantaka/features/search/presentation/global_search_dialog.dart';
import 'package:yamantaka/features/settings/presentation/settings_screen.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('global search switches between Spanish and English', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final documents = DocumentRepository(database);
    final repository = GlobalSearchRepository(database);
    final search = GlobalLibrarySearchController(repository);
    final engine = StubPdfDocumentEngine();
    final index = LibraryIndexController(documents, repository, engine);
    addTearDown(() async {
      search.dispose();
      index.dispose();
      await engine.dispose();
      database.close();
    });

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
          home: GlobalSearchDialog(
            controller: search,
            indexController: index,
            onOpen: (_) {},
          ),
        ),
      );
      await tester.pump();

      expect(
        find.text(
          language == AppLanguage.spanish
              ? 'Buscar en la biblioteca'
              : 'Search library',
        ),
        findsOneWidget,
      );
      expect(
        find.text(
          language == AppLanguage.spanish
              ? 'Documentos, contenido, notas y etiquetas'
              : 'Documents, content, notes, and tags',
        ),
        findsOneWidget,
      );
    }
  });

  testWidgets('settings switches between Spanish and English', (tester) async {
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
          home: Scaffold(appBar: SettingsAppBar(onBack: () {})),
        ),
      );
      await tester.pump();

      expect(
        find.text(language == AppLanguage.spanish ? 'Ajustes' : 'Settings'),
        findsOneWidget,
      );
      expect(
        find.byTooltip(language == AppLanguage.spanish ? 'Volver' : 'Back'),
        findsOneWidget,
      );
    }
  });

  testWidgets('expanded global search relates its list to a detail preview', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final documents = DocumentRepository(database);
    final repository = GlobalSearchRepository(database);
    final search = GlobalLibrarySearchController(repository);
    final engine = StubPdfDocumentEngine();
    final index = LibraryIndexController(documents, repository, engine);
    search
      ..query = 'privada'
      ..results = const [
        GlobalSearchResult(
          documentId: 'precision',
          pageIndex: 2,
          kind: GlobalSearchKind.pageText,
          title: 'Lectura de precisión',
          snippet: 'Un fragmento indexado de forma privada en este equipo.',
          rank: 1,
        ),
      ];
    addTearDown(() async {
      search.dispose();
      index.dispose();
      await engine.dispose();
      database.close();
    });
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(
      MaterialApp(
        home: GlobalSearchDialog(
          controller: search,
          indexController: index,
          onOpen: (_) {},
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Lectura de precisión'), findsNWidgets(2));
    expect(find.text('Abrir resultado'), findsOneWidget);
    expect(find.text('Texto de página'), findsWidgets);
    expect(tester.takeException(), isNull);
  });

  testWidgets('compact global search opens detail and returns to its list', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final documents = DocumentRepository(database);
    final repository = GlobalSearchRepository(database);
    final search = GlobalLibrarySearchController(repository)
      ..query = 'privada'
      ..results = const [
        GlobalSearchResult(
          documentId: 'compact-result',
          pageIndex: 2,
          kind: GlobalSearchKind.pageText,
          title: 'Lectura compacta',
          snippet: 'Un fragmento local con contexto suficiente.',
          rank: 1,
        ),
      ];
    final engine = StubPdfDocumentEngine();
    final index = LibraryIndexController(documents, repository, engine);
    var opened = false;
    addTearDown(() async {
      search.dispose();
      index.dispose();
      await engine.dispose();
      database.close();
    });
    tester.view.physicalSize = const Size(640, 480);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(
      MaterialApp(
        home: GlobalSearchDialog(
          controller: search,
          indexController: index,
          onOpen: (_) => opened = true,
        ),
      ),
    );
    await tester.pump();

    await tester.tap(find.text('Lectura compacta'));
    await tester.pump();
    expect(
      find.byKey(const Key('global-search-compact-detail')),
      findsOneWidget,
    );
    expect(find.text('Volver a resultados'), findsOneWidget);
    expect(opened, isFalse);

    await tester.tap(find.byKey(const Key('global-search-back-to-results')));
    await tester.pump();
    expect(find.byKey(const Key('global-search-compact-detail')), findsNothing);
    expect(find.text('Lectura compacta'), findsOneWidget);

    await tester.tap(find.text('Lectura compacta'));
    await tester.pump();
    expect(
      find.byKey(const Key('global-search-compact-detail')),
      findsOneWidget,
    );
    await tester.binding.handlePopRoute();
    await tester.pump();
    expect(find.byKey(const Key('global-search-compact-detail')), findsNothing);
    expect(find.byType(GlobalSearchDialog), findsOneWidget);
  });

  testWidgets('global search keyboard opens the selected result in a new tab', (
    tester,
  ) async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final documents = DocumentRepository(database);
    final repository = GlobalSearchRepository(database);
    final search = GlobalLibrarySearchController(repository)
      ..query = 'atlas'
      ..results = const [
        GlobalSearchResult(
          documentId: 'first',
          pageIndex: 1,
          kind: GlobalSearchKind.pageText,
          title: 'Atlas primero',
          snippet: 'Primer resultado',
          rank: 1,
        ),
        GlobalSearchResult(
          documentId: 'second',
          pageIndex: 8,
          kind: GlobalSearchKind.annotation,
          title: 'Atlas segundo',
          snippet: 'Segundo resultado',
          rank: 1,
        ),
      ];
    final engine = StubPdfDocumentEngine();
    final index = LibraryIndexController(documents, repository, engine);
    GlobalSearchResult? openedInNewTab;
    addTearDown(() async {
      search.dispose();
      index.dispose();
      await engine.dispose();
      database.close();
    });

    await tester.pumpWidget(
      MaterialApp(
        home: GlobalSearchDialog(
          controller: search,
          indexController: index,
          onOpen: (_) => fail('Enter with Control must use the new-tab path'),
          onOpenInNewTab: (result) => openedInNewTab = result,
        ),
      ),
    );
    await tester.pump();
    await tester.tap(find.byKey(const Key('global-search-field')));
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pump();

    expect(openedInNewTab?.documentId, 'second');
    expect(openedInNewTab?.pageIndex, 8);
  });
}
