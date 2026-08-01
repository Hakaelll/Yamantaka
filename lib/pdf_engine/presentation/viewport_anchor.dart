import 'dart:math' as math;

import '../models/pdf_models.dart';

final class ViewportAnchor {
  const ViewportAnchor({
    required this.pageIndex,
    required this.x,
    required this.y,
    required this.zoom,
    required this.rotation,
  });

  final int pageIndex;
  final double x;
  final double y;
  final double zoom;
  final int rotation;

  ViewportAnchor normalized() => ViewportAnchor(
    pageIndex: math.max(0, pageIndex),
    x: x.clamp(0.0, 1.0),
    y: y.clamp(0.0, 1.0),
    zoom: zoom.clamp(0.1, 20.0),
    rotation: ((rotation % 360) + 360) % 360,
  );

  Map<String, Object> toJson() => {
    'pageIndex': pageIndex,
    'x': x,
    'y': y,
    'zoom': zoom,
    'rotation': rotation,
  };
}

abstract final class ViewportAnchorTransform {
  static ViewportAnchor fromScroll({
    required int pageIndex,
    required double scrollOffset,
    required double zoom,
    required int rotation,
  }) => ViewportAnchor(
    pageIndex: pageIndex,
    x: 0.5,
    y: scrollOffset,
    zoom: zoom,
    rotation: rotation,
  ).normalized();

  static ViewportAnchor rotate(ViewportAnchor anchor, int nextRotation) {
    final turns = (((nextRotation - anchor.rotation) % 360) + 360) % 360 ~/ 90;
    var x = anchor.x;
    var y = anchor.y;
    for (var turn = 0; turn < turns; turn++) {
      final nextX = 1 - y;
      y = x;
      x = nextX;
    }
    return ViewportAnchor(
      pageIndex: anchor.pageIndex,
      x: x,
      y: y,
      zoom: anchor.zoom,
      rotation: nextRotation,
    ).normalized();
  }

  static PdfRenderRegion project(
    ViewportAnchor anchor, {
    required double width,
    required double height,
    required double viewportWidth,
    required double viewportHeight,
  }) {
    final safeWidth = math.max(1, width);
    final safeHeight = math.max(1, height);
    final visibleWidth = math
        .min(safeWidth, viewportWidth / anchor.zoom)
        .toDouble();
    final visibleHeight = math
        .min(safeHeight, viewportHeight / anchor.zoom)
        .toDouble();
    final left = (anchor.x * safeWidth - visibleWidth / 2)
        .clamp(0.0, math.max(0.0, safeWidth - visibleWidth))
        .toDouble();
    final top = (anchor.y * safeHeight - visibleHeight / 2)
        .clamp(0.0, math.max(0.0, safeHeight - visibleHeight))
        .toDouble();
    return PdfRenderRegion(
      left: left,
      top: top,
      width: visibleWidth,
      height: visibleHeight,
    );
  }
}
