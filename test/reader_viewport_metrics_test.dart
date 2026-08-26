import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
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
        pageGap: 0,
        rightToLeft: false,
      );

      final left = layout.pageRects[0];
      final right = layout.pageRects[1];
      expect(right.left - left.right, 0);
      expect(left.top, right.top);

      final slotWidth = layout.documentSize.width / 3;
      final spread = left.expandToInclude(right);
      expect(spread.center.dx, closeTo(slotWidth / 2, 0.001));
    });

    test('supports one page, an unpaired final page and RTL ordering', () {
      final onePage = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800)],
        outerMargin: 24,
        pageGap: 0,
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
          Size(600, 800),
        ],
        outerMargin: 24,
        pageGap: 0,
        rightToLeft: false,
      );
      expect(ltr.pageRects[0].left, lessThan(ltr.pageRects[1].left));
      final finalUnit = ReaderViewportMetrics.readingUnitRect(
        pageRects: ltr.pageRects,
        pageIndex: 4,
        facing: true,
      );
      expect(finalUnit, ltr.pageRects[4]);

      final rtl = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800), Size(600, 800), Size(600, 800)],
        outerMargin: 24,
        pageGap: 0,
        rightToLeft: true,
      );
      expect(rtl.pageRects[1].left, lessThan(rtl.pageRects[0].left));
    });

    test('keeps the physical gap separate from the visual book channel', () {
      final layout = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800), Size(600, 800), Size(600, 800)],
        outerMargin: 24,
        rightToLeft: false,
        pageGap: 0,
      );

      expect(layout.pageRects[1].left - layout.pageRects[0].right, 0);
      expect(
        layout.spreadRectFor(0).width,
        closeTo(layout.pageRects[0].width + layout.pageRects[1].width, 0.001),
      );
    });

    test('isolates a fitted spread from the following spread', () {
      final layout = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [
          Size(600, 800),
          Size(600, 800),
          Size(600, 800),
          Size(600, 800),
        ],
        outerMargin: 40,
        rightToLeft: false,
      );
      final firstSpread = layout.pageRects[0].expandToInclude(
        layout.pageRects[1],
      );
      final zoom = ReaderViewportMetrics.fitPageZoom(
        viewport: const Size(1920, 1080),
        contentRect: firstSpread,
        margin: 0,
        minimum: 0.1,
        maximum: 8,
      );
      final visible = Rect.fromCenter(
        center: firstSpread.center,
        width: 1920 / zoom,
        height: 1080 / zoom,
      );

      expect(visible.overlaps(layout.pageRects[2]), isFalse);
    });

    test('places the first no-cover page on the configured side', () {
      final right = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800), Size(600, 800), Size(600, 800)],
        outerMargin: 24,
        rightToLeft: false,
        coverPolicy: ReaderCoverPolicy.noCover,
        initialSide: ReaderInitialSide.right,
      );
      expect(
        right.pageRects[0].right,
        closeTo(right.documentSize.width / 2 - 24, 0.001),
      );

      final left = ReaderViewportMetrics.facingPageLayout(
        pageSizes: const [Size(600, 800), Size(600, 800), Size(600, 800)],
        outerMargin: 24,
        rightToLeft: false,
        coverPolicy: ReaderCoverPolicy.noCover,
        initialSide: ReaderInitialSide.left,
      );
      final firstSpread = left.pageRects[0].expandToInclude(left.pageRects[1]);
      expect(left.pageRects[0].left, lessThan(left.pageRects[1].left));
      expect(
        firstSpread.center.dx,
        closeTo(left.documentSize.width / 4, 0.001),
      );
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

    test('can preserve a top-aligned fit-width viewport when content fits', () {
      final center = ReaderViewportMetrics.normalizeCenter(
        proposedCenter: const Offset(600, 180),
        viewport: const Size(1200, 800),
        contentRect: const Rect.fromLTWH(300, 100, 600, 600),
        zoom: 1,
        margin: 20,
        centerVerticallyWhenFits: false,
      );

      expect(center, const Offset(600, 180));
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

  test('margins adapt in bounded discrete steps', () {
    expect(
      ReaderViewportMetrics.adaptiveMargin(
        viewport: const Size(640, 460),
        preferred: 40,
      ),
      12,
    );
    expect(
      ReaderViewportMetrics.adaptiveMargin(
        viewport: const Size(1920, 1080),
        preferred: 40,
      ),
      32,
    );
  });
}
