import 'library_document.dart';
import 'document_enrichment.dart';
import '../../organization/domain/organization_models.dart';

const Object _keepLibraryQueryValue = Object();

final class LibraryQuery {
  const LibraryQuery({
    this.offset = 0,
    this.limit = 80,
    this.text = '',
    this.sort = LibraryQuerySort.lastOpened,
    this.favoritesOnly = false,
    this.includeMissing = true,
    this.readingState = LibraryQueryReadingState.all,
    this.authors = const {},
    this.topics = const {},
    this.collectionId,
    this.smartCollection,
  });

  final int offset;
  final int limit;
  final String text;
  final LibraryQuerySort sort;
  final bool favoritesOnly;
  final bool includeMissing;
  final LibraryQueryReadingState readingState;
  final Set<String> authors;
  final Set<LibraryTopic> topics;
  final String? collectionId;
  final SmartCollection? smartCollection;

  LibraryQuery copyWith({
    int? offset,
    int? limit,
    String? text,
    LibraryQuerySort? sort,
    bool? favoritesOnly,
    bool? includeMissing,
    LibraryQueryReadingState? readingState,
    Set<String>? authors,
    Set<LibraryTopic>? topics,
    Object? collectionId = _keepLibraryQueryValue,
    Object? smartCollection = _keepLibraryQueryValue,
  }) => LibraryQuery(
    offset: offset ?? this.offset,
    limit: limit ?? this.limit,
    text: text ?? this.text,
    sort: sort ?? this.sort,
    favoritesOnly: favoritesOnly ?? this.favoritesOnly,
    includeMissing: includeMissing ?? this.includeMissing,
    readingState: readingState ?? this.readingState,
    authors: authors ?? this.authors,
    topics: topics ?? this.topics,
    collectionId: identical(collectionId, _keepLibraryQueryValue)
        ? this.collectionId
        : collectionId as String?,
    smartCollection: identical(smartCollection, _keepLibraryQueryValue)
        ? this.smartCollection
        : smartCollection as SmartCollection?,
  );

  LibraryQuery normalized() => LibraryQuery(
    offset: offset.clamp(0, 1000000),
    limit: limit.clamp(1, 80),
    text: text.trim(),
    sort: sort,
    favoritesOnly: favoritesOnly,
    includeMissing: includeMissing,
    readingState: readingState,
    authors: {
      for (final author in authors)
        if (author.trim().isNotEmpty) author.trim().toLowerCase(),
    },
    topics: Set<LibraryTopic>.unmodifiable(topics),
    collectionId: collectionId?.trim().isEmpty == true
        ? null
        : collectionId?.trim(),
    smartCollection: smartCollection,
  );
}

enum LibraryQuerySort { lastOpened, added, title, progress }

enum LibraryQueryReadingState { all, inProgress, notStarted, completed }

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
