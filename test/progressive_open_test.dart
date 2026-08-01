import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/progressive_open.dart';

void main() {
  test('progressive phases publish only for the current generation', () {
    const state = ProgressiveOpenState(
      phase: ProgressiveOpenPhase.firstRender,
      generation: 4,
      pageIndex: 8,
    );
    expect(state.isUsable, isTrue);
    expect(state.canPublish(4), isTrue);
    expect(state.canPublish(5), isFalse);
    expect(
      ProgressiveOpenPolicy.cancel(state).phase,
      ProgressiveOpenPhase.cancelled,
    );
    expect(
      ProgressiveOpenPolicy.failure(state, 'corrupt_pdf').errorCode,
      'corrupt_pdf',
    );
  });
}
