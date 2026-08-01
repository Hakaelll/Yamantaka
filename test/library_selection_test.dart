import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/library_selection.dart';

void main() {
  test('selection supports additive ranges and pruning', () {
    final selection = LibrarySelection()..toggle('a');
    selection.selectRange(['a', 'b', 'c', 'd'], 'a', 'c', additive: true);
    expect(selection.ids, ['a', 'b', 'c']);
    selection.retain(['b', 'd']);
    expect(selection.ids, ['b']);
    selection.clear();
    expect(selection.isEmpty, isTrue);
  });

  test(
    'removing from library is confirmation-gated and never touches PDFs',
    () {
      expect(
        BulkOperationPolicy.requiresConfirmation(
          BulkLibraryAction.removeFromLibrary,
        ),
        isTrue,
      );
      expect(
        BulkOperationPolicy.changesPdfFile(BulkLibraryAction.favorite),
        isFalse,
      );
    },
  );

  test('bulk action summary keeps per-item failures', () {
    const summary = BulkOperationSummary([
      BulkItemResult(id: 'ok', success: true),
      BulkItemResult(
        id: 'failed',
        success: false,
        errorCode: 'operation_failed',
      ),
    ]);

    expect(summary.succeeded, 1);
    expect(summary.failed, 1);
    expect(summary.isPartial, isTrue);
    expect(
      BulkOperationPolicy.changesPdfFile(BulkLibraryAction.archive),
      isFalse,
    );
    expect(
      BulkOperationPolicy.changesPdfFile(BulkLibraryAction.assignTag),
      isFalse,
    );
  });
}
