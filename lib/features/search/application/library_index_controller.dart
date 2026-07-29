import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/global_search_repository.dart';
import '../domain/global_search_models.dart';

final class LibraryIndexController extends ChangeNotifier {
  LibraryIndexController(this._documents, this._search, this._engine);

  final DocumentRepository _documents;
  final GlobalSearchRepository _search;
  final PdfDocumentEngine _engine;

  bool enabled = false;
  bool _cancelled = false;
  bool _paused = false;
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
    _cancelled = false;
    final candidates = _documents
        .all()
        .where(
          (document) =>
              !document.isMissing &&
              !excludedDocumentIds.contains(document.id) &&
              _search.needsIndex(document),
        )
        .toList(growable: false);
    progress = LibraryIndexProgress(
      totalDocuments: candidates.length,
      completedDocuments: 0,
      currentPage: 0,
      totalPages: 0,
      isRunning: true,
      isPaused: false,
    );
    notifyListeners();
    var completed = 0;
    for (final document in candidates) {
      if (_cancelled) break;
      while (_paused && !_cancelled) {
        await Future<void>.delayed(const Duration(milliseconds: 80));
      }
      String? engineDocumentId;
      try {
        _search.beginDocument(document);
        final opened = await _engine.openDocument(document.path);
        engineDocumentId = opened.documentId;
        var hasText = false;
        for (var pageIndex = 0; pageIndex < opened.pageCount; pageIndex++) {
          if (_cancelled) break;
          while (_paused && !_cancelled) {
            await Future<void>.delayed(const Duration(milliseconds: 80));
          }
          final text = await _engine.extractPageText(
            documentId: opened.documentId,
            pageIndex: pageIndex,
          );
          if (text.trim().isNotEmpty) {
            hasText = true;
            _search.addPage(
              document: document,
              pageIndex: pageIndex,
              text: text,
            );
          }
          progress = LibraryIndexProgress(
            totalDocuments: candidates.length,
            completedDocuments: completed,
            currentPage: pageIndex + 1,
            totalPages: opened.pageCount,
            isRunning: true,
            isPaused: _paused,
          );
          notifyListeners();
          await Future<void>.delayed(Duration.zero);
        }
        if (!_cancelled) {
          _search.finishDocument(document.id, hasExtractableText: hasText);
        }
      } on AppFailure catch (failure) {
        _search.failDocument(document.id, failure.kind.name);
      } on Object {
        _search.failDocument(document.id, 'unknown');
      } finally {
        if (engineDocumentId != null) {
          await _engine.closeDocument(engineDocumentId);
        }
      }
      completed++;
      progress = LibraryIndexProgress(
        totalDocuments: candidates.length,
        completedDocuments: completed,
        currentPage: 0,
        totalPages: 0,
        isRunning: true,
        isPaused: _paused,
      );
      notifyListeners();
    }
    _search
      ..refreshAnnotationEntries()
      ..refreshOrganizationEntries();
    progress = LibraryIndexProgress(
      totalDocuments: candidates.length,
      completedDocuments: completed,
      currentPage: 0,
      totalPages: 0,
      isRunning: false,
      isPaused: false,
    );
    notifyListeners();
  }

  void setEnabled(bool value) {
    enabled = value;
    if (!value) cancel();
    notifyListeners();
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

  void cancel() {
    _cancelled = true;
    _paused = false;
  }

  void clearIndex() {
    cancel();
    _search.clear();
    notifyListeners();
  }
}
