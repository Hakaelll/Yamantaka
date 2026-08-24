import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/privacy/diagnostics_redactor.dart';

void main() {
  test('diagnostics allowlist excludes content-bearing fields', () {
    final safe = DiagnosticsRedactor.redact({
      'documents': 3,
      'path': r'C:\Users\private\book.pdf',
      'query': 'secret term',
      'engine': {'open_documents': 1},
      'performance': {
        'durations': {
          'page_render': {'p95_us': 12000},
        },
        'path': r'C:\Users\private\book.pdf',
      },
    });
    expect(safe, containsPair('documents', 3));
    expect(safe, isNot(contains('path')));
    expect(safe, isNot(contains('query')));
    expect(safe['performance'], isNot(contains('path')));
    expect(safe['performance'], contains('durations'));
    expect(DiagnosticsRedactor.manifest()['contains_private_content'], isFalse);
  });
}
