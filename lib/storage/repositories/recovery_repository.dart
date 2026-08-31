import 'dart:convert';

import '../../features/recovery/domain/recovery_models.dart';
import '../database/app_database.dart';

final class RecoveryRepository {
  const RecoveryRepository(this._database);

  final AppDatabase _database;
  static const _sessionKey = 'atomic_session_snapshot_v1';

  void saveSession(AtomicSessionSnapshot snapshot) {
    _database.writeTransaction(() {
      _database.database.execute(
        '''
        INSERT INTO app_metadata(key, value, updated_at) VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET
          value = excluded.value, updated_at = excluded.updated_at
        ''',
        [
          _sessionKey,
          jsonEncode(snapshot.toJson()),
          snapshot.savedAt.toUtc().toIso8601String(),
        ],
      );
    });
  }

  AtomicSessionSnapshot? session() {
    final rows = _database.database.select(
      'SELECT value FROM app_metadata WHERE key = ?',
      [_sessionKey],
    );
    if (rows.isEmpty) return null;
    try {
      return AtomicSessionSnapshot.fromJson(
        (jsonDecode(rows.first['value'] as String) as Map)
            .cast<String, Object?>(),
      );
    } on Object {
      return null;
    }
  }

  void discardSession() {
    _database.database.execute('DELETE FROM app_metadata WHERE key = ?', [
      _sessionKey,
    ]);
  }

  List<RecoveryProblem> problems({int limit = 80}) {
    final result = <RecoveryProblem>[];
    final journals = _database.database.select(
      "SELECT id, status, operation_kind, updated_at FROM operation_journal "
      "WHERE status IN ('running', 'interrupted', 'failed') "
      'ORDER BY updated_at DESC LIMIT ?',
      [limit.clamp(1, 80)],
    );
    for (final row in journals) {
      result.add(
        RecoveryProblem(
          id: row['id'] as String,
          kind: RecoveryProblemKind.interruptedOperation,
          code: 'operation_${row['status'] as String}',
          createdAt: DateTime.parse(row['updated_at'] as String),
          canResume: row['status'] == 'interrupted',
        ),
      );
    }
    final remaining = limit.clamp(1, 80) - result.length;
    if (remaining > 0) {
      final missing = _database.database.select(
        'SELECT id, last_opened_at FROM documents WHERE is_missing = 1 '
        'ORDER BY last_opened_at DESC, id LIMIT ?',
        [remaining],
      );
      for (final row in missing) {
        result.add(
          RecoveryProblem(
            id: row['id'] as String,
            kind: RecoveryProblemKind.inaccessibleDocument,
            code: 'document_inaccessible',
            createdAt: DateTime.parse(row['last_opened_at'] as String),
          ),
        );
      }
    }
    return List.unmodifiable(result);
  }

  void discardProblem(String id) {
    _database.database.execute(
      "UPDATE operation_journal SET status = 'cancelled', updated_at = ? "
      'WHERE id = ? AND status IN (\'running\', \'interrupted\', \'failed\')',
      [DateTime.now().toUtc().toIso8601String(), id],
    );
  }

  String integrityCheck() => _database.integrityCheck();
}
