import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/path_normalizer.dart';

void main() {
  test('normalizes Windows separators and casing', () {
    final normalized = PathNormalizer.normalize(
      r'C:/Books/../Books/Novel.PDF/',
      windows: true,
    );
    expect(normalized, r'c:\books\novel.pdf');
  });
}
