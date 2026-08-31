import '../../../app/app_config.dart';
import '../../../storage/repositories/automation_repository.dart';
import '../../library/domain/library_document.dart';
import '../domain/local_automation.dart';

final class AutomationCancellationToken {
  bool _cancelled = false;
  bool get isCancelled => _cancelled;
  void cancel() => _cancelled = true;
}

typedef AutomationActionApplier =
    Future<Map<String, Object?>> Function(
      LibraryDocument document,
      AutomationAction action,
    );

typedef AutomationUndoApplier =
    Future<void> Function(Map<String, Object?> undo);

final class AutomationApplyResult {
  const AutomationApplyResult({
    required this.journalId,
    required this.status,
    required this.completed,
    required this.total,
  });

  final String journalId;
  final JournalStatus status;
  final int completed;
  final int total;
}

final class LocalAutomationEngine {
  const LocalAutomationEngine(this._repository);

  final AutomationRepository _repository;

  AutomationSimulation simulate(
    LocalAutomationRule rule,
    Iterable<LibraryDocument> documents, {
    AutomationMatchContext context = const AutomationMatchContext(),
    int limit = AppConfig.maxAutomationDocuments,
    Duration timeLimit = AppConfig.maxAutomationDuration,
  }) {
    if (!rule.isValid) throw ArgumentError.value(rule, 'rule');
    if (timeLimit.isNegative) throw ArgumentError.value(timeLimit, 'timeLimit');
    final stopwatch = Stopwatch()..start();
    final matches = <String>[];
    var scanned = 0;
    var truncated = false;
    for (final document in documents) {
      if (stopwatch.elapsed >= timeLimit) {
        truncated = true;
        break;
      }
      if (scanned >= limit.clamp(1, AppConfig.maxAutomationDocuments)) {
        truncated = true;
        break;
      }
      scanned++;
      if (automationMatches(rule, document, context)) matches.add(document.id);
    }
    return AutomationSimulation(
      documentIds: List.unmodifiable(matches),
      scanned: scanned,
      truncated: truncated,
    );
  }

  Future<AutomationSimulation> simulateAsync(
    LocalAutomationRule rule,
    Iterable<LibraryDocument> documents, {
    AutomationMatchContext context = const AutomationMatchContext(),
    int limit = AppConfig.maxAutomationDocuments,
    Duration timeLimit = AppConfig.maxAutomationDuration,
    AutomationCancellationToken? cancellationToken,
  }) async {
    if (!rule.isValid) throw ArgumentError.value(rule, 'rule');
    if (timeLimit.isNegative) throw ArgumentError.value(timeLimit, 'timeLimit');
    final stopwatch = Stopwatch()..start();
    final matches = <String>[];
    var scanned = 0;
    var truncated = false;
    for (final document in documents) {
      if (cancellationToken?.isCancelled == true ||
          stopwatch.elapsed >= timeLimit) {
        truncated = true;
        break;
      }
      if (scanned >= limit.clamp(1, AppConfig.maxAutomationDocuments)) {
        truncated = true;
        break;
      }
      scanned++;
      if (automationMatches(rule, document, context)) matches.add(document.id);
      if (scanned % AppConfig.automationBatchSize == 0) {
        await Future<void>.delayed(Duration.zero);
      }
    }
    return AutomationSimulation(
      documentIds: List.unmodifiable(matches),
      scanned: scanned,
      truncated: truncated,
    );
  }

