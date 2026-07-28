import 'package:flutter/foundation.dart';

import '../../../storage/repositories/bookmark_repository.dart';
import '../domain/bookmark.dart';

final class BookmarkController extends ChangeNotifier {
  BookmarkController(this._repository);

  final BookmarkRepository _repository;
  String? _documentId;
  List<Bookmark> bookmarks = const [];

  bool containsPage(int pageIndex) =>
      bookmarks.any((bookmark) => bookmark.pageIndex == pageIndex);

  void load(String documentId) {
    _documentId = documentId;
    bookmarks = _repository.forDocument(documentId);
    notifyListeners();
  }

  void clear() {
    _documentId = null;
    bookmarks = const [];
    notifyListeners();
  }

  void toggle(int pageIndex, {String? label}) {
    final documentId = _documentId;
    if (documentId == null) return;
    final existing = bookmarks
        .where((bookmark) => bookmark.pageIndex == pageIndex)
        .firstOrNull;
    if (existing == null) {
      _repository.add(documentId, pageIndex, label: label);
    } else {
      _repository.remove(existing.id);
    }
    bookmarks = _repository.forDocument(documentId);
    notifyListeners();
  }

  void add(int pageIndex, {String? label}) {
    final documentId = _documentId;
    if (documentId == null) return;
    _repository.add(documentId, pageIndex, label: label);
    bookmarks = _repository.forDocument(documentId);
    notifyListeners();
  }

  void remove(Bookmark bookmark) {
    _repository.remove(bookmark.id);
    final documentId = _documentId;
    if (documentId != null) {
      bookmarks = _repository.forDocument(documentId);
      notifyListeners();
    }
  }
}
