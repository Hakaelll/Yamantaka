// ignore_for_file: prefer_initializing_formals

import 'dart:async';
import 'dart:collection';

import '../lifecycle/lifecycle_tracker.dart';

/// The coarse work buckets used by the application scheduler.
///
/// These names intentionally describe product work rather than implementation
/// details. They are safe to expose in diagnostics because they never contain
/// a path, title, query, or document identifier.
enum JobCategory {
  render,
  thumbnail,
  indexing,
  import,
  backup,
  recovery,
  maintenance,
}

enum JobPriority { visible, interactive, background, idle }

extension JobPriorityWeight on JobPriority {
  int get weight => switch (this) {
    JobPriority.visible => 400,
    JobPriority.interactive => 300,
    JobPriority.background => 200,
    JobPriority.idle => 100,
  };
}

/// A cancellation error is deliberately small and does not include the job
/// key. Keys are allowed to contain private paths and must never reach logs.
final class JobCancelled implements Exception {
  const JobCancelled();

  @override
  String toString() => 'JobCancelled';
}

final class JobCancellationToken {
  bool _cancelled = false;
  final Set<void Function()> _listeners = <void Function()>{};

  bool get isCancelled => _cancelled;

  void addListener(void Function() listener) {
    if (_cancelled) {
      try {
        listener();
      } on Object {
        // A late listener must not turn cancellation into a new failure.
      }
      return;
    }
    _listeners.add(listener);
  }

  void removeListener(void Function() listener) => _listeners.remove(listener);

  void cancel() {
    if (_cancelled) return;
    _cancelled = true;
    final listeners = _listeners.toList(growable: false);
    _listeners.clear();
    for (final listener in listeners) {
      try {
        listener();
      } on Object {
        // Cancellation is best effort. One listener must not block cleanup.
      }
    }
  }
}

final class JobDescriptor {
  const JobDescriptor({
    required this.category,
    required this.priority,
    required this.generation,
  });

  final JobCategory category;
  final JobPriority priority;
  final int generation;
}

final class JobSnapshot {
  const JobSnapshot({
    required this.queued,
    required this.active,
    required this.consumers,
    required this.completed,
    required this.cancelled,
    required this.failed,
    required this.averageDurationMs,
  });

  final int queued;
  final int active;
  final int consumers;
  final int completed;
  final int cancelled;
  final int failed;
  final double averageDurationMs;

  Map<String, Object> toJson() => {
    'queued': queued,
    'active': active,
    'consumers': consumers,
    'completed': completed,
    'cancelled': cancelled,
    'failed': failed,
    'average_duration_ms': averageDurationMs,
  };
}

final class JobHandle<T> {
  JobHandle._(this.descriptor, this._consumer)
    : future = _consumer.completer.future.then((value) => value as T);

  final JobDescriptor descriptor;
  final _JobConsumer _consumer;
  final Future<T> future;
  Future<void> get settled => _consumer.shared.settled.future;

  bool get isCancelled => _consumer.cancelled;

  void cancel() => _consumer.cancel();
}

/// Coordinates cancellable, deduplicated work without persisting private job
/// input. A key is only used in memory to share an identical operation.
final class JobCoordinator {
  JobCoordinator({
    this.maxConcurrent = 4,
    Map<JobCategory, int>? categoryLimits,
    LifecycleTracker? lifecycle,
  }) : assert(maxConcurrent > 0),
       _categoryLimits = {
         for (final category in JobCategory.values)
           category: (categoryLimits?[category] ?? maxConcurrent).clamp(
             1,
             maxConcurrent,
           ),
       },
       _lifecycle = lifecycle;

  final int maxConcurrent;
  final Map<JobCategory, int> _categoryLimits;
  final LifecycleTracker? _lifecycle;
  final Map<String, _SharedJob> _jobs = <String, _SharedJob>{};
  final Queue<_SharedJob> _queue = Queue<_SharedJob>();
  final Map<JobCategory, int> _activeByCategory = <JobCategory, int>{};
  bool _disposed = false;
  bool _backgroundPaused = false;
  int _active = 0;
  int _completed = 0;
  int _cancelled = 0;
  int _failed = 0;
  int _durationTotalMs = 0;
  int _consecutiveHighPriorityStarts = 0;

