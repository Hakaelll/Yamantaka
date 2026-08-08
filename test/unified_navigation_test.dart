import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/navigation/domain/unified_navigation.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';

void main() {
  test('canonical navigation delegates to the reading unit policy', () {
    const target = UnifiedNavigationTarget(
      pageIndex: 4,
      kind: UnifiedNavigationKind.thumbnail,
      offset: 0.3,
      label: 'Página',
    );
    final canonical = target.canonical(pageCount: 8, mode: ReaderMode.book);
    expect(canonical.pageIndex, 4);
    expect(canonical.offset, 0.3);
  });

  test('commands are ignored after document generation changes', () {
    const command = NavigationCommand(
      target: UnifiedNavigationTarget(
        pageIndex: 1,
        kind: UnifiedNavigationKind.page,
      ),
      generation: 2,
      origin: 'keyboard',
    );
    expect(command.isCurrent(2), isTrue);
    expect(command.isCurrent(3), isFalse);
  });
}
