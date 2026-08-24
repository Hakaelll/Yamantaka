import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/shortcuts/app_commands.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/errors/failure_presentation.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';

void main() {
  test('catalog has Spanish and English fallbacks with parameters', () {
    const spanish = YamantakaLocalizations(AppLanguage.spanish);
    const english = YamantakaLocalizations(AppLanguage.english);
    expect(
      spanish.get('reader.pageIndicator', parameters: {'page': 2, 'count': 8}),
      '2 de 8',
    );
    expect(
      english.get('reader.pageIndicator', parameters: {'page': 2, 'count': 8}),
      '2 of 8',
    );
    expect(english.get('missing.key'), 'missing.key');
    expect(english.count('count.documents', 2), '2 documents');
  });

  test('Spanish and English catalogs have complete key parity', () {
    final spanishKeys = YamantakaLocalizations.catalogKeys(AppLanguage.spanish);
    final englishKeys = YamantakaLocalizations.catalogKeys(AppLanguage.english);

    expect(englishKeys.difference(spanishKeys), isEmpty);
    expect(spanishKeys.difference(englishKeys), isEmpty);
    expect(spanishKeys, isNotEmpty);
    const spanish = YamantakaLocalizations(AppLanguage.spanish);
    const english = YamantakaLocalizations(AppLanguage.english);
    for (final key in spanishKeys) {
      expect(spanish.get(key), isNot(key), reason: 'Spanish key $key');
      expect(english.get(key), isNot(key), reason: 'English key $key');
    }
  });

  test('typed failures open with localized private-safe copy', () {
    const english = YamantakaLocalizations(AppLanguage.english);
    final presentation = FailurePresentations.forFailure(
      const AppFailure(FailureKind.fileNotFound, ''),
      strings: english,
    );

    expect(presentation.title, 'The file is no longer available');
    expect(presentation.message, isNot(contains(r'\\')));
    expect(presentation.action, FailureAction.chooseAnotherFile);
  });

  test('shortcut names are localized without changing their tokens', () {
    const spanish = YamantakaLocalizations(AppLanguage.spanish);
    const english = YamantakaLocalizations(AppLanguage.english);
    final shiftChoice = AppCommandRegistry.shortcutChoices.firstWhere(
      (choice) => choice.token == 'ctrl-shift-f',
    );
    final pageChoice = AppCommandRegistry.shortcutChoices.firstWhere(
      (choice) => choice.token == 'page-down',
    );

    expect(
      AppCommandRegistry.shortcutChoiceLabel(shiftChoice, strings: spanish),
      'Ctrl+Mayús+F',
    );
    expect(
      AppCommandRegistry.shortcutChoiceLabel(shiftChoice, strings: english),
      'Ctrl+Shift+F',
    );
    expect(
      AppCommandRegistry.shortcutChoiceLabel(pageChoice, strings: english),
      'Page Down',
    );
  });
}
