import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/automations/domain/local_automation.dart';
import '../database/app_database.dart';

enum JournalStatus {
  planned,
  running,
  completed,
  cancelled,
  interrupted,
  rolledBack,
  failed,
}

enum JournalResumeCapability { none, retry, resume, rollback }

final class JournalEntry {
  const JournalEntry({
    required this.id,
    required this.kind,
    required this.status,
    required this.totalItems,
    required this.completedItems,
    required this.startedAt,
    required this.updatedAt,
    this.phase = 'planned',
    this.resumeCapability = JournalResumeCapability.none,
    this.interruptionReason,
    this.compatibilityHash,
    this.summary = const {},
    this.undo,
    this.errorCode,
  });

  final String id;
  final String kind;
  final JournalStatus status;
  final int totalItems;
  final int completedItems;
  final String phase;
  final JournalResumeCapability resumeCapability;
  final String? interruptionReason;
  final String? compatibilityHash;
  final Map<String, Object?> summary;
  final Map<String, Object?>? undo;
  final String? errorCode;
  final DateTime startedAt;
  final DateTime updatedAt;
}

final class AutomationRepository {
  const AutomationRepository(this._database);

  final AppDatabase _database;

  List<LocalAutomationRule> all() => _database.database
      .select('SELECT * FROM automation_rules ORDER BY updated_at DESC, id')
      .map(_ruleFromRow)
      .toList(growable: false);

  LocalAutomationRule save(LocalAutomationRule rule) {
    if (!rule.isValid) throw ArgumentError.value(rule, 'rule');
    final now = DateTime.now().toUtc();
    final id = rule.id.isEmpty
        ? sha256
              .convert(
                utf8.encode('rule:${rule.name}:${now.microsecondsSinceEpoch}'),
              )
              .toString()
        : rule.id;
    _database.database.execute(
      '''
      INSERT INTO automation_rules(
        id, name, enabled, trigger_json, actions_json, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(id) DO UPDATE SET
        name = excluded.name,
        enabled = excluded.enabled,
        trigger_json = excluded.trigger_json,
        actions_json = excluded.actions_json,
        updated_at = excluded.updated_at
      ''',
      [
        id,
        rule.name.trim(),
        rule.enabled ? 1 : 0,
        jsonEncode(rule.trigger.toJson()),
        jsonEncode(rule.actions.map((action) => action.toJson()).toList()),
        rule.createdAt.toUtc().toIso8601String(),
        now.toIso8601String(),
      ],
    );
    return all().firstWhere((item) => item.id == id);
  }

  void delete(String id) {
    _database.database.execute('DELETE FROM automation_rules WHERE id = ?', [
      id,
    ]);
  }

  JournalEntry startJournal({
    required String kind,
    required int totalItems,
    Map<String, Object?> summary = const {},
    String phase = 'planned',
    JournalResumeCapability resumeCapability = JournalResumeCapability.none,
    String? compatibilityHash,
  }) {
    final now = DateTime.now().toUtc();
    final id = sha256
        .convert(utf8.encode('journal:$kind:${now.microsecondsSinceEpoch}'))
        .toString();
    _database.database.execute(
      'INSERT INTO operation_journal('
      'id, operation_kind, status, summary_json, total_items, '
      'completed_items, started_at, updated_at, phase, resume_capability, '
      'compatibility_hash) '
      'VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?)',
      [
        id,
        kind,
        JournalStatus.running.name,
        jsonEncode(summary),
        totalItems.clamp(0, 100000),
        now.toIso8601String(),
        now.toIso8601String(),
        phase,
        resumeCapability.name,
        compatibilityHash,
      ],
    );
    return journal(id)!;
  }

  void updateJournal(
    String id, {
    required JournalStatus status,
    int? completedItems,
    Map<String, Object?>? undo,
    String? errorCode,
    String? phase,
    String? interruptionReason,
    JournalResumeCapability? resumeCapability,
  }) {
    _database.database.execute(
      '''
      UPDATE operation_journal SET
        status = ?,
        completed_items = COALESCE(?, completed_items),
        undo_json = COALESCE(?, undo_json),
        error_code = ?,
        phase = COALESCE(?, phase),
        interruption_reason = ?,
        resume_capability = COALESCE(?, resume_capability),
        updated_at = ?
      WHERE id = ?
      ''',
      [
        status.name,
        completedItems,
        undo == null ? null : jsonEncode(undo),
        errorCode,
        phase,
        interruptionReason,
        resumeCapability?.name,
        DateTime.now().toUtc().toIso8601String(),
        id,
      ],
    );
  }

  JournalEntry? journal(String id) {
    final rows = _database.database.select(
      'SELECT * FROM operation_journal WHERE id = ?',
      [id],
    );
    return rows.isEmpty ? null : _journalFromRow(rows.first);
  }

  List<JournalEntry> unfinished() => _database.database
      .select(
        "SELECT * FROM operation_journal WHERE status IN "
        "('planned', 'running', 'interrupted') ORDER BY updated_at DESC",
      )
      .map(_journalFromRow)
      .toList(growable: false);

  void markRunningAsInterrupted() {
    _database.database.execute(
      "UPDATE operation_journal SET status = 'interrupted', "
      "interruption_reason = 'process_interrupted', "
      "resume_capability = CASE WHEN undo_json IS NULL THEN 'retry' "
      "ELSE 'rollback' END, updated_at = ? "
      "WHERE status = 'running'",
      [DateTime.now().toUtc().toIso8601String()],
    );
  }

  static LocalAutomationRule _ruleFromRow(dynamic row) {
    final trigger = (jsonDecode(row['trigger_json'] as String) as Map)
        .cast<String, Object?>();
    final actions = jsonDecode(row['actions_json'] as String) as List;
    return LocalAutomationRule(
      id: row['id'] as String,
      name: row['name'] as String,
      trigger: AutomationTrigger.fromJson(trigger),
      actions: actions
          .whereType<Map>()
          .map(
            (item) => AutomationAction.fromJson(item.cast<String, Object?>()),
          )
          .toList(growable: false),
      enabled: (row['enabled'] as int) == 1,
      createdAt: DateTime.parse(row['created_at'] as String),
      updatedAt: DateTime.parse(row['updated_at'] as String),
    );
  }

  static JournalEntry _journalFromRow(dynamic row) => JournalEntry(
    id: row['id'] as String,
    kind: row['operation_kind'] as String,
    status: JournalStatus.values.firstWhere(
      (item) => item.name == row['status'],
      orElse: () => JournalStatus.failed,
    ),
    totalItems: row['total_items'] as int,
    completedItems: row['completed_items'] as int,
    phase: row['phase'] as String? ?? 'planned',
    resumeCapability: JournalResumeCapability.values.firstWhere(
      (item) => item.name == row['resume_capability'],
      orElse: () => JournalResumeCapability.none,
    ),
    interruptionReason: row['interruption_reason'] as String?,
    compatibilityHash: row['compatibility_hash'] as String?,
    summary: (jsonDecode(row['summary_json'] as String) as Map)
        .cast<String, Object?>(),
    undo: switch (row['undo_json'] as String?) {
      final value? => (jsonDecode(value) as Map).cast<String, Object?>(),
      null => null,
    },
    errorCode: row['error_code'] as String?,
    startedAt: DateTime.parse(row['started_at'] as String),
    updatedAt: DateTime.parse(row['updated_at'] as String),
  );
}
