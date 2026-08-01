import 'library_document.dart';

final class LibraryQuery {
  const LibraryQuery({
    this.offset = 0,
    this.limit = 80,
    this.text = '',
    this.sort = LibraryQuerySort.lastOpened,
    this.favoritesOnly = false,
    this.includeMissing = true,
  });

  final int offset;
  final int limit;
  final String text;
  final LibraryQuerySort sort;
  final bool favoritesOnly;
  final bool includeMissing;

  LibraryQuery copyWith({
    int? offset,
    int? limit,
    String? text,
    LibraryQuerySort? sort,
    bool? favoritesOnly,
    bool? includeMissing,
  }) => LibraryQuery(
    offset: offset ?? this.offset,
    limit: limit ?? this.limit,
    text: text ?? this.text,
    sort: sort ?? this.sort,
    favoritesOnly: favoritesOnly ?? this.favoritesOnly,
    includeMissing: includeMissing ?? this.includeMissing,
  );

  LibraryQuery normalized() => LibraryQuery(
    offset: offset.clamp(0, 1000000),
    limit: limit.clamp(1, 200),
    text: text.trim(),
    sort: sort,
    favoritesOnly: favoritesOnly,
    includeMissing: includeMissing,
  );
}

enum LibraryQuerySort { lastOpened, added, title, progress }

final class LibraryPage {
  const LibraryPage({
    required this.documents,
    required this.offset,
    required this.total,
    required this.hasMore,
  });

  final List<LibraryDocument> documents;
  final int offset;
  final int total;
  final bool hasMore;
}
