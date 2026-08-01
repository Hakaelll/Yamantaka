import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../../../core/async/job_coordinator.dart';
import '../../../core/errors/app_failure.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/cache/thumbnail_service.dart';
import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/import_queue_repository.dart';
import '../domain/import_queue.dart';
import 'document_service.dart';

enum FolderCandidateStatus { newDocument, duplicate, inaccessible, invalid }

final class FolderImportCandidate {
  const FolderImportCandidate({
    required this.path,
    required this.fileName,
    required this.status,
    this.inspected,
    this.errorCode,
  });

  final String path;
  final String fileName;
  final FolderCandidateStatus status;
  final InspectedFile? inspected;
  final String? errorCode;
}

final class FolderImportSummary {
  const FolderImportSummary({
    required this.found,
    required this.imported,
    required this.duplicates,
    required this.inaccessible,
    required this.invalid,
    required this.cancelled,
  });

  final int found;
  final int imported;
  final int duplicates;
  final int inaccessible;
  final int invalid;
  final bool cancelled;
}

final class FolderImportController extends ChangeNotifier {
  FolderImportController(
    this._documents,
    this._documentService,
    this._engine, [
    this._thumbnails,
    this._jobs,
    ImportQueueRepository? queueRepository,
  ]) : _queueRepository = queueRepository,
       queue = queueRepository?.load() ?? ImportQueue(maxPending: 5000);

  final DocumentRepository _documents;
  final DocumentService _documentService;
  final PdfDocumentEngine _engine;
  final ThumbnailService? _thumbnails;
  final JobCoordinator? _jobs;
  final ImportQueueRepository? _queueRepository;

  List<FolderImportCandidate> candidates = const [];
  final ImportQueue queue;
  bool isScanning = false;
  bool isImporting = false;
  int processed = 0;
  int total = 0;
  bool _cancelled = false;
  int _generation = 0;

  Future<List<FolderImportCandidate>> scan(
    String directoryPath, {
    bool includeSubdirectories = true,
  }) async {
    if (isScanning || isImporting) return candidates;
    final jobs = _jobs;
    if (jobs != null) {
      final generation = ++_generation;
      final handle = jobs.submit<List<FolderImportCandidate>>(
        key: 'folder-scan',
        category: JobCategory.import,
        priority: JobPriority.interactive,
        generation: generation,
        task: (token) async {
          void cancel() => _cancelled = true;
          token.addListener(cancel);
          try {
            return await _scanInternal(
              directoryPath,
              includeSubdirectories: includeSubdirectories,
            );
          } finally {
            token.removeListener(cancel);
          }
        },
      );
      try {
        return await handle.future;
      } on JobCancelled {
        return candidates;
      }
    }
    return _scanInternal(
      directoryPath,
      includeSubdirectories: includeSubdirectories,
    );
  }

  Future<List<FolderImportCandidate>> _scanInternal(
    String directoryPath, {
    required bool includeSubdirectories,
  }) async {
    if (isScanning || isImporting) return candidates;
    _cancelled = false;
    isScanning = true;
    processed = 0;
    candidates = const [];
    notifyListeners();
    final directory = Directory(directoryPath);
    final paths = <String>[];
    try {
      await for (final entity in directory.list(
        recursive: includeSubdirectories,
        followLinks: false,
      )) {
        if (_cancelled) break;
        if (entity is File && entity.path.toLowerCase().endsWith('.pdf')) {
          paths.add(entity.path);
          if (paths.length >= 5000) break;
        }
      }
    } on FileSystemException {
      isScanning = false;
      notifyListeners();
      return candidates;
    }
    total = paths.length;
    final results = <FolderImportCandidate>[];
    for (final path in paths) {
      if (_cancelled) break;
      try {
        final inspected = await _documentService.inspect(path);
        final duplicate =
            _documents.byNormalizedPath(inspected.normalizedPath) != null ||
            _documents.byFingerprint(inspected.fingerprint) != null;
        results.add(
          FolderImportCandidate(
            path: path,
            fileName: inspected.fileName,
            status: duplicate
                ? FolderCandidateStatus.duplicate
                : FolderCandidateStatus.newDocument,
            inspected: inspected,
          ),
        );
      } on AppFailure catch (failure) {
        results.add(
          FolderImportCandidate(
            path: path,
            fileName: File(path).uri.pathSegments.last,
            status: failure.kind == FailureKind.accessDenied
                ? FolderCandidateStatus.inaccessible
                : FolderCandidateStatus.invalid,
            errorCode: failure.kind.name,
          ),
        );
      } on Object {
        results.add(
          FolderImportCandidate(
            path: path,
            fileName: File(path).uri.pathSegments.last,
            status: FolderCandidateStatus.invalid,
            errorCode: 'unknown',
          ),
        );
      }
      processed++;
      candidates = List.unmodifiable(results);
      notifyListeners();
      await Future<void>.delayed(Duration.zero);
    }
    isScanning = false;
    notifyListeners();
    return candidates;
  }

