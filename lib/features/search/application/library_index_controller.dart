import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../core/async/job_coordinator.dart';
import '../../../core/errors/app_failure.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/global_search_repository.dart';
import '../domain/global_search_models.dart';

final class LibraryIndexController extends ChangeNotifier {
  LibraryIndexController(
    this._documents,
    this._search,
    this._engine, [
    this._jobs,
  ]);

  final DocumentRepository _documents;
  final GlobalSearchRepository _search;
  final PdfDocumentEngine _engine;
  final JobCoordinator? _jobs;

  bool enabled = false;
  bool _cancelled = false;
  bool _paused = false;
  bool _backgroundPaused = false;
  bool _interactionPaused = false;
  final List<String> _priorityDocumentIds = [];
  final Map<String, int> _priorityPages = {};
  bool get _effectivelyPaused =>
      _paused || _backgroundPaused || _interactionPaused;
  int _generation = 0;
  LibraryIndexProgress progress = const LibraryIndexProgress(
    totalDocuments: 0,
    completedDocuments: 0,
    currentPage: 0,
    totalPages: 0,
    isRunning: false,
    isPaused: false,
  );

  Future<void> rebuild({Set<String> excludedDocumentIds = const {}}) async {
    if (progress.isRunning) return;
    _search.clear();
    await indexIncremental(excludedDocumentIds: excludedDocumentIds);
  }

  Future<void> indexIncremental({
    Set<String> excludedDocumentIds = const {},
  }) async {
    if (progress.isRunning || !enabled) return;
    final generation = ++_generation;
    final jobs = _jobs;
    if (jobs != null) {
      final handle = jobs.submit<void>(
        key: 'library-index',
        category: JobCategory.indexing,
        priority: JobPriority.background,
        generation: generation,
        task: (token) async {
          void cancel() => _cancelled = true;
          token.addListener(cancel);
          try {
            await _indexIncrementalInternal(
              excludedDocumentIds: excludedDocumentIds,
              jobToken: token,
            );
          } finally {
            token.removeListener(cancel);
          }
        },
      );
      try {
        await handle.future;
      } on JobCancelled {
        // The controller already publishes its final non-running state.
      }
      return;
    }
    await _indexIncrementalInternal(excludedDocumentIds: excludedDocumentIds);
  }

  Future<void> _indexIncrementalInternal({
    required Set<String> excludedDocumentIds,
    JobCancellationToken? jobToken,
  }) async {
    _cancelled = false;
    final operationToken = PdfOperationToken();
    void cancelOperation() => operationToken.cancel();
    jobToken?.addListener(cancelOperation);
    try {
      final candidates = _documents
          .all()
          .where(
            (document) =>
                !document.isMissing &&
                !excludedDocumentIds.contains(document.id) &&
                _search.needsIndex(document),
          )
          .toList();
      final totalCandidates = candidates.length;
      progress = LibraryIndexProgress(
        totalDocuments: totalCandidates,
        completedDocuments: 0,
        currentPage: 0,
        totalPages: 0,
        isRunning: true,
        isPaused: _effectivelyPaused,
      );
      notifyListeners();
      var completed = 0;
      while (candidates.isNotEmpty) {
        if (_cancelled || operationToken.isCancelled) break;
        var nextDocumentIndex = 0;
        for (final priorityId in _priorityDocumentIds) {
          final candidateIndex = candidates.indexWhere(
            (candidate) => candidate.id == priorityId,
          );
          if (candidateIndex >= 0) {
            nextDocumentIndex = candidateIndex;
            break;
          }
        }
        final document = candidates.removeAt(nextDocumentIndex);
        while (_effectivelyPaused &&
            !_cancelled &&
            !operationToken.isCancelled) {
          await Future<void>.delayed(const Duration(milliseconds: 80));
        }
        String? engineDocumentId;
        try {
          final opened = await _engine.openDocument(document.path);
          engineDocumentId = opened.documentId;
          final storedCursor = _search.resumableCursor(document);
          var cursor = storedCursor ?? 0;
          final canResume = storedCursor != null && cursor <= opened.pageCount;
          if (!canResume) cursor = 0;
          _search.beginDocument(document, resume: canResume);
          final indexedPages = canResume
              ? _search.indexedPageIndexes(document.id)
              : <int>{};
          var hasText = indexedPages.isNotEmpty;
          final completedPages = <int>{
            for (var page = 0; page < cursor; page++) page,
            ...indexedPages,
          };
          final remainingPages = List<int>.generate(
            opened.pageCount,
            (index) => index,
          ).where((index) => !completedPages.contains(index)).toList();
          while (remainingPages.isNotEmpty) {
            if (_cancelled || operationToken.isCancelled) break;
            final prioritizedPage = _priorityPages[document.id];
            final prioritizedIndex = prioritizedPage == null
                ? -1
                : remainingPages.indexOf(prioritizedPage);
            final pageIndex = remainingPages.removeAt(
              prioritizedIndex < 0 ? 0 : prioritizedIndex,
            );
            while (_effectivelyPaused &&
                !_cancelled &&
                !operationToken.isCancelled) {
              await Future<void>.delayed(const Duration(milliseconds: 80));
            }
            final text = await _engine.extractPageText(
              documentId: opened.documentId,
              pageIndex: pageIndex,
              cancellationToken: operationToken,
            );
            if (text.trim().isNotEmpty) {
              hasText = true;
              _search.addPage(
                document: document,
                pageIndex: pageIndex,
                text: text,
              );
            }
            completedPages.add(pageIndex);
            while (completedPages.contains(cursor)) {
              cursor++;
            }
            _search.updateCursor(document.id, cursor);
            progress = LibraryIndexProgress(
              totalDocuments: totalCandidates,
              completedDocuments: completed,
              currentPage: pageIndex + 1,
              totalPages: opened.pageCount,
              isRunning: true,
              isPaused: _effectivelyPaused,
            );
            notifyListeners();
            await Future<void>.delayed(Duration.zero);
          }
          if (!_cancelled && !operationToken.isCancelled) {
            _search.finishDocument(document.id, hasExtractableText: hasText);
          }
        } on AppFailure catch (failure) {
          if (!_cancelled &&
              !operationToken.isCancelled &&
              failure.kind != FailureKind.cancelled) {
            _search.failDocument(document.id, failure.kind.name);
          }
        } on Object {
          if (!_cancelled && !operationToken.isCancelled) {
            _search.failDocument(document.id, 'unknown');
          }
        } finally {
          if (engineDocumentId != null) {
            await _engine.closeDocument(engineDocumentId);
          }
        }
        completed++;
        progress = LibraryIndexProgress(
          totalDocuments: totalCandidates,
          completedDocuments: completed,
          currentPage: 0,
          totalPages: 0,
          isRunning: true,
          isPaused: _effectivelyPaused,
        );
        notifyListeners();
      }
      _search
        ..refreshAnnotationEntries()
        ..refreshBookmarkEntries()
        ..refreshOrganizationEntries();
      progress = LibraryIndexProgress(
        totalDocuments: totalCandidates,
        completedDocuments: completed,
        currentPage: 0,
        totalPages: 0,
        isRunning: false,
        isPaused: false,
      );
      notifyListeners();
    } finally {
      jobToken?.removeListener(cancelOperation);
    }
  }

