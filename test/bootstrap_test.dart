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

  test('collects multiple unique PDF command-line arguments', () {
    expect(
      initialPdfPaths([
        'yamantaka.exe',
        r'C:\one.pdf',
        r'C:\two.PDF',
        r'C:\one.pdf',
        '--safe',
      ]),
      [r'C:\one.pdf', r'C:\two.PDF'],
    );
  });
}
