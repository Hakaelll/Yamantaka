import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/cache/pdf_tile_planner.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';

void main() {
  test('ordinary pages stay on the full-page path', () {
    final plan = PdfTilePlanner.plan(
      const PdfTilePlanRequest(
        documentId: 'doc',
        pageIndex: 0,
        pageWidth: 600,
        pageHeight: 800,
        scale: 1,
        rotation: 0,
        viewport: PdfRenderRegion(left: 0, top: 0, width: 600, height: 800),
        maxBytes: 16 * 1024 * 1024,
      ),
    );
    expect(plan.useTiles, isFalse);
    expect(plan.requests.single.region.width, 600);
  });

  test('extreme pages plan a visible tile ring with stable keys', () {
    const request = PdfTilePlanRequest(
      documentId: 'doc',
      pageIndex: 4,
      pageWidth: 5000,
      pageHeight: 7000,
      scale: 2,
      rotation: 90,
      viewport: PdfRenderRegion(left: 2000, top: 3000, width: 800, height: 600),
      maxBytes: 16 * 1024 * 1024,
      tileSize: 512,
      generation: 3,
    );
    final first = PdfTilePlanner.plan(request);
    final second = PdfTilePlanner.plan(request);
    expect(first.useTiles, isTrue);
    expect(first.requests, isNotEmpty);
    expect(first.requests.first.key, second.requests.first.key);
    expect(first.requests.every((item) => item.region.width > 0), isTrue);
  });
}
