import 'dart:convert';
import 'dart:io';

import '../../../app/app_config.dart';
import '../../../pdf_engine/adapters/pdfrx_document_engine.dart';
import '../../../storage/database/app_database.dart';
import '../../../storage/paths/app_paths.dart';

final class SafeDiagnosticsReport {
  const SafeDiagnosticsReport(this.values);

  final Map<String, Object?> values;

  String toJson() => const JsonEncoder.withIndent(
    '  ',
  ).convert({'format': 'yamantaka-safe-diagnostics', 'version': 1, ...values});
}

final class DiagnosticsService {
  const DiagnosticsService({
    required this.database,
    required this.paths,
    required this.engine,
    required this.safeMode,
  });

  final AppDatabase database;
  final AppPaths paths;
  final PdfrxDocumentEngine engine;
  final bool safeMode;

  Future<SafeDiagnosticsReport> collect() async {
    final databaseFile = File(paths.databasePath);
    return SafeDiagnosticsReport({
      'yamantaka_version': '0.1.0+1',
      'flutter_version': const String.fromEnvironment(
        'FLUTTER_VERSION',
        defaultValue: 'bundled',
      ),
      'platform': Platform.operatingSystem,
      'platform_version': _safePlatformVersion(),
      'architecture':
          Platform.environment['PROCESSOR_ARCHITECTURE'] ?? 'unknown',
      'pdf_engine': 'PDFium via pdfrx',
      'database_schema': AppConfig.databaseVersion,
      'database_bytes': await databaseFile.exists()
          ? await databaseFile.length()
          : 0,
      'cache_bytes': await _directoryBytes(paths.cacheDirectory),
      'documents': database.tableCount('documents'),
      'missing_documents': database.database
          .select('SELECT COUNT(*) FROM documents WHERE is_missing = 1')
          .first
          .columnAt(0),
      'indexed_documents': database.tableCount('search_index_state'),
      'annotations': database.tableCount('annotations'),
      'collections': database.tableCount('collections'),
      'tags': database.tableCount('tags'),
      'workspaces': database.tableCount('workspaces'),
      'integrity': database.integrityCheck(),
      'safe_mode': safeMode,
      'data_location': _maskedPath(paths.dataDirectory.path),
      'engine': engine.diagnostics.toJson(),
      'contains_paths_titles_or_queries': false,
    });
  }

  void optimizeDatabase() => database.optimize();

  String verifyDatabase() => database.integrityCheck();

  static Future<int> _directoryBytes(Directory directory) async {
    if (!await directory.exists()) return 0;
    var total = 0;
    await for (final entity in directory.list(recursive: true)) {
      if (entity is File) {
        try {
          total += await entity.length();
        } on FileSystemException {
          // A cache file may disappear while diagnostics are collected.
        }
      }
    }
    return total;
  }

  static String _maskedPath(String path) {
    final separator = Platform.pathSeparator;
    final parts = path.split(separator).where((part) => part.isNotEmpty);
    final tail = parts.isEmpty ? 'data' : parts.last;
    return '…$separator$tail';
  }

  static String _safePlatformVersion() {
    final raw = Platform.operatingSystemVersion;
    return raw.length <= 120 ? raw : raw.substring(0, 120);
  }
}
