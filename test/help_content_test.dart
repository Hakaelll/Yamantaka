import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/features/help/domain/help_content.dart';

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
}
