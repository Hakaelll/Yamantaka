import '../domain/library_document.dart';
import '../domain/document_enrichment.dart';

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
  Set<String> authors = const {},
  Set<LibraryTopic> topics = const {},
}) {
  final normalizedQuery = query.trim().toLowerCase();
  final visible = <LibraryDocument>[];
  final counts = <LibraryReadingState, int>{
    for (final state in LibraryReadingState.values) state: 0,
  };
  for (final document in documents) {
    final normalizedAuthor = document.effectiveAuthor?.trim().toLowerCase();
    final matchesAuthor =
        authors.isEmpty ||
        (normalizedAuthor != null && authors.contains(normalizedAuthor));
    final documentTopics = document.topics
        .map((assignment) => assignment.topic)
        .toSet();
    final matchesTopic =
        topics.isEmpty || documentTopics.intersection(topics).isNotEmpty;
    final matchesText =
        normalizedQuery.isEmpty ||
        document.effectiveTitle.toLowerCase().contains(normalizedQuery) ||
        document.effectiveAuthor?.toLowerCase().contains(normalizedQuery) ==
            true ||
        document.fileName.toLowerCase().contains(normalizedQuery) ||
        document.libraryNote?.toLowerCase().contains(normalizedQuery) == true ||
        document.topics.any(
          (assignment) =>
              assignment.topic.label.toLowerCase().contains(normalizedQuery),
        );
    if (!matchesText ||
        !matchesAuthor ||
        !matchesTopic ||
        (favoritesOnly && !document.isFavorite)) {
      continue;
    }
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
  Set<String> authors = const {},
  Set<LibraryTopic> topics = const {},
}) => evaluateLibraryDocuments(
  documents,
  query: query,
  sortOrder: sortOrder,
  readingState: readingState,
  favoritesOnly: favoritesOnly,
  authors: authors,
  topics: topics,
).documents;

Map<String, List<LibraryDocument>> groupLibraryDocumentsByAuthor(
  Iterable<LibraryDocument> documents,
) {
  final groups = <String, ({String label, List<LibraryDocument> documents})>{};
  for (final document in documents) {
    final author = document.effectiveAuthor
        ?.replaceAll(RegExp(r'\s+'), ' ')
        .trim();
    final label = author == null || author.isEmpty
        ? 'Autor desconocido'
        : author;
    final key = label == 'Autor desconocido' ? '' : label.toLowerCase();
    final group = groups.putIfAbsent(
      key,
      () => (label: label, documents: <LibraryDocument>[]),
    );
    group.documents.add(document);
  }
  final entries = groups.values.toList()
    ..sort((left, right) {
      if (left.label == 'Autor desconocido') return 1;
      if (right.label == 'Autor desconocido') return -1;
      return left.label.toLowerCase().compareTo(right.label.toLowerCase());
    });
  return {for (final entry in entries) entry.label: entry.documents};
}

LibraryReadingState readingStateOf(LibraryDocument document) {
  if (document.readingProgress >= 1) return LibraryReadingState.completed;
  if (document.readingProgress > 0) return LibraryReadingState.inProgress;
  return LibraryReadingState.notStarted;
}

String documentLabel(LibraryDocument document) {
  final title = document.effectiveTitle.trim();
  return (title.isEmpty ? document.fileName : title).toLowerCase();
}
