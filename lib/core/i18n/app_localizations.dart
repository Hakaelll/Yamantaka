import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

enum AppLanguage { spanish, english }

extension AppLanguageCode on AppLanguage {
  String get code => switch (this) {
    AppLanguage.spanish => 'es',
    AppLanguage.english => 'en',
  };

  static AppLanguage fromCode(String? code) =>
      code?.toLowerCase() == 'en' ? AppLanguage.english : AppLanguage.spanish;
}

/// Small typed catalog. Feature-specific copy can be added without allowing
/// a missing key to leak into a visible surface.
final class YamantakaLocalizations {
  const YamantakaLocalizations(this.language);

  final AppLanguage language;

  static const LocalizationsDelegate<YamantakaLocalizations> delegate =
      _YamantakaLocalizationsDelegate();

  static YamantakaLocalizations of(BuildContext context) =>
      Localizations.of<YamantakaLocalizations>(
        context,
        YamantakaLocalizations,
      ) ??
      const YamantakaLocalizations(AppLanguage.spanish);

  String get(String key, {Map<String, Object?> parameters = const {}}) {
    final value =
        _catalog[language]?[key] ?? _catalog[AppLanguage.spanish]![key] ?? key;
    return value.replaceAllMapped(RegExp(r'\{(\w+)\}'), (match) {
      final parameter = parameters[match.group(1)];
      return parameter?.toString() ?? match.group(0)!;
    });
  }

  String count(
    String key,
    int value, {
    Map<String, Object?> parameters = const {},
  }) => get(
    value == 1 ? '$key.one' : '$key.other',
    parameters: {'count': value, ...parameters},
  );

  static const Map<AppLanguage, Map<String, String>> _catalog = {
    AppLanguage.spanish: {
      'app.title': 'Yamantaka',
      'action.open': 'Abrir',
      'action.import': 'Importar',
      'action.cancel': 'Cancelar',
      'action.retry': 'Reintentar',
      'action.close': 'Cerrar',
      'action.save': 'Guardar',
      'action.reset': 'Restablecer',
      'action.clear': 'Limpiar',
      'library.title': 'Biblioteca',
      'library.empty': 'Aún no hay documentos locales.',
      'library.noPdfsDeleted': 'Los PDF no se borran desde esta acción.',
      'reader.title': 'Lector',
      'reader.pageIndicator': '{page} de {count}',
      'settings.title': 'Ajustes',
      'settings.language': 'Idioma',
      'settings.language.spanish': 'Castellano',
      'settings.language.english': 'English',
      'privacy.title': 'Privacidad y almacenamiento',
      'privacy.localOnly': 'Los datos permanecen en este dispositivo.',
      'backup.title': 'Copia local',
      'help.title': 'Ayuda',
      'help.search': 'Buscar en la ayuda',
      'help.noResults': 'No hay temas que coincidan.',
      'welcome.title': 'Empieza con tu biblioteca local',
      'welcome.body':
          'Abre un PDF, conserva tu posición y mantén tus datos en este '
          'dispositivo. No hay cuentas ni copias automáticas en la nube.',
      'welcome.open': 'Abrir un PDF',
      'welcome.help': 'Ver ayuda',
      'count.documents.one': '{count} documento',
      'count.documents.other': '{count} documentos',
    },
    AppLanguage.english: {
      'app.title': 'Yamantaka',
      'action.open': 'Open',
      'action.import': 'Import',
      'action.cancel': 'Cancel',
      'action.retry': 'Retry',
      'action.close': 'Close',
      'action.save': 'Save',
      'action.reset': 'Reset',
      'action.clear': 'Clear',
      'library.title': 'Library',
      'library.empty': 'There are no local documents yet.',
      'library.noPdfsDeleted': 'PDFs are not deleted by this action.',
      'reader.title': 'Reader',
      'reader.pageIndicator': '{page} of {count}',
      'settings.title': 'Settings',
      'settings.language': 'Language',
      'settings.language.spanish': 'Castilian Spanish',
      'settings.language.english': 'English',
      'privacy.title': 'Privacy and storage',
      'privacy.localOnly': 'Data stays on this device.',
      'backup.title': 'Local backup',
      'help.title': 'Help',
      'help.search': 'Search help',
      'help.noResults': 'No topics match your search.',
      'welcome.title': 'Start with your local library',
      'welcome.body':
          'Open a PDF, keep your position, and keep your data on this device. '
          'There are no accounts or automatic cloud copies.',
      'welcome.open': 'Open a PDF',
      'welcome.help': 'View help',
      'count.documents.one': '{count} document',
      'count.documents.other': '{count} documents',
    },
  };
}

final class _YamantakaLocalizationsDelegate
    extends LocalizationsDelegate<YamantakaLocalizations> {
  const _YamantakaLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) =>
      const ['es', 'en'].contains(locale.languageCode);

  @override
  Future<YamantakaLocalizations> load(Locale locale) => SynchronousFuture(
    YamantakaLocalizations(AppLanguageCode.fromCode(locale.languageCode)),
  );

  @override
  bool shouldReload(_YamantakaLocalizationsDelegate old) => false;
}
