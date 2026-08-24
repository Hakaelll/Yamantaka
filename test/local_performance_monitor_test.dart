import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/local_performance_monitor.dart';

void main() {
  test('keeps bounded aggregate-only duration samples and counters', () {
    final monitor = LocalPerformanceMonitor(sampleLimit: 3);
    for (final milliseconds in [1, 2, 3, 4]) {
      monitor.recordDuration(
        'page_render',
        Duration(milliseconds: milliseconds),
      );
    }
    monitor.increment('cache_hits', 2);

    final json = monitor.toJson();
    final durations = json['durations']! as Map<String, Object>;
    final pageRender = durations['page_render']! as Map<String, Object>;
    final counters = json['counters']! as Map<String, int>;
    expect(pageRender, {
      'count': 4,
      'retained_samples': 3,
      'p50_us': 3000,
      'p95_us': 4000,
      'max_us': 4000,
    });
    expect(counters['cache_hits'], 2);
    expect(json.toString(), isNot(contains('path')));
  });
}
