import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/workspaces/domain/workspace_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/annotation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/global_search_repository.dart';
import 'package:yamantaka/storage/repositories/organization_repository.dart';
import 'package:yamantaka/storage/repositories/workspace_repository.dart';

Future<void> main(List<String> arguments) async {
  final outputPath =
      arguments.firstOrNull ?? 'build/performance/advanced-features.json';
  final baselineRss = ProcessInfo.currentRss;
  final database = AppDatabase.inMemory(
    AppLogger(minimumLevel: LogLevel.error),
  );
  final documents = DocumentRepository(database);
  final annotations = AnnotationRepository(database);
  final organization = OrganizationRepository(database);
  final search = GlobalSearchRepository(database);
  final now = DateTime.utc(2026, 7, 29);

  final insert = Stopwatch()..start();
  for (var index = 0; index < 500; index++) {
    final document = _document(index, now);
    documents.save(document);
    search.beginDocument(document);
    if (index % 10 == 0) {
      search.addPage(
        document: document,
        pageIndex: index % 20,
        text: 'fixture local indexable term-${index % 7}',
      );
      search.finishDocument(document.id, hasExtractableText: true);
    } else {
      search.finishDocument(document.id, hasExtractableText: false);
    }
  }
  insert.stop();

  for (var index = 0; index < 100; index++) {
    annotations.create(
      documentId: 'document-${index % 40}',
      pageIndex: index % 20,
      type: AnnotationType.values[index % AnnotationType.values.length],
      color: AnnotationColor.values[index % AnnotationColor.values.length],
      documentFingerprint: 'fingerprint-${index % 40}',
      note: 'nota fixture tema-${index % 5}',
      tags: ['etiqueta-${index % 4}'],
    );
  }
  search.refreshAnnotationEntries();

  for (var index = 0; index < 10; index++) {
    final collection = organization.createCollection('Colección $index');
    for (var documentIndex = index; documentIndex < 500; documentIndex += 10) {
      organization.addDocument(collection.id, 'document-$documentIndex');
    }
  }

  final allDocuments = documents.all();
  final tabs = [
    for (var index = 0; index < 5; index++)
      DocumentTab(
        id: 'tab-$index',
        documentId: allDocuments[index].id,
        path: allDocuments[index].path,
        title: allDocuments[index].title!,
        pageIndex: index,
        zoomLevel: 1,
        pageLayout: 'continuous',
        readingMode: 'document',
        scrollOffset: 0,
        readingProgress: index / 5,
        isPinned: index == 0,
        resourceState: index == 4
            ? TabResourceState.active
            : TabResourceState.suspended,
        readingTheme: 'ivory',
      ),
  ];
  final workspaces = WorkspaceRepository(database, documents);
  workspaces.save(
    name: 'Fixture',
    tabs: tabs,
    activeTabId: tabs.last.id,
    panelSection: 'annotations',
    split: const SplitViewState(enabled: true, syncZoom: true),
  );

  final searchSamples = <int>[];
  final collectionSamples = <int>[];
  final workspaceSamples = <int>[];
  for (var pass = 0; pass < 7; pass++) {
    searchSamples.add(
      _measure(() => search.search('fixture local', limit: 100)),
    );
    collectionSamples.add(
      _measure(
        () =>
            organization.documentIds(organization.collections()[pass % 10].id),
      ),
    );
    workspaceSamples.add(_measure(workspaces.all));
  }

  final finalRss = ProcessInfo.currentRss;
  final peakRss = math.max(baselineRss, finalRss);
  final pageCount =
      database.database.select('PRAGMA page_count').first.columnAt(0) as int;
  final pageSize =
      database.database.select('PRAGMA page_size').first.columnAt(0) as int;
  final report = <String, Object?>{
    'format': 'yamantaka-advanced-performance',
    'version': 1,
    'generated_at': DateTime.now().toUtc().toIso8601String(),
    'mode': 'dart-vm-local-synthetic',
    'passes': 7,
    'contains_paths_or_titles': false,
    'fixtures': {
      'documents': 500,
      'annotations': 100,
      'collections': 10,
      'workspace_tabs': 5,
    },
    'bulk_insert_ms': insert.elapsedMicroseconds / 1000,
    'global_search_us': _distribution(searchSamples),
    'collection_query_us': _distribution(collectionSamples),
    'workspace_restore_us': _distribution(workspaceSamples),
    'database_bytes': pageCount * pageSize,
    'rss_baseline_bytes': baselineRss,
    'rss_peak_bytes': peakRss,
    'rss_final_bytes': finalRss,
    'rss_delta_bytes': math.max(0, peakRss - baselineRss),
    'integrity': database.integrityCheck(),
  };
  database.close();
  final output = File(outputPath);
  output.parent.createSync(recursive: true);
  await output.writeAsString(
    const JsonEncoder.withIndent('  ').convert(report),
    flush: true,
  );
  stdout.writeln(output.absolute.path);
}

int _measure(Object? Function() operation) {
  final stopwatch = Stopwatch()..start();
  operation();
  stopwatch.stop();
  return stopwatch.elapsedMicroseconds;
}

Map<String, num> _distribution(List<int> samples) {
  final sorted = [...samples]..sort();
  final middle = sorted.length ~/ 2;
  final p95Index = ((sorted.length - 1) * 0.95).ceil();
  return {
    'median': sorted[middle],
    'p95': sorted[p95Index],
    'min': sorted.first,
    'max': sorted.last,
  };
}

LibraryDocument _document(int index, DateTime now) => LibraryDocument(
  id: 'document-$index',
  path: 'fixture-$index.pdf',
  normalizedPath: 'fixture-$index.pdf',
  fileName: 'fixture-$index.pdf',
  title: 'Documento fixture $index',
  author: 'Autor fixture',
  pageCount: 20 + index % 200,
  fileSize: 4096 + index,
  fileModifiedAt: now,
  addedAt: now,
  lastOpenedAt: now.subtract(Duration(minutes: index)),
  lastPageIndex: index % 20,
  readingProgress: (index % 100) / 100,
  fingerprint: 'fingerprint-$index',
);
