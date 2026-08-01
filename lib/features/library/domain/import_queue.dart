import 'dart:collection';

enum ImportWorkState { pending, running, completed, failed, cancelled }

final class ImportQueueItem {
  const ImportQueueItem({
    required this.id,
    required this.path,
    required this.normalizedPath,
    this.fingerprint,
    this.state = ImportWorkState.pending,
    this.attempts = 0,
    this.errorCode,
  });

  final String id;
  final String path;
  final String normalizedPath;
  final String? fingerprint;
  final ImportWorkState state;
  final int attempts;
  final String? errorCode;

  ImportQueueItem copyWith({
    ImportWorkState? state,
    int? attempts,
    Object? errorCode = _keepError,
  }) => ImportQueueItem(
    id: id,
    path: path,
    normalizedPath: normalizedPath,
    fingerprint: fingerprint,
    state: state ?? this.state,
    attempts: attempts ?? this.attempts,
    errorCode: identical(errorCode, _keepError)
        ? this.errorCode
        : errorCode as String?,
  );

  Map<String, Object?> toJson() => {
    'id': id,
    'path': path,
    'normalized_path': normalizedPath,
    if (fingerprint != null) 'fingerprint': fingerprint,
    'state': state.name,
    'attempts': attempts,
    if (errorCode != null) 'error_code': errorCode,
  };

  static ImportQueueItem? fromJson(Object? raw) {
    if (raw is! Map) return null;
    final id = raw['id'];
    final path = raw['path'];
    final normalizedPath = raw['normalized_path'];
    if (id is! String ||
        path is! String ||
        normalizedPath is! String ||
        id.isEmpty ||
        path.isEmpty ||
        normalizedPath.isEmpty) {
      return null;
    }
    final stateName = raw['state'];
    final state = ImportWorkState.values
        .where((candidate) => candidate.name == stateName)
        .firstOrNull;
    final attempts = raw['attempts'];
    return ImportQueueItem(
      id: id,
      path: path,
      normalizedPath: normalizedPath,
      fingerprint: raw['fingerprint'] is String
          ? raw['fingerprint'] as String
          : null,
      state: state ?? ImportWorkState.pending,
      attempts: attempts is int && attempts >= 0 ? attempts : 0,
      errorCode: raw['error_code'] is String
          ? raw['error_code'] as String
          : null,
    );
  }
}

final class ImportQueue {
  ImportQueue({this.maxPending = 5000}) {
    if (maxPending < 1) throw ArgumentError.value(maxPending, 'maxPending');
  }

  final int maxPending;
  final LinkedHashMap<String, ImportQueueItem> _items =
      LinkedHashMap<String, ImportQueueItem>();
  bool _paused = false;

  bool get isPaused => _paused;
  int get pendingCount => _items.values
      .where((item) => item.state == ImportWorkState.pending)
      .length;
  int get activeCount => _items.values
      .where((item) => item.state == ImportWorkState.running)
      .length;
  List<ImportQueueItem> get items => List.unmodifiable(_items.values);

  bool enqueue(ImportQueueItem item) {
    if (_items.containsKey(item.id)) return false;
    if (_items.values.any(
      (current) =>
          current.normalizedPath == item.normalizedPath ||
          item.fingerprint != null &&
              item.fingerprint!.isNotEmpty &&
              current.fingerprint == item.fingerprint,
    )) {
      return false;
    }
    if (pendingCount >= maxPending) return false;
    _items[item.id] = item.copyWith(
      state: ImportWorkState.pending,
      attempts: 0,
      errorCode: null,
    );
    return true;
  }

  ImportQueueItem? claimNext() {
    if (_paused) return null;
    for (final item in _items.values) {
      if (item.state == ImportWorkState.pending) {
        final claimed = item.copyWith(
          state: ImportWorkState.running,
          attempts: item.attempts + 1,
          errorCode: null,
        );
        _items[item.id] = claimed;
        return claimed;
      }
    }
    return null;
  }

  void complete(String id) =>
      _update(id, state: ImportWorkState.completed, errorCode: null);

  void fail(String id, String errorCode, {bool retryable = true}) {
    final item = _required(id);
    _update(
      id,
      state: retryable && item.attempts < 3
          ? ImportWorkState.pending
          : ImportWorkState.failed,
      errorCode: _safeErrorCode(errorCode),
    );
  }

  void cancelPending() {
    for (final item in _items.values) {
      if (item.state == ImportWorkState.pending) {
        _items[item.id] = item.copyWith(state: ImportWorkState.cancelled);
      }
    }
  }

  void pause() => _paused = true;
  void resume() => _paused = false;

  void prepareForResume() {
    for (final item in _items.values) {
      if (item.state == ImportWorkState.running) {
        _items[item.id] = item.copyWith(state: ImportWorkState.pending);
      }
    }
  }

  Map<String, Object?> toJson() => {
    'version': 1,
    'paused': _paused,
    'items': [for (final item in _items.values) item.toJson()],
  };

  static ImportQueue fromJson(
    Map<String, Object?> raw, {
    int maxPending = 5000,
  }) {
    final queue = ImportQueue(maxPending: maxPending);
    queue._paused = raw['paused'] as bool? ?? false;
    final items = raw['items'];
    if (items is List) {
      for (final item in items) {
        final parsed = ImportQueueItem.fromJson(item);
        if (parsed == null || queue._items.length >= maxPending) continue;
        if (queue._items.containsKey(parsed.id)) continue;
        if (queue._items.values.any(
          (current) => current.normalizedPath == parsed.normalizedPath,
        )) {
          continue;
        }
        queue._items[parsed.id] = parsed;
      }
    }
    queue.prepareForResume();
    return queue;
  }

  void _update(
    String id, {
    required ImportWorkState state,
    required Object? errorCode,
  }) {
    final item = _required(id);
    _items[id] = item.copyWith(state: state, errorCode: errorCode);
  }

  ImportQueueItem _required(String id) =>
      _items[id] ?? (throw StateError('Unknown import queue item.'));

  static String _safeErrorCode(String value) {
    final clean = value.trim().toLowerCase().replaceAll(
      RegExp(r'[^a-z0-9_-]'),
      '_',
    );
    if (clean.isEmpty) return 'unknown';
    final end = clean.length > 64 ? 64 : clean.length;
    return clean.substring(0, end);
  }
}

const Object _keepError = Object();
