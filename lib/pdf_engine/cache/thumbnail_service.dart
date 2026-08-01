// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:path/path.dart' as p;

import '../../core/errors/app_failure.dart';
import '../../core/async/job_coordinator.dart';
import '../../core/logging/app_logger.dart';
import '../../features/library/domain/document_cover.dart';
import '../../features/library/domain/library_document.dart';
import '../../storage/paths/app_paths.dart';
import '../../storage/repositories/document_repository.dart';
import '../models/pdf_models.dart';
import '../pdf_document_engine.dart';

final class ThumbnailService {
  static const double coverPreviewScale = 0.40;
  static const double smallCoverScale = 0.48;
  static const double largeCoverScale = 1.00;

  ThumbnailService({
    required PdfDocumentEngine engine,
    required AppPaths paths,
    required DocumentRepository documents,
    required AppLogger logger,
    required int Function() maxCacheBytes,
    JobCoordinator? jobs,
  }) : _engine = engine,
       _paths = paths,
       _documents = documents,
       _logger = logger,
       _maxCacheBytes = maxCacheBytes,
       _jobs = jobs;

  final PdfDocumentEngine _engine;
  final AppPaths _paths;
  final DocumentRepository _documents;
  final AppLogger _logger;
  final int Function() _maxCacheBytes;
  final JobCoordinator? _jobs;
  void Function()? onCoverUpdated;
  final Set<String> _cancelledDocumentIds = <String>{};
  final Map<String, PdfOperationToken> _activeCoverTokens = {};
  final Map<String, JobHandle<void>> _activeCoverJobs = {};
  final Map<String, int> _coverGenerations = <String, int>{};
  int _coverQueueGeneration = 0;
  Future<void> _backgroundQueue = Future<void>.value();

  Future<String?> generateCover({
    required String libraryDocumentId,
    required String engineDocumentId,
    PdfOperationToken? cancellationToken,
  }) async {
    final document = _documents.byId(libraryDocumentId);
    if (document == null) return null;
    final valid = document.coverMetadata?.isValidFor(
      modifiedAt: document.fileModifiedAt,
      currentAlgorithmVersion: CoverSelectionPolicy.algorithmVersion,
    );
    if (valid == true && await _coverFilesExist(document.coverMetadata!)) {
      return document.coverMetadata!.smallThumbnailPath;
    }
    final manual = document.coverMetadata;
    if (manual?.selectionMode == CoverSelectionMode.manual) {
      return _renderAndPersist(
        document: document,
        engineDocumentId: engineDocumentId,
        pageIndex: manual!.sourcePageIndex.clamp(0, document.pageCount - 1),
        confidence: 1,
        selectionMode: CoverSelectionMode.manual,
        cancellationToken: cancellationToken,
      );
    }
    return _detectAndGenerate(
      document,
      engineDocumentId,
      cancellationToken: cancellationToken,
    );
  }

  void enqueueCover(
    LibraryDocument document, {
    JobPriority priority = JobPriority.background,
  }) {
    _activeCoverTokens.remove(document.id)?.cancel();
    _cancelledDocumentIds.remove(document.id);
    final documentGeneration = (_coverGenerations[document.id] ?? 0) + 1;
    _coverGenerations[document.id] = documentGeneration;
    final queueGeneration = _coverQueueGeneration;
    Future<void> schedule() => _scheduleCover(
      document,
      documentGeneration: documentGeneration,
      queueGeneration: queueGeneration,
      priority: priority,
    );
    if (_jobs == null) {
      // The fallback is used by small unit/integration harnesses without the
      // application scheduler; retain serial IO there.
      _backgroundQueue = _backgroundQueue
          .catchError((Object error) {
            _logger.warning('Previous cover queue task failed.', error);
          })
          .then((_) => schedule());
    } else {
      // With the coordinator, submit immediately so a visible card can jump
      // ahead of background covers while the category limit still bounds IO.
      unawaited(schedule());
    }
  }

  /// Promotes a card that entered the viewport without exposing scheduler
  /// details to presentation widgets.
  void promoteVisibleCover(LibraryDocument document) =>
      enqueueCover(document, priority: JobPriority.visible);

