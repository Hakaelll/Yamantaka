import 'package:flutter/scheduler.dart';

/// Bounded, local-only performance instrumentation.
///
/// Metric names are compile-time product identifiers. Samples never contain a
/// document identifier, path, title, page text or search query.
final class LocalPerformanceMonitor {
  LocalPerformanceMonitor({this.sampleLimit = 256, this.slowFrameMs = 16.7});

  final int sampleLimit;
  final double slowFrameMs;
  final Map<String, _DurationSeries> _durations = <String, _DurationSeries>{};
  final Map<String, int> _counters = <String, int>{};
  bool _listeningToFrames = false;

  void startFrameMonitoring() {
    if (_listeningToFrames) return;
    _listeningToFrames = true;
    SchedulerBinding.instance.addTimingsCallback(_onFrameTimings);
  }

  void stopFrameMonitoring() {
    if (!_listeningToFrames) return;
    SchedulerBinding.instance.removeTimingsCallback(_onFrameTimings);
    _listeningToFrames = false;
  }

  void recordDuration(String metric, Duration duration) {
    final series = _durations.putIfAbsent(
      metric,
      () => _DurationSeries(sampleLimit),
    );
    series.add(duration.inMicroseconds);
  }

  void increment(String counter, [int amount = 1]) {
    if (amount <= 0) return;
    _counters[counter] = (_counters[counter] ?? 0) + amount;
  }

  Map<String, Object> toJson() => {
    'sample_limit': sampleLimit,
    'durations': {
      for (final entry in _durations.entries) entry.key: entry.value.toJson(),
    },
    'counters': Map<String, int>.unmodifiable(_counters),
  };

  void _onFrameTimings(List<FrameTiming> timings) {
    for (final timing in timings) {
      recordDuration('frame_build', timing.buildDuration);
      recordDuration('frame_raster', timing.rasterDuration);
      recordDuration('frame_total', timing.totalSpan);
      if (timing.totalSpan.inMicroseconds > slowFrameMs * 1000) {
        increment('slow_frames');
      }
    }
  }
}

final class _DurationSeries {
  _DurationSeries(this.limit);

  final int limit;
  final List<int> _samples = <int>[];
  int total = 0;

  void add(int microseconds) {
    total++;
    if (_samples.length == limit) _samples.removeAt(0);
    _samples.add(microseconds);
  }

  Map<String, Object> toJson() {
    final sorted = _samples.toList(growable: false)..sort();
    return {
      'count': total,
      'retained_samples': sorted.length,
      'p50_us': _percentile(sorted, 0.50),
      'p95_us': _percentile(sorted, 0.95),
      'max_us': sorted.isEmpty ? 0 : sorted.last,
    };
  }

  static int _percentile(List<int> values, double percentile) {
    if (values.isEmpty) return 0;
    final index = ((values.length - 1) * percentile).ceil();
    return values[index.clamp(0, values.length - 1)];
  }
}
