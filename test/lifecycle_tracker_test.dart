import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/lifecycle/lifecycle_tracker.dart';

void main() {
  test('leases are idempotent and counts return to zero', () {
    final tracker = LifecycleTracker();
    final document = tracker.acquire(ResourceKind.document);
    final image = tracker.acquire(ResourceKind.image);
    expect(tracker.snapshot.total, 2);
    image
      ..release()
      ..release();
    document.release();
    expect(tracker.snapshot.isEmpty, isTrue);
  });

  test('generation rejects callbacks from a previous surface', () {
    final tracker = LifecycleTracker();
    final first = tracker.newGeneration();
    final second = tracker.newGeneration();
    expect(tracker.isCurrent(first), isFalse);
    expect(tracker.isCurrent(second), isTrue);
  });
}
