import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/privacy/storage_catalog.dart';

void main() {
  test('storage catalog distinguishes rebuildable data from user data', () {
    final covers = StorageCatalog.describe(StorageCategory.covers, 100);
    final database = StorageCatalog.describe(StorageCategory.database, 200);
    expect(covers.rebuildable, isTrue);
    expect(covers.canClear, isTrue);
    expect(database.personal, isTrue);
    expect(database.canClear, isFalse);
  });
}
