enum ReaderPane { primary, secondary }

final class SplitPaneState {
  const SplitPaneState({
    this.documentId,
    this.pageIndex = 0,
    this.zoom = 1,
    this.offset = 0,
    this.closed = false,
  });

  final String? documentId;
  final int pageIndex;
  final double zoom;
  final double offset;
  final bool closed;

  SplitPaneState copyWith({
    Object? documentId = _keepDocument,
    int? pageIndex,
    double? zoom,
    double? offset,
    bool? closed,
  }) => SplitPaneState(
    documentId: identical(documentId, _keepDocument)
        ? this.documentId
        : documentId as String?,
    pageIndex: pageIndex ?? this.pageIndex,
    zoom: zoom ?? this.zoom,
    offset: offset ?? this.offset,
    closed: closed ?? this.closed,
  );
}

final class SplitReaderSession {
  const SplitReaderSession({
    this.primary = const SplitPaneState(),
    this.secondary = const SplitPaneState(),
    this.focused = ReaderPane.primary,
    this.generation = 0,
  });

  final SplitPaneState primary;
  final SplitPaneState secondary;
  final ReaderPane focused;
  final int generation;

  SplitPaneState pane(ReaderPane pane) =>
      pane == ReaderPane.primary ? primary : secondary;

  SplitReaderSession focus(ReaderPane pane) => SplitReaderSession(
    primary: primary,
    secondary: secondary,
    focused: pane,
    generation: generation + 1,
  );

  SplitReaderSession open(ReaderPane pane, String documentId) {
    final next = this
        .pane(pane)
        .copyWith(documentId: documentId, closed: false);
    return SplitReaderSession(
      primary: pane == ReaderPane.primary ? next : primary,
      secondary: pane == ReaderPane.secondary ? next : secondary,
      focused: pane,
      generation: generation + 1,
    );
  }

  SplitReaderSession close(ReaderPane pane) {
    final next = this.pane(pane).copyWith(documentId: null, closed: true);
    final nextFocus = pane == focused
        ? (pane == ReaderPane.primary && !secondary.closed
              ? ReaderPane.secondary
              : ReaderPane.primary)
        : focused;
    return SplitReaderSession(
      primary: pane == ReaderPane.primary ? next : primary,
      secondary: pane == ReaderPane.secondary ? next : secondary,
      focused: nextFocus,
      generation: generation + 1,
    );
  }
}

const Object _keepDocument = Object();
