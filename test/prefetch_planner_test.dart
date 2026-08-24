import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/performance/prefetch_planner.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';

void main() {
  final now = DateTime.utc(2026, 1, 1);

  test('prefetch follows direction and bounds a document unit', () {
    final plan = PrefetchPlanner.plan(
      samples: [
        ScrollSample(pageIndex: 3, offset: 0, at: now),
        ScrollSample(
          pageIndex: 4,
          offset: 0,
          at: now.add(const Duration(milliseconds: 40)),
        ),
      ],
      currentPage: 3,
      pageCount: 5,
      mode: ReaderMode.document,
      generation: 7,
    );
    expect(plan.pages, [4]);
    expect(plan.direction, 1);
    expect(plan.generation, 7);
    expect(plan.velocityPagesPerSecond, greaterThan(8));
  });

  test(
    'book prefetch can request the next spread but pauses under pressure',
    () {
      final plan = PrefetchPlanner.plan(
        samples: [
          ScrollSample(pageIndex: 2, offset: 0, at: now),
          ScrollSample(
            pageIndex: 1,
            offset: 0,
            at: now.add(const Duration(milliseconds: 40)),
          ),
        ],
        currentPage: 1,
        pageCount: 8,
        mode: ReaderMode.book,
        generation: 3,
      );
      expect(plan.pages, [0]);
      expect(
        PrefetchPlanner.plan(
          samples: const [],
          currentPage: 1,
          pageCount: 8,
          mode: ReaderMode.book,
          generation: 4,
          memoryPressure: true,
        ).paused,
        isTrue,
      );
    },
  );

  test('fast navigation expands lookahead only in its current direction', () {
    final plan = PrefetchPlanner.plan(
      samples: [
        ScrollSample(pageIndex: 2, offset: 0, at: now),
        ScrollSample(
          pageIndex: 8,
          offset: 0,
          at: now.add(const Duration(milliseconds: 100)),
        ),
      ],
      currentPage: 8,
      pageCount: 20,
      mode: ReaderMode.document,
      generation: 9,
    );

    expect(plan.pages, [9, 10, 11]);
    expect(plan.velocityPagesPerSecond, 60);
  });
}
