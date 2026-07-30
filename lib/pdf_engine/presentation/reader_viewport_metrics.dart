import 'dart:math' as math;
import 'dart:ui';

final class ReaderPageLayoutGeometry {
  const ReaderPageLayoutGeometry({
    required this.pageRects,
    required this.documentSize,
  });

  final List<Rect> pageRects;
  final Size documentSize;
}

/// Single source of truth for reader page geometry and viewport constraints.
///
/// The PDF adapter owns the native transform, while this class keeps all
/// layout, fit and centering calculations deterministic and independently
/// testable.
abstract final class ReaderViewportMetrics {
  static double adaptiveMargin({
    required Size viewport,
    required double preferred,
  }) {
    final shortestSide = math.min(viewport.width, viewport.height);
    final maximum = switch (shortestSide) {
      < 520 => 16.0,
      < 720 => 24.0,
      < 1000 => 32.0,
      _ => 40.0,
    };
    return preferred.clamp(12.0, maximum).toDouble();
  }

  static double adaptiveSpineGap(double viewportWidth) =>
      switch (viewportWidth) {
        < 800 => 10.0,
        < 1400 => 14.0,
        _ => 16.0,
      };

  static ReaderPageLayoutGeometry facingPageLayout({
    required List<Size> pageSizes,
    required double outerMargin,
    required double spineGap,
    required bool rightToLeft,
  }) {
    if (pageSizes.isEmpty) {
      return const ReaderPageLayoutGeometry(
        pageRects: [],
        documentSize: Size.zero,
      );
    }

    var widestUnit = pageSizes.first.width;
    var tallestPage = pageSizes.first.height;
    for (var index = 1; index < pageSizes.length; index += 2) {
      final first = pageSizes[index];
      final second = index + 1 < pageSizes.length ? pageSizes[index + 1] : null;
      widestUnit = math.max(
        widestUnit,
        first.width + (second == null ? 0 : spineGap + second.width),
      );
      tallestPage = math.max(tallestPage, first.height);
      if (second != null) {
        tallestPage = math.max(tallestPage, second.height);
      }
    }

    final slotWidth = widestUnit + outerMargin * 2;
    final documentHeight = tallestPage + outerMargin * 2;
    final rects = List<Rect>.filled(pageSizes.length, Rect.zero);
    var slotLeft = 0.0;

    final cover = pageSizes.first;
    rects[0] = Rect.fromLTWH(
      slotLeft + (slotWidth - cover.width) / 2,
      (documentHeight - cover.height) / 2,
      cover.width,
      cover.height,
    );
    slotLeft += slotWidth;

    for (var index = 1; index < pageSizes.length; index += 2) {
      final first = pageSizes[index];
      final second = index + 1 < pageSizes.length ? pageSizes[index + 1] : null;
      if (second == null) {
        rects[index] = Rect.fromLTWH(
          slotLeft + (slotWidth - first.width) / 2,
          (documentHeight - first.height) / 2,
          first.width,
          first.height,
        );
        slotLeft += slotWidth;
        continue;
      }

      final left = rightToLeft ? second : first;
      final right = rightToLeft ? first : second;
      final pairWidth = left.width + spineGap + right.width;
      final pairTop =
          (documentHeight - math.max(left.height, right.height)) / 2;
      final pairLeft = slotLeft + (slotWidth - pairWidth) / 2;
      final leftIndex = rightToLeft ? index + 1 : index;
      final rightIndex = rightToLeft ? index : index + 1;
      rects[leftIndex] = Rect.fromLTWH(
        pairLeft,
        pairTop,
        left.width,
        left.height,
      );
      rects[rightIndex] = Rect.fromLTWH(
        pairLeft + left.width + spineGap,
        pairTop,
        right.width,
        right.height,
      );
      slotLeft += slotWidth;
    }

    return ReaderPageLayoutGeometry(
      pageRects: rects,
      documentSize: Size(slotLeft, documentHeight),
    );
  }

