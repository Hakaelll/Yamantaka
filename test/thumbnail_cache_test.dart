import 'dart:io';
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/async/job_coordinator.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  test(
    'cover cache removes oldest files and clears their SQLite references',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-cover-cache-',
      );
      final covers = Directory(
        '${directory.path}${Platform.pathSeparator}covers',
      )..createSync();
      final logger = AppLogger(minimumLevel: LogLevel.error);
      final database = AppDatabase.inMemory(logger);
      final documents = DocumentRepository(database);
      addTearDown(() async {
        database.close();
        await directory.delete(recursive: true);
      });

      final files = <File>[];
      for (var index = 0; index < 3; index++) {
        final file = File(
          '${covers.path}${Platform.pathSeparator}cover-$index.png',
        );
        await file.writeAsBytes(List<int>.filled(10, index));
        await file.setLastModified(DateTime.utc(2026, 7, 28, 10, index));
        files.add(file);
        documents.save(_document(index, file.path));
      }
      final service = ThumbnailService(
        engine: StubPdfDocumentEngine(),
        paths: AppPaths(
          dataDirectory: directory,
          cacheDirectory: directory,
          thumbnailDirectory: covers,
        ),
        documents: documents,
        logger: logger,
        maxCacheBytes: () => 15,
      );

      await service.trimToLimit();

      expect(await files[0].exists(), isFalse);
      expect(await files[1].exists(), isFalse);
      expect(await files[2].exists(), isTrue);
      expect(documents.byId('doc-0')!.thumbnailPath, isNull);
      expect(documents.byId('doc-1')!.thumbnailPath, isNull);
      expect(documents.byId('doc-2')!.thumbnailPath, files[2].path);
    },
  );

  test('offscreen cancellation reports cover decodes avoided', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-cover-viewport-',
    );
    final covers = Directory('${directory.path}${Platform.pathSeparator}covers')
      ..createSync();
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    final documents = DocumentRepository(database);
    final document = _document(9, '');
    documents.save(document);
    final releaseBlocker = Completer<void>();
    final jobs = JobCoordinator(
      maxConcurrent: 1,
      categoryLimits: const {JobCategory.thumbnail: 1},
    );
    final blocker = jobs.submit<void>(
      key: 'viewport-test-blocker',
      category: JobCategory.thumbnail,
      priority: JobPriority.visible,
      generation: 1,
      task: (_) => releaseBlocker.future,
    );
    final service = ThumbnailService(
      engine: StubPdfDocumentEngine(),
      paths: AppPaths(
        dataDirectory: directory,
        cacheDirectory: directory,
        thumbnailDirectory: covers,
      ),
      documents: documents,
      logger: logger,
      maxCacheBytes: () => 1024 * 1024,
      jobs: jobs,
    );
    addTearDown(() async {
      if (!releaseBlocker.isCompleted) releaseBlocker.complete();
      await blocker.future;
      await jobs.dispose();
      database.close();
      await directory.delete(recursive: true);
    });

    service.promoteVisibleCover(document);
    for (var turn = 0; turn < 20; turn++) {
      await Future<void>.delayed(Duration.zero);
      if (service.diagnostics['pending_cover_decodes']! > 0) break;
    }
    expect(service.diagnostics['pending_cover_decodes'], greaterThan(0));

    service.cancelOffscreen(document.id);

    expect(service.diagnostics['cover_decodes_avoided'], greaterThan(0));
    expect(service.diagnostics['offscreen_cancellations'], 1);
    expect(service.diagnostics['pending_cover_decodes'], 0);
    releaseBlocker.complete();
    await blocker.future;
  });
}

LibraryDocument _document(int index, String thumbnailPath) {
  final now = DateTime.utc(2026, 7, 28, 10, index);
  return LibraryDocument(
    id: 'doc-$index',
    path: 'C:\\Books\\book-$index.pdf',
    normalizedPath: 'c:\\books\\book-$index.pdf',
    fileName: 'book-$index.pdf',
    pageCount: 10,
    fileSize: 100,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint-$index',
    thumbnailPath: thumbnailPath,
  );
}
