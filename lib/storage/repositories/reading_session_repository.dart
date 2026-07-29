import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/reader/domain/reading_session.dart';
import '../database/app_database.dart';

int _lastSessionStartMicros = 0;

final class ReadingSessionRepository {
  const ReadingSessionRepository(this._database);

  final AppDatabase _database;

  ReadingSession start(String documentId, int pageIndex) {
    final wallClockMicros = DateTime.now().toUtc().microsecondsSinceEpoch;
    final startMicros = wallClockMicros > _lastSessionStartMicros
        ? wallClockMicros
        : _lastSessionStartMicros + 1;
    _lastSessionStartMicros = startMicros;
    final now = DateTime.fromMicrosecondsSinceEpoch(startMicros, isUtc: true);
    final id = sha256
        .convert(utf8.encode('$documentId:${now.microsecondsSinceEpoch}'))
        .toString();
    _database.database.execute(
      '''
      INSERT INTO reading_sessions (
        id, document_id, started_at, start_page, end_page, active_seconds
      ) VALUES (?, ?, ?, ?, ?, 0)
      ''',
      [id, documentId, now.toIso8601String(), pageIndex, pageIndex],
    );
    return ReadingSession(
      id: id,
      documentId: documentId,
      startedAt: now,
      startPage: pageIndex,
      endPage: pageIndex,
      activeSeconds: 0,
    );
  }

  List<ReadingHistoryEntry> history({int limit = 250}) {
    final rows = _database.database.select(
      '''
      SELECT s.*, COALESCE(d.title, d.file_name) AS document_title, d.path
      FROM reading_sessions s
      JOIN documents d ON d.id = s.document_id
      WHERE s.ended_at IS NOT NULL
      ORDER BY s.started_at DESC
      LIMIT ?
      ''',
      [limit.clamp(1, 1000)],
    );
    return rows
        .map(
          (row) => ReadingHistoryEntry(
            session: ReadingSession(
              id: row['id'] as String,
              documentId: row['document_id'] as String,
              startedAt: DateTime.parse(row['started_at'] as String),
              endedAt: DateTime.parse(row['ended_at'] as String),
              startPage: row['start_page'] as int,
              endPage: row['end_page'] as int,
              activeSeconds: row['active_seconds'] as int,
            ),
            documentTitle: row['document_title'] as String,
            documentPath: row['path'] as String,
          ),
        )
        .toList(growable: false);
  }

  void delete(String id) {
    _database.database.execute('DELETE FROM reading_sessions WHERE id = ?', [
      id,
    ]);
  }

  void clear() {
    _database.database.execute('DELETE FROM reading_sessions');
  }

  void trimToLimit(int limit) {
    final bounded = limit.clamp(25, 1000);
    _database.database.execute(
      '''
      DELETE FROM reading_sessions
      WHERE id IN (
        SELECT id FROM reading_sessions
        ORDER BY started_at DESC
        LIMIT -1 OFFSET ?
      )
      ''',
      [bounded],
    );
  }

  void finish({
    required String id,
    required int endPage,
    required int activeSeconds,
  }) {
    _database.database.execute(
      '''
      UPDATE reading_sessions SET
        ended_at = ?, end_page = ?, active_seconds = ?
      WHERE id = ?
      ''',
      [
        DateTime.now().toUtc().toIso8601String(),
        endPage,
        activeSeconds.clamp(0, 24 * 60 * 60),
        id,
      ],
    );
  }
}
