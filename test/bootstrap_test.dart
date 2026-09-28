import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/bootstrap.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

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

  test('startup maximizes while retaining bounded normal geometry', () {
    final presentation = resolveInitialWindowPresentation(
      const AppSettings(
        windowWidth: 3200,
        windowHeight: 400,
        windowX: 120,
        windowY: 80,
      ),
    );

    expect(presentation.maximized, isTrue);
    expect(presentation.fullScreen, isTrue);
    expect(presentation.normalSize.width, 2400);
    expect(presentation.normalSize.height, 480);
    expect(presentation.normalPosition?.dx, 120);
    expect(presentation.normalPosition?.dy, 80);
  });
}
