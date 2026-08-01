enum SplitSyncMode { independent, page, zoom, position }

final class SplitSyncEvent {
  const SplitSyncEvent({
    required this.sourcePane,
    required this.mode,
    required this.generation,
    this.pageIndex,
    this.zoom,
    this.offset,
  });

  final int sourcePane;
  final SplitSyncMode mode;
  final int generation;
  final int? pageIndex;
  final double? zoom;
  final double? offset;
}

final class SplitSyncResult {
  const SplitSyncResult({
    required this.applied,
    required this.targetPane,
    this.pageIndex,
    this.zoom,
    this.offset,
  });

  final bool applied;
  final int targetPane;
  final int? pageIndex;
  final double? zoom;
  final double? offset;
}

/// Origin and generation are carried through every propagation to prevent
/// two readers from echoing the same update forever.
abstract final class SplitSynchronizer {
  static SplitSyncResult propagate({
    required SplitSyncEvent event,
    required int targetPane,
    required int currentGeneration,
    int pageOffset = 0,
  }) {
    if (event.sourcePane == targetPane ||
        event.generation != currentGeneration) {
      return SplitSyncResult(applied: false, targetPane: targetPane);
    }
    return SplitSyncResult(
      applied: true,
      targetPane: targetPane,
      pageIndex: event.pageIndex == null ? null : event.pageIndex! + pageOffset,
      zoom: event.zoom,
      offset: event.offset,
    );
  }
}