  Future<FolderImportSummary> importNew() async {
    if (isScanning || isImporting) {
      return _summary(imported: 0, cancelled: _cancelled);
    }
    _cancelled = false;
    isImporting = true;
    processed = 0;
    final pending = <String, FolderImportCandidate>{};
    for (final candidate in candidates) {
      final inspected = candidate.inspected;
      if (candidate.status != FolderCandidateStatus.newDocument ||
          inspected == null) {
        continue;
      }
      final id = inspected.normalizedPath;
      queue.enqueue(
        ImportQueueItem(
          id: id,
          path: candidate.path,
          normalizedPath: inspected.normalizedPath,
          fingerprint: inspected.fingerprint,
        ),
      );
      pending[id] = candidate;
      _queueRepository?.save(queue);
    }
    total = queue.pendingCount;
    notifyListeners();
    var imported = 0;
    while (!_cancelled) {
      final work = queue.claimNext();
      if (work == null) break;
      _queueRepository?.save(queue);
      final candidate = pending[work.id];
      InspectedFile? inspected = candidate?.inspected;
      if (inspected == null) {
        try {
          inspected = await _documentService.inspect(work.path);
        } on Object {
          queue.fail(work.id, 'candidate_unavailable', retryable: false);
          _queueRepository?.save(queue);
          continue;
        }
      }
      if (_cancelled) break;
      if (work.fingerprint != null &&
          work.fingerprint!.isNotEmpty &&
          inspected.fingerprint != work.fingerprint) {
        queue.fail(work.id, 'file_changed', retryable: false);
        _queueRepository?.save(queue);
        continue;
      }
      String? engineDocumentId;
      try {
        final opened = await _engine.openDocument(candidate?.path ?? work.path);
        engineDocumentId = opened.documentId;
        final document = _documentService.register(
          file: inspected,
          pageCount: opened.pageCount,
        );
        _thumbnails?.enqueueCover(document);
        queue.complete(work.id);
        _queueRepository?.save(queue);
        imported++;
      } on Object {
        // The per-file result remains available in the final summary.
        queue.fail(work.id, 'import_failed');
        _queueRepository?.save(queue);
      } finally {
        if (engineDocumentId != null) {
          await _engine.closeDocument(engineDocumentId);
        }
      }
      processed++;
      notifyListeners();
      await Future<void>.delayed(Duration.zero);
    }
    if (_cancelled) {
      queue.prepareForResume();
      _queueRepository?.save(queue);
    }
    isImporting = false;
    notifyListeners();
    return _summary(imported: imported, cancelled: _cancelled);
  }

  void cancel() {
    _cancelled = true;
    queue.cancelPending();
    _queueRepository?.save(queue);
  }

  FolderImportSummary _summary({
    required int imported,
    required bool cancelled,
  }) => FolderImportSummary(
    found: candidates.length,
    imported: imported,
    duplicates: candidates
        .where((item) => item.status == FolderCandidateStatus.duplicate)
        .length,
    inaccessible: candidates
        .where((item) => item.status == FolderCandidateStatus.inaccessible)
        .length,
    invalid: candidates
        .where((item) => item.status == FolderCandidateStatus.invalid)
        .length,
    cancelled: cancelled,
  );
}
