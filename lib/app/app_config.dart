/// Compile-time product configuration.
abstract final class AppConfig {
  static const String productName = String.fromEnvironment(
    'YAMANTAKA_PRODUCT_NAME',
    defaultValue: 'Yamantaka',
  );
  static const String databaseFileName = 'yamantaka.sqlite3';
  static const String cacheDirectoryName = 'render-cache';
  static const String thumbnailDirectoryName = 'covers';
  static const int databaseVersion = 10;
  static const int maxRecentDocuments = 40;
  static const int maxOpenTabs = 12;
  static const int maxWarmTabs = 2;
  static const int maxSearchResults = 100;
  static const int maxLibraryPageSize = 80;
  static const int maxAutomationDocuments = 100000;
  static const int automationBatchSize = 80;
  static const Duration maxAutomationDuration = Duration(seconds: 30);
  static const int maxAnnotationNoteLength = 4000;
  static const int maxRenderCacheBytes = 96 * 1024 * 1024;
  static const int maxPageCacheBytes = 72 * 1024 * 1024;
  static const int maxTileCacheBytes = 24 * 1024 * 1024;
  static const int maxTextCacheBytes = 8 * 1024 * 1024;
  static const int maxPdfiumDimension = 16384;
  static const int maxSingleRenderBytes = 192 * 1024 * 1024;
  static const double previewRenderDpi = 110;
  static const Duration settledRenderDelay = Duration(milliseconds: 75);
  static const Duration positionSaveDebounce = Duration(milliseconds: 450);
}