  static const int _highPriorityBurstLimit = 4;

  JobHandle<T> submit<T>({
    required String key,
    required JobCategory category,
    required JobPriority priority,
    required int generation,
    required Future<T> Function(JobCancellationToken token) task,
    void Function()? onCleanup,
  }) {
    if (_disposed) throw StateError('Job coordinator is disposed.');
    if (key.trim().isEmpty) throw ArgumentError.value(key, 'key');
    final mapKey = _mapKey(key, category, generation);
    var shared = _jobs[mapKey];
    if (shared == null) {
      shared = _SharedJob(
        mapKey: mapKey,
        descriptor: JobDescriptor(
          category: category,
          priority: priority,
          generation: generation,
        ),
        task: (token) => task(token),
        onCleanup: onCleanup,
      );
      _jobs[mapKey] = shared;
      _queue.add(shared);
    } else if (!shared.started &&
        priority.weight > shared.descriptor.priority.weight) {
      shared.descriptor = JobDescriptor(
        category: category,
        priority: priority,
        generation: generation,
      );
    }
    final consumer = _JobConsumer(shared, _pump);
    shared.consumers.add(consumer);
    _pump();
    return JobHandle<T>._(shared.descriptor, consumer);
  }

  /// Cancels work from older document/view generations while leaving current
  /// consumers untouched. It is used when a document is replaced quickly.
  void cancelBefore(int generation) {
    for (final job in _jobs.values.toList(growable: false)) {
      if (job.descriptor.generation < generation) job.cancelUnderlying();
    }
    _pump();
  }

  /// Prevents new background/idle work while the desktop surface is hidden.
  /// Visible and interactive work remains eligible so an explicit user action
  /// is never stranded behind an inactive-window policy.
  void setBackgroundPaused(bool value) {
    if (_backgroundPaused == value) return;
    _backgroundPaused = value;
    _pump();
  }

