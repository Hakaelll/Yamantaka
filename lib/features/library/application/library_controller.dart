import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/library_document.dart';

final class LibraryController extends ChangeNotifier {
  LibraryController(this._repository, this._logger);

  final DocumentRepository _repository;
  final AppLogger _logger;

  List<LibraryDocument> documents = const [];
  bool isLoading = false;
  AppFailure? error;
  String? selectedDocumentId;
  double scrollOffset = 0;
  String query = '';
  String sortOrder = 'lastOpened';
  String readingState = 'all';
  bool favoritesOnly = false;

  LibraryDocument? get continueReading => documents
      .where(
        (document) =>
            !document.isMissing &&
            document.readingProgress > 0 &&
            document.readingProgress < 1,
      )
      .firstOrNull;

  Future<void> load() async {
    isLoading = true;
    error = null;
    notifyListeners();
    try {
      _repository.refreshMissingState();
      documents = _repository.all();
      if (selectedDocumentId != null &&
          !documents.any((document) => document.id == selectedDocumentId)) {
        selectedDocumentId = null;
      }
      if (selectedDocumentId == null && documents.length == 1) {
        selectedDocumentId = documents.single.id;
      }
    } on Object catch (exception, stackTrace) {
      _logger.error('Library load failed.', exception, stackTrace);
      error = AppFailure(
        FailureKind.databaseFailed,
        'No se pudo cargar la biblioteca local.',
        debugMessage: exception.toString(),
      );
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }

  Future<void> deleteHistory() async {
    _repository.deleteHistory();
    await load();
  }

  void selectDocument(String? documentId) {
    if (selectedDocumentId == documentId) return;
    selectedDocumentId = documentId;
    notifyListeners();
  }

  Future<void> toggleFavorite(String documentId) async {
    _repository.toggleFavorite(documentId);
    await load();
  }
}