  static ReaderPageLayoutGeometry singlePageLayout({
    required List<Size> pageSizes,
    required double outerMargin,
  }) {
    if (pageSizes.isEmpty) {
      return const ReaderPageLayoutGeometry(
        pageRects: [],
        documentSize: Size.zero,
      );
    }
    final widestPage = pageSizes.fold<double>(
      0,
      (width, page) => math.max(width, page.width),
    );
    final tallestPage = pageSizes.fold<double>(
      0,
      (height, page) => math.max(height, page.height),
    );
    // A wide slot prevents a neighbouring page leaking into one-page mode.
    final slotWidth = widestPage * 2 + outerMargin * 3;
    final documentHeight = tallestPage + outerMargin * 2;
    final rects = <Rect>[];
    for (var index = 0; index < pageSizes.length; index++) {
      final page = pageSizes[index];
      rects.add(
        Rect.fromLTWH(
          index * slotWidth + (slotWidth - page.width) / 2,
          (documentHeight - page.height) / 2,
          page.width,
          page.height,
        ),
      );
    }
    return ReaderPageLayoutGeometry(
      pageRects: rects,
      documentSize: Size(slotWidth * pageSizes.length, documentHeight),
    );
  }

  static Rect readingUnitRect({
    required List<Rect> pageRects,
    required int pageIndex,
    required bool facing,
  }) {
    if (pageRects.isEmpty) return Rect.zero;
    final index = pageIndex.clamp(0, pageRects.length - 1);
    if (!facing || index == 0) return pageRects[index];
    final start = index.isOdd ? index : index - 1;
    var result = pageRects[start];
    if (start + 1 < pageRects.length) {
      result = result.expandToInclude(pageRects[start + 1]);
    }
    return result;
  }

  static Rect nearestReadingUnitRect({
    required List<Rect> pageRects,
    required Offset proposedCenter,
    required bool facing,
  }) {
    if (pageRects.isEmpty) return Rect.zero;
    if (!facing) {
      return pageRects.reduce(
        (closest, candidate) =>
            (candidate.center.dx - proposedCenter.dx).abs() <
                (closest.center.dx - proposedCenter.dx).abs()
            ? candidate
            : closest,
      );
    }

    final units = <Rect>[pageRects.first];
    for (var index = 1; index < pageRects.length; index += 2) {
      var unit = pageRects[index];
      if (index + 1 < pageRects.length) {
        unit = unit.expandToInclude(pageRects[index + 1]);
      }
      units.add(unit);
    }
    return units.reduce(
      (closest, candidate) =>
          (candidate.center.dx - proposedCenter.dx).abs() <
              (closest.center.dx - proposedCenter.dx).abs()
          ? candidate
          : closest,
    );
  }

  static Offset normalizeCenter({
    required Offset proposedCenter,
    required Size viewport,
    required Rect contentRect,
    required double zoom,
    double margin = 0,
  }) {
    if (contentRect.isEmpty || viewport.isEmpty || zoom <= 0) {
      return proposedCenter;
    }
    final boundedRect = contentRect.inflate(margin);
    final visibleWidth = viewport.width / zoom;
    final visibleHeight = viewport.height / zoom;
    final x = boundedRect.width <= visibleWidth
        ? boundedRect.center.dx
        : proposedCenter.dx.clamp(
            boundedRect.left + visibleWidth / 2,
            boundedRect.right - visibleWidth / 2,
          );
    final y = boundedRect.height <= visibleHeight
        ? boundedRect.center.dy
        : proposedCenter.dy.clamp(
            boundedRect.top + visibleHeight / 2,
            boundedRect.bottom - visibleHeight / 2,
          );
    return Offset(x.toDouble(), y.toDouble());
  }

  static double fitPageZoom({
    required Size viewport,
    required Rect contentRect,
    required double margin,
    required double minimum,
    required double maximum,
  }) {
    if (viewport.isEmpty || contentRect.isEmpty) return minimum;
    final availableWidth = math.max(1.0, viewport.width - margin * 2);
    final availableHeight = math.max(1.0, viewport.height - margin * 2);
    return math
        .min(
          availableWidth / contentRect.width,
          availableHeight / contentRect.height,
        )
        .clamp(minimum, maximum)
        .toDouble();
  }

  static double fitWidthZoom({
    required Size viewport,
    required Rect contentRect,
    required double margin,
    required double minimum,
    required double maximum,
  }) {
    if (viewport.isEmpty || contentRect.isEmpty) return minimum;
    final availableWidth = math.max(1.0, viewport.width - margin * 2);
    return (availableWidth / contentRect.width)
        .clamp(minimum, maximum)
        .toDouble();
  }
}
