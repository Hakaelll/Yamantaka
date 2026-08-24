// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../../../core/async/job_coordinator.dart';
import '../../../core/logging/app_logger.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/file_validation.dart';

/// Validates the library incrementally without putting file-system work on the
/// library query path. SQLite always remains the immediate source of the last
/// known state.
final class LibraryFileValidationController extends ChangeNotifier {
  LibraryFileValidationController({
    required DocumentRepository documents,
    required JobCoordinator jobs,
    required AppLogger logger,
    this.batchSize = 24,
    this.onDocumentsChanged,
  }) : assert(batchSize > 0 && batchSize <= 100),
       _documents = documents,
       _jobs = jobs,
       _logger = logger;

  final DocumentRepository _documents;
  final JobCoordinator _jobs;
  final AppLogger _logger;
  final int batchSize;
  final FutureOr<void> Function(Set<String> documentIds)? onDocumentsChanged;

  FileValidationProgress progress = const FileValidationProgress.idle();
  JobHandle<FileValidationSummary>? _handle;
  int _generation = 0;
  String? _resumeAfterId;
  bool _backgroundPaused = false;
  bool _readerPaused = false;
  bool _disposed = false;

  bool get isPaused => _backgroundPaused || _readerPaused;

  Future<FileValidationSummary> checkAll() => _start(manual: true);

  void startIncremental() {
    if (_disposed || isPaused || progress.isRunning) return;
    unawaited(_start(manual: false));
  }

  void setBackgroundPaused(bool value) {
    if (_backgroundPaused == value) return;
    _backgroundPaused = value;
    if (value) {
      cancel();
    } else {
      startIncremental();
    }
  }

  void setReaderPaused(bool value) {
    if (_readerPaused == value) return;
    _readerPaused = value;
    if (value) {
      cancel();
    } else {
      startIncremental();
    }
  }

  void cancel() {
    _generation++;
    _handle?.cancel();
    _handle = null;
    if (progress.isRunning) {
      final remaining = (progress.total - progress.summary.checked).clamp(
        0,
        progress.total,
      );
      progress = FileValidationProgress(
        isRunning: false,
        isManual: progress.isManual,
        summary: progress.summary.copyWith(cancelled: remaining),
        total: progress.total,
      );
      notifyListeners();
    }
  }

  Future<FileValidationSummary> _start({required bool manual}) async {
    if (_disposed) return const FileValidationSummary();
    if (isPaused) {
      final total = _documents.fileValidationCount;
      return FileValidationSummary(cancelled: total);
    }
    cancel();
    final generation = ++_generation;
    final total = _documents.fileValidationCount;
    final initialAfterId = manual ? null : _resumeAfterId;
    progress = FileValidationProgress(
      isRunning: true,
      isManual: manual,
      summary: const FileValidationSummary(),
      total: total,
    );
    notifyListeners();

    final handle = _jobs.submit<FileValidationSummary>(
      key: manual ? 'manual-library-validation' : 'library-validation',
      category: JobCategory.maintenance,
      priority: manual ? JobPriority.interactive : JobPriority.idle,
      generation: generation,
      task: (token) => _run(
        token,
        generation: generation,
        total: total,
        initialAfterId: initialAfterId,
        manual: manual,
      ),
    );
    _handle = handle;
    try {
      final summary = await handle.future;
      if (_disposed || generation != _generation) {
        return summary.copyWith(
          cancelled: (total - summary.checked).clamp(0, total),
        );
      }
      _handle = null;
      _resumeAfterId = null;
      progress = FileValidationProgress(
        isRunning: false,
        isManual: manual,
        summary: summary,
        total: total,
      );
      notifyListeners();
      return summary;
    } on JobCancelled {
      final summary = progress.summary.copyWith(
        cancelled: (total - progress.summary.checked).clamp(0, total),
      );
      if (!_disposed && generation == _generation) {
        _handle = null;
        progress = FileValidationProgress(
          isRunning: false,
          isManual: manual,
          summary: summary,
          total: total,
        );
        notifyListeners();
      }
      return summary;
    } on Object catch (error, stackTrace) {
      _logger.error('Library file validation failed.', error, stackTrace);
      final summary = progress.summary.copyWith(
        failed: progress.summary.failed + 1,
        cancelled: (total - progress.summary.checked - 1).clamp(0, total),
      );
      if (!_disposed && generation == _generation) {
        _handle = null;
        progress = FileValidationProgress(
          isRunning: false,
          isManual: manual,
          summary: summary,
          total: total,
        );
        notifyListeners();
      }
      return summary;
    }
  }

  Future<FileValidationSummary> _run(
    JobCancellationToken token, {
    required int generation,
    required int total,
    required String? initialAfterId,
    required bool manual,
  }) async {
    var summary = const FileValidationSummary();
    var afterId = initialAfterId;
    while (!token.isCancelled && generation == _generation) {
      final batch = _documents.fileValidationBatch(
        afterId: afterId,
        limit: batchSize,
      );
      if (batch.isEmpty) break;
      final results = <FileValidationResult>[];
      for (final candidate in batch) {
        if (token.isCancelled || generation != _generation) break;
        final status = await _validate(candidate);
        if (token.isCancelled || generation != _generation) break;
        results.add(FileValidationResult(id: candidate.id, status: status));
        summary = summary.add(status);
      }
      if (token.isCancelled || generation != _generation) break;
      final changed = _documents.applyFileValidationBatch(results);
      afterId = batch.last.id;
      _resumeAfterId = afterId;
      if (changed.isNotEmpty && onDocumentsChanged != null) {
        await onDocumentsChanged!(changed);
      }
      if (!_disposed && generation == _generation) {
        progress = FileValidationProgress(
          isRunning: true,
          isManual: manual,
          summary: summary,
          total: total,
        );
        notifyListeners();
      }
      // Yield after every small transaction so input and rendering can run.
      await Future<void>.delayed(Duration.zero);
    }
    return summary;
  }

  static Future<FileValidationStatus> _validate(
    FileValidationCandidate candidate,
  ) async {
    try {
      final stat = await File(candidate.path).stat();
      if (stat.type == FileSystemEntityType.notFound) {
        return FileValidationStatus.missing;
      }
      final modified =
          stat.size != candidate.fileSize ||
          stat.modified.toUtc() != candidate.fileModifiedAt.toUtc();
      return modified
          ? FileValidationStatus.modified
          : FileValidationStatus.available;
    } on FileSystemException {
      return FileValidationStatus.failed;
    } on Object {
      return FileValidationStatus.failed;
    }
  }

  @override
  void dispose() {
    _disposed = true;
    cancel();
    super.dispose();
  }
}
