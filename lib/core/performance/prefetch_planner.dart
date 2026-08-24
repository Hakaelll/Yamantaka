import '../../features/reader/domain/reader_models.dart';

final class ScrollSample {
  const ScrollSample({
    required this.pageIndex,
    required this.offset,
    required this.at,
  });

  final int pageIndex;
  final double offset;
  final DateTime at;
}

final class PrefetchPlan {
  const PrefetchPlan({
    required this.pages,
    required this.direction,
    required this.generation,
    required this.paused,
    required this.velocityPagesPerSecond,
  });

  final List<int> pages;
  final int direction;
  final int generation;
  final bool paused;
  final double velocityPagesPerSecond;
}

/// Chooses only the next reading unit. Prefetch has no user-visible progress
/// and is deliberately disabled when the viewport is under pressure.
abstract final class PrefetchPlanner {
  static PrefetchPlan plan({
    required List<ScrollSample> samples,
    required int currentPage,
    required int pageCount,
    required ReaderMode mode,
    required int generation,
    bool memoryPressure = false,
    bool windowInactive = false,
    bool randomNavigation = false,
  }) {
    if (pageCount <= 0 ||
        memoryPressure ||
        windowInactive ||
        randomNavigation) {
      return PrefetchPlan(
        pages: const [],
        direction: 0,
        generation: generation,
        paused: true,
        velocityPagesPerSecond: 0,
      );
    }
    final direction = _direction(samples);
    if (direction == 0) {
      return PrefetchPlan(
        pages: const [],
        direction: 0,
        generation: generation,
        paused: false,
        velocityPagesPerSecond: 0,
      );
    }
    final velocity = _velocityPagesPerSecond(samples);
    final units = mode == ReaderMode.book ? 2 : 1;
    final lookahead = velocity >= 8
        ? 3
        : velocity >= 3
        ? 2
        : 1;
    final pages = <int>[];
    for (var step = 1; step <= units * lookahead; step++) {
      final target = currentPage + direction * step;
      if (target < 0 || target >= pageCount) continue;
      if (!pages.contains(target)) pages.add(target);
    }
    return PrefetchPlan(
      pages: List.unmodifiable(pages),
      direction: direction,
      generation: generation,
      paused: false,
      velocityPagesPerSecond: velocity,
    );
  }

  static int _direction(List<ScrollSample> samples) {
    if (samples.length < 2) return 0;
    final current = samples.last;
    final previous = samples[samples.length - 2];
    final pageDelta = current.pageIndex - previous.pageIndex;
    if (pageDelta != 0) return pageDelta.sign;
    final offsetDelta = current.offset - previous.offset;
    if (offsetDelta.abs() < 1) return 0;
    return offsetDelta.sign.toInt();
  }

  static double _velocityPagesPerSecond(List<ScrollSample> samples) {
    if (samples.length < 2) return 0;
    final first = samples.first;
    final last = samples.last;
    final elapsedMicros = last.at.difference(first.at).inMicroseconds;
    if (elapsedMicros <= 0) return 0;
    final elapsedSeconds = elapsedMicros / Duration.microsecondsPerSecond;
    final pageDistance = (last.pageIndex - first.pageIndex).abs().toDouble();
    if (pageDistance > 0) return pageDistance / elapsedSeconds;
    // A typical viewport is used only as a stable normalization unit. Actual
    // rendering remains bounded by the cache ring and memory pressure.
    return (last.offset - first.offset).abs() / 800 / elapsedSeconds;
  }
}
