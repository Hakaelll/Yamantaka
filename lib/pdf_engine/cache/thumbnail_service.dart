// ignore_for_file: prefer_initializing_formals

import 'dart:io';
import 'dart:ui' as ui;

import 'package:path/path.dart' as p;

import '../../core/errors/app_failure.dart';
import '../../core/logging/app_logger.dart';
import '../../storage/paths/app_paths.dart';
import '../../storage/repositories/document_repository.dart';
import '../pdf_document_engine.dart';

final class ThumbnailService {
  const ThumbnailService({
    required PdfDocumentEngine engine,
    required AppPaths paths,
    required DocumentRepository documents,
    required AppLogger logger,
    required int Function() maxCacheBytes,
  }) : _engine = engine,
       _paths = paths,
       _documents = documents,
       _logger = logger,
       _maxCacheBytes = maxCacheBytes;

  final PdfDocumentEngine _engine;
  final AppPaths _paths;
  final DocumentRepository _documents;
  final AppLogger _logger;
  final int Function() _maxCacheBytes;

  Future<String?> generateCover({
    required String libraryDocumentId,
    required String engineDocumentId,
  }) async {
    final outputPath = p.join(
      _paths.thumbnailDirectory.path,
      '$libraryDocumentId.png',
    );
    try {
      final rendered = await _engine.renderPage(
        documentId: engineDocumentId,
        pageIndex: 0,
        scale: 0.3,
      );
      final data = await rendered.image.toByteData(
        format: ui.ImageByteFormat.png,
      );
      if (data == null) return null;
      await File(
        outputPath,
      ).writeAsBytes(data.buffer.asUint8List(), flush: true);
      _documents.updateThumbnail(libraryDocumentId, outputPath);
      await trimToLimit();
      return outputPath;
    } on Object catch (error, stackTrace) {
      _logger.warning('Cover generation failed.', error);
      Error.throwWithStackTrace(
        AppFailure(
          FailureKind.thumbnailFailed,
          'No se pudo crear la portada.',
          debugMessage: error.toString(),
        ),
        stackTrace,
      );
    }
  }

  Future<void> clear() async {
    final directory = _paths.thumbnailDirectory;
    if (await directory.exists()) {
      await for (final entity in directory.list(followLinks: false)) {
        if (entity is File) await entity.delete();
      }
    }
    _documents.clearThumbnailPaths();
  }

  /// Removes oldest cached covers until the configured byte budget is met.
  Future<void> trimToLimit() async {
    final files = <({File file, FileStat stat})>[];
    var totalBytes = 0;
    await for (final entity in _paths.thumbnailDirectory.list(
      followLinks: false,
    )) {
      if (entity is! File) continue;
      final stat = await entity.stat();
      totalBytes += stat.size;
      files.add((file: entity, stat: stat));
    }
    final limit = _maxCacheBytes();
    if (totalBytes <= limit) return;
    files.sort((a, b) => a.stat.modified.compareTo(b.stat.modified));
    for (final entry in files) {
      if (totalBytes <= limit) break;
      await entry.file.delete();
      _documents.clearThumbnailPath(entry.file.path);
      totalBytes -= entry.stat.size;
    }
  }
}
