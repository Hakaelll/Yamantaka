import 'package:flutter/foundation.dart';

import '../../../storage/repositories/global_search_repository.dart';
import '../domain/global_search_models.dart';

final class GlobalLibrarySearchController extends ChangeNotifier {
  GlobalLibrarySearchController(this._repository);

  final GlobalSearchRepository _repository;
  String query = '';
  List<GlobalSearchResult> results = const [];
  bool isSearching = false;
  int offset = 0;
  bool hasMore = false;

  Future<void> search(String value, {bool loadMore = false}) async {
    final clean = value.trim();
    if (!loadMore) {
      query = clean;
      offset = 0;
      results = const [];
    }
    if (query.isEmpty) {
      notifyListeners();
      return;
    }
    isSearching = true;
    notifyListeners();
    await Future<void>.delayed(Duration.zero);
    _repository
      ..refreshAnnotationEntries()
      ..refreshOrganizationEntries();
    final page = _repository.search(query, limit: 40, offset: offset);
    results = loadMore ? [...results, ...page] : page;
    offset = results.length;
    hasMore = page.length == 40;
    isSearching = false;
    notifyListeners();
  }

  void clear() {
    query = '';
    results = const [];
    offset = 0;
    hasMore = false;
    notifyListeners();
  }
}
