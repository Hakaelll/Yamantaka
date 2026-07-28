import 'package:flutter/foundation.dart';

import '../../../core/errors/app_failure.dart';
import '../../../core/logging/app_logger.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';

final class SearchController extends ChangeNotifier {
  SearchController(this._engine, this._viewerController, this._logger);

  final PdfDocumentEngine _engine;
  final PdfViewController _viewerController;
  final AppLogger _logger;

  String? _documentId;
  int _pageCount = 0;
  PdfOperationToken? _token;
  String query = '';
  List<PdfSearchMatch> matches = const [];
  int activeIndex = -1;
  bool isSearching = false;
  bool noExtractableText = false;
  AppFailure? error;

  PdfSearchMatch? get activeMatch =>
      activeIndex >= 0 && activeIndex < matches.length
      ? matches[activeIndex]
      : null;

  void attach(String documentId, int pageCount) {
    cancel();
    _documentId = documentId;
    _pageCount = pageCount;
    reset();
  }

  Future<void> search(String value) async {
    final documentId = _documentId;
    query = value.trim();
    cancel();
    if (documentId == null || query.isEmpty) {
      reset();
      return;
    }
    final token = PdfOperationToken();
    _token = token;
    isSearching = true;
    noExtractableText = false;
    error = null;
    matches = const [];
    activeIndex = -1;
    notifyListeners();
    try {
      final results = await _engine.searchText(
        documentId: documentId,
        query: query,
        cancellationToken: token,
      );
      if (token.isCancelled) return;
      matches = results;
      activeIndex = results.isEmpty ? -1 : 0;
      if (results.isEmpty) {
        noExtractableText = await _hasNoExtractableText(documentId, token);
      } else {
        await _goToActive();
      }
    } on AppFailure catch (failure) {
      if (failure.kind != FailureKind.cancelled) error = failure;
    } on Object catch (exception, stackTrace) {
      _logger.error('PDF search failed.', exception, stackTrace);
      error = AppFailure(
        FailureKind.unknown,
        'No se pudo completar la búsqueda.',
        debugMessage: exception.toString(),
      );
    } finally {
      if (identical(_token, token)) {
        isSearching = false;
        notifyListeners();
      }
    }
  }

  Future<bool> _hasNoExtractableText(
    String documentId,
    PdfOperationToken token,
  ) async {
    for (var page = 0; page < _pageCount; page++) {
      if (token.isCancelled) return false;
      final text = await _engine.extractPageText(
        documentId: documentId,
        pageIndex: page,
      );
      if (text.trim().isNotEmpty) return false;
    }
    return true;
  }

  Future<void> next() async {
    if (matches.isEmpty) return;
    activeIndex = (activeIndex + 1) % matches.length;
    notifyListeners();
    await _goToActive();
  }

  Future<void> previous() async {
    if (matches.isEmpty) return;
    activeIndex = (activeIndex - 1 + matches.length) % matches.length;
    notifyListeners();
    await _goToActive();
  }

  Future<void> select(int index) async {
    if (index < 0 || index >= matches.length) return;
    activeIndex = index;
    notifyListeners();
    await _goToActive();
  }

  Future<void> _goToActive() async {
    final match = activeMatch;
    if (match != null) await _viewerController.goToPage(match.pageIndex);
  }

  void cancel() {
    final changed = _token != null || isSearching;
    _token?.cancel();
    _token = null;
    isSearching = false;
    if (changed) notifyListeners();
  }

  void reset() {
    cancel();
    query = '';
    matches = const [];
    activeIndex = -1;
    noExtractableText = false;
    error = null;
    notifyListeners();
  }

  @override
  void dispose() {
    cancel();
    super.dispose();
  }
}
