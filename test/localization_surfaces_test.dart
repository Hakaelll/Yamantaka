import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/search/application/global_search_controller.dart';
import 'package:yamantaka/features/search/application/library_index_controller.dart';
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
}
