import 'dart:math' as math;

import '../models/pdf_models.dart';

final class PdfTilePlanRequest {
  const PdfTilePlanRequest({
    required this.documentId,
    required this.pageIndex,
    required this.pageWidth,
    required this.pageHeight,
    required this.scale,
    required this.rotation,
    required this.viewport,
    required this.maxBytes,
    this.tileSize = 1024,
    this.overlap = 2,
    this.generation = 0,
    this.extremePageBytes = 32 * 1024 * 1024,
  });

  final String documentId;
  final int pageIndex;
  final double pageWidth;
  final double pageHeight;
  final double scale;
  final int rotation;
  final PdfRenderRegion viewport;
  final int maxBytes;
  final int tileSize;
  final int overlap;
  final int generation;
  final int extremePageBytes;
}

final class PdfTileRequest {
  const PdfTileRequest({
    required this.key,
    required this.region,
    required this.priority,
  });

  final PdfTileKey key;
  final PdfRenderRegion region;
  final int priority;
}

final class PdfTileKey {
  const PdfTileKey({
    required this.documentId,
    required this.pageIndex,
    required this.scale,
    required this.rotation,
    required this.region,
    required this.generation,
  });

  final String documentId;
  final int pageIndex;
  final int scale;
  final int rotation;
  final PdfRenderRegion region;
  final int generation;

  @override
  int get hashCode => Object.hash(
    documentId,
    pageIndex,
    scale,
    rotation,
    generation,
    region.left,
    region.top,
    region.width,
    region.height,
  );

  @override
  bool operator ==(Object other) =>
      other is PdfTileKey &&
      other.documentId == documentId &&
      other.pageIndex == pageIndex &&
      other.scale == scale &&
      other.rotation == rotation &&
      other.generation == generation &&
      other.region.left == region.left &&
      other.region.top == region.top &&
      other.region.width == region.width &&
      other.region.height == region.height;
}

final class PdfTilePlan {
  const PdfTilePlan({required this.useTiles, required this.requests});

  final bool useTiles;
  final List<PdfTileRequest> requests;
}

/// Chooses a full-page render for ordinary documents and a viewport ring for
/// pathological pages. The planner is pure: it never allocates images or
/// writes tiles to disk.
abstract final class PdfTilePlanner {
  static PdfTilePlan plan(PdfTilePlanRequest request) {
    final width = math.max(1.0, request.pageWidth * request.scale);
    final height = math.max(1.0, request.pageHeight * request.scale);
    final estimatedBytes = width.ceil() * height.ceil() * 4;
    if (estimatedBytes <= request.extremePageBytes ||
        estimatedBytes <= request.maxBytes) {
      final region = PdfRenderRegion(
        left: 0,
        top: 0,
        width: request.pageWidth,
        height: request.pageHeight,
      );
      return PdfTilePlan(
        useTiles: false,
        requests: [
          PdfTileRequest(
            key: _key(request, region),
            region: region,
            priority: 0,
          ),
        ],
      );
    }

    final tileSize = request.tileSize.clamp(128, 4096);
    final overlap = request.overlap.clamp(0, 16);
    final viewport = _clampRegion(
      request.viewport,
      request.pageWidth,
      request.pageHeight,
    );
    final firstX = math.max(0, (viewport.left / tileSize).floor() - 1);
    final firstY = math.max(0, (viewport.top / tileSize).floor() - 1);
    final lastX = math.min(
      (request.pageWidth / tileSize).ceil() - 1,
      ((viewport.left + viewport.width) / tileSize).ceil(),
    );
    final lastY = math.min(
      (request.pageHeight / tileSize).ceil() - 1,
      ((viewport.top + viewport.height) / tileSize).ceil(),
    );
    final center = viewport;
    final result = <PdfTileRequest>[];
    for (var y = firstY; y <= lastY; y++) {
      for (var x = firstX; x <= lastX; x++) {
        final left = math.max(0.0, x * tileSize - overlap).toDouble();
        final top = math.max(0.0, y * tileSize - overlap).toDouble();
        final right = math
            .min(request.pageWidth, (x + 1) * tileSize + overlap)
            .toDouble();
        final bottom = math
            .min(request.pageHeight, (y + 1) * tileSize + overlap)
            .toDouble();
        final region = PdfRenderRegion(
          left: left,
          top: top,
          width: math.max(1, right - left),
          height: math.max(1, bottom - top),
        );
        final distance =
            (region.left + region.width / 2 - center.left).abs() +
            (region.top + region.height / 2 - center.top).abs();
        result.add(
          PdfTileRequest(
            key: _key(request, region),
            region: region,
            priority: distance.round(),
          ),
        );
      }
    }
    result.sort((left, right) => left.priority.compareTo(right.priority));
    return PdfTilePlan(useTiles: true, requests: result);
  }

  static PdfTileKey _key(PdfTilePlanRequest request, PdfRenderRegion region) =>
      PdfTileKey(
        documentId: request.documentId,
        pageIndex: request.pageIndex,
        scale: (request.scale * 1000).round(),
        rotation: request.rotation % 360,
        region: region,
        generation: request.generation,
      );

  static PdfRenderRegion _clampRegion(
    PdfRenderRegion region,
    double pageWidth,
    double pageHeight,
  ) {
    final left = region.left.clamp(0.0, pageWidth);
    final top = region.top.clamp(0.0, pageHeight);
    final right = (region.left + region.width).clamp(left, pageWidth);
    final bottom = (region.top + region.height).clamp(top, pageHeight);
    return PdfRenderRegion(
      left: left,
      top: top,
      width: math.max(1, right - left),
      height: math.max(1, bottom - top),
    );
  }
}
