import 'dart:collection';

import '../../../storage/repositories/document_repository.dart';
import '../../../storage/repositories/organization_repository.dart';
import '../domain/library_selection.dart';

typedef BulkCoverRegenerator = Future<void> Function(String documentId);

typedef BulkDocumentCleanup = Future<void> Function(String documentId);

final class LibraryBulkOperationService {
  const LibraryBulkOperationService({
    required DocumentRepository documents,
    required OrganizationRepository organization,
    this.regenerateCover,
    this.cleanupBeforeRemoval,
  }) : _documents = documents,
       _organization = organization;

  final DocumentRepository _documents;
  final OrganizationRepository _organization;
  final BulkCoverRegenerator? regenerateCover;
  final BulkDocumentCleanup? cleanupBeforeRemoval;

  Future<BulkOperationSummary> execute(
    BulkLibraryAction action,
    Iterable<String> documentIds, {
    String? targetId,
  }) async {
    final results = <BulkItemResult>[];
    for (final id in LinkedHashSet<String>.from(documentIds)) {
      try {
        if (_documents.byId(id) == null) {
          throw StateError('Document is no longer in the library.');
        }
        switch (action) {
          case BulkLibraryAction.favorite:
            _documents.setFavorite(id, true);
          case BulkLibraryAction.unfavorite:
            _documents.setFavorite(id, false);
          case BulkLibraryAction.assignTag:
            if (targetId == null || targetId.isEmpty) {
              throw ArgumentError('tagId is required');
            }
            _organization.assignTagToDocument(targetId, id);
          case BulkLibraryAction.addToCollection:
            if (targetId == null || targetId.isEmpty) {
              throw ArgumentError('collectionId is required');
            }
            _organization.addDocument(targetId, id);
          case BulkLibraryAction.removeFromCollection:
            if (targetId == null || targetId.isEmpty) {
              throw ArgumentError('collectionId is required');
            }
            _organization.removeDocument(targetId, id);
          case BulkLibraryAction.archive:
            final archive = _organization.ensureCollection(
              'Archivado',
              icon: 'archive',
            );
            _organization.addDocument(archive.id, id);
          case BulkLibraryAction.regenerateCover:
            final regenerate = regenerateCover;
            if (regenerate == null) {
              throw StateError('Cover regeneration is unavailable.');
            }
            await regenerate(id);
          case BulkLibraryAction.removeFromLibrary:
            // This only removes the local record. The original PDF is never
            // touched by a bulk action.
            await cleanupBeforeRemoval?.call(id);
            _documents.remove(id);
        }
        results.add(BulkItemResult(id: id, success: true));
      } on Object catch (error) {
        results.add(
          BulkItemResult(
            id: id,
            success: false,
            errorCode: describeIdentity(error),
          ),
        );
      }
    }
    return BulkOperationSummary(List.unmodifiable(results));
  }

  static String describeIdentity(Object error) =>
      error is ArgumentError ? 'invalid_input' : 'operation_failed';
}