  Future<void> _scheduleCover(
    LibraryDocument requested, {
    required int documentGeneration,
    required int queueGeneration,
    required JobPriority priority,
  }) async {
    if (queueGeneration != _coverQueueGeneration ||
        _coverGenerations[requested.id] != documentGeneration ||
        _cancelledDocumentIds.contains(requested.id)) {
      return;
    }
    final current = _documents.byId(requested.id);
    if (current == null ||
        current.isMissing ||
        current.coverMetadata?.selectionMode == CoverSelectionMode.manual) {
      return;
    }
    final valid = current.coverMetadata?.isValidFor(
      modifiedAt: current.fileModifiedAt,
      currentAlgorithmVersion: CoverSelectionPolicy.algorithmVersion,
    );
    if (valid == true && await _coverFilesExist(current.coverMetadata!)) return;

    Future<void> execute(JobCancellationToken? schedulerToken) async {
      String? engineDocumentId;
      final cancellationToken = PdfOperationToken();
      void cancelNative() => cancellationToken.cancel();
      schedulerToken?.addListener(cancelNative);
      _activeCoverTokens[current.id] = cancellationToken;
      try {
        if (cancellationToken.isCancelled) return;
        final opened = await _engine.openDocument(current.path);
        engineDocumentId = opened.documentId;
        if (!_cancelledDocumentIds.contains(current.id) &&
            !cancellationToken.isCancelled) {
          await _detectAndGenerate(
            current,
            opened.documentId,
            cancellationToken: cancellationToken,
          );
        }
      } on Object catch (error) {
        _logger.warning('Queued cover generation failed.', error);
      } finally {
        schedulerToken?.removeListener(cancelNative);
        if (identical(_activeCoverTokens[current.id], cancellationToken)) {
          _activeCoverTokens.remove(current.id);
        }
        if (engineDocumentId != null) {
          await _engine.closeDocument(engineDocumentId);
        }
      }
    }

    final jobs = _jobs;
    if (jobs == null) {
      await execute(null);
      return;
    }
    final handle = jobs.submit<void>(
      key: 'cover:${current.id}',
      category: JobCategory.thumbnail,
      priority: priority,
      generation: documentGeneration,
      task: execute,
    );
    _activeCoverJobs[current.id] = handle;
    try {
      await handle.future;
    } on JobCancelled {
      // Cancellation is expected when the document changes or closes.
    } finally {
      if (identical(_activeCoverJobs[current.id], handle)) {
        _activeCoverJobs.remove(current.id);
      }
    }
  }

  void cancel(String documentId) {
    _cancelledDocumentIds.add(documentId);
    _coverGenerations[documentId] = (_coverGenerations[documentId] ?? 0) + 1;
    _activeCoverTokens.remove(documentId)?.cancel();
    _activeCoverJobs.remove(documentId)?.cancel();
  }

  Future<List<CoverPreview>> previewPages(
    LibraryDocument document, {
    int maximumPages = CoverSelectionPolicy.maximumCandidatePages,
  }) async {
    final opened = await _engine.openDocument(document.path);
    try {
      final previews = <CoverPreview>[];
      final count = math.min(maximumPages, opened.pageCount);
      for (var pageIndex = 0; pageIndex < count; pageIndex++) {
        final rendered = await _engine.renderPage(
          documentId: opened.documentId,
          pageIndex: pageIndex,
          scale: coverPreviewScale,
        );
        final bytes = await _pngBytes(rendered.image);
        if (bytes != null) {
          previews.add(CoverPreview(pageIndex: pageIndex, pngBytes: bytes));
        }
      }
      return previews;
    } finally {
      await _engine.closeDocument(opened.documentId);
    }
  }

  Future<void> setManualCover(LibraryDocument document, int pageIndex) async {
    final opened = await _engine.openDocument(document.path);
    try {
      if (pageIndex < 0 || pageIndex >= opened.pageCount) {
        throw RangeError.range(pageIndex, 0, opened.pageCount - 1, 'pageIndex');
      }
      await _renderAndPersist(
        document: document,
        engineDocumentId: opened.documentId,
        pageIndex: pageIndex,
        confidence: 1,
        selectionMode: CoverSelectionMode.manual,
      );
    } finally {
      await _engine.closeDocument(opened.documentId);
    }
  }

