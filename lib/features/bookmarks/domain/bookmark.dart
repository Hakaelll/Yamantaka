final class Bookmark {
  const Bookmark({
    required this.id,
    required this.documentId,
    required this.pageIndex,
    required this.createdAt,
    required this.updatedAt,
    this.label,
  });

  final String id;
  final String documentId;
  final int pageIndex;
  final String? label;
  final DateTime createdAt;
  final DateTime updatedAt;
}
