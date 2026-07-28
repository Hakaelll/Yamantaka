import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
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
