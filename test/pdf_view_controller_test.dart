import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';

void main() {
  test('restores a saved position and detaches to a neutral state', () async {
    final controller = PdfViewController();
    double? restoredZoom;
    double? restoredOffset;

    controller.attach(
      pageCount: 24,
      currentPageIndex: 2,
      zoom: 1.4,
      goToPage: (_) async {},
      zoomBy: (_) async {},
      fitWidth: () async {},
      fitPage: () async {},
      restorePosition: (zoom, offset) async {
        restoredZoom = zoom;
        restoredOffset = offset;
      },
    );
    await controller.restorePosition(zoom: 1.8, scrollOffset: 0.42);

    expect(restoredZoom, 1.8);
    expect(restoredOffset, 0.42);

    controller.detach();
    expect(controller.isReady, false);
    expect(controller.pageCount, 0);
    expect(controller.currentPageIndex, 0);
    expect(controller.zoom, 1);
    expect(controller.scrollOffset, 0);
  });
}
