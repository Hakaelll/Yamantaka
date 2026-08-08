enum ReaderMode { document, book }

enum ReaderCoverPolicy { firstPageIsCover, noCover }

enum ReaderInitialSide { right, left }

enum ReaderTheme {
  light(0xFFF3F3F1, 'Claro'),
  ivory(0xFFEAE5D8, 'Marfil'),
  sepia(0xFFD8CAB2, 'Sepia'),
  gray(0xFFB9BAB6, 'Gris'),
  dark(0xFF222320, 'Oscuro'),
  black(0xFF000000, 'Negro');

  const ReaderTheme(this.colorValue, this.label);

  final int colorValue;
  final String label;
}

enum PageLayoutMode { single, facing, continuous }

enum ZoomMode { custom, fitWidth, fitPage }

enum ReadingDirection { leftToRight, rightToLeft }

enum ReaderVisualProfile { standard, electronicInk }

final class DocumentPreferences {
  const DocumentPreferences({
    required this.documentId,
    this.viewMode = ReaderMode.document,
    this.readingTheme = ReaderTheme.ivory,
    this.zoomMode = ZoomMode.fitPage,
    this.zoomLevel = 1,
    this.pageLayout = PageLayoutMode.single,
    this.coverPolicy = ReaderCoverPolicy.noCover,
    this.initialSide = ReaderInitialSide.left,
    this.pageRotation = 0,
    this.horizontalMargin = 32,
    this.verticalMargin = 24,
    this.controlsAutoHide = true,
    this.scrollOffset = 0,
    this.updatedAt,
  });

  final String documentId;
  final ReaderMode viewMode;
  final ReaderTheme readingTheme;
  final ZoomMode zoomMode;
  final double zoomLevel;
  final PageLayoutMode pageLayout;
  final ReaderCoverPolicy coverPolicy;
  final ReaderInitialSide initialSide;
  final int pageRotation;
  final double horizontalMargin;
  final double verticalMargin;
  final bool controlsAutoHide;
  final double scrollOffset;
  final DateTime? updatedAt;

  DocumentPreferences copyWith({
    ReaderMode? viewMode,
    ReaderTheme? readingTheme,
    ZoomMode? zoomMode,
    double? zoomLevel,
    PageLayoutMode? pageLayout,
    ReaderCoverPolicy? coverPolicy,
    ReaderInitialSide? initialSide,
    int? pageRotation,
    double? horizontalMargin,
    double? verticalMargin,
    bool? controlsAutoHide,
    double? scrollOffset,
    DateTime? updatedAt,
  }) => DocumentPreferences(
    documentId: documentId,
    viewMode: viewMode ?? this.viewMode,
    readingTheme: readingTheme ?? this.readingTheme,
    zoomMode: zoomMode ?? this.zoomMode,
    zoomLevel: zoomLevel ?? this.zoomLevel,
    pageLayout: pageLayout ?? this.pageLayout,
    coverPolicy: coverPolicy ?? this.coverPolicy,
    initialSide: initialSide ?? this.initialSide,
    pageRotation: pageRotation ?? this.pageRotation,
    horizontalMargin: horizontalMargin ?? this.horizontalMargin,
    verticalMargin: verticalMargin ?? this.verticalMargin,
    controlsAutoHide: controlsAutoHide ?? this.controlsAutoHide,
    scrollOffset: scrollOffset ?? this.scrollOffset,
    updatedAt: updatedAt ?? this.updatedAt,
  );
}

abstract final class ReadingProgress {
  static double calculate({required int pageIndex, required int pageCount}) {
    if (pageCount <= 1) return pageCount == 1 ? 1 : 0;
    return (pageIndex.clamp(0, pageCount - 1) / (pageCount - 1)).clamp(0, 1);
  }
}

final class FacingPageSpread {
  const FacingPageSpread({this.leftPageIndex, this.rightPageIndex});

  final int? leftPageIndex;
  final int? rightPageIndex;

  int? get firstPageIndex {
    final pages = [?leftPageIndex, ?rightPageIndex];
    if (pages.isEmpty) return null;
    return pages.reduce((first, page) => first < page ? first : page);
  }

  int? get lastPageIndex {
    final pages = [?leftPageIndex, ?rightPageIndex];
    if (pages.isEmpty) return null;
    return pages.reduce((last, page) => last > page ? last : page);
  }

