import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';

void main() {
  test('logger keeps error category but never forwards private details', () {
    String? observedMessage;
    Object? observedError;
    StackTrace? observedStack;
    final logger = AppLogger(
      minimumLevel: LogLevel.debug,
      sink: (level, message, error, stackTrace) {
        observedMessage = message;
        observedError = error;
        observedStack = stackTrace;
      },
    );

    logger.error(
      'Document operation failed.',
      FileSystemException('Denied', r'C:\Private\secret.pdf'),
      StackTrace.current,
    );

    expect(observedMessage, 'Document operation failed. [FileSystemException]');
    expect(observedMessage, isNot(contains('secret.pdf')));
    expect(observedMessage, isNot(contains(r'C:\Private')));
    expect(observedError, isNull);
    expect(observedStack, isNull);
  });
}
