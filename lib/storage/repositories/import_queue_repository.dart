import 'dart:convert';

import '../../features/library/domain/import_queue.dart';
import '../database/app_database.dart';

/// Persists the confirmed import queue in the local metadata store.
///
/// The queue contains paths selected by the user, never PDF bytes or extracted
/// text. Running items are normalized back to pending when loaded so an
/// interrupted import can be resumed safely.
final class ImportQueueRepository {
  const ImportQueueRepository(this._database);

  static const _key = 'import_queue';
  static const _maxPending = 5000;

  final AppDatabase _database;

  ImportQueue load() {
    final rows = _database.database.select(
      'SELECT value FROM app_metadata WHERE key = ?',
      [_key],
    );
    if (rows.isEmpty) return ImportQueue(maxPending: _maxPending);
    try {
      final decoded = jsonDecode(rows.single['value'] as String);
      if (decoded is Map) {
        return ImportQueue.fromJson(<String, Object?>{
          for (final entry in decoded.entries)
            if (entry.key is String) entry.key as String: entry.value,
        }, maxPending: _maxPending);
      }
    } on Object {
      // A malformed queue must never prevent the library from opening.
    }
    return ImportQueue(maxPending: _maxPending);
  }

  void save(ImportQueue queue) {
    final now = DateTime.now().toUtc().toIso8601String();
    _database.database.execute(
      '''
      INSERT INTO app_metadata(key, value, updated_at) VALUES (?, ?, ?)
      ON CONFLICT(key) DO UPDATE SET
        value = excluded.value,
        updated_at = excluded.updated_at
      ''',
      [_key, const JsonEncoder().convert(queue.toJson()), now],
    );
  }

  void clear() {
    _database.database.execute('DELETE FROM app_metadata WHERE key = ?', [
      _key,
    ]);
  }
}
