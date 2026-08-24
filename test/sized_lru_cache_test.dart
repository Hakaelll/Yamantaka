import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/cache/sized_lru_cache.dart';

void main() {
  test('evicts least recently used values by byte budget', () {
    final evicted = <String>[];
    final cache = SizedLruCache<String, String>(
      maxBytes: 5,
      sizeOf: (value) => value.length,
      onEvict: evicted.add,
    );
    cache
      ..put('a', 'aa')
      ..put('b', 'bb');
    expect(cache.get('a'), 'aa');
    cache.put('c', 'cc');

    expect(cache.get('b'), isNull);
    expect(cache.get('a'), 'aa');
    expect(cache.get('c'), 'cc');
    expect(evicted, ['bb']);
  });

  test('shrinks an existing cache immediately when pressure rises', () {
    final evicted = <String>[];
    final cache = SizedLruCache<String, String>(
      maxBytes: 8,
      sizeOf: (value) => value.length,
      onEvict: evicted.add,
    );
    cache
      ..put('oldest', 'aaaa')
      ..put('newest', 'bbbb')
      ..setMaxBytes(4);

    expect(cache.maxBytes, 4);
    expect(cache.currentBytes, 4);
    expect(cache.containsKey('oldest'), isFalse);
    expect(cache.get('newest'), 'bbbb');
    expect(evicted, ['aaaa']);
  });
}
