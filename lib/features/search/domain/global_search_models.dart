enum GlobalSearchKind {
  metadata('Documento'),
  pageText('Contenido'),
  annotation('Anotación'),
  bookmark('Marcador'),
  collection('Colección'),
  tag('Etiqueta');

  const GlobalSearchKind(this.label);
  final String label;
}

final class GlobalSearchResult {
  const GlobalSearchResult({
    required this.documentId,
    required this.pageIndex,
    required this.kind,
    required this.title,
    required this.snippet,
    required this.rank,
    this.isMissing = false,
    this.isStale = false,
  });

  final String documentId;
  final int? pageIndex;
  final GlobalSearchKind kind;
  final String title;
  final String snippet;
  final double rank;
  final bool isMissing;
  final bool isStale;
}

enum GlobalSearchState { idle, searching, results, noResults, cancelled }

enum IndexDocumentStatus {
  pending,
  indexing,
  indexed,
  notIndexable,
  excluded,
  failed,
}

final class LibraryIndexProgress {
  const LibraryIndexProgress({
    required this.totalDocuments,
    required this.completedDocuments,
    required this.currentPage,
    required this.totalPages,
    required this.isRunning,
    required this.isPaused,
  });

  final int totalDocuments;
  final int completedDocuments;
  final int currentPage;
  final int totalPages;
  final bool isRunning;
  final bool isPaused;

  double get fraction =>
      totalDocuments == 0 ? 0 : completedDocuments / totalDocuments;
}
