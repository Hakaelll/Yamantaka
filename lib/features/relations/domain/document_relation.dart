enum DocumentRelationType { related, replaces, sourceOf, attachmentOf }

enum DocumentAnchorKind { page, bookmark, annotation }

final class DocumentAnchor {
  const DocumentAnchor({required this.kind, required this.id});

  final DocumentAnchorKind kind;
  final String id;
}

final class DocumentRelation {
  const DocumentRelation({
    required this.id,
    required this.sourceDocumentId,
    required this.targetDocumentId,
    required this.type,
    required this.createdAt,
    this.sourceAnchor,
    this.targetAnchor,
  });

  final String id;
  final String sourceDocumentId;
  final String targetDocumentId;
  final DocumentRelationType type;
  final DocumentAnchor? sourceAnchor;
  final DocumentAnchor? targetAnchor;
  final DateTime createdAt;
}

final class DocumentRelationView {
  const DocumentRelationView({
    required this.relation,
    required this.otherDocumentId,
    required this.otherDocumentTitle,
    required this.isBacklink,
  });

  final DocumentRelation relation;
  final String otherDocumentId;
  final String otherDocumentTitle;
  final bool isBacklink;

  /// Replaces is directional. The inverse is presented as "is replaced by"
  /// without persisting a second, contradictory row.
  bool get isReplacedBy =>
      isBacklink && relation.type == DocumentRelationType.replaces;
}

final class DocumentRevisionGroup {
  const DocumentRevisionGroup({
    required this.id,
    required this.name,
    required this.documentIds,
    required this.createdAt,
    required this.updatedAt,
    this.currentDocumentId,
  });

  final String id;
  final String name;
  final List<String> documentIds;
  final String? currentDocumentId;
  final DateTime createdAt;
  final DateTime updatedAt;
}
