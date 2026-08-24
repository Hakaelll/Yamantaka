import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/features/help/domain/help_content.dart';
import 'package:yamantaka/features/help/presentation/help_center_dialog.dart';

void main() {
  test(
    'help is local, searchable, and complete in both supported languages',
    () {
      expect(HelpContent.articles(AppLanguage.spanish), hasLength(7));
      expect(HelpContent.articles(AppLanguage.english), hasLength(7));
      expect(
        HelpContent.search(AppLanguage.english, 'backup').single.topic,
        HelpTopic.backup,
      );
    },
  );

  testWidgets('help center opens in Spanish and English', (tester) async {
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
          home: HelpCenterDialog(language: language),
        ),
      );
      await tester.pump();

      expect(
        find.text(language == AppLanguage.spanish ? 'Ayuda' : 'Help'),
        findsOneWidget,
      );
    }
  });
}
