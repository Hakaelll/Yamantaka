import 'dart:convert';
import 'dart:io';

import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main(List<String> arguments) {
  final outputPath =
      arguments.firstOrNull ?? 'build/verification/library-benchmark.json';
  final results = <Map<String, Object?>>[];
  for (final count in const [5000, 25000, 100000]) {
    results.add(_benchmark(count));
  }
  final report = <String, Object?>{
    'format': 'yamantaka-library-benchmark',
    'version': 2,
    'generatedAt': DateTime.now().toUtc().toIso8601String(),
    'pageSize': 80,
    'results': results,
    'containsPersonalData': false,
  };
  final file = File(outputPath)..parent.createSync(recursive: true);
  file.writeAsStringSync(
    const JsonEncoder.withIndent('  ').convert(report),
    flush: true,
  );
  stdout.writeln(file.absolute.path);
}

Map<String, Object?> _benchmark(int documentCount) {
  final logger = AppLogger(minimumLevel: LogLevel.error);
  final database = AppDatabase.inMemory(logger);
  final repository = DocumentRepository(database);
  final now = DateTime.utc(2026, 1, 1);
  final insert = Stopwatch()..start();
  for (var index = 0; index < documentCount; index++) {
    repository.save(
      LibraryDocument(
        id: 'fixture-${index.toString().padLeft(6, '0')}',
        path: 'fixture-$index.pdf',
        normalizedPath: 'fixture-$index.pdf',
        fileName: 'fixture-$index.pdf',
        title: 'Synthetic document $index',
        author: 'Synthetic author ${index % 200}',
        pageCount: 100 + index % 900,
        fileSize: 1024 + index,
        fileModifiedAt: now,
        addedAt: now.subtract(Duration(minutes: index)),
        lastOpenedAt: now.subtract(Duration(seconds: index)),
        readingProgress: (index % 101) / 100,
        fingerprint: 'fingerprint-$index',
        isFavorite: index % 17 == 0,
      ),
    );
  }
  insert.stop();

  final firstLoad = Stopwatch()..start();
  const firstQuery = LibraryQuery(limit: 80);
  final firstPage = repository.page(firstQuery);
  repository.authorFacets(firstQuery);
  repository.topicFacets(firstQuery);
  repository.readingStateCounts(firstQuery);
  repository.documentCount;
  firstLoad.stop();

  final samples = <int>[];
  for (var pass = 0; pass < 20; pass++) {
    final stopwatch = Stopwatch()..start();
    repository.page(
      LibraryQuery(
        offset: (pass * 80) % (documentCount - 100),
        limit: 80,
        text: pass.isEven ? '' : 'Synthetic document',
        favoritesOnly: pass % 5 == 0,
      ),
    );
    stopwatch.stop();
    samples.add(stopwatch.elapsedMicroseconds);
  }
  samples.sort();
  final plan = database.database.select(
    'EXPLAIN QUERY PLAN SELECT d.id FROM documents d '
    'WHERE d.is_missing = 0 AND d.is_favorite = 1 '
    'ORDER BY d.last_opened_at DESC, d.id ASC LIMIT 80',
  );
  final result = <String, Object?>{
    'documents': documentCount,
    'insertMs': insert.elapsedMicroseconds / 1000,
    'firstLoadUs': firstLoad.elapsedMicroseconds,
    'firstLoadFileSystemWalks': 0,
    'queryMedianUs': samples[samples.length ~/ 2],
    'queryP95Us': samples[((samples.length - 1) * 0.95).ceil()],
    'materializedDocuments': firstPage.documents.length,
    'totalDocuments': firstPage.total,
    'rssBytes': ProcessInfo.currentRss,
    'integrity': database.integrityCheck(),
    'queryPlan': [for (final row in plan) row.values.join(' ')],
  };
  database.close();
  return result;
}