  Future<AutomationApplyResult> apply({
    required LocalAutomationRule rule,
    required Iterable<LibraryDocument> documents,
    required AutomationActionApplier applyAction,
    AutomationMatchContext context = const AutomationMatchContext(),
    AutomationCancellationToken? cancellationToken,
    AutomationUndoApplier? rollbackAction,
    Duration timeLimit = AppConfig.maxAutomationDuration,
  }) async {
    if (timeLimit.isNegative) throw ArgumentError.value(timeLimit, 'timeLimit');
    final stopwatch = Stopwatch()..start();
    final candidates = <LibraryDocument>[];
    var scanned = 0;
    var scanCancelled = false;
    var scanTimedOut = false;
    for (final document in documents) {
      if (cancellationToken?.isCancelled == true) {
        scanCancelled = true;
        break;
      }
      if (stopwatch.elapsed >= timeLimit) {
        scanTimedOut = true;
        break;
      }
      if (scanned >= AppConfig.maxAutomationDocuments) break;
      scanned++;
      if (automationMatches(rule, document, context)) candidates.add(document);
      if (scanned % AppConfig.automationBatchSize == 0) {
        await Future<void>.delayed(Duration.zero);
      }
    }
    final journal = _repository.startJournal(
      kind: 'automation',
      totalItems: candidates.length,
      summary: {'ruleId': rule.id, 'actions': rule.actions.length},
    );
    final undo = <String, Object?>{};
    var completed = 0;
    try {
      if (scanCancelled || scanTimedOut) {
        _repository.updateJournal(
          journal.id,
          status: JournalStatus.cancelled,
          completedItems: 0,
          errorCode: scanTimedOut ? 'time_limit_exceeded' : null,
        );
        return AutomationApplyResult(
          journalId: journal.id,
          status: JournalStatus.cancelled,
          completed: 0,
          total: candidates.length,
        );
      }
      for (
        var start = 0;
        start < candidates.length;
        start += AppConfig.automationBatchSize
      ) {
        final timedOut = stopwatch.elapsed >= timeLimit;
        if (cancellationToken?.isCancelled == true || timedOut) {
          final rollbackOk =
              rollbackAction == null ||
              await _tryRollback(undo, rollbackAction);
          final status = rollbackOk
              ? JournalStatus.cancelled
              : JournalStatus.failed;
          _repository.updateJournal(
            journal.id,
            status: status,
            completedItems: completed,
            undo: undo,
            errorCode: rollbackOk
                ? (timedOut ? 'time_limit_exceeded' : null)
                : 'rollback_failed',
          );
          return AutomationApplyResult(
            journalId: journal.id,
            status: status,
            completed: completed,
            total: candidates.length,
          );
        }
        final end = (start + AppConfig.automationBatchSize).clamp(
          0,
          candidates.length,
        );
        for (final document in candidates.sublist(start, end)) {
          for (
            var actionIndex = 0;
            actionIndex < rule.actions.length;
            actionIndex++
          ) {
            final action = rule.actions[actionIndex];
            undo['${document.id}:$actionIndex:${action.kind.name}'] =
                await applyAction(document, action);
          }
          completed++;
        }
        _repository.updateJournal(
          journal.id,
          status: JournalStatus.running,
          completedItems: completed,
          undo: undo,
        );
        await Future<void>.delayed(Duration.zero);
      }
      _repository.updateJournal(
        journal.id,
        status: JournalStatus.completed,
        completedItems: completed,
        undo: undo,
      );
      return AutomationApplyResult(
        journalId: journal.id,
        status: JournalStatus.completed,
        completed: completed,
        total: candidates.length,
      );
    } on Object {
      final rollbackOk =
          rollbackAction == null || await _tryRollback(undo, rollbackAction);
      _repository.updateJournal(
        journal.id,
        status: JournalStatus.failed,
        completedItems: completed,
        undo: undo,
        errorCode: rollbackOk ? 'apply_failed' : 'rollback_failed',
      );
      rethrow;
    }
  }

  static Future<void> _rollback(
    Map<String, Object?> undo,
    AutomationUndoApplier applyUndo,
  ) async {
    for (final value in undo.values.toList().reversed) {
      if (value is Map) await applyUndo(value.cast<String, Object?>());
    }
  }

  static Future<bool> _tryRollback(
    Map<String, Object?> undo,
    AutomationUndoApplier applyUndo,
  ) async {
    try {
      await _rollback(undo, applyUndo);
      return true;
    } on Object {
      return false;
    }
  }

  Future<bool> undo(String journalId, AutomationUndoApplier applyUndo) async {
    final journal = _repository.journal(journalId);
    if (journal == null || journal.undo == null) return false;
    for (final value in journal.undo!.values.toList().reversed) {
      if (value is Map) await applyUndo(value.cast<String, Object?>());
    }
    _repository.updateJournal(
      journalId,
      status: JournalStatus.rolledBack,
      completedItems: journal.completedItems,
    );
    return true;
  }
}
