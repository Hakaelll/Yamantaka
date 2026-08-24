import 'dart:convert';
import 'dart:io';

import '../../../app/app_config.dart';
import '../../../core/async/job_coordinator.dart';
import '../../../core/lifecycle/lifecycle_tracker.dart';
import '../../../core/memory/memory_governor.dart';
import '../../../core/performance/startup_timeline.dart';
import '../../../core/performance/local_performance_monitor.dart';
import '../../../core/privacy/diagnostics_redactor.dart';
import '../../../pdf_engine/adapters/pdfrx_document_engine.dart';
import '../../../storage/database/app_database.dart';
import '../../../storage/paths/app_paths.dart';

final class SafeDiagnosticsReport {
  const SafeDiagnosticsReport(this.values);

  final Map<String, Object?> values;

  String toJson() {
    DiagnosticsRedactor.assertSafe(values);
    return const JsonEncoder.withIndent('  ').convert({
      'format': 'yamantaka-safe-diagnostics',
      'version': 2,
      'manifest': DiagnosticsRedactor.manifest(),
      ...DiagnosticsRedactor.redact(values),
    });
  }

  String preview() => DiagnosticsRedactor.preview(values);
}

final class DiagnosticsService {
  const DiagnosticsService({
    required this.database,
    required this.paths,
    required this.engine,
    required this.safeMode,
    this.jobs,
    this.memory,
    this.lifecycle,
    this.startup,
    this.performance,
  });

  final AppDatabase database;
  final AppPaths paths;
  final PdfrxDocumentEngine engine;
  final bool safeMode;
  final JobCoordinator? jobs;
  final MemoryGovernor? memory;
  final LifecycleTracker? lifecycle;
  final StartupTimeline? startup;
  final LocalPerformanceMonitor? performance;

  Future<SafeDiagnosticsReport> collect() async {
    final databaseFile = File(paths.databasePath);
    return SafeDiagnosticsReport({
      'yamantaka_version': '0.5.0+6',
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
      'library_notes': database.tableCount('document_notes'),
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
      'storage_mode': paths.mode.name,
      'engine': engine.diagnostics.toJson(),
      'jobs': jobs?.snapshot.toJson() ?? const <String, Object>{},
      'memory': memory?.snapshot.toJson() ?? const <String, Object>{},
      'lifecycle': lifecycle?.snapshot.toJson() ?? const <String, int>{},
      'startup': startup?.toJson() ?? const <String, Object>{},
      'performance': performance?.toJson() ?? const <String, Object>{},
      'contains_paths_titles_or_queries': false,
    });
  }

  Future<void> exportTo(SafeDiagnosticsReport report, String path) async {
    final destination = File(path);
    final partial = File('$path.part');
    final previous = File('$path.previous');
    var movedPrevious = false;
    var installed = false;
    try {
      await partial.writeAsString(report.toJson(), flush: true);
      if (await previous.exists()) await previous.delete();
      if (await destination.exists()) {
        await destination.rename(previous.path);
        movedPrevious = true;
      }
      await partial.rename(destination.path);
      installed = true;
      if (movedPrevious && await previous.exists()) {
        await previous.delete();
      }
    } on Object {
      if (await partial.exists()) await partial.delete();
      if (!installed &&
          movedPrevious &&
          !await destination.exists() &&
          await previous.exists()) {
        await previous.rename(destination.path);
      }
      rethrow;
    }
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