  Future<void> restoreAutomaticCover(LibraryDocument document) async {
    await _deleteCoverFiles(document.coverMetadata);
    _documents.invalidateCover(document.id);
    final opened = await _engine.openDocument(document.path);
    try {
      await _detectAndGenerate(document, opened.documentId);
    } finally {
      await _engine.closeDocument(opened.documentId);
    }
  }

  Future<void> removeForDocument(LibraryDocument document) async {
    cancel(document.id);
    await _deleteCoverFiles(document.coverMetadata);
    _documents.invalidateCover(document.id);
  }

  Future<String?> _detectAndGenerate(
    LibraryDocument document,
    String engineDocumentId, {
    PdfOperationToken? cancellationToken,
  }) async {
    try {
      final candidateCount = math.min(
        CoverSelectionPolicy.maximumCandidatePages,
        document.pageCount,
      );
      final metrics = <CoverCandidateMetrics>[];
      for (var pageIndex = 0; pageIndex < candidateCount; pageIndex++) {
        if (_cancelledDocumentIds.contains(document.id) ||
            (cancellationToken?.isCancelled ?? false)) {
          throw const AppFailure(
            FailureKind.cancelled,
            'Generación de portada cancelada.',
          );
        }
        final rendered = await _engine.renderPage(
          documentId: engineDocumentId,
          pageIndex: pageIndex,
          scale: 0.16,
          cancellationToken: cancellationToken,
        );
        String text = '';
        try {
          text = await _engine.extractPageText(
            documentId: engineDocumentId,
            pageIndex: pageIndex,
            cancellationToken: cancellationToken,
          );
        } on Object {
          // Visual scoring remains available for scanned documents.
        }
        metrics.add(await _measure(rendered.image, pageIndex, text));
        if (cancellationToken?.isCancelled ?? false) {
          throw const AppFailure(
            FailureKind.cancelled,
            'Generación de portada cancelada.',
          );
        }
      }
      if (metrics.isEmpty) return null;
      final selected = CoverSelectionPolicy.choose(metrics);
      return _renderAndPersist(
        document: document,
        engineDocumentId: engineDocumentId,
        pageIndex: selected.metrics.pageIndex,
        confidence: selected.confidence,
        selectionMode: CoverSelectionMode.automatic,
        cancellationToken: cancellationToken,
      );
    } on AppFailure {
      rethrow;
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

  Future<String?> _renderAndPersist({
    required LibraryDocument document,
    required String engineDocumentId,
    required int pageIndex,
    required double confidence,
    required CoverSelectionMode selectionMode,
    PdfOperationToken? cancellationToken,
  }) async {
    final previousMetadata = document.coverMetadata;
    final cacheStamp = DateTime.now().microsecondsSinceEpoch;
    final smallPath = p.join(
      _paths.thumbnailDirectory.path,
      '${document.id}-v${CoverSelectionPolicy.algorithmVersion}'
      '-$cacheStamp-small.png',
    );
    final largePath = p.join(
      _paths.thumbnailDirectory.path,
      '${document.id}-v${CoverSelectionPolicy.algorithmVersion}'
      '-$cacheStamp-large.png',
    );
    final renders = <({double scale, String path})>[
      (scale: smallCoverScale, path: smallPath),
      (scale: largeCoverScale, path: largePath),
    ];
    var pageAspectRatio = 0.704;
    var persisted = false;
    try {
      for (final target in renders) {
        if (cancellationToken?.isCancelled ?? false) {
          throw const AppFailure(
            FailureKind.cancelled,
            'Generación de portada cancelada.',
          );
        }
        final rendered = await _engine.renderPage(
          documentId: engineDocumentId,
          pageIndex: pageIndex,
          scale: target.scale,
          cancellationToken: cancellationToken,
        );
        pageAspectRatio = rendered.image.width / rendered.image.height;
        final bytes = await _pngBytes(rendered.image);
        if (bytes == null) return null;
        await File(target.path).writeAsBytes(bytes, flush: true);
      }
      if (cancellationToken?.isCancelled ?? false) {
        throw const AppFailure(
          FailureKind.cancelled,
          'Generación de portada cancelada.',
        );
      }
      final metadata = DocumentCoverMetadata(
        documentId: document.id,
        sourcePageIndex: pageIndex,
        smallThumbnailPath: smallPath,
        largeThumbnailPath: largePath,
        generatedAt: DateTime.now().toUtc(),
        sourceModifiedAt: document.fileModifiedAt,
        confidence: confidence,
        selectionMode: selectionMode,
        algorithmVersion: CoverSelectionPolicy.algorithmVersion,
        cropMode: CoverCropMode.contain,
        pageAspectRatio: pageAspectRatio.clamp(0.1, 10),
      );
      _documents.saveCover(metadata);
      await _deleteCoverFiles(previousMetadata);
      await trimToLimit();
      onCoverUpdated?.call();
      persisted = true;
      return smallPath;
    } finally {
      if (!persisted) {
        for (final path in [smallPath, largePath]) {
          final file = File(path);
          if (await file.exists()) await file.delete();
        }
      }
    }
  }

  Future<CoverCandidateMetrics> _measure(
    ui.Image image,
    int pageIndex,
    String text,
  ) async {
    final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    if (data == null || image.width == 0 || image.height == 0) {
      return CoverCandidateMetrics(
        pageIndex: pageIndex,
        uniformSpaceRatio: 1,
        edgeDensity: 0,
        luminanceContrast: 0,
        textLength: text.trim().length,
        lineCount: _lineCount(text),
        hasContentsMarker: _hasContentsMarker(text),
      );
    }
    final bytes = data.buffer.asUint8List(
      data.offsetInBytes,
      data.lengthInBytes,
    );
    final stepX = math.max(1, image.width ~/ 48);
    final stepY = math.max(1, image.height ~/ 48);
    var samples = 0;
    var uniformSamples = 0;
    var edges = 0;
    var sum = 0.0;
    var squareSum = 0.0;
    for (var y = 0; y < image.height; y += stepY) {
      double? previous;
      for (var x = 0; x < image.width; x += stepX) {
        final offset = (y * image.width + x) * 4;
        final luminance =
            (bytes[offset] * 0.2126 +
                bytes[offset + 1] * 0.7152 +
                bytes[offset + 2] * 0.0722) /
            255;
        samples++;
        sum += luminance;
        squareSum += luminance * luminance;
        if (luminance >= 0.94 || luminance <= 0.06) uniformSamples++;
        if (previous != null && (luminance - previous).abs() > 0.16) edges++;
        previous = luminance;
      }
    }
    final mean = sum / samples;
    final variance = math.max(0, squareSum / samples - mean * mean);
    return CoverCandidateMetrics(
      pageIndex: pageIndex,
      uniformSpaceRatio: uniformSamples / samples,
      edgeDensity: edges / samples,
      luminanceContrast: math.sqrt(variance),
      textLength: text.trim().length,
      lineCount: _lineCount(text),
      hasContentsMarker: _hasContentsMarker(text),
    );
  }

  Future<void> clear() async {
    _coverQueueGeneration++;
    for (final token in _activeCoverTokens.values) {
      token.cancel();
    }
    for (final job in _activeCoverJobs.values) {
      job.cancel();
    }
    _activeCoverTokens.clear();
    _activeCoverJobs.clear();
    _coverGenerations.clear();
    final directory = _paths.thumbnailDirectory;
    if (await directory.exists()) {
      await for (final entity in directory.list(followLinks: false)) {
        if (entity is File) await entity.delete();
      }
    }
    _documents.clearThumbnailPaths();
  }

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

  static Future<Uint8List?> _pngBytes(ui.Image image) async {
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    return data?.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes);
  }

  static int _lineCount(String text) =>
      text.trim().isEmpty ? 0 : text.trim().split(RegExp(r'\r?\n')).length;

  static bool _hasContentsMarker(String text) {
    final normalized = text.toLowerCase();
    return RegExp(
      r'\b(índice|indice|contenido|contents|table of contents|chapter|capítulo)\b',
    ).hasMatch(normalized);
  }

  static Future<bool> _coverFilesExist(DocumentCoverMetadata metadata) async =>
      File(metadata.smallThumbnailPath).existsSync() &&
      File(metadata.largeThumbnailPath).existsSync();

  static Future<void> _deleteCoverFiles(DocumentCoverMetadata? metadata) async {
    if (metadata == null) return;
    for (final path in {
      metadata.smallThumbnailPath,
      metadata.largeThumbnailPath,
    }) {
      final file = File(path);
      if (await file.exists()) await file.delete();
    }
  }
}
