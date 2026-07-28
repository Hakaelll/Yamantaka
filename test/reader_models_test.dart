import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';

void main() {
  group('ReadingProgress', () {
    test('clamps invalid pages and reaches one on the final page', () {
      expect(ReadingProgress.calculate(pageIndex: -4, pageCount: 10), 0);
      expect(ReadingProgress.calculate(pageIndex: 4, pageCount: 10), 4 / 9);
      expect(ReadingProgress.calculate(pageIndex: 99, pageCount: 10), 1);
    });

    test('handles empty and single-page documents', () {
      expect(ReadingProgress.calculate(pageIndex: 0, pageCount: 0), 0);
      expect(ReadingProgress.calculate(pageIndex: 0, pageCount: 1), 1);
    });
  });

  group('FacingPageSpread', () {
    test('keeps cover alone and then pairs odd-even page indexes', () {
      expect(FacingPageSpread.forPage(0, 8).rightPageIndex, 0);
      final spread = FacingPageSpread.forPage(2, 8);
      expect(spread.leftPageIndex, 1);
      expect(spread.rightPageIndex, 2);
    });

    test('supports a final unpaired page and RTL direction', () {
      final finalSpread = FacingPageSpread.forPage(4, 5);
      expect(finalSpread.leftPageIndex, 3);
      expect(finalSpread.rightPageIndex, 4);

      final rtl = FacingPageSpread.forPage(
        1,
        5,
        direction: ReadingDirection.rightToLeft,
      );
      expect(rtl.leftPageIndex, 2);
      expect(rtl.rightPageIndex, 1);
    });
  });

  test(
    'visual profiles and reading directions remain explicit domain values',
    () {
      expect(ReaderVisualProfile.values, hasLength(2));
      expect(ReadingDirection.values, hasLength(2));
      expect(ReaderVisualProfile.electronicInk.name, 'electronicInk');
    },
  );
}
