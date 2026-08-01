import 'dart:io';

import '../../../core/platform/file_root_policy.dart';

enum FolderWatchDecision { ignored, paused, queuedForRevalidation }

final class FolderWatchEvent {
  const FolderWatchEvent({required this.path, required this.at});

  final String path;
  final DateTime at;
}

/// In-memory watcher policy. Native filesystem events are deliberately
/// converted into debounced revalidation requests; they never import or
/// delete a document by themselves.
final class AuthorizedFolderWatch {
  AuthorizedFolderWatch({
    this.debounce = const Duration(milliseconds: 350),
    this.maxPending = 256,
  });

  final Duration debounce;
  final int maxPending;
  final List<AuthorizedFileRoot> _roots = [];
  final Map<String, FolderWatchEvent> _pending = {};
  bool _paused = false;

  bool get isPaused => _paused;
  List<String> get roots =>
      List.unmodifiable([for (final root in _roots) root.path]);
  int get pendingCount => _pending.length;

  void authorize(String path) {
    final root = AuthorizedFileRoot(path);
    if (!_roots.any((item) => _same(item.path, root.path))) _roots.add(root);
  }

  void revoke(String path) {
    final normalized = AuthorizedFileRoot(path).path;
    _roots.removeWhere((root) => _same(root.path, normalized));
    _pending.removeWhere(
      (candidate, _) => !FileRootPolicy.mayRead(_roots, candidate),
    );
  }

  void setPaused(bool value) {
    _paused = value;
    if (value) _pending.clear();
  }

  FolderWatchDecision accept(String path, {DateTime? at}) {
    if (_paused) return FolderWatchDecision.paused;
    if (!FileRootPolicy.mayRead(_roots, path)) {
      return FolderWatchDecision.ignored;
    }
    if (_pending.length >= maxPending && !_pending.containsKey(path)) {
      return FolderWatchDecision.ignored;
    }
    final normalized = File(path).absolute.path;
    _pending[normalized] = FolderWatchEvent(
      path: normalized,
      at: at ?? DateTime.now(),
    );
    return FolderWatchDecision.queuedForRevalidation;
  }

  List<FolderWatchEvent> drain({DateTime? now}) {
    if (_paused || _pending.isEmpty) return const [];
    final clock = now ?? DateTime.now();
    final ready = <FolderWatchEvent>[];
    for (final event in _pending.values.toList(growable: false)) {
      if (clock.difference(event.at) >= debounce) {
        ready.add(event);
        _pending.remove(event.path);
      }
    }
    return List.unmodifiable(ready);
  }

  static bool _same(String left, String right) => Platform.isWindows
      ? left.toLowerCase() == right.toLowerCase()
      : left == right;
}
