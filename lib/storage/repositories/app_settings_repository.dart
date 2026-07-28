import 'package:shared_preferences/shared_preferences.dart';

import '../../features/reader/domain/reader_models.dart';

abstract interface class AppPreferencesStore {
  Future<bool?> getBool(String key);
  Future<String?> getString(String key);
  Future<double?> getDouble(String key);
  Future<int?> getInt(String key);
  Future<void> setBool(String key, bool value);
  Future<void> setString(String key, String value);
  Future<void> setDouble(String key, double value);
  Future<void> setInt(String key, int value);
  Future<void> clear();
}

final class SharedPreferencesStore implements AppPreferencesStore {
  SharedPreferencesStore([SharedPreferencesAsync? preferences])
    : _preferences = preferences ?? SharedPreferencesAsync();

  final SharedPreferencesAsync _preferences;

  @override
  Future<bool?> getBool(String key) => _preferences.getBool(key);

  @override
  Future<String?> getString(String key) => _preferences.getString(key);

  @override
  Future<double?> getDouble(String key) => _preferences.getDouble(key);

  @override
  Future<int?> getInt(String key) => _preferences.getInt(key);

  @override
  Future<void> setBool(String key, bool value) =>
      _preferences.setBool(key, value);

  @override
  Future<void> setString(String key, String value) =>
      _preferences.setString(key, value);

  @override
  Future<void> setDouble(String key, double value) =>
      _preferences.setDouble(key, value);

  @override
  Future<void> setInt(String key, int value) => _preferences.setInt(key, value);

  @override
  Future<void> clear() => _preferences.clear();
}

final class AppSettings {
  const AppSettings({
    this.darkMode = false,
    this.defaultReaderTheme = 'ivory',
    this.autoHideControls = true,
    this.showProgress = true,
    this.reduceMotion = false,
    this.disableAnimations = false,
    this.defaultViewMode = 'document',
    this.defaultPageLayout = 'continuous',
    this.defaultZoomMode = 'fitPage',
    this.readingDirection = ReadingDirection.leftToRight,
    this.visualProfile = ReaderVisualProfile.standard,
    this.defaultMargin = 32,
    this.restoreLastPosition = true,
    this.showRecents = true,
    this.generateCovers = true,
    this.maxCacheMegabytes = 256,
    this.recordReadingSessions = true,
    this.languageCode = 'es',
    this.windowWidth = 1120,
    this.windowHeight = 760,
    this.windowX,
    this.windowY,
  });

  final bool darkMode;
  final String defaultReaderTheme;
  final bool autoHideControls;
  final bool showProgress;
  final bool reduceMotion;
  final bool disableAnimations;
  final String defaultViewMode;
  final String defaultPageLayout;
  final String defaultZoomMode;
  final ReadingDirection readingDirection;
  final ReaderVisualProfile visualProfile;
  final double defaultMargin;
  final bool restoreLastPosition;
  final bool showRecents;
  final bool generateCovers;
  final int maxCacheMegabytes;
  final bool recordReadingSessions;
  final String languageCode;
  final double windowWidth;
  final double windowHeight;
  final double? windowX;
  final double? windowY;

  AppSettings copyWith({
    bool? darkMode,
    String? defaultReaderTheme,
    bool? autoHideControls,
    bool? showProgress,
    bool? reduceMotion,
    bool? disableAnimations,
    String? defaultViewMode,
    String? defaultPageLayout,
    String? defaultZoomMode,
    ReadingDirection? readingDirection,
    ReaderVisualProfile? visualProfile,
    double? defaultMargin,
    bool? restoreLastPosition,
    bool? showRecents,
    bool? generateCovers,
    int? maxCacheMegabytes,
    bool? recordReadingSessions,
    String? languageCode,
    double? windowWidth,
    double? windowHeight,
    double? windowX,
    double? windowY,
  }) => AppSettings(
    darkMode: darkMode ?? this.darkMode,
    defaultReaderTheme: defaultReaderTheme ?? this.defaultReaderTheme,
    autoHideControls: autoHideControls ?? this.autoHideControls,
    showProgress: showProgress ?? this.showProgress,
    reduceMotion: reduceMotion ?? this.reduceMotion,
    disableAnimations: disableAnimations ?? this.disableAnimations,
    defaultViewMode: defaultViewMode ?? this.defaultViewMode,
    defaultPageLayout: defaultPageLayout ?? this.defaultPageLayout,
    defaultZoomMode: defaultZoomMode ?? this.defaultZoomMode,
    readingDirection: readingDirection ?? this.readingDirection,
    visualProfile: visualProfile ?? this.visualProfile,
    defaultMargin: defaultMargin ?? this.defaultMargin,
    restoreLastPosition: restoreLastPosition ?? this.restoreLastPosition,
    showRecents: showRecents ?? this.showRecents,
    generateCovers: generateCovers ?? this.generateCovers,
    maxCacheMegabytes: maxCacheMegabytes ?? this.maxCacheMegabytes,
    recordReadingSessions: recordReadingSessions ?? this.recordReadingSessions,
    languageCode: languageCode ?? this.languageCode,
    windowWidth: windowWidth ?? this.windowWidth,
    windowHeight: windowHeight ?? this.windowHeight,
    windowX: windowX ?? this.windowX,
    windowY: windowY ?? this.windowY,
  );
}

