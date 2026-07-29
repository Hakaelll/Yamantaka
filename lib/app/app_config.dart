/// Compile-time product configuration.
abstract final class AppConfig {
  static const String productName = String.fromEnvironment(
    'YAMANTAKA_PRODUCT_NAME',
    defaultValue: 'Yamantaka',
  );
  static const String databaseFileName = 'yamantaka.sqlite3';
  static const String cacheDirectoryName = 'render-cache';
  static const String thumbnailDirectoryName = 'covers';
  static const int databaseVersion = 1;
  static const int maxRecentDocuments = 40;
  static const int maxRenderCacheBytes = 96 * 1024 * 1024;
  static const Duration positionSaveDebounce = Duration(milliseconds: 450);
}
