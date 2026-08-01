enum BackgroundActivity { visible, idle, inactive, minimized }

final class BackgroundActivityPolicy {
  const BackgroundActivityPolicy({
    required this.activity,
    required this.memoryPressure,
  });

  final BackgroundActivity activity;
  final bool memoryPressure;

  bool get allowPrefetch =>
      activity == BackgroundActivity.visible && !memoryPressure;
  bool get allowIndexing =>
      activity != BackgroundActivity.minimized && !memoryPressure;
  bool get allowMaintenance =>
      activity == BackgroundActivity.idle && !memoryPressure;

  Map<String, Object> toJson() => {
    'activity': activity.name,
    'memory_pressure': memoryPressure,
    'allow_prefetch': allowPrefetch,
    'allow_indexing': allowIndexing,
    'allow_maintenance': allowMaintenance,
  };
}
