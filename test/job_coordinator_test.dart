import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/async/job_coordinator.dart';

void main() {
  test('deduplicates identical work and keeps both consumers alive', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    var executions = 0;

    Future<int> task(JobCancellationToken token) async {
      executions++;
      await Future<void>.delayed(Duration.zero);
      return 42;
    }

    final first = coordinator.submit<int>(
      key: 'same',
      category: JobCategory.indexing,
      priority: JobPriority.background,
      generation: 1,
      task: task,
    );
    final second = coordinator.submit<int>(
      key: 'same',
      category: JobCategory.indexing,
      priority: JobPriority.background,
      generation: 1,
      task: task,
    );

    expect(await first.future, 42);
    expect(await second.future, 42);
    expect(executions, 1);
  });

  test('cancelling one consumer does not cancel a shared job', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    final pending = Completer<int>();
    var tokenCancelled = false;

    JobHandle<int> submit() => coordinator.submit<int>(
      key: 'shared',
      category: JobCategory.render,
      priority: JobPriority.visible,
      generation: 2,
      task: (token) async {
        token.addListener(() => tokenCancelled = true);
        return pending.future;
      },
    );

    final first = submit();
    final second = submit();
    first.cancel();
    await expectLater(first.future, throwsA(isA<JobCancelled>()));
    expect(tokenCancelled, isFalse);
    pending.complete(7);
    expect(await second.future, 7);
  });

  test('older generations can be cancelled without exposing keys', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    final handle = coordinator.submit<int>(
      key: r'C:\private\book.pdf',
      category: JobCategory.thumbnail,
      priority: JobPriority.idle,
      generation: 1,
      task: (token) async {
        await Future<void>.delayed(const Duration(milliseconds: 1));
        return 1;
      },
    );
    coordinator.cancelBefore(2);
    await expectLater(handle.future, throwsA(isA<JobCancelled>()));
    expect(coordinator.snapshot.queued, 0);
  });

  test('cancelling the only queued consumer removes the queued job', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    final blocker = Completer<void>();
    final first = coordinator.submit<void>(
      key: 'blocker',
      category: JobCategory.render,
      priority: JobPriority.visible,
      generation: 1,
      task: (_) => blocker.future,
    );
    final queued = coordinator.submit<int>(
      key: 'queued-private-work',
      category: JobCategory.thumbnail,
      priority: JobPriority.background,
      generation: 1,
      task: (_) async => 1,
    );

    queued.cancel();
    await expectLater(queued.future, throwsA(isA<JobCancelled>()));
    expect(coordinator.snapshot.queued, 0);
    blocker.complete();
    await first.future;
  });

  test(
    'settled waits for native-style cleanup after consumer cancellation',
    () async {
      final coordinator = JobCoordinator(maxConcurrent: 1);
      final release = Completer<void>();
      final started = Completer<void>();
      final handle = coordinator.submit<void>(
        key: 'native-cleanup',
        category: JobCategory.render,
        priority: JobPriority.visible,
        generation: 1,
        task: (token) async {
          started.complete();
          token.addListener(() {});
          await release.future;
        },
      );
      await started.future;
      handle.cancel();
      await expectLater(handle.future, throwsA(isA<JobCancelled>()));
      var settled = false;
      handle.settled.then((_) => settled = true);
      await Future<void>.delayed(Duration.zero);
      expect(settled, isFalse);
      release.complete();
      await handle.settled;
      expect(settled, isTrue);
      await coordinator.dispose();
    },
  );

  test('a visible consumer promotes queued deduplicated work', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    final blocker = Completer<void>();
    final first = coordinator.submit<void>(
      key: 'blocker',
      category: JobCategory.render,
      priority: JobPriority.visible,
      generation: 1,
      task: (_) => blocker.future,
    );
    final background = coordinator.submit<int>(
      key: 'promoted',
      category: JobCategory.render,
      priority: JobPriority.background,
      generation: 1,
      task: (_) async => 7,
    );
    final visible = coordinator.submit<int>(
      key: 'promoted',
      category: JobCategory.render,
      priority: JobPriority.visible,
      generation: 1,
      task: (_) async => 99,
    );

    blocker.complete();
    await first.future;
    expect(await background.future, 7);
    expect(await visible.future, 7);
    expect(visible.descriptor.priority, JobPriority.visible);
  });

  test('background work pauses while visible work remains eligible', () async {
    final coordinator = JobCoordinator(maxConcurrent: 1);
    addTearDown(coordinator.dispose);
    final blocker = Completer<void>();
    coordinator.setBackgroundPaused(true);
    final background = coordinator.submit<int>(
      key: 'background',
      category: JobCategory.indexing,
      priority: JobPriority.background,
      generation: 1,
      task: (_) async => 1,
    );
    expect(coordinator.snapshot.queued, 1);

    final visible = coordinator.submit<int>(
      key: 'visible',
      category: JobCategory.render,
      priority: JobPriority.visible,
      generation: 1,
      task: (_) async {
        blocker.complete();
        return 2;
      },
    );
    expect(await visible.future, 2);
    expect(coordinator.snapshot.queued, 1);

    coordinator.setBackgroundPaused(false);
    expect(await background.future, 1);
    expect(blocker.isCompleted, isTrue);
  });

  test(
    'fairness lets queued background work run after a bounded burst',
    () async {
      final coordinator = JobCoordinator(maxConcurrent: 1);
      addTearDown(coordinator.dispose);
      final blocker = Completer<void>();
      final started = <String>[];

      final first = coordinator.submit<void>(
        key: 'fairness-blocker',
        category: JobCategory.render,
        priority: JobPriority.visible,
        generation: 1,
        task: (_) => blocker.future,
      );
      final background = coordinator.submit<String>(
        key: 'fairness-background',
        category: JobCategory.indexing,
        priority: JobPriority.background,
        generation: 1,
        task: (_) async {
          started.add('background');
          return 'background';
        },
      );
      final visible = [
        for (var index = 0; index < 5; index++)
          coordinator.submit<String>(
            key: 'fairness-visible-$index',
            category: JobCategory.render,
            priority: JobPriority.visible,
            generation: 1,
            task: (_) async {
              started.add('visible-$index');
              await Future<void>.delayed(Duration.zero);
              return 'visible';
            },
          ),
      ];

      blocker.complete();
      await first.future;
      expect(await background.future, 'background');
      await Future.wait(visible.map((handle) => handle.future));

      expect(started.indexOf('background'), lessThan(started.length - 1));
      expect(started.sublist(0, 5), contains('background'));
    },
  );
}
