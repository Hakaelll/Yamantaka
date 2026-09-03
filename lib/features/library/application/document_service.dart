import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as p;

import '../../../core/errors/app_failure.dart';
import '../../../core/platform/path_normalizer.dart';
import '../../../core/security/local_input_policy.dart';
import '../../reader/domain/reader_models.dart';
import '../domain/library_document.dart';
import '../domain/document_cover.dart';
import '../../../storage/repositories/document_repository.dart';

final class InspectedFile {
  const InspectedFile({
    required this.path,
    required this.normalizedPath,
    required this.fileName,
    required this.fileSize,
    required this.modifiedAt,
    required this.fingerprint,
  });

  final String path;
  final String normalizedPath;
  final String fileName;
  final int fileSize;
  final DateTime modifiedAt;
  final String fingerprint;
}

abstract interface class LocalDocumentFileAccess {
  const LocalDocumentFileAccess();

  Future<bool> exists(String path);
  Future<FileStat> stat(String path);
  Stream<List<int>> openRead(String path, int start, int end);
  String absolutePath(String path);
}

final class DartLocalDocumentFileAccess implements LocalDocumentFileAccess {
  const DartLocalDocumentFileAccess();

  @override
  Future<bool> exists(String path) => File(path).exists();

  @override
  Future<FileStat> stat(String path) => File(path).stat();

  @override
  Stream<List<int>> openRead(String path, int start, int end) =>
      File(path).openRead(start, end);

  @override
  String absolutePath(String path) => File(path).absolute.path;
}

final class DocumentService {
  const DocumentService(
    this._repository, {
    this.fileAccess = const DartLocalDocumentFileAccess(),
  });

  final DocumentRepository _repository;
  final LocalDocumentFileAccess fileAccess;

  Future<InspectedFile> inspect(String inputPath) async {
    final path = LocalInputPolicy.boundedPath(inputPath);
    if (path.isEmpty || p.extension(path).toLowerCase() != '.pdf') {
      throw const AppFailure(
        FailureKind.unsupportedFormat,
        'Selecciona un archivo PDF.',
      );
    }
    try {
      if (!await fileAccess.exists(path)) {
        throw const AppFailure(
          FailureKind.fileNotFound,
          'El archivo ya no existe en esa ubicación.',
        );
      }
      final stat = await fileAccess.stat(path);
      if (stat.size == 0) {
        throw const AppFailure(
          FailureKind.emptyPdf,
          'El archivo PDF está vacío.',
        );
      }
      final end = math.min(stat.size, 64 * 1024);
      final partialDigest = await sha256
          .bind(fileAccess.openRead(path, 0, end))
          .first;
      final fingerprint = sha256
          .convert(utf8.encode('${stat.size}:$partialDigest'))
          .toString();
      return InspectedFile(
        path: fileAccess.absolutePath(path),
        normalizedPath: PathNormalizer.normalize(path),
        fileName: p.basename(path),
        fileSize: stat.size,
        modifiedAt: stat.modified,
        fingerprint: fingerprint,
      );
    } on AppFailure {
      rethrow;
    } on FileSystemException catch (error) {
      throw AppFailure(
        error.osError?.errorCode == 5
            ? FailureKind.accessDenied
            : FailureKind.fileNotFound,
        error.osError?.errorCode == 5
            ? 'No hay permiso para leer este archivo.'
            : 'No se pudo acceder al archivo.',
        debugMessage:
            'Local file access failed with OS error '
            '${error.osError?.errorCode ?? 'unknown'}.',
      );
    }
  }

  LibraryDocument register({
    required InspectedFile file,
    required int pageCount,
  }) {
    final now = DateTime.now().toUtc();
    final existing =
        _repository.byNormalizedPath(file.normalizedPath) ??
        _repository.byFingerprint(file.fingerprint);
    final id =
        existing?.id ??
        sha256
            .convert(
              utf8.encode('${file.fingerprint}:${now.microsecondsSinceEpoch}'),
            )
            .toString();
    final restoredPage = existing?.lastPageIndex.clamp(0, pageCount - 1) ?? 0;
    final document = LibraryDocument(
      id: id,
      path: file.path,
      normalizedPath: file.normalizedPath,
      fileName: file.fileName,
      title: existing?.title,
      author: existing?.author,
      sourceTitle: existing?.sourceTitle,
      sourceAuthor: existing?.sourceAuthor,
      titleOverride: existing?.titleOverride,
      authorOverride: existing?.authorOverride,
      libraryNote: existing?.libraryNote,
      pageCount: pageCount,
      fileSize: file.fileSize,
      fileModifiedAt: file.modifiedAt,
      addedAt: existing?.addedAt ?? now,
      lastOpenedAt: now,
      lastPageIndex: restoredPage,
      readingProgress: ReadingProgress.calculate(
        pageIndex: restoredPage,
        pageCount: pageCount,
      ),
      preferredViewMode: existing?.preferredViewMode ?? ReaderMode.document,
      preferredTheme: existing?.preferredTheme ?? ReaderTheme.ivory,
      fingerprint: file.fingerprint,
      thumbnailPath:
          existing?.fingerprint == file.fingerprint ||
              existing?.coverMetadata?.selectionMode ==
                  CoverSelectionMode.manual
          ? existing?.thumbnailPath
          : null,
      coverMetadata:
          existing?.fingerprint == file.fingerprint ||
              existing?.coverMetadata?.selectionMode ==
                  CoverSelectionMode.manual
          ? existing?.coverMetadata
          : null,
      isFavorite: existing?.isFavorite ?? false,
      topics: existing?.topics ?? const [],
      enrichment: existing?.enrichment,
    );
    if (existing != null &&
        existing.fingerprint != file.fingerprint &&
        existing.coverMetadata?.selectionMode != CoverSelectionMode.manual) {
      _repository.invalidateCover(existing.id);
    }
    _repository.save(document);
    return document;
  }
}
