import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';

void main() {
  test('cancellation supports listener removal and idempotency', () {
    final token = PdfOperationToken();
    var called = 0;
    void listener() => called++;

    token.addCancellationListener(listener);
    token.removeCancellationListener(listener);
    token.cancel();
    token.cancel();

    expect(token.isCancelled, true);
    expect(called, 0);
  });

  test('a listener added after cancellation runs immediately', () {
    final token = PdfOperationToken()..cancel();
    var called = 0;

    token.addCancellationListener(() => called++);

    expect(called, 1);
  });

  test('cancellation notifies an active listener exactly once', () {
    final token = PdfOperationToken();
    var called = 0;

    token.addCancellationListener(() => called++);
    token.cancel();
    token.cancel();

    expect(called, 1);
  });
}
