import 'dart:collection';

enum StartupPhase {
  bootstrap,
  binding,
  paths,
  settings,
  nativePdf,
  window,
  database,
  services,
  firstLibrary,
  firstFrame,
  firstPdf,
}

final class StartupPhaseTiming {
  const StartupPhaseTiming({required this.phase, required this.durationMs});

  final StartupPhase phase;
  final int durationMs;

  Map<String, Object> toJson() => {
    'phase': phase.name,
    'duration_ms': durationMs,
  };
}

final class StartupTimeline {
  StartupTimeline({DateTime Function()? clock})
    : _clock = clock ?? DateTime.now;

  final DateTime Function() _clock;
  final Queue<StartupPhaseTiming> _completed = Queue<StartupPhaseTiming>();
  final Map<StartupPhase, DateTime> _active = <StartupPhase, DateTime>{};

  List<StartupPhaseTiming> get timings => List.unmodifiable(_completed);

  StartupSpan start(StartupPhase phase) {
    _active[phase] = _clock();
    return StartupSpan._(this, phase);
  }

  bool startIfAbsent(StartupPhase phase) {
    if (_active.containsKey(phase) ||
        _completed.any((timing) => timing.phase == phase)) {
      return false;
    }
    _active[phase] = _clock();
    return true;
  }

  StartupPhaseTiming? complete(StartupPhase phase) {
    final started = _active.remove(phase);
    if (started == null) return null;
    final timing = StartupPhaseTiming(
      phase: phase,
      durationMs: _clock().difference(started).inMilliseconds,
    );
    _completed.add(timing);
    return timing;
  }

  Map<String, Object> toJson() => {
    'phases': [for (final timing in timings) timing.toJson()],
    'completed': _active.isEmpty,
  };
}

final class StartupSpan {
  StartupSpan._(this._timeline, this.phase);

  final StartupTimeline _timeline;
  final StartupPhase phase;
  bool _ended = false;

  StartupPhaseTiming? end() {
    if (_ended) return null;
    _ended = true;
    return _timeline.complete(phase);
  }
}
