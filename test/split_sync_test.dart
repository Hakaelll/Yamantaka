import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/workspaces/domain/split_sync.dart';

void main() {
  test('propagation carries page offset and does not echo to its source', () {
    const event = SplitSyncEvent(
      sourcePane: 0,
      mode: SplitSyncMode.page,
      generation: 7,
      pageIndex: 10,
      zoom: 1.25,
    );
    final result = SplitSynchronizer.propagate(
      event: event,
      targetPane: 1,
      currentGeneration: 7,
      pageOffset: -2,
    );
    expect(result.applied, isTrue);
    expect(result.pageIndex, 8);
    expect(
      SplitSynchronizer.propagate(
        event: event,
        targetPane: 0,
        currentGeneration: 7,
      ).applied,
      isFalse,
    );
  });
}