final class AppSettingsRepository {
  AppSettingsRepository([AppPreferencesStore? preferences])
    : _preferences = preferences ?? SharedPreferencesStore();

  final AppPreferencesStore _preferences;

  Future<AppSettings> load() async => AppSettings(
    darkMode: await _preferences.getBool('darkMode') ?? false,
    defaultReaderTheme:
        await _preferences.getString('defaultReaderTheme') ?? 'ivory',
    autoHideControls: await _preferences.getBool('autoHideControls') ?? true,
    showProgress: await _preferences.getBool('showProgress') ?? true,
    reduceMotion: await _preferences.getBool('reduceMotion') ?? false,
    disableAnimations: await _preferences.getBool('disableAnimations') ?? false,
    defaultViewMode:
        await _preferences.getString('defaultViewMode') ?? 'document',
    defaultPageLayout:
        await _preferences.getString('defaultPageLayout') ?? 'continuous',
    defaultZoomMode:
        await _preferences.getString('defaultZoomMode') ?? 'fitPage',
    readingDirection: _enumByName(
      ReadingDirection.values,
      await _preferences.getString('readingDirection'),
      ReadingDirection.leftToRight,
    ),
    visualProfile: _enumByName(
      ReaderVisualProfile.values,
      await _preferences.getString('visualProfile'),
      ReaderVisualProfile.standard,
    ),
    defaultMargin: await _preferences.getDouble('defaultMargin') ?? 32,
    restoreLastPosition:
        await _preferences.getBool('restoreLastPosition') ?? true,
    showRecents: await _preferences.getBool('showRecents') ?? true,
    generateCovers: await _preferences.getBool('generateCovers') ?? true,
    maxCacheMegabytes: await _preferences.getInt('maxCacheMegabytes') ?? 256,
    recordReadingSessions:
        await _preferences.getBool('recordReadingSessions') ?? true,
    languageCode: await _preferences.getString('languageCode') ?? 'es',
    windowWidth: await _preferences.getDouble('windowWidth') ?? 1120,
    windowHeight: await _preferences.getDouble('windowHeight') ?? 760,
    windowX: await _preferences.getDouble('windowX'),
    windowY: await _preferences.getDouble('windowY'),
  );

  Future<void> save(AppSettings settings) async {
    await Future.wait([
      _preferences.setBool('darkMode', settings.darkMode),
      _preferences.setString('defaultReaderTheme', settings.defaultReaderTheme),
      _preferences.setBool('autoHideControls', settings.autoHideControls),
      _preferences.setBool('showProgress', settings.showProgress),
      _preferences.setBool('reduceMotion', settings.reduceMotion),
      _preferences.setBool('disableAnimations', settings.disableAnimations),
      _preferences.setString('defaultViewMode', settings.defaultViewMode),
      _preferences.setString('defaultPageLayout', settings.defaultPageLayout),
      _preferences.setString('defaultZoomMode', settings.defaultZoomMode),
      _preferences.setString(
        'readingDirection',
        settings.readingDirection.name,
      ),
      _preferences.setString('visualProfile', settings.visualProfile.name),
      _preferences.setDouble('defaultMargin', settings.defaultMargin),
      _preferences.setBool('restoreLastPosition', settings.restoreLastPosition),
      _preferences.setBool('showRecents', settings.showRecents),
      _preferences.setBool('generateCovers', settings.generateCovers),
      _preferences.setInt('maxCacheMegabytes', settings.maxCacheMegabytes),
      _preferences.setBool(
        'recordReadingSessions',
        settings.recordReadingSessions,
      ),
      _preferences.setString('languageCode', settings.languageCode),
      _preferences.setDouble('windowWidth', settings.windowWidth),
      _preferences.setDouble('windowHeight', settings.windowHeight),
      if (settings.windowX != null)
        _preferences.setDouble('windowX', settings.windowX!),
      if (settings.windowY != null)
        _preferences.setDouble('windowY', settings.windowY!),
    ]);
  }

  Future<void> clear() => _preferences.clear();

  static T _enumByName<T extends Enum>(
    List<T> values,
    String? storedName,
    T fallback,
  ) {
    for (final value in values) {
      if (value.name == storedName) return value;
    }
    return fallback;
  }
}
