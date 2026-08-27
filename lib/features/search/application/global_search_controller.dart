import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../storage/repositories/global_search_repository.dart';
import '../../../core/performance/local_performance_monitor.dart';
import '../domain/global_search_models.dart';

final class GlobalLibrarySearchController extends ChangeNotifier {
  GlobalLibrarySearchController(this._repository, [this._performance]);

  final GlobalSearchRepository _repository;
  final LocalPerformanceMonitor? _performance;
  String query = '';
  List<GlobalSearchResult> results = const [];
  bool isSearching = false;
  GlobalSearchState state = GlobalSearchState.idle;
  Set<GlobalSearchKind> selectedKinds = Set.unmodifiable(
    GlobalSearchKind.values,
  );
  int offset = 0;
  bool hasMore = false;
  int _generation = 0;
  bool _derivedEntriesFresh = false;

  Future<void> search(String value, {bool loadMore = false}) async {
    final clean = value.trim();
    final generation = ++_generation;
    if (!loadMore) {
      query = clean;
      offset = 0;
      results = const [];
    }
    if (query.isEmpty) {
      isSearching = false;
      state = GlobalSearchState.idle;
      notifyListeners();
      return;
    }
    isSearching = true;
    state = GlobalSearchState.searching;
    notifyListeners();
    await Future<void>.delayed(
      loadMore ? Duration.zero : const Duration(milliseconds: 180),
    );
    if (generation != _generation) return;
    if (!_derivedEntriesFresh) {
      _repository
        ..refreshAnnotationEntries()
        ..refreshBookmarkEntries()
        ..refreshOrganizationEntries();
      _derivedEntriesFresh = true;
    }
    final stopwatch = Stopwatch()..start();
    final page = _repository.search(
      query,
      limit: 40,
      offset: offset,
      kinds: selectedKinds,
    );
    stopwatch.stop();
    _performance?.recordDuration('global_search', stopwatch.elapsed);
    if (generation != _generation) return;
    results = loadMore ? [...results, ...page] : page;
    offset = results.length;
    hasMore = page.length == 40;
    isSearching = false;
    state = results.isEmpty
        ? GlobalSearchState.noResults
        : GlobalSearchState.results;
    notifyListeners();
  }

  void toggleKind(GlobalSearchKind kind) {
    final next = selectedKinds.toSet();
    if (next.contains(kind)) {
      if (next.length == 1) return;
      next.remove(kind);
    } else {
      next.add(kind);
    }
    selectedKinds = Set.unmodifiable(next);
    unawaited(search(query));
  }

  void cancel() {
    if (!isSearching) return;
    _generation++;
    isSearching = false;
    state = GlobalSearchState.cancelled;
    notifyListeners();
  }

  void invalidateDerivedEntries() {
    _derivedEntriesFresh = false;
  }

  void clear() {
    _generation++;
    query = '';
    results = const [];
    offset = 0;
    hasMore = false;
    isSearching = false;
    state = GlobalSearchState.idle;
    notifyListeners();
  }
}
