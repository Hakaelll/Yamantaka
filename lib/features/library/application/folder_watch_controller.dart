import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../../../core/platform/file_root_policy.dart';
import '../domain/authorized_folder_watch.dart';

/// Bridges Windows filesystem events to the authorization/debounce policy.
///
/// A ready event is only a revalidation request. This controller never imports,
/// moves, deletes, or changes a document by itself.
final class FolderWatchController extends ChangeNotifier {
  FolderWatchController({AuthorizedFolderWatch? policy})
    : policy = policy ?? AuthorizedFolderWatch();

  final AuthorizedFolderWatch policy;
  final Map<String, StreamSubscription<FileSystemEvent>> _subscriptions = {};
  Timer? _drainTimer;

  ValueChanged<List<FolderWatchEvent>>? onRevalidationReady;
  Object? lastError;

  bool get isPaused => policy.isPaused;
  List<String> get roots => policy.roots;
  int get pendingCount => policy.pendingCount;

  bool authorize(String path) {
    final root = AuthorizedFileRoot(path).path;
    final directory = Directory(root);
    if (!directory.existsSync()) return false;
    policy.authorize(root);
    if (_subscriptions.keys.any((key) => _samePath(key, root))) {
      notifyListeners();
      return true;
    }
    try {
      final subscription = directory
          .watch(recursive: true, events: FileSystemEvent.all)
          .listen(
            _acceptEvent,
            onError: (Object error) {
              lastError = error is FileSystemException
                  ? 'watch_unavailable'
                  : 'watch_failed';
              notifyListeners();
            },
            cancelOnError: false,
          );
      _subscriptions[root] = subscription;
      lastError = null;
      notifyListeners();
      return true;
    } on Object {
      policy.revoke(root);
      lastError = 'watch_failed';
      notifyListeners();
      return false;
    }
  }

  void revoke(String path) {
    final root = AuthorizedFileRoot(path).path;
    final keys = _subscriptions.keys
        .where((key) => _samePath(key, root))
        .toList(growable: false);
    for (final key in keys) {
      unawaited(_subscriptions.remove(key)?.cancel());
    }
    policy.revoke(root);
    _drainTimer?.cancel();
    _drainTimer = null;
    notifyListeners();
  }

  void setPaused(bool value) {
    policy.setPaused(value);
    _drainTimer?.cancel();
    _drainTimer = null;
    if (!value) _scheduleDrain();
    notifyListeners();
  }

  void flush({DateTime? now}) {
    final ready = policy.drain(now: now);
    if (ready.isNotEmpty) onRevalidationReady?.call(ready);
    notifyListeners();
  }

  @override
  void dispose() {
    _drainTimer?.cancel();
    for (final subscription in _subscriptions.values) {
      unawaited(subscription.cancel());
    }
    _subscriptions.clear();
    super.dispose();
  }

  void _acceptEvent(FileSystemEvent event) {
    final path = event.path;
    if (!path.toLowerCase().endsWith('.pdf')) return;
    policy.accept(path);
    _scheduleDrain();
    notifyListeners();
  }

  void _scheduleDrain() {
    if (policy.isPaused || policy.pendingCount == 0 || _drainTimer != null) {
      return;
    }
    _drainTimer = Timer(policy.debounce, () {
      _drainTimer = null;
      flush();
      if (policy.pendingCount > 0) _scheduleDrain();
    });
  }

  static bool _samePath(String left, String right) => Platform.isWindows
      ? left.toLowerCase() == right.toLowerCase()
      : left == right;
}
