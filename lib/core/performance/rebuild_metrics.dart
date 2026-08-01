final class RebuildMetrics {
  RebuildMetrics({this.sampleLimit = 1000});

  final int sampleLimit;
  final Map<String, int> _counts = {};

  void record(String surface) {
    if (_counts.length >= sampleLimit && !_counts.containsKey(surface)) return;
    _counts[surface] = (_counts[surface] ?? 0) + 1;
  }

  Map<String, int> get counts => Map.unmodifiable(_counts);

  Map<String, Object> toJson() => {
    'surfaces': counts,
    'total': _counts.values.fold<int>(0, (sum, value) => sum + value),
  };
}
