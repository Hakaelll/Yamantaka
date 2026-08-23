import 'models/pdf_models.dart';

abstract interface class PdfDocumentEngine {
  Future<OpenedPdfDocument> openDocument(String path, {String? password});

  Future<void> closeDocument(String documentId);

  /// Returns an image borrowed from the engine cache.
  ///
  /// Callers must not dispose the returned image; closing the document or
  /// clearing the engine cache releases it.
  Future<RenderedPdfPage> renderPage({
    required String documentId,
    required int pageIndex,
    required double scale,
    int rotation = 0,
    PdfRenderRegion? region,
    PdfOperationToken? cancellationToken,
  });

  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  });

  Future<PdfDocumentMetadata> readDocumentMetadata(String documentId);

  Future<List<PdfStructuredTextBlock>> extractStructuredPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  });

  Future<List<PdfSearchMatch>> searchText({
    required String documentId,
    required String query,
    PdfOperationToken? cancellationToken,
  });

  Future<List<PdfOutlineItem>> getOutline(String documentId);

  Future<List<PdfLink>> getPageLinks({
    required String documentId,
    required int pageIndex,
  });

  void clearCaches();

  Future<void> dispose();
}
