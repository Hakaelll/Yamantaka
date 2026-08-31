import 'package:flutter/foundation.dart';

import '../../../storage/repositories/organization_repository.dart';
import '../domain/organization_models.dart';

final class OrganizationController extends ChangeNotifier {
  OrganizationController(this._repository) {
    reload();
  }

  final OrganizationRepository _repository;
  List<DocumentCollection> collections = const [];
  List<DocumentTag> tags = const [];
  Map<String, Set<String>> collectionMemberships = const {};
  Map<String, Set<String>> tagMemberships = const {};
  String? selectedCollectionId;
  SmartCollection? selectedSmartCollection;
  Set<String>? visibleDocumentIds;

  void reload() {
    collections = _repository.collections();
    tags = _repository.tags();
    collectionMemberships = _repository.collectionMemberships();
    tagMemberships = _repository.tagMemberships();
    _refreshVisibleDocuments();
    notifyListeners();
  }

  DocumentCollection createCollection(String name, {String icon = 'folder'}) {
    final created = _repository.createCollection(name, icon: icon);
    reload();
    return created;
  }

  void renameCollection(String id, String name) {
    _repository.renameCollection(id, name);
    reload();
  }

  void deleteCollection(String id) {
    _repository.deleteCollection(id);
    if (selectedCollectionId == id) selectedCollectionId = null;
    reload();
  }

  void togglePinned(DocumentCollection collection) {
    _repository.setPinned(collection.id, !collection.isPinned);
    reload();
  }

  void addDocument(String collectionId, String documentId) {
    _repository.addDocument(collectionId, documentId);
    reload();
  }

  void removeDocument(String collectionId, String documentId) {
    _repository.removeDocument(collectionId, documentId);
    reload();
  }

  void assignTagToDocument(String tagId, String documentId) {
    _repository.assignTagToDocument(tagId, documentId);
    reload();
  }

  void assignTagSilently(String tagId, String documentId) {
    _repository.assignTagToDocument(tagId, documentId);
  }

  DocumentTag ensureTag(String name) {
    final tag = _repository.ensureTag(name);
    reload();
    return tag;
  }

  void removeTagFromDocument(String tagId, String documentId) {
    _repository.removeTagFromDocument(tagId, documentId);
    reload();
  }

  void removeTagSilently(String tagId, String documentId) {
    _repository.removeTagFromDocument(tagId, documentId);
  }

  void addDocumentSilently(String collectionId, String documentId) {
    _repository.addDocument(collectionId, documentId);
  }

  void removeDocumentSilently(String collectionId, String documentId) {
    _repository.removeDocument(collectionId, documentId);
  }

  void selectCollection(String? id) {
    selectedCollectionId = id;
    selectedSmartCollection = null;
    _refreshVisibleDocuments();
    notifyListeners();
  }

  void selectSmartCollection(SmartCollection? smart) {
    selectedSmartCollection = smart;
    selectedCollectionId = null;
    _refreshVisibleDocuments();
    notifyListeners();
  }

  void clearSelection() {
    selectedCollectionId = null;
    selectedSmartCollection = null;
    visibleDocumentIds = null;
    notifyListeners();
  }

  bool includes(String documentId) =>
      visibleDocumentIds == null || visibleDocumentIds!.contains(documentId);

  void _refreshVisibleDocuments() {
    final collectionId = selectedCollectionId;
    final smart = selectedSmartCollection;
    visibleDocumentIds = collectionId != null
        ? _repository.documentIds(collectionId)
        : smart != null
        ? _repository.smartDocumentIds(smart)
        : null;
  }
}
