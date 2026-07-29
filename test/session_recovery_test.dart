import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/app.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

void main() {
  test('session recovery is safe, opt-out aware and loop resistant', () {
    const recoverable = AppSettings(
      restoreLastSession: true,
      lastDocumentPath: 'book.pdf',
    );
    expect(shouldRestoreReaderSession(recoverable, pathExists: true), isTrue);
    expect(shouldRestoreReaderSession(recoverable, pathExists: false), isFalse);
    expect(
      shouldRestoreReaderSession(
        recoverable,
        pathExists: true,
        initialPathProvided: true,
      ),
      isFalse,
    );
    expect(
      shouldRestoreReaderSession(
        recoverable.copyWith(startInLibrary: true),
        pathExists: true,
      ),
      isFalse,
    );
    expect(
      shouldRestoreReaderSession(
        recoverable.copyWith(restoreLastSession: false),
        pathExists: true,
      ),
      isFalse,
    );
    expect(
      shouldRestoreReaderSession(
        recoverable.copyWith(lastDocumentPath: ''),
        pathExists: true,
      ),
      isFalse,
    );
  });
}
