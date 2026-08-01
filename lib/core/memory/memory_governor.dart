enum MemoryOwner {
  activePage,
  renderCache,
  thumbnailCache,
  prefetch,
  indexing,
  split,
}

enum MemoryPriority { visible, cache, prefetch, background }

extension MemoryPriorityOrder on MemoryPriority {
  int get rank => switch (this) {
    MemoryPriority.visible => 400,
    MemoryPriority.cache => 300,
    MemoryPriority.prefetch => 200,
    MemoryPriority.background => 100,
  };
}

final class MemorySnapshot {
  const MemorySnapshot({
    required this.limitBytes,
    required this.retainedBytes,
    required this.reservationCount,
    required this.pressure,
    required this.bytesByOwner,
  });

  final int limitBytes;
  final int retainedBytes;
  final int reservationCount;
  final bool pressure;
  final Map<MemoryOwner, int> bytesByOwner;

  Map<String, Object> toJson() => {
    'limit_bytes': limitBytes,
    'retained_bytes': retainedBytes,
    'reservation_count': reservationCount,
    'pressure': pressure,
    'bytes_by_owner': {
      for (final entry in bytesByOwner.entries) entry.key.name: entry.value,
    },
  };
}

/// A logical application-level memory budget. It does not attempt to control
/// the Dart allocator; it makes image/cache/prefetch ownership measurable and
/// gives recreable work a chance to leave before visible content.
final class MemoryGovernor {
  MemoryGovernor({required int limitBytes, int minimumVisibleBytes = 0})
    : _limitBytes = limitBytes,
      minimumVisibleBytes = minimumVisibleBytes.clamp(0, limitBytes);

  int _limitBytes;
  final int minimumVisibleBytes;
  final List<MemoryReservation> _reservations = <MemoryReservation>[];

  int get limitBytes => _limitBytes;

  MemorySnapshot get snapshot {
    final byOwner = <MemoryOwner, int>{};
    var retained = 0;
    for (final reservation in _reservations) {
      if (reservation.isReleased) continue;
      retained += reservation.bytes;
      byOwner[reservation.owner] =
          (byOwner[reservation.owner] ?? 0) + reservation.bytes;
    }
    return MemorySnapshot(
      limitBytes: _limitBytes,
      retainedBytes: retained,
      reservationCount: _reservations.where((item) => !item.isReleased).length,
      pressure: retained >= (_limitBytes * 0.85).round(),
      bytesByOwner: Map.unmodifiable(byOwner),
    );
  }

  MemoryReservation? reserve({
    required MemoryOwner owner,
    required int bytes,
    required MemoryPriority priority,
    void Function()? onEvicted,
  }) {
    if (bytes <= 0 || bytes > _limitBytes) return null;
    _trimFor(bytes, priority);
    final retained = snapshot.retainedBytes;
    final visibleReserve = priority.rank < MemoryPriority.visible.rank
        ? minimumVisibleBytes
        : 0;
    if (retained + bytes > _limitBytes - visibleReserve) return null;
    final reservation = MemoryReservation._(
      governor: this,
      owner: owner,
      priority: priority,
      bytes: bytes,
      onEvicted: onEvicted,
    );
    _reservations.add(reservation);
    return reservation;
  }

  void setLimit(int bytes) {
    if (bytes <= 0) throw ArgumentError.value(bytes, 'bytes');
    _limitBytes = bytes;
    _trimFor(0, MemoryPriority.visible);
  }

  void releaseOwner(MemoryOwner owner) {
    for (final reservation in _reservations.toList(growable: false)) {
      if (reservation.owner == owner) reservation.release();
    }
    _compact();
  }

  void _trimFor(int requested, MemoryPriority incomingPriority) {
    while (snapshot.retainedBytes + requested > _limitBytes) {
      final candidates =
          _reservations
              .where(
                (item) =>
                    !item.isReleased &&
                    item.priority.rank < incomingPriority.rank,
              )
              .toList(growable: false)
            ..sort(
              (left, right) =>
                  left.priority.rank.compareTo(right.priority.rank),
            );
      if (candidates.isEmpty) return;
      candidates.first.release(evicted: true);
    }
    _compact();
  }

  void _remove(MemoryReservation reservation) {
    reservation._released = true;
    _compact();
  }

  void _compact() {
    _reservations.removeWhere((item) => item.isReleased);
  }
}

final class MemoryReservation {
  MemoryReservation._({
    required MemoryGovernor governor,
    required this.owner,
    required this.priority,
    required this.bytes,
    required this.onEvicted,
  }) : _governor = governor;

  final MemoryGovernor _governor;
  final MemoryOwner owner;
  final MemoryPriority priority;
  final int bytes;
  final void Function()? onEvicted;
  bool _released = false;

  bool get isReleased => _released;

  void release({bool evicted = false}) {
    if (_released) return;
    _governor._remove(this);
    if (evicted) {
      try {
        onEvicted?.call();
      } on Object {
        // Eviction callbacks cannot compromise the budget.
      }
    }
  }
}
