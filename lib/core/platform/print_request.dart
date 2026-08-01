final class PrintRequest {
  const PrintRequest({this.firstPage, this.lastPage, this.landscape = false});

  final int? firstPage;
  final int? lastPage;
  final bool landscape;

  PrintRequest normalized(int pageCount) {
    if (pageCount <= 0) throw ArgumentError.value(pageCount, 'pageCount');
    final first = (firstPage ?? 0).clamp(0, pageCount - 1);
    final last = (lastPage ?? pageCount - 1).clamp(first, pageCount - 1);
    return PrintRequest(firstPage: first, lastPage: last, landscape: landscape);
  }

  Map<String, Object?> toJson() => {
    'first_page': firstPage,
    'last_page': lastPage,
    'landscape': landscape,
  };
}
