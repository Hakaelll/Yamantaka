import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/pdf_engine/pdf_document_engine.dart';

base class StubPdfDocumentEngine implements PdfDocumentEngine {
  @override
  Future<void> closeDocument(String documentId) async {}

  @override
  void clearCaches() {}

  @override
  Future<void> dispose() async {}

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  }) async => '';

  @override
  Future<PdfDocumentMetadata> readDocumentMetadata(String documentId) async =>
      const PdfDocumentMetadata();

  @override
  Future<List<PdfStructuredTextBlock>> extractStructuredPageText({
    required String documentId,
    required int pageIndex,
    PdfOperationToken? cancellationToken,
  }) async => const [];

  @override
  Future<List<PdfOutlineItem>> getOutline(String documentId) async => const [];

  @override
  Future<List<PdfLink>> getPageLinks({
    required String documentId,
    required int pageIndex,
  }) async => const [];

  @override
  Future<OpenedPdfDocument> openDocument(String path, {String? password}) =>
      throw UnimplementedError();

  @override
  Future<RenderedPdfPage> renderPage({
    required String documentId,
    required int pageIndex,
    required double scale,
    int rotation = 0,
    PdfRenderRegion? region,
    PdfOperationToken? cancellationToken,
    double devicePixelRatio = 1,
    int? generation,
    PdfRenderPriority priority = PdfRenderPriority.visible,
  }) => throw UnimplementedError();

  @override
  void beginRenderGeneration(String documentId, int generation) {}

  @override
  Future<List<PdfSearchMatch>> searchText({
    required String documentId,
    required String query,
    PdfOperationToken? cancellationToken,
  }) async => const [];
}
