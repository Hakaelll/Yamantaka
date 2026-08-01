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
    required this.horizontalCellExtent,
    required this.cellExtent,
  });

  final int columns;
  final double artifactWidth;
  final double artifactHeight;

  /// The horizontal slot allocated to each artifact in the sparse canvas.
  final double horizontalCellExtent;

  /// The vertical slot allocated to each artifact row.
  final double cellExtent;
}

abstract final class LibraryArtifactLayout {
  static const double gridGap = 16;

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
    final usableWidth = math.max(1.0, viewportWidth);
    if (documentCount <= 1) {
      final width = math.min(210.0, usableWidth * 0.66);
      return ArtifactGridMetrics(
        columns: 1,
        artifactWidth: width,
        artifactHeight: width * 1.42,
        horizontalCellExtent: width,
        cellExtent: width * 1.42 + 32,
      );
    }
    if (documentCount <= 6) {
      final desiredColumns = documentCount <= 3 ? documentCount : 3;
      final columns = math.min(
        desiredColumns,
        math.max(1, ((usableWidth + 16) / 188).floor()),
      );
      final slotWidth = (usableWidth - gridGap * (columns - 1)) / columns;
      final width = math.min(172.0, slotWidth * 0.78);
      return ArtifactGridMetrics(
        columns: columns,
        artifactWidth: width,
        artifactHeight: width * 1.42,
        horizontalCellExtent: slotWidth,
        cellExtent: width * 1.42 + 32,
      );
    }
    final targetCellWidth = switch (usableWidth) {
      < 680 => 176.0,
      < 1600 => 204.0,
      < 2200 => 220.0,
      _ => 236.0,
    };
    final maximumColumns = switch (usableWidth) {
      < 680 => 4,
      < 1600 => 7,
      < 2200 => 8,
      _ => 10,
    };
    final minimumColumns = usableWidth < 188 ? 1 : 2;
    final columns = (usableWidth / targetCellWidth).floor().clamp(
      minimumColumns,
      maximumColumns,
    );
    final cellWidth = (usableWidth - gridGap * (columns - 1)) / columns;
    final width = math.min(184.0, cellWidth * 0.72);
    return ArtifactGridMetrics(
      columns: columns,
      artifactWidth: width,
      artifactHeight: width * 1.42,
      horizontalCellExtent: cellWidth,
      cellExtent: width * 1.42 + 32,
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
