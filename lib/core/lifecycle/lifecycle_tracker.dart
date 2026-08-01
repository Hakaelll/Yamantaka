enum ResourceKind { document, render, image, listener, timer, job, overlay }

final class LifecycleSnapshot {
  const LifecycleSnapshot(this.counts);

  final Map<ResourceKind, int> counts;

  int get total => counts.values.fold(0, (sum, value) => sum + value);

  bool get isEmpty => total == 0;

  Map<String, int> toJson() => {
    for (final entry in counts.entries) entry.key.name: entry.value,
  };
}

/// Tracks resources owned by Yamantaka. Labels are intentionally not retained
/// in the public snapshot so diagnostics cannot leak document or path data.
final class LifecycleTracker {
  final Map<ResourceKind, int> _counts = {
    for (final kind in ResourceKind.values) kind: 0,
  };
  int _generation = 0;
  bool _disposed = false;

  LifecycleLease acquire(ResourceKind kind) {
    if (_disposed) throw StateError('Lifecycle tracker is disposed.');
    _counts[kind] = (_counts[kind] ?? 0) + 1;
    return LifecycleLease._(this, kind);
  }

  LifecycleSnapshot get snapshot =>
      LifecycleSnapshot(Map.unmodifiable({..._counts}));

  int newGeneration() => ++_generation;

  bool isCurrent(int generation) => !_disposed && generation == _generation;

  void dispose() {
    _disposed = true;
    for (final kind in ResourceKind.values) {
      _counts[kind] = 0;
    }
  }

  void _release(ResourceKind kind) {
    final count = _counts[kind] ?? 0;
    if (count <= 0) return;
    _counts[kind] = count - 1;
  }
}

final class LifecycleLease {
  LifecycleLease._(this._tracker, this.kind);

  final LifecycleTracker _tracker;
  final ResourceKind kind;
  bool _released = false;

  bool get isReleased => _released;

  void release() {
    if (_released) return;
    _released = true;
    _tracker._release(kind);
  }
}
