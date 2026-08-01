import 'package:flutter_test/flutter_test.dart';
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
}
