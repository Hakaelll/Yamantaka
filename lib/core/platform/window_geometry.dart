import 'dart:math' as math;

final class WindowBounds {
  const WindowBounds({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    this.isMaximized = false,
  });

  final double x;
  final double y;
  final double width;
  final double height;
  final bool isMaximized;

  WindowBounds normalized({
    required WindowWorkArea workArea,
    double minWidth = 640,
    double minHeight = 480,
    double safeMargin = 24,
  }) {
    final availableWidth = math.max(1, workArea.width);
    final availableHeight = math.max(1, workArea.height);
    final safeWidth = width
        .clamp(math.min(minWidth, availableWidth), availableWidth)
        .toDouble();
    final safeHeight = height
        .clamp(math.min(minHeight, availableHeight), availableHeight)
        .toDouble();
    final maxX = workArea.right - safeMargin;
    final maxY = workArea.bottom - safeMargin;
    final minX = workArea.left - safeWidth + safeMargin;
    final minY = workArea.top - safeHeight + safeMargin;
    return WindowBounds(
      x: x.clamp(minX, maxX).toDouble(),
      y: y.clamp(minY, maxY).toDouble(),
      width: safeWidth,
      height: safeHeight,
      isMaximized: isMaximized,
    );
  }

  Map<String, Object> toJson() => {
    'x': x,
    'y': y,
    'width': width,
    'height': height,
    'maximized': isMaximized,
  };
}

final class WindowWorkArea {
  const WindowWorkArea({
    required this.left,
    required this.top,
    required this.width,
    required this.height,
  });

  final double left;
  final double top;
  final double width;
  final double height;

  double get right => left + width;
  double get bottom => top + height;
}