  JobSnapshot get snapshot {
    var consumers = 0;
    var queued = 0;
    for (final job in _jobs.values) {
      consumers += job.consumers.where((item) => !item.cancelled).length;
      if (!job.started && !job.token.isCancelled) queued++;
    }
    return JobSnapshot(
      queued: queued,
      active: _active,
      consumers: consumers,
      completed: _completed,
      cancelled: _cancelled,
      failed: _failed,
      averageDurationMs: _completed == 0 ? 0 : _durationTotalMs / _completed,
    );
  }

  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    final jobs = _jobs.values.toList(growable: false);
    for (final job in jobs) {
      job.cancelUnderlying();
      for (final consumer in job.consumers.toList(growable: false)) {
        consumer.completeError(const JobCancelled());
      }
    }
    for (final job in jobs.where((item) => !item.started)) {
      if (!job.settled.isCompleted) job.settled.complete();
    }
    _jobs.clear();
    _queue.clear();
    await Future.wait(jobs.map((job) => job.settled.future));
  }

  void _pump() {
    if (_disposed) return;
    _discardCancelledQueuedJobs();
    while (_active < maxConcurrent) {
      final next = _nextRunnable();
      if (next == null) return;
      next.started = true;
      _active++;
      _activeByCategory[next.descriptor.category] =
          (_activeByCategory[next.descriptor.category] ?? 0) + 1;
      unawaited(_run(next));
    }
  }

  void _discardCancelledQueuedJobs() {
    final stale = _queue
        .where((job) => job.token.isCancelled || job.consumers.isEmpty)
        .toList(growable: false);
    for (final job in stale) {
      _queue.remove(job);
      if (_jobs.remove(job.mapKey) != null) {
        _cancelled++;
        try {
          job.onCleanup?.call();
        } on Object {
          // Cleanup is isolated from scheduler accounting.
        }
        if (!job.settled.isCompleted) job.settled.complete();
      }
    }
  }

  _SharedJob? _nextRunnable() {
    _SharedJob? bestHighPriority;
    _SharedJob? bestBackground;
    for (final job in _queue) {
      if (job.started || job.token.isCancelled || job.consumers.isEmpty) {
        continue;
      }
      if (_backgroundPaused &&
          (job.descriptor.priority == JobPriority.background ||
              job.descriptor.priority == JobPriority.idle)) {
        continue;
      }
      final limit = _categoryLimits[job.descriptor.category] ?? maxConcurrent;
      if ((_activeByCategory[job.descriptor.category] ?? 0) >= limit) {
        continue;
      }
      if (job.descriptor.priority == JobPriority.visible ||
          job.descriptor.priority == JobPriority.interactive) {
        if (bestHighPriority == null ||
            job.descriptor.priority.weight >
                bestHighPriority.descriptor.priority.weight) {
          bestHighPriority = job;
        }
      } else if (bestBackground == null ||
          job.descriptor.priority.weight >
              bestBackground.descriptor.priority.weight) {
        bestBackground = job;
      }
    }
    final best =
        bestBackground != null &&
            _consecutiveHighPriorityStarts >= _highPriorityBurstLimit
        ? bestBackground
        : bestHighPriority ?? bestBackground;
    if (best != null) _queue.remove(best);
    if (best == null) return null;
    if (best.descriptor.priority == JobPriority.visible ||
        best.descriptor.priority == JobPriority.interactive) {
      _consecutiveHighPriorityStarts++;
    } else {
      _consecutiveHighPriorityStarts = 0;
    }
    return best;
  }

  Future<void> _run(_SharedJob job) async {
    final stopwatch = Stopwatch()..start();
    final jobLease = _lifecycle?.acquire(ResourceKind.job);
    Object? value;
    StackTrace? stackTrace;
    Object? failure;
    try {
      value = await job.task(job.token);
      if (job.token.isCancelled) {
        failure = const JobCancelled();
      }
    } on Object catch (error, stack) {
      failure = error;
      stackTrace = stack;
    } finally {
      stopwatch.stop();
      _active--;
      final category = job.descriptor.category;
      _activeByCategory[category] = (_activeByCategory[category] ?? 1) - 1;
      _jobs.remove(job.mapKey);
      _completed++;
      _durationTotalMs += stopwatch.elapsedMilliseconds;
      if (failure != null) {
        if (failure is JobCancelled) {
          _cancelled++;
        } else {
          _failed++;
        }
      }
      for (final consumer in job.consumers.toList(growable: false)) {
        if (consumer.cancelled) continue;
        if (failure != null) {
          consumer.completeError(failure, stackTrace);
        } else {
          consumer.complete(value);
        }
      }
      try {
        job.onCleanup?.call();
      } on Object {
        // Cleanup is isolated from scheduler accounting.
      }
      jobLease?.release();
      if (!job.settled.isCompleted) job.settled.complete();
      _pump();
    }
  }

  static String _mapKey(String key, JobCategory category, int generation) =>
      '${category.name}:$generation:$key';
}

final class _SharedJob {
  _SharedJob({
    required this.mapKey,
    required this.descriptor,
    required this.task,
    required this.onCleanup,
  });

  final String mapKey;
  JobDescriptor descriptor;
  final Future<Object?> Function(JobCancellationToken token) task;
  final void Function()? onCleanup;
  final JobCancellationToken token = JobCancellationToken();
  final Set<_JobConsumer> consumers = <_JobConsumer>{};
  final Completer<void> settled = Completer<void>();
  bool started = false;

  void cancelUnderlying() {
    token.cancel();
    for (final consumer in consumers) {
      consumer.completeError(const JobCancelled());
    }
  }
}

final class _JobConsumer {
  _JobConsumer(this.shared, this._onCancelled);

  final _SharedJob shared;
  final void Function() _onCancelled;
  final Completer<Object?> completer = Completer<Object?>();
  bool cancelled = false;

  void cancel() {
    if (cancelled || completer.isCompleted) return;
    cancelled = true;
    shared.consumers.remove(this);
    completeError(const JobCancelled());
    if (shared.consumers.isEmpty) shared.cancelUnderlying();
    _onCancelled();
  }

  void complete(Object? value) {
    if (!completer.isCompleted) completer.complete(value);
  }

  void completeError(Object error, [StackTrace? stackTrace]) {
    if (!completer.isCompleted) completer.completeError(error, stackTrace);
  }
}
