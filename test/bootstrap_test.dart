import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/bootstrap.dart';

void main() {
  test('finds a PDF command-line argument without accepting other files', () {
    expect(
      initialPdfPath(['--verbose', r'C:\Books\Example.PDF']),
      r'C:\Books\Example.PDF',
    );
    expect(initialPdfPath(['--verbose', 'notes.txt']), isNull);
  });
}
