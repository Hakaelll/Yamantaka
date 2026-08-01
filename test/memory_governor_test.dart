import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/memory/memory_governor.dart';

void main() {
  test('evicts recreable work before visible content', () {
    final governor = MemoryGovernor(limitBytes: 100);
    var evicted = 0;
    final prefetch = governor.reserve(
      owner: MemoryOwner.prefetch,
      bytes: 60,
      priority: MemoryPriority.prefetch,
      onEvicted: () => evicted++,
    );
    expect(prefetch, isNotNull);

    final visible = governor.reserve(
      owner: MemoryOwner.activePage,
      bytes: 70,
      priority: MemoryPriority.visible,
    );
    expect(visible, isNotNull);
    expect(evicted, 1);
    expect(governor.snapshot.retainedBytes, 70);
  });

  test('released reservations return quota and release is idempotent', () {
    final governor = MemoryGovernor(limitBytes: 32);
    final reservation = governor.reserve(
      owner: MemoryOwner.renderCache,
      bytes: 32,
      priority: MemoryPriority.cache,
    )!;
    expect(
      governor.reserve(
        owner: MemoryOwner.prefetch,
        bytes: 1,
        priority: MemoryPriority.prefetch,
      ),
      isNull,
    );
    reservation
      ..release()
      ..release();
    expect(governor.snapshot.retainedBytes, 0);
    expect(
      governor.reserve(
        owner: MemoryOwner.activePage,
        bytes: 16,
        priority: MemoryPriority.visible,
      ),
      isNotNull,
    );
  });

  test('reserves a visible minimum before admitting background work', () {
    final governor = MemoryGovernor(limitBytes: 100, minimumVisibleBytes: 24);
    final visible = governor.reserve(
      owner: MemoryOwner.activePage,
      bytes: 60,
      priority: MemoryPriority.visible,
    );
    expect(visible, isNotNull);
    expect(
      governor.reserve(
        owner: MemoryOwner.prefetch,
        bytes: 16,
        priority: MemoryPriority.prefetch,
      ),
      isNotNull,
    );
    expect(
      governor.reserve(
        owner: MemoryOwner.indexing,
        bytes: 1,
        priority: MemoryPriority.background,
      ),
      isNull,
    );
  });
}
