import '../domain/library_document.dart';

enum LibrarySortOrder { lastOpened, dateAdded, title, progress }

enum LibraryReadingState { all, inProgress, notStarted, completed }

extension LibraryReadingStateLabel on LibraryReadingState {
  String get label => switch (this) {
    LibraryReadingState.all => 'Todos',
    LibraryReadingState.inProgress => 'En curso',
    LibraryReadingState.notStarted => 'Sin empezar',
    LibraryReadingState.completed => 'Terminados',
  };
}

final class LibraryFilterResult {
  const LibraryFilterResult({required this.documents, required this.counts});

  final List<LibraryDocument> documents;
  final Map<LibraryReadingState, int> counts;
}

LibraryFilterResult evaluateLibraryDocuments(
  Iterable<LibraryDocument> documents, {
  String query = '',
  LibrarySortOrder sortOrder = LibrarySortOrder.lastOpened,
  LibraryReadingState readingState = LibraryReadingState.all,
  bool favoritesOnly = false,
}) {
  final normalizedQuery = query.trim().toLowerCase();
  final visible = <LibraryDocument>[];
  final counts = <LibraryReadingState, int>{
    for (final state in LibraryReadingState.values) state: 0,
  };
  for (final document in documents) {
    final matchesText =
        normalizedQuery.isEmpty ||
        document.title?.toLowerCase().contains(normalizedQuery) == true ||
        document.author?.toLowerCase().contains(normalizedQuery) == true ||
        document.fileName.toLowerCase().contains(normalizedQuery);
    if (!matchesText || (favoritesOnly && !document.isFavorite)) continue;
    final documentState = readingStateOf(document);
    counts[LibraryReadingState.all] = counts[LibraryReadingState.all]! + 1;
    counts[documentState] = counts[documentState]! + 1;
    if (readingState == LibraryReadingState.all ||
        readingState == documentState) {
      visible.add(document);
    }
  }
  visible.sort((left, right) {
    final comparison = switch (sortOrder) {
      LibrarySortOrder.lastOpened => right.lastOpenedAt.compareTo(
        left.lastOpenedAt,
      ),
      LibrarySortOrder.dateAdded => right.addedAt.compareTo(left.addedAt),
      LibrarySortOrder.title => documentLabel(
        left,
      ).compareTo(documentLabel(right)),
      LibrarySortOrder.progress => right.readingProgress.compareTo(
        left.readingProgress,
      ),
    };
    if (comparison != 0) return comparison;
    return right.lastOpenedAt.compareTo(left.lastOpenedAt);
  });
  return LibraryFilterResult(documents: visible, counts: counts);
}

List<LibraryDocument> filterAndSortLibraryDocuments(
  Iterable<LibraryDocument> documents, {
  String query = '',
  LibrarySortOrder sortOrder = LibrarySortOrder.lastOpened,
  LibraryReadingState readingState = LibraryReadingState.all,
  bool favoritesOnly = false,
}) => evaluateLibraryDocuments(
  documents,
  query: query,
  sortOrder: sortOrder,
  readingState: readingState,
  favoritesOnly: favoritesOnly,
).documents;

LibraryReadingState readingStateOf(LibraryDocument document) {
  if (document.readingProgress >= 1) return LibraryReadingState.completed;
  if (document.readingProgress > 0) return LibraryReadingState.inProgress;
  return LibraryReadingState.notStarted;
}

String documentLabel(LibraryDocument document) {
  final title = document.title?.trim();
  return (title == null || title.isEmpty ? document.fileName : title)
      .toLowerCase();
}
