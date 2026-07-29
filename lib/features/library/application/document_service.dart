import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as p;

import '../../../core/errors/app_failure.dart';
import '../../../core/platform/path_normalizer.dart';
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

final class DocumentService {
  const DocumentService(this._repository);

  final DocumentRepository _repository;

  Future<InspectedFile> inspect(String inputPath) async {
    final path = inputPath.trim().replaceAll('"', '');
    if (path.isEmpty || p.extension(path).toLowerCase() != '.pdf') {
      throw const AppFailure(
        FailureKind.unsupportedFormat,
        'Selecciona un archivo PDF.',
      );
    }
    final file = File(path);
    try {
      if (!await file.exists()) {
        throw const AppFailure(
          FailureKind.fileNotFound,
          'El archivo ya no existe en esa ubicación.',
        );
      }
      final stat = await file.stat();
      if (stat.size == 0) {
        throw const AppFailure(
          FailureKind.emptyPdf,
          'El archivo PDF está vacío.',
        );
      }
      final end = math.min(stat.size, 64 * 1024);
      final partialDigest = await sha256.bind(file.openRead(0, end)).first;
      final fingerprint = sha256
          .convert(utf8.encode('${stat.size}:$partialDigest'))
          .toString();
      return InspectedFile(
        path: file.absolute.path,
        normalizedPath: PathNormalizer.normalize(file.path),
        fileName: p.basename(file.path),
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
        debugMessage: error.toString(),
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
