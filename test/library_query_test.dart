import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';

void main() {
  test('library query bounds the materialized window', () {
    final query = const LibraryQuery(offset: -10, limit: 500).normalized();
    expect(query.offset, 0);
    expect(query.limit, 200);
  });
}
