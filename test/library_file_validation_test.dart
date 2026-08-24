import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/async/job_coordinator.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/application/library_file_validation_controller.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main() {
  late Directory directory;
  late AppDatabase database;
  late DocumentRepository repository;
  late JobCoordinator jobs;

  setUp(() async {
    directory = await Directory.systemTemp.createTemp(
      'yamantaka-file-validation-',
    );
    database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
    repository = DocumentRepository(database);
    jobs = JobCoordinator(maxConcurrent: 1);
  });

  tearDown(() async {
    await jobs.dispose();
    database.close();
    await directory.delete(recursive: true);
  });

  test(
    'first library load uses persisted state without walking files',
    () async {
      final missingPath =
          '${directory.path}${Platform.pathSeparator}absent.pdf';
      repository.save(_document(id: 'known', path: missingPath));
      final controller = LibraryController(
        repository,
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(controller.dispose);

      await controller.load();

      expect(controller.documents, hasLength(1));
      expect(controller.documents.single.isMissing, isFalse);
      expect(controller.documents.single.hasFileChanged, isFalse);
    },
  );

  test(
    'incremental validation persists one transactional batch at a time',
    () async {
      final available = File(
        '${directory.path}${Platform.pathSeparator}available.pdf',
      );
      await available.writeAsBytes([1, 2, 3], flush: true);
      final availableStat = await available.stat();
      final modified = File(
        '${directory.path}${Platform.pathSeparator}modified.pdf',
      );
      await modified.writeAsBytes([1, 2, 3, 4], flush: true);
      final modifiedStat = await modified.stat();
      final absentPath = '${directory.path}${Platform.pathSeparator}absent.pdf';
      repository
        ..save(
          _document(
            id: 'available',
            path: available.path,
            size: availableStat.size,
            modifiedAt: availableStat.modified,
          ),
        )
        ..save(
          _document(
            id: 'modified',
            path: modified.path,
            size: modifiedStat.size + 10,
            modifiedAt: modifiedStat.modified,
          ),
        )
        ..save(_document(id: 'missing', path: absentPath));
      final changedBatches = <Set<String>>[];
      final validation = LibraryFileValidationController(
        documents: repository,
        jobs: jobs,
        logger: AppLogger(minimumLevel: LogLevel.error),
        batchSize: 2,
        onDocumentsChanged: (ids) => changedBatches.add(ids),
      );
      addTearDown(validation.dispose);

      final summary = await validation.checkAll();

      expect(summary.available, 1);
      expect(summary.modified, 1);
      expect(summary.missing, 1);
      expect(summary.failed, 0);
      expect(summary.cancelled, 0);
      expect(repository.byId('available')!.isMissing, isFalse);
      expect(repository.byId('available')!.hasFileChanged, isFalse);
      expect(repository.byId('modified')!.hasFileChanged, isTrue);
      expect(repository.byId('missing')!.isMissing, isTrue);
      expect(
        changedBatches.expand((batch) => batch),
        containsAll(['modified', 'missing']),
      );
      expect(
        database.database
            .select(
              'SELECT COUNT(*) FROM documents WHERE last_validated_at IS NOT NULL',
            )
            .first
            .columnAt(0),
        3,
      );
    },
  );

  test(
    'cancellation prevents obsolete batches and reports remaining work',
    () async {
      for (var index = 0; index < 80; index++) {
        repository.save(
          _document(
            id: 'item-${index.toString().padLeft(3, '0')}',
            path: '${directory.path}${Platform.pathSeparator}$index.pdf',
          ),
        );
      }
      final validation = LibraryFileValidationController(
        documents: repository,
        jobs: jobs,
        logger: AppLogger(minimumLevel: LogLevel.error),
        batchSize: 1,
      );
      addTearDown(validation.dispose);

      final pending = validation.checkAll();
      validation.setReaderPaused(true);
      final summary = await pending;

      expect(summary.cancelled, greaterThan(0));
      expect(validation.progress.isRunning, isFalse);
    },
  );

  test(
    'stable SQL ordering and append de-duplication keep pages exact',
    () async {
      final stamp = DateTime.utc(2026, 8, 1);
      for (var index = 0; index < 205; index++) {
        repository.save(
          _document(
            id: 'item-${index.toString().padLeft(3, '0')}',
            path: 'item-$index.pdf',
            modifiedAt: stamp,
            openedAt: stamp,
          ),
        );
      }
      final controller = LibraryController(
        repository,
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(controller.dispose);

      await controller.load(query: const LibraryQuery(limit: 80));
      await controller.loadNextPage();
      await controller.loadNextPage();

      expect(controller.documents, hasLength(205));
      expect(
        controller.documents.map((item) => item.id).toSet(),
        hasLength(205),
      );
      expect(controller.hasMore, isFalse);
    },
  );
}

LibraryDocument _document({
  required String id,
  required String path,
  int size = 0,
  DateTime? modifiedAt,
  DateTime? openedAt,
}) {
  final stamp = modifiedAt ?? DateTime.utc(2026, 8, 1);
  return LibraryDocument(
    id: id,
    path: path,
    normalizedPath: path.toLowerCase(),
    fileName: '$id.pdf',
    pageCount: 1,
    fileSize: size,
    fileModifiedAt: stamp,
    addedAt: stamp,
    lastOpenedAt: openedAt ?? stamp,
    fingerprint: 'fingerprint-$id',
  );
}
