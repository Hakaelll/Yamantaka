import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/library_document.dart';
import '../domain/library_query.dart';
import '../domain/document_enrichment.dart';
import '../../organization/domain/organization_models.dart';

final class LibraryController extends ChangeNotifier {
  LibraryController(this._repository, this._logger);

  final DocumentRepository _repository;
  final AppLogger _logger;

  List<LibraryDocument> documents = const [];
  int totalDocuments = 0;
  int libraryDocumentCount = 0;
  bool hasMore = false;
  bool isLoading = false;
  bool isLoadingMore = false;
  AppFailure? error;
  final ValueNotifier<String?> selectedDocumentListenable =
      ValueNotifier<String?>(null);
  final Map<String, ValueNotifier<bool>> _documentSelectionListenables = {};
  double scrollOffset = 0;
  String query = '';
  String sortOrder = 'lastOpened';
  String readingState = 'all';
  bool favoritesOnly = false;
  Set<String> selectedAuthors = const {};
  Set<LibraryTopic> selectedTopics = const {};
  bool groupByAuthor = false;
  String? selectedCollectionId;
  SmartCollection? selectedSmartCollection;
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

  String? get selectedDocumentId => selectedDocumentListenable.value;

  ValueListenable<bool> selectionListenableFor(String documentId) =>
      _documentSelectionListenables.putIfAbsent(
        documentId,
        () => ValueNotifier<bool>(selectedDocumentId == documentId),
      );

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
      final page = _repository.page(requested);
      if (generation != _loadGeneration) return;
      if (append) {
        final byId = <String, LibraryDocument>{
          for (final document in documents) document.id: document,
        };
        for (final document in page.documents) {
          byId[document.id] = document;
        }
        documents = List.unmodifiable(byId.values);
      } else {
        documents = page.documents;
      }
      totalDocuments = page.total;
      libraryDocumentCount = _repository.documentCount;
      hasMore = page.hasMore;
      authorFacets = _repository.authorFacets(requested);
      topicFacets = _repository.topicFacets(requested);
      readingStateCounts = _repository.readingStateCounts(requested);
      if (selectedDocumentId != null &&
          _repository.byId(selectedDocumentId!) == null) {
        selectDocument(null);
      }
      if (selectedDocumentId == null && documents.length == 1) {
        selectDocument(documents.single.id);
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

  /// Refreshes only materialized rows whose persisted file state changed.
  /// Filtered queries that exclude missing files are re-queried because their
  /// membership can change; the normal inclusive library remains targeted.
  Future<void> refreshFileStates(Set<String> changedIds) async {
    if (changedIds.isEmpty) return;
    final active = _activeQuery;
    if (active != null && !active.includeMissing) {
      await reload();
      return;
    }
    var changed = false;
    final next = <LibraryDocument>[];
    for (final document in documents) {
      if (!changedIds.contains(document.id)) {
        next.add(document);
        continue;
      }
      final refreshed = _repository.byId(document.id);
      if (refreshed == null) {
        changed = true;
        continue;
      }
      next.add(refreshed);
      changed = true;
    }
    if (!changed) return;
    documents = List.unmodifiable(next);
    notifyListeners();
  }

  void setQuery(String value) {
    query = value;
    unawaited(reload());
  }

  Future<void> setReadingState(LibraryQueryReadingState value) {
    readingState = switch (value) {
      LibraryQueryReadingState.inProgress => 'inProgress',
      LibraryQueryReadingState.notStarted => 'notStarted',
      LibraryQueryReadingState.completed => 'completed',
      LibraryQueryReadingState.all => 'all',
    };
    return reload();
  }

  Future<void> setSort(LibraryQuerySort value) {
    sortOrder = switch (value) {
      LibraryQuerySort.added => 'added',
      LibraryQuerySort.title => 'title',
      LibraryQuerySort.progress => 'progress',
      LibraryQuerySort.lastOpened => 'lastOpened',
    };
    return reload();
  }

  Future<void> setFavoritesOnly(bool value) {
    favoritesOnly = value;
    return reload();
  }

  Future<void> resetFilters() {
    query = '';
    readingState = 'all';
    sortOrder = 'lastOpened';
    favoritesOnly = false;
    selectedAuthors = const {};
    selectedTopics = const {};
    groupByAuthor = false;
    return reload();
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

  Future<void> setOrganizationScope({
    String? collectionId,
    SmartCollection? smartCollection,
  }) {
    selectedCollectionId = collectionId;
    selectedSmartCollection = smartCollection;
    return reload();
  }

  LibraryQuery _queryFromState() => LibraryQuery(
    text: query,
    favoritesOnly: favoritesOnly,
    authors: selectedAuthors,
    topics: selectedTopics,
    collectionId: selectedCollectionId,
    smartCollection: selectedSmartCollection,
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
    final previous = selectedDocumentId;
    selectedDocumentListenable.value = documentId;
    if (previous != null) {
      _documentSelectionListenables[previous]?.value = false;
    }
    if (documentId != null) {
      final selected = _documentSelectionListenables.putIfAbsent(
        documentId,
        () => ValueNotifier<bool>(false),
      );
      selected.value = true;
    }
  }

  Future<void> toggleFavorite(String documentId) async {
    _repository.toggleFavorite(documentId);
    await load();
  }

  @override
  void dispose() {
    selectedDocumentListenable.dispose();
    for (final listenable in _documentSelectionListenables.values) {
      listenable.dispose();
    }
    _documentSelectionListenables.clear();
    super.dispose();
  }
}