  static FacingPageSpread forPage(
    int pageIndex,
    int pageCount, {
    ReadingDirection direction = ReadingDirection.leftToRight,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return const FacingPageSpread();
    final index = pageIndex.clamp(0, pageCount - 1).toInt();
    final cover = firstPageIsCover;
    final resolvedInitialSide = initialSide;
    final firstPageIsSolo =
        cover || (!cover && resolvedInitialSide == ReaderInitialSide.right);
    if (firstPageIsSolo && index == 0) {
      if (cover) {
        return direction == ReadingDirection.leftToRight
            ? const FacingPageSpread(rightPageIndex: 0)
            : const FacingPageSpread(leftPageIndex: 0);
      }
      return resolvedInitialSide == ReaderInitialSide.left
          ? const FacingPageSpread(leftPageIndex: 0)
          : const FacingPageSpread(rightPageIndex: 0);
    }
    final first = cover
        ? (index.isOdd ? index : index - 1)
        : firstPageIsSolo
        ? (index.isOdd ? index : index - 1)
        : (index.isEven ? index : index - 1);
    final normalizedFirst = first.clamp(0, pageCount - 1).toInt();
    final second = normalizedFirst + 1 < pageCount ? normalizedFirst + 1 : null;
    return direction == ReadingDirection.leftToRight
        ? FacingPageSpread(
            leftPageIndex: normalizedFirst,
            rightPageIndex: second,
          )
        : FacingPageSpread(
            leftPageIndex: second,
            rightPageIndex: normalizedFirst,
          );
  }
}

abstract final class ReadingUnitNavigation {
  static bool isFacing(ReaderMode mode) => mode == ReaderMode.book;

  static int canonicalPage({
    required int pageIndex,
    required int pageCount,
    required ReaderMode mode,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return 0;
    final index = pageIndex.clamp(0, pageCount - 1).toInt();
    if (!isFacing(mode) ||
        (firstPageIsCover && index == 0) ||
        (!firstPageIsCover &&
            initialSide == ReaderInitialSide.right &&
            index == 0)) {
      return index;
    }
    if (firstPageIsCover || initialSide == ReaderInitialSide.right) {
      return index.isOdd ? index : index - 1;
    }
    return index.isEven ? index : index - 1;
  }

  static int previousTarget({
    required int pageIndex,
    required int pageCount,
    required ReaderMode mode,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return 0;
    final current = canonicalPage(
      pageIndex: pageIndex,
      pageCount: pageCount,
      mode: mode,
      firstPageIsCover: firstPageIsCover,
      initialSide: initialSide,
    );
    if (!isFacing(mode)) return (current - 1).clamp(0, pageCount - 1);
    if (current == 0 ||
        (firstPageIsCover && current == 1) ||
        (!firstPageIsCover &&
            initialSide == ReaderInitialSide.right &&
            current == 1)) {
      return 0;
    }
    return (current - 2).clamp(0, pageCount - 1);
  }

  static int nextTarget({
    required int pageIndex,
    required int pageCount,
    required ReaderMode mode,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return 0;
    final current = canonicalPage(
      pageIndex: pageIndex,
      pageCount: pageCount,
      mode: mode,
      firstPageIsCover: firstPageIsCover,
      initialSide: initialSide,
    );
    if (!isFacing(mode)) return (current + 1).clamp(0, pageCount - 1);
    if (current == 0) {
      if (firstPageIsCover || initialSide == ReaderInitialSide.right) {
        return pageCount > 1 ? 1 : 0;
      }
      return pageCount > 2 ? 2 : 0;
    }
    final target = current + 2;
    return target < pageCount ? target : current;
  }

  static int lastTarget({
    required int pageCount,
    required ReaderMode mode,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return 0;
    return canonicalPage(
      pageIndex: pageCount - 1,
      pageCount: pageCount,
      mode: mode,
      firstPageIsCover: firstPageIsCover,
      initialSide: initialSide,
    );
  }

  static String indicator({
    required int pageIndex,
    required int pageCount,
    required ReaderMode mode,
    ReadingDirection direction = ReadingDirection.leftToRight,
    bool firstPageIsCover = false,
    ReaderInitialSide initialSide = ReaderInitialSide.left,
  }) {
    if (pageCount <= 0) return '0 / 0';
    if (!isFacing(mode)) {
      return '${pageIndex.clamp(0, pageCount - 1) + 1} / $pageCount';
    }
    final spread = FacingPageSpread.forPage(
      pageIndex,
      pageCount,
      direction: direction,
      firstPageIsCover: firstPageIsCover,
      initialSide: initialSide,
    );
    final first = spread.firstPageIndex;
    final last = spread.lastPageIndex;
    if (first == null || last == null) return '0 / $pageCount';
    if (first == last) return '${first + 1} / $pageCount';
    return '${first + 1}–${last + 1} / $pageCount';
  }
}
