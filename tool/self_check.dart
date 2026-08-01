import 'package:yamantaka/core/async/job_coordinator.dart';
import 'package:yamantaka/core/lifecycle/lifecycle_tracker.dart';
import 'package:yamantaka/core/memory/memory_governor.dart';
import 'package:yamantaka/core/performance/prefetch_planner.dart';
import 'package:yamantaka/core/performance/progressive_open.dart';
import 'package:yamantaka/core/platform/print_request.dart';
import 'package:yamantaka/core/platform/single_instance_protocol.dart';
import 'package:yamantaka/core/platform/window_geometry.dart';
import 'package:yamantaka/core/privacy/diagnostics_redactor.dart';
import 'package:yamantaka/core/recovery/recovery_policy.dart';
import 'package:yamantaka/core/security/local_input_policy.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';
import 'package:yamantaka/features/navigation/domain/unified_navigation.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/workspaces/domain/split_sync.dart';
import 'sha256.dart';

Future<void> main() async {
  final jobs = JobCoordinator(maxConcurrent: 1);
  var executions = 0;
  final first = jobs.submit<int>(
    key: 'self-check',
    category: JobCategory.maintenance,
    priority: JobPriority.visible,
    generation: 1,
    task: (_) async {
      executions++;
      return 42;
    },
  );
  final second = jobs.submit<int>(
    key: 'self-check',
    category: JobCategory.maintenance,
    priority: JobPriority.visible,
    generation: 1,
    task: (_) async => 99,
  );
  assert(await first.future == 42);
  assert(await second.future == 42);
  assert(executions == 1);
  await jobs.dispose();

  final memory = MemoryGovernor(limitBytes: 100, minimumVisibleBytes: 20);
  final cache = memory.reserve(
    owner: MemoryOwner.renderCache,
    bytes: 80,
    priority: MemoryPriority.cache,
  );
  final visible = memory.reserve(
    owner: MemoryOwner.activePage,
    bytes: 40,
    priority: MemoryPriority.visible,
  );
  assert(cache?.isReleased == true);
  assert(visible != null);

  final lifecycle = LifecycleTracker();
  final lease = lifecycle.acquire(ResourceKind.document);
  assert(lifecycle.snapshot.counts[ResourceKind.document] == 1);
  lease.release();
  assert(lifecycle.snapshot.counts[ResourceKind.document] == 0);
  lifecycle.dispose();

  final bounds = const WindowBounds(x: -5000, y: -5000, width: 100, height: 100)
      .normalized(
        workArea: const WindowWorkArea(
          left: 0,
          top: 0,
          width: 1920,
          height: 1080,
        ),
      );
  assert(bounds.width >= 640 && bounds.height >= 480);
  assert(bounds.x >= -616 && bounds.y >= -456);

  assert(
    LocalInputPolicy.authorizedExternalUri(Uri.parse('javascript:alert(1)')) ==
        null,
  );
  assert(
    SingleInstanceProtocol.fromArguments(['a.pdf', 'a.pdf']).pdfPaths.length ==
        1,
  );
  assert(
    const PrintRequest(firstPage: -1, lastPage: 99).normalized(3).lastPage == 2,
  );
  assert(RecoveryPolicy.decide(RecoveryCause.uncleanShutdown).safeMode);
  assert(
    !DiagnosticsRedactor.redact({
      'path': 'private',
      'documents': 2,
    }).containsKey('path'),
  );
  assert(const LibraryQuery(limit: 999).normalized().limit == 200);
  assert(
    const UnifiedNavigationTarget(
          pageIndex: 2,
          kind: UnifiedNavigationKind.page,
        ).canonical(pageCount: 5, mode: ReaderMode.book).pageIndex ==
        1,
  );
  assert(
    SplitSynchronizer.propagate(
      event: const SplitSyncEvent(
        sourcePane: 0,
        mode: SplitSyncMode.page,
        generation: 1,
        pageIndex: 3,
      ),
      targetPane: 1,
      currentGeneration: 1,
    ).applied,
  );
  assert(
    PrefetchPlanner.plan(
      samples: const [],
      currentPage: 0,
      pageCount: 3,
      mode: ReaderMode.document,
      generation: 1,
    ).pages.isEmpty,
  );
  assert(
    const ProgressiveOpenState(
      phase: ProgressiveOpenPhase.firstRender,
      generation: 1,
    ).canPublish(1),
  );
  assert(
    sha256Hex('abc'.codeUnits) ==
        'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
  );
}
