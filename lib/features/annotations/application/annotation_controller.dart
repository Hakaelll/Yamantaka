import 'package:flutter/foundation.dart';

import '../../../storage/repositories/annotation_repository.dart';
import '../domain/annotation.dart';

enum AnnotationSort { page, newest }

final class AnnotationController extends ChangeNotifier {
  AnnotationController(this._repository);

  final AnnotationRepository _repository;
  String? _documentId;
  String? _fingerprint;
  List<PdfAnnotation> annotations = const [];
  AnnotationType? typeFilter;
  AnnotationColor? colorFilter;
  String? tagFilter;
  String query = '';
  AnnotationSort sort = AnnotationSort.page;
  PdfAnnotation? _lastDeleted;

  bool get canUndoDelete => _lastDeleted != null;
  int get reviewRequiredCount => annotations
      .where((item) => item.status == AnnotationAttachmentStatus.reviewRequired)
      .length;

  bool pageHasAnnotations(int pageIndex) =>
      annotations.any((item) => item.pageIndex == pageIndex);

  void load(String documentId, String fingerprint) {
    _documentId = documentId;
    _fingerprint = fingerprint;
    _repository.markDocumentChanged(documentId, fingerprint);
    _reload();
  }

  void clear() {
    _documentId = null;
    _fingerprint = null;
    annotations = const [];
    _lastDeleted = null;
    notifyListeners();
  }

  PdfAnnotation? create({
    required int pageIndex,
    AnnotationType type = AnnotationType.pageComment,
    AnnotationColor color = AnnotationColor.yellow,
    String? note,
    String? selectedText,
    String? contextBefore,
    String? contextAfter,
    List<NormalizedRect> rects = const [],
    int pageRotation = 0,
    List<String> tags = const [],
  }) {
    final documentId = _documentId;
    final fingerprint = _fingerprint;
    if (documentId == null || fingerprint == null) return null;
    final created = _repository.create(
      documentId: documentId,
      pageIndex: pageIndex,
      type: type,
      color: color,
      documentFingerprint: fingerprint,
      note: note,
      selectedText: selectedText,
      contextBefore: contextBefore,
      contextAfter: contextAfter,
      rects: rects,
      pageRotation: pageRotation,
      tags: tags,
    );
    _lastDeleted = null;
    _reload();
    return created;
  }

  void update(PdfAnnotation annotation) {
    _repository.save(annotation.copyWith(updatedAt: DateTime.now().toUtc()));
    _reload();
  }

  void delete(PdfAnnotation annotation) {
    _repository.remove(annotation.id);
    _lastDeleted = annotation;
    _reload();
  }

  bool undoDelete() {
    final deleted = _lastDeleted;
    if (deleted == null) return false;
    _repository.save(deleted.copyWith(updatedAt: DateTime.now().toUtc()));
    _lastDeleted = null;
    _reload();
    return true;
  }

  void setFilters({
    AnnotationType? type,
    AnnotationColor? color,
    String? tag,
    String? search,
    AnnotationSort? order,
    bool clearType = false,
    bool clearColor = false,
    bool clearTag = false,
  }) {
    typeFilter = clearType ? null : type ?? typeFilter;
    colorFilter = clearColor ? null : color ?? colorFilter;
    tagFilter = clearTag ? null : tag ?? tagFilter;
    query = search ?? query;
    sort = order ?? sort;
    _reload();
  }

  void _reload() {
    final documentId = _documentId;
    annotations = documentId == null
        ? const []
        : _repository.forDocument(
            documentId,
            type: typeFilter,
            color: colorFilter,
            query: query,
            tag: tagFilter,
            newestFirst: sort == AnnotationSort.newest,
          );
    notifyListeners();
  }
}
