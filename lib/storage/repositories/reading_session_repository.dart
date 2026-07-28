import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/reader/domain/reading_session.dart';
import '../database/app_database.dart';

final class ReadingSessionRepository {
  const ReadingSessionRepository(this._database);

  final AppDatabase _database;

  ReadingSession start(String documentId, int pageIndex) {
    final now = DateTime.now().toUtc();
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
