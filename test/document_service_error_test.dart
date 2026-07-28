import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main() {
  late Directory directory;
  late AppDatabase database;
  late DocumentService service;

  setUp(() async {
    directory = await Directory.systemTemp.createTemp(
      'yamantaka-document-errors-',
    );
    database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
    service = DocumentService(DocumentRepository(database));
  });

  tearDown(() async {
    database.close();
    await directory.delete(recursive: true);
  });

  test('rejects a non-PDF extension with a typed failure', () async {
    await expectLater(
      service.inspect('${directory.path}${Platform.pathSeparator}notes.txt'),
      throwsA(_failure(FailureKind.unsupportedFormat)),
    );
  });

  test('reports a missing PDF with a typed failure', () async {
    await expectLater(
      service.inspect('${directory.path}${Platform.pathSeparator}missing.pdf'),
      throwsA(_failure(FailureKind.fileNotFound)),
    );
  });

  test('reports an empty PDF with a typed failure', () async {
    final file = File('${directory.path}${Platform.pathSeparator}empty.pdf');
    await file.create();

    await expectLater(
      service.inspect(file.path),
      throwsA(_failure(FailureKind.emptyPdf)),
    );
  });
}

Matcher _failure(FailureKind kind) =>
    isA<AppFailure>().having((failure) => failure.kind, 'kind', kind);
