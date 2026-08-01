import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/workspaces/domain/split_reader_session.dart';

void main() {
  test('split panes have independent focus and lifecycle', () {
    var session = const SplitReaderSession().open(ReaderPane.primary, 'a');
    session = session.open(ReaderPane.secondary, 'b');
    expect(session.primary.documentId, 'a');
    expect(session.secondary.documentId, 'b');
    expect(session.focused, ReaderPane.secondary);
    expect(session.close(ReaderPane.secondary).secondary.closed, isTrue);
  });
}
