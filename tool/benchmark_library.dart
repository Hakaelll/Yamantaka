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
  final database = AppDatabase.inMemory(
    AppLogger(minimumLevel: LogLevel.error),
  );
  final repository = DocumentRepository(database);
  final now = DateTime.utc(2026, 1, 1);
  final insert = Stopwatch()..start();
  for (var index = 0; index < 5000; index++) {
    repository.save(
      LibraryDocument(
        id: 'fixture-$index',
        path: 'fixture-$index.pdf',
        normalizedPath: 'fixture-$index.pdf',
        fileName: 'fixture-$index.pdf',
        title: 'Synthetic document $index',
        author: 'Synthetic author',
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

  final samples = <int>[];
  for (var pass = 0; pass < 20; pass++) {
    final stopwatch = Stopwatch()..start();
    repository.page(
      LibraryQuery(
        offset: (pass * 80) % 4000,
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
    'ORDER BY d.last_opened_at DESC LIMIT 80',
  );
  final report = <String, Object?>{
    'format': 'yamantaka-library-benchmark',
    'version': 1,
    'generatedAt': DateTime.now().toUtc().toIso8601String(),
    'documents': 5000,
    'pageSize': 80,
    'insertMs': insert.elapsedMicroseconds / 1000,
    'queryMedianUs': samples[samples.length ~/ 2],
    'queryP95Us': samples[((samples.length - 1) * 0.95).ceil()],
    'rssBytes': ProcessInfo.currentRss,
    'integrity': database.integrityCheck(),
    'queryPlan': [for (final row in plan) row.values.join(' ')],
    'containsPersonalData': false,
  };
  database.close();
  final file = File(outputPath)..parent.createSync(recursive: true);
  file.writeAsStringSync(
    const JsonEncoder.withIndent('  ').convert(report),
    flush: true,
  );
  stdout.writeln(file.absolute.path);
}
