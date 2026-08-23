import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/library_document.dart';
import '../domain/library_query.dart';
import '../domain/document_enrichment.dart';

final class LibraryController extends ChangeNotifier {
  LibraryController(this._repository, this._logger);

  final DocumentRepository _repository;
  final AppLogger _logger;

  List<LibraryDocument> documents = const [];
  int totalDocuments = 0;
  bool hasMore = false;
  bool isLoading = false;
  bool isLoadingMore = false;
  AppFailure? error;
  String? selectedDocumentId;
  double scrollOffset = 0;
  String query = '';
  String sortOrder = 'lastOpened';
  String readingState = 'all';
  bool favoritesOnly = false;
  Set<String> selectedAuthors = const {};
  Set<LibraryTopic> selectedTopics = const {};
  bool groupByAuthor = false;
  List<LibraryFacet> authorFacets = const [];
  Map<LibraryTopic, int> topicFacets = const {};
  Map<LibraryQueryReadingState, int> readingStateCounts = const {};
  LibraryQuery? _activeQuery;
  int _loadGeneration = 0;

  LibraryDocument? get continueReading => documents
      .where(
        (document) =>
            !document.isMissing &&
            document.readingProgress > 0 &&
            document.readingProgress < 1,
      )
      .firstOrNull;

  Future<void> load({LibraryQuery? query}) async {
    final requested = (query ?? _queryFromState()).normalized();
    _activeQuery = requested;
    final generation = ++_loadGeneration;
    final append = requested.offset > 0 && documents.isNotEmpty;
    if (append) {
      isLoadingMore = true;
    } else {
      isLoading = true;
      isLoadingMore = false;
    }
    error = null;
    notifyListeners();
    try {
      _repository.refreshMissingState();
      final page = _repository.page(requested);
      if (generation != _loadGeneration) return;
      documents = append
          ? List.unmodifiable([...documents, ...page.documents])
          : page.documents;
      totalDocuments = page.total;
      hasMore = page.hasMore;
      authorFacets = _repository.authorFacets(requested);
      topicFacets = _repository.topicFacets(requested);
      readingStateCounts = _repository.readingStateCounts(requested);
      if (selectedDocumentId != null &&
          _repository.byId(selectedDocumentId!) == null) {
        selectedDocumentId = null;
      }
      if (selectedDocumentId == null && documents.length == 1) {
        selectedDocumentId = documents.single.id;
      }
    } on Object catch (exception, stackTrace) {
      _logger.error('Library load failed.', exception, stackTrace);
      error = AppFailure(
        FailureKind.databaseFailed,
        'No se pudo cargar la biblioteca local.',
        debugMessage: exception.toString(),
      );
    } finally {
      if (generation == _loadGeneration) {
        if (append) {
          isLoadingMore = false;
        } else {
          isLoading = false;
        }
        notifyListeners();
      }
    }
  }

  Future<void> reload() => load(query: _queryFromState());

  Future<void> loadNextPage() {
    final active = _activeQuery;
    if (active == null || !hasMore || isLoading || isLoadingMore) {
      return Future<void>.value();
    }
    return load(query: active.copyWith(offset: active.offset + active.limit));
  }

  void setQuery(String value) {
    query = value;
    unawaited(reload());
  }

  Future<void> setFacets({
    required Set<String> authors,
    required Set<LibraryTopic> topics,
    required bool groupAuthors,
  }) {
    selectedAuthors = {
      for (final author in authors) author.trim().toLowerCase(),
    };
    selectedTopics = Set.unmodifiable(topics);
    groupByAuthor = groupAuthors;
    return reload();
  }

  LibraryQuery _queryFromState() => LibraryQuery(
    text: query,
    favoritesOnly: favoritesOnly,
    authors: selectedAuthors,
    topics: selectedTopics,
    readingState: switch (readingState) {
      'inProgress' => LibraryQueryReadingState.inProgress,
      'notStarted' => LibraryQueryReadingState.notStarted,
      'completed' => LibraryQueryReadingState.completed,
      _ => LibraryQueryReadingState.all,
    },
    sort: switch (sortOrder) {
      'added' => LibraryQuerySort.added,
      'title' => LibraryQuerySort.title,
      'progress' => LibraryQuerySort.progress,
      _ => LibraryQuerySort.lastOpened,
    },
  );

  Future<void> deleteHistory() async {
    _repository.deleteHistory();
    await load();
  }

  void selectDocument(String? documentId) {
    if (selectedDocumentId == documentId) return;
    selectedDocumentId = documentId;
    notifyListeners();
  }

  Future<void> toggleFavorite(String documentId) async {
    _repository.toggleFavorite(documentId);
    await load();
  }
}
