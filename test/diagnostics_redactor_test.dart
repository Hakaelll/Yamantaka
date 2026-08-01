import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/privacy/diagnostics_redactor.dart';

void main() {
  test('diagnostics allowlist excludes content-bearing fields', () {
    final safe = DiagnosticsRedactor.redact({
      'documents': 3,
      'path': r'C:\Users\private\book.pdf',
      'query': 'secret term',
      'engine': {'open_documents': 1},
    });
    expect(safe, containsPair('documents', 3));
    expect(safe, isNot(contains('path')));
    expect(safe, isNot(contains('query')));
    expect(DiagnosticsRedactor.manifest()['contains_private_content'], isFalse);
  });
}
