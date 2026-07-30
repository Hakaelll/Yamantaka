import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/presentation/reader_viewport_metrics.dart';

void main() {
  group('facing page geometry', () {
    test('centers a compact spread with a fixed spine and aligned tops', () {
      final layout = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [
          Size(580, 800),
          Size(600, 820),
          Size(560, 760),
          Size(590, 810),
          Size(590, 810),
        ],
        outerMargin: 32,
        spineGap: 14,
        rightToLeft: false,
      );

      final left = layout.pageRects[1];
      final right = layout.pageRects[2];
      expect(right.left - left.right, 14);
      expect(left.top, right.top);

      final slotWidth = layout.documentSize.width / 3;
      final spread = left.expandToInclude(right);
      expect(spread.center.dx, closeTo(slotWidth * 1.5, 0.001));
      expect(layout.pageRects.first.center.dx, closeTo(slotWidth / 2, 0.001));
    });

    test('supports one page, an unpaired final page and RTL ordering', () {
      final onePage = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800)],
        outerMargin: 24,
        spineGap: 12,
        rightToLeft: false,
      );
      expect(onePage.pageRects, hasLength(1));
      expect(
        onePage.pageRects.single.center.dx,
        onePage.documentSize.width / 2,
      );

      final ltr = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [
          Size(600, 800),
          Size(600, 800),
          Size(600, 800),
          Size(600, 800),
        ],
        outerMargin: 24,
        spineGap: 12,
        rightToLeft: false,
      );
      expect(ltr.pageRects[1].left, lessThan(ltr.pageRects[2].left));
      final finalUnit = ReaderViewportMetrics.readingUnitRect(
        pageRects: ltr.pageRects,
        pageIndex: 3,
        facing: true,
      );
      expect(finalUnit, ltr.pageRects[3]);

      final rtl = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800), Size(600, 800), Size(600, 800)],
        outerMargin: 24,
        spineGap: 12,
        rightToLeft: true,
      );
      expect(rtl.pageRects[2].left, lessThan(rtl.pageRects[1].left));
    });
  });

  group('viewport normalization', () {
    test('drops stale pan offsets once the complete unit fits', () {
      final center = ReaderViewportMetrics.normalizeCenter(
        proposedCenter: const Offset(1200, -400),
        viewport: const Size(1200, 800),
        contentRect: const Rect.fromLTWH(300, 100, 600, 800),
        zoom: 0.7,
        margin: 20,
      );

      expect(center, const Offset(600, 500));
    });

    test('keeps larger content pannable but clamps inaccessible areas', () {
      final center = ReaderViewportMetrics.normalizeCenter(
        proposedCenter: const Offset(-100, 2000),
        viewport: const Size(800, 600),
        contentRect: const Rect.fromLTWH(100, 100, 900, 1200),
        zoom: 2,
      );

      expect(center.dx, 300);
      expect(center.dy, 1150);
    });

    test('fit modes use the complete spread and stable limits', () {
      const viewport = Size(1200, 800);
      const spread = Rect.fromLTWH(0, 0, 1214, 800);
      final fitPage = ReaderViewportMetrics.fitPageZoom(
        viewport: viewport,
        contentRect: spread,
        margin: 20,
        minimum: 0.1,
        maximum: 8,
      );
      final fitWidth = ReaderViewportMetrics.fitWidthZoom(
        viewport: viewport,
        contentRect: spread,
        margin: 20,
        minimum: 0.1,
        maximum: 8,
      );

      expect(fitPage, closeTo(0.95, 0.001));
      expect(fitWidth, closeTo(1160 / 1214, 0.001));
    });
  });

  test('margins and spine adapt in bounded discrete steps', () {
    expect(
      ReaderViewportMetrics.adaptiveMargin(
        viewport: const Size(640, 460),
        preferred: 40,
      ),
      16,
    );
    expect(
      ReaderViewportMetrics.adaptiveMargin(
        viewport: const Size(1920, 1080),
        preferred: 40,
      ),
      40,
    );
    expect(ReaderViewportMetrics.adaptiveSpineGap(640), 10);
    expect(ReaderViewportMetrics.adaptiveSpineGap(1200), 14);
    expect(ReaderViewportMetrics.adaptiveSpineGap(1920), 16);
  });
}
