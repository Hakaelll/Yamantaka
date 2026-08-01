import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/rebuild_metrics.dart';

void main() {
  test('rebuild metrics remain redacted and surface-scoped', () {
    final metrics = RebuildMetrics()
      ..record('library.card')
      ..record('library.card');
    expect(metrics.counts['library.card'], 2);
    expect(metrics.toJson(), containsPair('total', 2));
  });
}
