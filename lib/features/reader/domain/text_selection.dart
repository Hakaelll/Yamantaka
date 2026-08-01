import 'dart:math' as math;

import '../../../features/annotations/domain/annotation.dart';

final class TextSelectionPosition {
  const TextSelectionPosition({required this.pageIndex, required this.offset});

  final int pageIndex;
  final int offset;

  TextSelectionPosition normalized() => TextSelectionPosition(
    pageIndex: math.max(0, pageIndex),
    offset: math.max(0, offset),
  );
}

final class TextSelectionRange {
  const TextSelectionRange({required this.start, required this.end});

  final TextSelectionPosition start;
  final TextSelectionPosition end;

  bool get isEmpty =>
      start.pageIndex == end.pageIndex && start.offset == end.offset;

  TextSelectionRange ordered() {
    final left = start.normalized();
    final right = end.normalized();
    final isReversed =
        left.pageIndex > right.pageIndex ||
        (left.pageIndex == right.pageIndex && left.offset > right.offset);
    return isReversed
        ? TextSelectionRange(start: right, end: left)
        : TextSelectionRange(start: left, end: right);
  }
}

final class TextSelectionPageGeometry {
  const TextSelectionPageGeometry({
    required this.pageIndex,
    required this.text,
    required this.rects,
  });

  final int pageIndex;
  final String text;
  final List<NormalizedRect> rects;

  TextSelectionPageGeometry normalized() => TextSelectionPageGeometry(
    pageIndex: math.max(0, pageIndex),
    text: text,
    rects: [
      for (final rect in rects)
        NormalizedRect(
          left: rect.left.clamp(0.0, 1.0),
          top: rect.top.clamp(0.0, 1.0),
          right: rect.right.clamp(0.0, 1.0),
          bottom: rect.bottom.clamp(0.0, 1.0),
        ),
    ],
  );
}

final class TextSelection {
  const TextSelection({required this.range, required this.pages});

  final TextSelectionRange range;
  final List<TextSelectionPageGeometry> pages;

  String get selectedText {
    final ordered = range.ordered();
    final parts = <String>[];
    for (final page in pages) {
      if (page.pageIndex < ordered.start.pageIndex ||
          page.pageIndex > ordered.end.pageIndex) {
        continue;
      }
      final start = page.pageIndex == ordered.start.pageIndex
          ? ordered.start.offset.clamp(0, page.text.length)
          : 0;
      final end = page.pageIndex == ordered.end.pageIndex
          ? ordered.end.offset.clamp(start, page.text.length)
          : page.text.length;
      if (end > start) parts.add(page.text.substring(start, end));
    }
    return parts.join('\n');
  }

  List<NormalizedRect> get normalizedRects => [
    for (final page in pages) ...page.normalized().rects,
  ];

  bool get hasText => selectedText.trim().isNotEmpty;
}
