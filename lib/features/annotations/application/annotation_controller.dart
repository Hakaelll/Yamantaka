import 'package:flutter/foundation.dart';

import '../../../storage/repositories/annotation_repository.dart';
import '../domain/annotation.dart';

enum AnnotationSort { page, newest }

final class AnnotationController extends ChangeNotifier {
  AnnotationController(this._repository, {this.historyLimit = 100})
    : assert(historyLimit > 0);

  final AnnotationRepository _repository;
  final int historyLimit;
  String? _documentId;
  String? _fingerprint;
  List<PdfAnnotation> annotations = const [];
  AnnotationType? typeFilter;
  AnnotationColor? colorFilter;
  String? tagFilter;
  String query = '';
  AnnotationSort sort = AnnotationSort.page;
  final List<_AnnotationHistoryEntry> _undoHistory = [];
  final List<_AnnotationHistoryEntry> _redoHistory = [];

  bool get canUndo => _undoHistory.isNotEmpty;
  bool get canRedo => _redoHistory.isNotEmpty;
  bool get canUndoDelete => _undoHistory.lastOrNull?.isDeletion == true;
  int get reviewRequiredCount => annotations
      .where((item) => item.status == AnnotationAttachmentStatus.reviewRequired)
      .length;

  bool pageHasAnnotations(int pageIndex) =>
      annotations.any((item) => item.pageIndex == pageIndex);

  void load(String documentId, String fingerprint) {
    _documentId = documentId;
    _fingerprint = fingerprint;
    _repository.markDocumentChanged(documentId, fingerprint);
    _clearHistory();
    _reload();
  }

  void clear() {
    _documentId = null;
    _fingerprint = null;
    annotations = const [];
    _clearHistory();
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
    _record(_AnnotationHistoryEntry(before: null, after: created));
    _reload();
    return created;
  }

  bool update(PdfAnnotation annotation) {
    final before = _repository.byId(annotation.id);
    if (before == null || before.documentId != _documentId) return false;
    _repository.save(annotation.copyWith(updatedAt: DateTime.now().toUtc()));
    final after = _repository.byId(annotation.id)!;
    _record(_AnnotationHistoryEntry(before: before, after: after));
    _reload();
    return true;
  }

  bool delete(PdfAnnotation annotation) {
    final before = _repository.byId(annotation.id);
    if (before == null || before.documentId != _documentId) return false;
    _repository.remove(annotation.id);
    _record(_AnnotationHistoryEntry(before: before, after: null));
    _reload();
    return true;
  }

  bool undoDelete() {
    if (!canUndoDelete) return false;
    return undo();
  }

  bool undo() {
    if (_undoHistory.isEmpty || _documentId == null) return false;
    final entry = _undoHistory.removeLast();
    try {
      _apply(entry.before, removeId: entry.after?.id);
      _redoHistory.add(entry);
      _reload();
      return true;
    } on Object {
      _undoHistory.add(entry);
      return false;
    }
  }

  bool redo() {
    if (_redoHistory.isEmpty || _documentId == null) return false;
    final entry = _redoHistory.removeLast();
    try {
      _apply(entry.after, removeId: entry.before?.id);
      _undoHistory.add(entry);
      _reload();
      return true;
    } on Object {
      _redoHistory.add(entry);
      return false;
    }
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

  void _record(_AnnotationHistoryEntry entry) {
    _undoHistory.add(entry);
    if (_undoHistory.length > historyLimit) _undoHistory.removeAt(0);
    _redoHistory.clear();
  }

  void _apply(PdfAnnotation? snapshot, {required String? removeId}) {
    if (snapshot == null) {
      if (removeId != null) _repository.remove(removeId);
      return;
    }
    _repository.save(snapshot);
  }

  void _clearHistory() {
    _undoHistory.clear();
    _redoHistory.clear();
  }
}

final class _AnnotationHistoryEntry {
  const _AnnotationHistoryEntry({required this.before, required this.after});

  final PdfAnnotation? before;
  final PdfAnnotation? after;

  bool get isDeletion => before != null && after == null;
}
