import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/startup_timeline.dart';

void main() {
  test('records each phase once and remains safe to end twice', () {
    var now = DateTime.utc(2026, 8, 1);
    final timeline = StartupTimeline(clock: () => now);
    final span = timeline.start(StartupPhase.services);
    now = now.add(const Duration(milliseconds: 12));
    expect(span.end()!.durationMs, 12);
    expect(span.end(), isNull);
    expect(timeline.toJson()['completed'], isTrue);
  });
}
