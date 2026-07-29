import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../storage/repositories/document_repository.dart';
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
  FolderImportController(this._documents, this._documentService, this._engine);

  final DocumentRepository _documents;
  final DocumentService _documentService;
  final PdfDocumentEngine _engine;

  List<FolderImportCandidate> candidates = const [];
  bool isScanning = false;
  bool isImporting = false;
  int processed = 0;
  int total = 0;
  bool _cancelled = false;

  Future<List<FolderImportCandidate>> scan(
    String directoryPath, {
    bool includeSubdirectories = true,
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
    final pending = candidates
        .where(
          (candidate) =>
              candidate.status == FolderCandidateStatus.newDocument &&
              candidate.inspected != null,
        )
        .toList(growable: false);
    total = pending.length;
    notifyListeners();
    var imported = 0;
    for (final candidate in pending) {
      if (_cancelled) break;
      String? engineDocumentId;
      try {
        final opened = await _engine.openDocument(candidate.path);
        engineDocumentId = opened.documentId;
        _documentService.register(
          file: candidate.inspected!,
          pageCount: opened.pageCount,
        );
        imported++;
      } on Object {
        // The per-file result remains available in the final summary.
      } finally {
        if (engineDocumentId != null) {
          await _engine.closeDocument(engineDocumentId);
        }
      }
      processed++;
      notifyListeners();
      await Future<void>.delayed(Duration.zero);
    }
    isImporting = false;
    notifyListeners();
    return _summary(imported: imported, cancelled: _cancelled);
  }

  void cancel() {
    _cancelled = true;
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
