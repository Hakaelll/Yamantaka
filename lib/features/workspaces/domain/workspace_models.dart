enum TabResourceState { active, warm, suspended }

const Object _keepPanelSection = Object();

abstract final class WorkspaceLayoutConstraints {
  static const double sidePanelMinWidth = 288;
  static const double sidePanelMaxWidth = 480;
}

final class DocumentTab {
  const DocumentTab({
    required this.id,
    required this.documentId,
    required this.path,
    required this.title,
    required this.pageIndex,
    required this.zoomLevel,
    required this.pageLayout,
    required this.readingMode,
    required this.scrollOffset,
    required this.readingProgress,
    required this.isPinned,
    required this.resourceState,
    this.readingTheme = 'ivory',
    this.panelSection,
    this.isMissing = false,
    this.isExternallyModified = false,
  });

  final String id;
  final String documentId;
  final String path;
  final String title;
  final int pageIndex;
  final double zoomLevel;
  final String pageLayout;
  final String readingMode;
  final double scrollOffset;
  final double readingProgress;
  final bool isPinned;
  final TabResourceState resourceState;
  final String readingTheme;
  final String? panelSection;
  final bool isMissing;
  final bool isExternallyModified;

  DocumentTab copyWith({
    int? pageIndex,
    double? zoomLevel,
    String? pageLayout,
    String? readingMode,
    double? scrollOffset,
    double? readingProgress,
    bool? isPinned,
    TabResourceState? resourceState,
    String? readingTheme,
    Object? panelSection = _keepPanelSection,
    bool? isMissing,
    bool? isExternallyModified,
  }) => DocumentTab(
    id: id,
    documentId: documentId,
    path: path,
    title: title,
    pageIndex: pageIndex ?? this.pageIndex,
    zoomLevel: zoomLevel ?? this.zoomLevel,
    pageLayout: pageLayout ?? this.pageLayout,
    readingMode: readingMode ?? this.readingMode,
    scrollOffset: scrollOffset ?? this.scrollOffset,
    readingProgress: readingProgress ?? this.readingProgress,
    isPinned: isPinned ?? this.isPinned,
    resourceState: resourceState ?? this.resourceState,
    readingTheme: readingTheme ?? this.readingTheme,
    panelSection: identical(panelSection, _keepPanelSection)
        ? this.panelSection
        : panelSection as String?,
    isMissing: isMissing ?? this.isMissing,
    isExternallyModified: isExternallyModified ?? this.isExternallyModified,
  );
}

enum SplitAxis { vertical, horizontal }

final class SplitViewState {
  const SplitViewState({
    this.enabled = false,
    this.axis = SplitAxis.vertical,
    this.ratio = 0.5,
    this.secondaryTabId,
    this.syncPage = false,
    this.syncZoom = false,
    this.comparisonMode = false,
    this.overlayOpacity,
  });

  final bool enabled;
  final SplitAxis axis;
  final double ratio;
  final String? secondaryTabId;
  final bool syncPage;
  final bool syncZoom;
  final bool comparisonMode;
  final double? overlayOpacity;

  bool get hasSafeOverlay =>
      comparisonMode &&
      overlayOpacity != null &&
      overlayOpacity! >= 0 &&
      overlayOpacity! <= 1;

  SplitViewState copyWith({
    bool? enabled,
    SplitAxis? axis,
    double? ratio,
    String? secondaryTabId,
    bool? syncPage,
    bool? syncZoom,
    bool? comparisonMode,
    double? overlayOpacity,
    bool clearOverlay = false,
  }) => SplitViewState(
    enabled: enabled ?? this.enabled,
    axis: axis ?? this.axis,
    ratio: (ratio ?? this.ratio).clamp(0.2, 0.8),
    secondaryTabId: secondaryTabId ?? this.secondaryTabId,
    syncPage: syncPage ?? this.syncPage,
    syncZoom: syncZoom ?? this.syncZoom,
    comparisonMode: comparisonMode ?? this.comparisonMode,
    overlayOpacity: clearOverlay ? null : overlayOpacity ?? this.overlayOpacity,
  );
}

final class WorkspaceSnapshot {
  const WorkspaceSnapshot({
    required this.id,
    required this.name,
    required this.tabs,
    required this.createdAt,
    required this.updatedAt,
    this.isDefault = false,
    this.activeTabId,
    this.panelSection,
    this.panelWidth = 320,
    this.split = const SplitViewState(),
    this.collectionId,
  });

  final String id;
  final String name;
  final List<DocumentTab> tabs;
  final bool isDefault;
  final String? activeTabId;
  final String? panelSection;
  final double panelWidth;
  final SplitViewState split;
  final String? collectionId;
  final DateTime createdAt;
  final DateTime updatedAt;
}

abstract final class SplitViewPolicy {
  static bool isAvailable(double width) => width >= 1100;

  static double cacheBudgetFactor(bool enabled) => enabled ? 0.62 : 1;
}
