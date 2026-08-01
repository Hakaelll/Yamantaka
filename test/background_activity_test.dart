import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/background_activity.dart';

void main() {
  test('minimized windows pause non-critical work', () {
    const policy = BackgroundActivityPolicy(
      activity: BackgroundActivity.minimized,
      memoryPressure: false,
    );
    expect(policy.allowPrefetch, isFalse);
    expect(policy.allowIndexing, isFalse);
    expect(policy.allowMaintenance, isFalse);
  });

  test('visible windows allow indexing but not maintenance by default', () {
    const policy = BackgroundActivityPolicy(
      activity: BackgroundActivity.visible,
      memoryPressure: false,
    );
    expect(policy.allowPrefetch, isTrue);
    expect(policy.allowIndexing, isTrue);
    expect(policy.allowMaintenance, isFalse);
  });
}
