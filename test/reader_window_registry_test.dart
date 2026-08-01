import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/reader_window_registry.dart';

void main() {
  test('window registry deduplicates documents and caps windows', () {
    final registry = ReaderWindowRegistry();
    final first = registry.open('a');
    expect(registry.open('a')?.id, first?.id);
    expect(registry.open('b'), isNotNull);
    expect(registry.open('c'), isNull);
  });

  test('window IDs remain unique after closing a lower-numbered window', () {
    final registry = ReaderWindowRegistry(maxWindows: 2);
    final first = registry.open('a')!;
    final second = registry.open('b')!;
    registry.close(first.id);
    final third = registry.open('c')!;
    expect(third.id, isNot(second.id));
    expect(registry.windows.map((item) => item.id).toSet(), hasLength(2));
  });
}
