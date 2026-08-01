enum ProgressiveOpenPhase {
  idle,
  nativeOpen,
  firstLayout,
  firstRender,
  enrichment,
  complete,
  cancelled,
  failed,
}

final class ProgressiveOpenState {
  const ProgressiveOpenState({
    required this.phase,
    required this.generation,
    this.pageIndex = 0,
    this.errorCode,
  });

  final ProgressiveOpenPhase phase;
  final int generation;
  final int pageIndex;
  final String? errorCode;

  bool get isUsable =>
      phase.index >= ProgressiveOpenPhase.firstRender.index &&
      phase != ProgressiveOpenPhase.cancelled &&
      phase != ProgressiveOpenPhase.failed;

  bool canPublish(int currentGeneration) =>
      generation == currentGeneration &&
      phase != ProgressiveOpenPhase.cancelled;

  Map<String, Object?> toJson() => {
    'phase': phase.name,
    'generation': generation,
    'page_index': pageIndex,
    if (errorCode != null) 'error_code': errorCode,
  };
}

abstract final class ProgressiveOpenPolicy {
  static const phases = <ProgressiveOpenPhase>[
    ProgressiveOpenPhase.nativeOpen,
    ProgressiveOpenPhase.firstLayout,
    ProgressiveOpenPhase.firstRender,
    ProgressiveOpenPhase.enrichment,
    ProgressiveOpenPhase.complete,
  ];

  static ProgressiveOpenState cancel(ProgressiveOpenState current) =>
      ProgressiveOpenState(
        phase: ProgressiveOpenPhase.cancelled,
        generation: current.generation,
        pageIndex: current.pageIndex,
      );

  static ProgressiveOpenState failure(
    ProgressiveOpenState current,
    String errorCode,
  ) => ProgressiveOpenState(
    phase: ProgressiveOpenPhase.failed,
    generation: current.generation,
    pageIndex: current.pageIndex,
    errorCode: errorCode,
  );
}
