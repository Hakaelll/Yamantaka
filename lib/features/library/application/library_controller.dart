import 'package:flutter/foundation.dart';

import '../../../app/app_config.dart';
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

  LibraryDocument? get continueReading =>
      documents.where((document) => !document.isMissing).firstOrNull;

  Future<void> load() async {
    isLoading = true;
    error = null;
    notifyListeners();
    try {
      _repository.refreshMissingState();
      documents = _repository.recent(limit: AppConfig.maxRecentDocuments);
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
}
