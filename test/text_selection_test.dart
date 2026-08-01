import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/annotations/domain/annotation.dart';
import 'package:yamantaka/features/reader/domain/text_selection.dart';

void main() {
  test('multi-page selection returns only the selected text', () {
    const selection = TextSelection(
      range: TextSelectionRange(
        start: TextSelectionPosition(pageIndex: 0, offset: 2),
        end: TextSelectionPosition(pageIndex: 1, offset: 3),
      ),
      pages: [
        TextSelectionPageGeometry(
          pageIndex: 0,
          text: 'abcdef',
          rects: [NormalizedRect(left: 0, top: 0, right: 0.2, bottom: 0.1)],
        ),
        TextSelectionPageGeometry(
          pageIndex: 1,
          text: 'ghijkl',
          rects: [NormalizedRect(left: 0.1, top: 0.1, right: 0.4, bottom: 0.2)],
        ),
      ],
    );
    expect(selection.selectedText, 'cdef\nghi');
    expect(selection.hasText, isTrue);
    expect(selection.normalizedRects, hasLength(2));
  });
}