  void setEnabled(bool value) {
    enabled = value;
    if (!value) cancel();
    notifyListeners();
  }

  /// Moves visible work to the front without interrupting the current native
  /// operation. The next document and, when known, its visible page are
  /// indexed first; the bounded list avoids retaining the whole viewport.
  void prioritizeDocument(String documentId, {int? pageIndex}) {
    _priorityDocumentIds
      ..remove(documentId)
      ..insert(0, documentId);
    if (_priorityDocumentIds.length > 24) {
      final removed = _priorityDocumentIds.removeLast();
      _priorityPages.remove(removed);
    }
    if (pageIndex != null && pageIndex >= 0) {
      _priorityPages[documentId] = pageIndex;
    }
  }

  /// Adds pinned or recently used documents behind the visible/active item.
  /// This secondary queue remains bounded and contains identifiers only.
  void prioritizeSecondary(Iterable<String> documentIds) {
    for (final documentId in documentIds) {
      if (_priorityDocumentIds.contains(documentId)) continue;
      _priorityDocumentIds.add(documentId);
      if (_priorityDocumentIds.length > 24) {
        final removed = _priorityDocumentIds.removeLast();
        _priorityPages.remove(removed);
      }
    }
  }

  void pause() {
    if (!progress.isRunning) return;
    _paused = true;
    progress = LibraryIndexProgress(
      totalDocuments: progress.totalDocuments,
      completedDocuments: progress.completedDocuments,
      currentPage: progress.currentPage,
      totalPages: progress.totalPages,
      isRunning: true,
      isPaused: true,
    );
    notifyListeners();
  }

  void resume() {
    _paused = false;
    notifyListeners();
  }

  /// Pauses only because the application is hidden or inactive. A manual
  /// pause remains independent and is not cleared by lifecycle changes.
  void setBackgroundPaused(bool value) {
    _backgroundPaused = value;
    if (progress.isRunning) {
      progress = LibraryIndexProgress(
        totalDocuments: progress.totalDocuments,
        completedDocuments: progress.completedDocuments,
        currentPage: progress.currentPage,
        totalPages: progress.totalPages,
        isRunning: true,
        isPaused: _effectivelyPaused,
      );
      notifyListeners();
    }
  }

  /// Pauses recreable indexing while the reader is zooming or scrolling.
  void setInteractionPaused(bool value) {
    if (_interactionPaused == value) return;
    _interactionPaused = value;
    if (progress.isRunning) {
      progress = LibraryIndexProgress(
        totalDocuments: progress.totalDocuments,
        completedDocuments: progress.completedDocuments,
        currentPage: progress.currentPage,
        totalPages: progress.totalPages,
        isRunning: true,
        isPaused: _effectivelyPaused,
      );
      notifyListeners();
    }
  }

  void cancel() {
    _cancelled = true;
    _paused = false;
  }

  void clearIndex() {
    cancel();
    _search.clear();
    notifyListeners();
  }

  void refreshMetadata(String documentId) {
    final document = _documents.byId(documentId);
    if (document == null) return;
    _search.refreshMetadataEntry(document);
  }
}
