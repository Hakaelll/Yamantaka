import 'dart:collection';

enum BulkLibraryAction {
  favorite,
  unfavorite,
  assignTag,
  addToCollection,
  removeFromCollection,
  archive,
  regenerateCover,
  removeFromLibrary,
}

typedef BulkOperationHandler =
    Future<BulkOperationSummary> Function(
      BulkLibraryAction action,
      Iterable<String> documentIds, {
      String? targetId,
    });

final class LibrarySelection {
  LibrarySelection({Iterable<String> initial = const []})
    : _selected = LinkedHashSet<String>.from(initial.where(_validId));

  final LinkedHashSet<String> _selected;

  List<String> get ids => List.unmodifiable(_selected);
  int get length => _selected.length;
  bool get isEmpty => _selected.isEmpty;
  bool get isNotEmpty => _selected.isNotEmpty;
  bool contains(String id) => _selected.contains(id);

  void toggle(String id, {bool? selected}) {
    if (!_validId(id)) return;
    final next = selected ?? !_selected.contains(id);
    if (next) {
      _selected.add(id);
    } else {
      _selected.remove(id);
    }
  }

  void selectRange(
    List<String> orderedIds,
    String anchor,
    String target, {
    bool additive = false,
  }) {
    if (!additive) _selected.clear();
    final start = orderedIds.indexOf(anchor);
    final end = orderedIds.indexOf(target);
    if (start < 0 || end < 0) {
      toggle(target, selected: true);
      return;
    }
    final low = start < end ? start : end;
    final high = start < end ? end : start;
    for (final id in orderedIds.sublist(low, high + 1)) {
      toggle(id, selected: true);
    }
  }

  void retain(Iterable<String> validIds) {
    final valid = validIds.toSet();
    _selected.removeWhere((id) => !valid.contains(id));
  }

  void clear() => _selected.clear();

  static bool _validId(String id) => id.trim().isNotEmpty;
}

final class BulkItemResult {
  const BulkItemResult({
    required this.id,
    required this.success,
    this.errorCode,
  });

  final String id;
  final bool success;
  final String? errorCode;
}

final class BulkOperationSummary {
  const BulkOperationSummary(this.results);

  final List<BulkItemResult> results;

  int get succeeded => results.where((item) => item.success).length;
  int get failed => results.length - succeeded;
  bool get isPartial => succeeded > 0 && failed > 0;
}

abstract final class BulkOperationPolicy {
  static bool requiresConfirmation(BulkLibraryAction action) =>
      action == BulkLibraryAction.removeFromLibrary;

  static bool changesPdfFile(BulkLibraryAction _) => false;
}
