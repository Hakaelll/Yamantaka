import 'dart:math' as math;

final class ArtifactJitter {
  const ArtifactJitter({
    required this.offsetX,
    required this.offsetY,
    required this.rotationDegrees,
    required this.scale,
  });

  final double offsetX;
  final double offsetY;
  final double rotationDegrees;
  final double scale;
}

final class ArtifactGridMetrics {
  const ArtifactGridMetrics({
    required this.columns,
    required this.artifactWidth,
    required this.artifactHeight,
    required this.cellExtent,
  });

  final int columns;
  final double artifactWidth;
  final double artifactHeight;
  final double cellExtent;
}

abstract final class LibraryArtifactLayout {
  static ArtifactJitter jitterFor(String stableId) {
    final hash = _stableHash(stableId);
    double signed(int shift, int mask) {
      final normalized = ((hash >> shift) & mask) / mask;
      return normalized * 2 - 1;
    }

    return ArtifactJitter(
      offsetX: signed(0, 0xff) * 3,
      offsetY: signed(8, 0xff) * 4,
      rotationDegrees: signed(16, 0xff) * 0.65,
      scale: 1 + signed(24, 0x7f) * 0.006,
    );
  }

  static ArtifactGridMetrics metricsFor({
    required double viewportWidth,
    required int documentCount,
  }) {
    final usableWidth = math.max(280.0, viewportWidth);
    if (documentCount <= 1) {
      final width = math.min(210.0, usableWidth * 0.46);
      return ArtifactGridMetrics(
        columns: 1,
        artifactWidth: width,
        artifactHeight: width * 1.42,
        cellExtent: width * 1.78,
      );
    }
    if (documentCount <= 6) {
      final columns = documentCount <= 3 ? documentCount : 3;
      final width = math.min(172.0, (usableWidth - 96) / columns * 0.68);
      return ArtifactGridMetrics(
        columns: columns,
        artifactWidth: width,
        artifactHeight: width * 1.42,
        cellExtent: width * 1.72,
      );
    }
    final targetCellWidth = usableWidth < 680 ? 150.0 : 196.0;
    final columns = (usableWidth / targetCellWidth).floor().clamp(2, 7);
    final cellWidth = usableWidth / columns;
    final width = math.min(158.0, cellWidth * 0.68);
    return ArtifactGridMetrics(
      columns: columns,
      artifactWidth: width,
      artifactHeight: width * 1.42,
      cellExtent: width * 1.74,
    );
  }

  static int indexAfterNavigation({
    required int currentIndex,
    required int itemCount,
    required int columns,
    required ArtifactNavigationDirection direction,
  }) {
    if (itemCount <= 0) return -1;
    final current = currentIndex.clamp(0, itemCount - 1);
    return switch (direction) {
      ArtifactNavigationDirection.left => math.max(0, current - 1),
      ArtifactNavigationDirection.right => math.min(itemCount - 1, current + 1),
      ArtifactNavigationDirection.up => math.max(0, current - columns),
      ArtifactNavigationDirection.down => math.min(
        itemCount - 1,
        current + columns,
      ),
      ArtifactNavigationDirection.first => 0,
      ArtifactNavigationDirection.last => itemCount - 1,
    };
  }

  static int _stableHash(String value) {
    var hash = 0x811c9dc5;
    for (final codeUnit in value.codeUnits) {
      hash ^= codeUnit;
      hash = (hash * 0x01000193) & 0xffffffff;
    }
    return hash;
  }
}

enum ArtifactNavigationDirection { left, right, up, down, first, last }
