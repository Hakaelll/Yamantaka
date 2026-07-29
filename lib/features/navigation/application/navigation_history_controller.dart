import 'package:flutter/foundation.dart';

enum NavigationReason {
  searchResult,
  outline,
  bookmark,
  annotation,
  thumbnail,
  distantPage,
  internalLink,
  history,
}

final class NavigationLocation {
  const NavigationLocation({
    required this.pageIndex,
    required this.offset,
    required this.zoom,
    required this.reason,
    required this.label,
  });

  final int pageIndex;
  final double offset;
  final double zoom;
  final NavigationReason reason;
  final String label;
}

final class NavigationHistoryController extends ChangeNotifier {
  NavigationHistoryController({this.limit = 100});

  final int limit;
  final List<NavigationLocation> _back = [];
  final List<NavigationLocation> _forward = [];
  NavigationLocation? current;

  bool get canGoBack => _back.isNotEmpty;
  bool get canGoForward => _forward.isNotEmpty;
  List<NavigationLocation> get recent => [..._back.reversed.take(12), ?current];

  void reset(NavigationLocation initial) {
    _back.clear();
    _forward.clear();
    current = initial;
    notifyListeners();
  }

  void record(NavigationLocation location) {
    final previous = current;
    if (previous != null) {
      final sameArea =
          (previous.pageIndex - location.pageIndex).abs() <= 1 &&
          previous.reason == location.reason;
      if (sameArea) {
        current = location;
        return;
      }
      _back.add(previous);
      if (_back.length > limit) _back.removeAt(0);
    }
    current = location;
    _forward.clear();
    notifyListeners();
  }

  NavigationLocation? back() {
    if (_back.isEmpty) return null;
    final active = current;
    if (active != null) _forward.add(active);
    current = _back.removeLast();
    notifyListeners();
    return current;
  }

  NavigationLocation? forward() {
    if (_forward.isEmpty) return null;
    final active = current;
    if (active != null) _back.add(active);
    current = _forward.removeLast();
    notifyListeners();
    return current;
  }

  void clear() {
    _back.clear();
    _forward.clear();
    notifyListeners();
  }
}
