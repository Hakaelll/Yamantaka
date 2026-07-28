import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

void main() {
  test(
    'global settings persist privacy, profile, language and window bounds',
    () async {
      final repository = AppSettingsRepository(_MemoryPreferences());
      const expected = AppSettings(
        darkMode: true,
        defaultReaderTheme: 'sepia',
        autoHideControls: false,
        showProgress: false,
        reduceMotion: true,
        disableAnimations: true,
        defaultViewMode: 'book',
        defaultPageLayout: 'facing',
        defaultZoomMode: 'fitWidth',
        readingDirection: ReadingDirection.rightToLeft,
        visualProfile: ReaderVisualProfile.electronicInk,
        defaultMargin: 48,
        restoreLastPosition: false,
        showRecents: false,
        generateCovers: false,
        maxCacheMegabytes: 128,
        recordReadingSessions: false,
        languageCode: 'es',
        windowWidth: 1360,
        windowHeight: 820,
        windowX: -320,
        windowY: 72,
      );

      await repository.save(expected);
      final actual = await repository.load();

      expect(actual.darkMode, isTrue);
      expect(actual.defaultReaderTheme, 'sepia');
      expect(actual.readingDirection, ReadingDirection.rightToLeft);
      expect(actual.visualProfile, ReaderVisualProfile.electronicInk);
      expect(actual.recordReadingSessions, isFalse);
      expect(actual.languageCode, 'es');
      expect(actual.windowWidth, 1360);
      expect(actual.windowHeight, 820);
      expect(actual.windowX, -320);
      expect(actual.windowY, 72);
    },
  );

  test('global settings clear back to private local defaults', () async {
    final repository = AppSettingsRepository(_MemoryPreferences());
    await repository.save(
      const AppSettings(darkMode: true, recordReadingSessions: false),
    );

    await repository.clear();
    final actual = await repository.load();

    expect(actual.darkMode, isFalse);
    expect(actual.recordReadingSessions, isTrue);
    expect(actual.languageCode, 'es');
    expect(actual.windowWidth, 1120);
    expect(actual.windowHeight, 760);
  });

  test('unknown persisted enum names fall back to safe defaults', () async {
    final preferences = _MemoryPreferences()
      ..values['readingDirection'] = 'sideways'
      ..values['visualProfile'] = 'crt';
    final actual = await AppSettingsRepository(preferences).load();

    expect(actual.readingDirection, ReadingDirection.leftToRight);
    expect(actual.visualProfile, ReaderVisualProfile.standard);
  });
}

final class _MemoryPreferences implements AppPreferencesStore {
  final Map<String, Object> values = {};

  @override
  Future<void> clear() async => values.clear();

  @override
  Future<bool?> getBool(String key) async => values[key] as bool?;

  @override
  Future<double?> getDouble(String key) async => values[key] as double?;

  @override
  Future<int?> getInt(String key) async => values[key] as int?;

  @override
  Future<String?> getString(String key) async => values[key] as String?;

  @override
  Future<void> setBool(String key, bool value) async => values[key] = value;

  @override
  Future<void> setDouble(String key, double value) async => values[key] = value;

  @override
  Future<void> setInt(String key, int value) async => values[key] = value;

  @override
  Future<void> setString(String key, String value) async => values[key] = value;
}
