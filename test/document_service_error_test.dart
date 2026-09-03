import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/core/security/local_input_policy.dart';
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

  test('inspects a non-empty PDF through a Unicode path', () async {
    final nested = Directory(
      '${directory.path}${Platform.pathSeparator}Lecturas 長い nombre ñ',
    );
    await nested.create();
    final file = File(
      '${nested.path}${Platform.pathSeparator}藏書 — filosofía local.pdf',
    );
    await file.writeAsBytes(const [0x25, 0x50, 0x44, 0x46]);

    final inspected = await service.inspect(file.path);

    expect(inspected.fileName, '藏書 — filosofía local.pdf');
    expect(inspected.fileSize, 4);
    expect(inspected.path, file.absolute.path);
  });

  test('accepts a long bounded local path without truncating it', () {
    final segment = List.filled(24, 'lectura-local').join('-');
    final input = '${directory.path}${Platform.pathSeparator}$segment.pdf';

    final bounded = LocalInputPolicy.boundedPath(input);

    expect(bounded, File(input).absolute.path);
    expect(bounded, endsWith('$segment.pdf'));
  });

  test(
    'maps permission loss after discovery without exposing the path',
    () async {
      const privatePath = r'C:\private\restricted.pdf';
      final denied = DocumentService(
        DocumentRepository(database),
        fileAccess: const _FailingFileAccess(failOnStat: true, osErrorCode: 5),
      );

      late AppFailure failure;
      try {
        await denied.inspect(privatePath);
        fail('Expected accessDenied.');
      } on AppFailure catch (error) {
        failure = error;
      }

      expect(failure.kind, FailureKind.accessDenied);
      expect(failure.debugMessage, isNot(contains(privatePath)));
    },
  );

  test('maps a disconnected drive to a recoverable missing file', () async {
    final disconnected = DocumentService(
      DocumentRepository(database),
      fileAccess: const _FailingFileAccess(failOnExists: true, osErrorCode: 21),
    );

    await expectLater(
      disconnected.inspect(r'Z:\offline\library.pdf'),
      throwsA(_failure(FailureKind.fileNotFound)),
    );
  });
}

final class _FailingFileAccess implements LocalDocumentFileAccess {
  const _FailingFileAccess({
    required this.osErrorCode,
    this.failOnExists = false,
    this.failOnStat = false,
  });

  final int osErrorCode;
  final bool failOnExists;
  final bool failOnStat;

  FileSystemException get _error => FileSystemException(
    'Synthetic local access failure',
    r'Z:\private\never-log-this.pdf',
    OSError('Synthetic Windows error', osErrorCode),
  );

  @override
  String absolutePath(String path) => path;

  @override
  Future<bool> exists(String path) async {
    if (failOnExists) throw _error;
    return true;
  }

  @override
  Stream<List<int>> openRead(String path, int start, int end) =>
      const Stream<List<int>>.empty();

  @override
  Future<FileStat> stat(String path) async {
    if (failOnStat) throw _error;
    throw StateError('Unexpected stat call.');
  }
}

Matcher _failure(FailureKind kind) =>
    isA<AppFailure>().having((failure) => failure.kind, 'kind', kind);
