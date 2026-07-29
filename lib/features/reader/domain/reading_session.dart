final class ReadingSession {
  const ReadingSession({
    required this.id,
    required this.documentId,
    required this.startedAt,
    required this.startPage,
    required this.endPage,
    required this.activeSeconds,
    this.endedAt,
  });

  final String id;
  final String documentId;
  final DateTime startedAt;
  final DateTime? endedAt;
  final int startPage;
  final int endPage;
  final int activeSeconds;
}

final class ReadingHistoryEntry {
  const ReadingHistoryEntry({
    required this.session,
    required this.documentTitle,
    required this.documentPath,
  });

  final ReadingSession session;
  final String documentTitle;
  final String documentPath;
}
