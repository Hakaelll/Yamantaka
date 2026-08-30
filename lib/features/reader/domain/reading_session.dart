final class ReadingSession {
  const ReadingSession({
    required this.id,
    required this.documentId,
    required this.startedAt,
    required this.startPage,
    required this.endPage,
    required this.activeSeconds,
    this.endedAt,
    this.context,
  });

  final String id;
  final String documentId;
  final DateTime startedAt;
  final DateTime? endedAt;
  final int startPage;
  final int endPage;
  final int activeSeconds;
  final ReaderSessionContext? context;
}

final class ReaderSessionContext {
  const ReaderSessionContext({
    required this.zoomLevel,
    required this.pageLayout,
    required this.readingMode,
    required this.readingTheme,
    required this.scrollOffset,
    this.panelSection,
  });

  final double zoomLevel;
  final String pageLayout;
  final String readingMode;
  final String readingTheme;
  final double scrollOffset;
  final String? panelSection;

  Map<String, Object?> toJson() => {
    'zoomLevel': zoomLevel,
    'pageLayout': pageLayout,
    'readingMode': readingMode,
    'readingTheme': readingTheme,
    'scrollOffset': scrollOffset,
    'panelSection': panelSection,
  };

  factory ReaderSessionContext.fromJson(Map<String, Object?> json) =>
      ReaderSessionContext(
        zoomLevel: (json['zoomLevel'] as num?)?.toDouble() ?? 1,
        pageLayout: json['pageLayout'] as String? ?? 'single',
        readingMode: json['readingMode'] as String? ?? 'document',
        readingTheme: json['readingTheme'] as String? ?? 'ivory',
        scrollOffset: (json['scrollOffset'] as num?)?.toDouble() ?? 0,
        panelSection: json['panelSection'] as String?,
      );
}

final class RecentlyClosedDocument {
  const RecentlyClosedDocument({
    required this.documentId,
    required this.path,
    required this.title,
    required this.pageIndex,
    required this.zoomLevel,
    required this.pageLayout,
    required this.readingMode,
    required this.readingTheme,
    required this.scrollOffset,
    required this.closedAt,
    this.panelSection,
    this.isMissing = false,
    this.isModified = false,
  });

  final String documentId;
  final String path;
  final String title;
  final int pageIndex;
  final double zoomLevel;
  final String pageLayout;
  final String readingMode;
  final String readingTheme;
  final double scrollOffset;
  final String? panelSection;
  final DateTime closedAt;
  final bool isMissing;
  final bool isModified;

  Map<String, Object?> toJson() => {
    'documentId': documentId,
    'path': path,
    'title': title,
    'pageIndex': pageIndex,
    'zoomLevel': zoomLevel,
    'pageLayout': pageLayout,
    'readingMode': readingMode,
    'readingTheme': readingTheme,
    'scrollOffset': scrollOffset,
    'panelSection': panelSection,
    'closedAt': closedAt.toUtc().toIso8601String(),
    'isMissing': isMissing,
    'isModified': isModified,
  };

  factory RecentlyClosedDocument.fromJson(Map<String, Object?> json) =>
      RecentlyClosedDocument(
        documentId: json['documentId'] as String,
        path: json['path'] as String,
        title: json['title'] as String,
        pageIndex: json['pageIndex'] as int? ?? 0,
        zoomLevel: (json['zoomLevel'] as num?)?.toDouble() ?? 1,
        pageLayout: json['pageLayout'] as String? ?? 'single',
        readingMode: json['readingMode'] as String? ?? 'document',
        readingTheme: json['readingTheme'] as String? ?? 'ivory',
        scrollOffset: (json['scrollOffset'] as num?)?.toDouble() ?? 0,
        panelSection: json['panelSection'] as String?,
        closedAt: DateTime.parse(json['closedAt'] as String),
        isMissing: json['isMissing'] as bool? ?? false,
        isModified: json['isModified'] as bool? ?? false,
      );
}

final class ReadingGoal {
  const ReadingGoal({this.pages, this.minutes});

  final int? pages;
  final int? minutes;

  bool get isEmpty => pages == null && minutes == null;

  Map<String, Object?> toJson() => {'pages': pages, 'minutes': minutes};

  factory ReadingGoal.fromJson(Map<String, Object?> json) => ReadingGoal(
    pages: (json['pages'] as int?)?.clamp(1, 10000),
    minutes: (json['minutes'] as int?)?.clamp(1, 1440),
  );
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
