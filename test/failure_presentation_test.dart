import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/errors/failure_presentation.dart';

void main() {
  test('every typed failure has a safe presentation', () {
    for (final kind in FailureKind.values) {
      final presentation = FailurePresentations.forFailure(
        AppFailure(
          kind,
          'private message',
          debugMessage: r'C:\private\book.pdf',
        ),
      );
      expect(presentation.title, isNotEmpty);
      expect(presentation.message, isNot(contains(r'C:\')));
    }
  });

  test('cancellation is not classified as a critical error', () {
    expect(
      FailurePresentations.forFailure(
        const AppFailure(FailureKind.cancelled, 'cancelled'),
      ).isCancellation,
      isTrue,
    );
    expect(
      FailurePresentations.forFailure(
        const AppFailure(FailureKind.thumbnailFailed, 'thumbnail failed'),
      ).isCancellation,
      isFalse,
    );
  });

  test('resource and database failures offer the diagnostics center', () {
    for (final kind in <FailureKind>[
      FailureKind.databaseFailed,
      FailureKind.outOfMemory,
      FailureKind.unknown,
    ]) {
      expect(
        FailurePresentations.forFailure(
          AppFailure(kind, 'safe message'),
        ).action,
        FailureAction.openDiagnostics,
      );
    }
  });
}
