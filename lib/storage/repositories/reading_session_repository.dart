import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/reader/domain/reading_session.dart';
import '../database/app_database.dart';

int _lastSessionStartMicros = 0;

final class ReadingSessionRepository {
  const ReadingSessionRepository(this._database);

  final AppDatabase _database;

  static const _sessionContextsKey = 'reading_session_contexts_v1';
  static const _recentlyClosedKey = 'recently_closed_documents_v1';
  static const _readingGoalsKey = 'reading_goals_v1';

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
    final contexts = _jsonMap(_sessionContextsKey);
    final rows = _database.database.select(
      '''
      SELECT s.*, COALESCE(d.title_override, d.title, d.file_name)
        AS document_title, d.path
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
              context: switch (contexts[row['id'] as String]) {
                final Map value => ReaderSessionContext.fromJson(
                  value.cast<String, Object?>(),
                ),
                _ => null,
              },
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
    _deleteMetadata(_sessionContextsKey);
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
    ReaderSessionContext? context,
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
    if (context != null) {
      final contexts = _jsonMap(_sessionContextsKey);
      contexts[id] = context.toJson();
      _writeJson(_sessionContextsKey, contexts);
    }
  }

  List<RecentlyClosedDocument> recentlyClosed({int limit = 10}) {
    final raw = _jsonList(_recentlyClosedKey);
    final result = <RecentlyClosedDocument>[];
    for (final item in raw) {
      try {
        if (item is Map) {
          result.add(
            RecentlyClosedDocument.fromJson(item.cast<String, Object?>()),
          );
        }
      } on Object {
        // Ignore only the incompatible entry; the remaining local history is
        // still useful and will be rewritten on the next mutation.
      }
    }
    result.sort((left, right) => right.closedAt.compareTo(left.closedAt));
    return List.unmodifiable(result.take(limit.clamp(1, 50)));
  }

  void rememberClosed(RecentlyClosedDocument document) {
    final next = [
      document,
      ...recentlyClosed(
        limit: 50,
      ).where((item) => item.documentId != document.documentId),
    ].take(10).map((item) => item.toJson()).toList(growable: false);
    _writeJson(_recentlyClosedKey, next);
  }

  void removeRecentlyClosed(String documentId) {
    final next = recentlyClosed(limit: 50)
        .where((item) => item.documentId != documentId)
        .map((item) => item.toJson())
        .toList(growable: false);
    if (next.isEmpty) {
      _deleteMetadata(_recentlyClosedKey);
    } else {
      _writeJson(_recentlyClosedKey, next);
    }
  }

  ReadingGoal? goalFor(String documentId) {
    final value = _jsonMap(_readingGoalsKey)[documentId];
    return value is Map
        ? ReadingGoal.fromJson(value.cast<String, Object?>())
        : null;
  }

  void setGoal(String documentId, ReadingGoal? goal) {
    final goals = _jsonMap(_readingGoalsKey);
    if (goal == null || goal.isEmpty) {
      goals.remove(documentId);
    } else {
      goals[documentId] = goal.toJson();
    }
    if (goals.isEmpty) {
      _deleteMetadata(_readingGoalsKey);
    } else {
      _writeJson(_readingGoalsKey, goals);
    }
  }

  Map<String, dynamic> _jsonMap(String key) {
    final value = _readMetadata(key);
    if (value == null) return <String, dynamic>{};
    try {
      final decoded = jsonDecode(value);
      return decoded is Map<String, dynamic>
          ? Map<String, dynamic>.from(decoded)
          : <String, dynamic>{};
    } on FormatException {
      return <String, dynamic>{};
    }
  }

  List<dynamic> _jsonList(String key) {
    final value = _readMetadata(key);
    if (value == null) return <dynamic>[];
    try {
      final decoded = jsonDecode(value);
      return decoded is List ? List<dynamic>.from(decoded) : <dynamic>[];
    } on FormatException {
      return <dynamic>[];
    }
  }

  String? _readMetadata(String key) =>
      _database.database.select(
            'SELECT value FROM app_metadata WHERE key = ?',
            [key],
          ).firstOrNull?['value']
          as String?;

  void _writeJson(String key, Object value) {
    _database.database.execute(
      '''
      INSERT INTO app_metadata(key, value, updated_at) VALUES (?, ?, ?)
      ON CONFLICT(key) DO UPDATE SET
        value = excluded.value, updated_at = excluded.updated_at
      ''',
      [key, jsonEncode(value), DateTime.now().toUtc().toIso8601String()],
    );
  }

  void _deleteMetadata(String key) {
    _database.database.execute('DELETE FROM app_metadata WHERE key = ?', [key]);
  }
}
