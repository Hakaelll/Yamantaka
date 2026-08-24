import 'dart:collection';

final class SizedLruCache<K, V extends Object> {
  SizedLruCache({
    required int maxBytes,
    required this.sizeOf,
    this.onEvict,
    this.onEvictEntry,
  }) : assert(maxBytes >= 0),
       _maxBytes = maxBytes;

  int _maxBytes;
  int get maxBytes => _maxBytes;
  final int Function(V value) sizeOf;
  final void Function(V value)? onEvict;
  final void Function(K key, V value)? onEvictEntry;
  final LinkedHashMap<K, V> _entries = LinkedHashMap<K, V>();
  int _currentBytes = 0;

  int get currentBytes => _currentBytes;
  int get length => _entries.length;
  Iterable<K> get keys => _entries.keys;

  bool containsKey(K key) => _entries.containsKey(key);

  void setMaxBytes(int value) {
    if (value < 0) throw ArgumentError.value(value, 'value');
    _maxBytes = value;
    _trim();
  }

  V? get(K key) {
    final value = _entries.remove(key);
    if (value == null) return null;
    _entries[key] = value;
    return value;
  }

  void put(K key, V value) {
    final replaced = _entries.remove(key);
    if (replaced != null) {
      _currentBytes -= sizeOf(replaced);
      onEvictEntry?.call(key, replaced);
      onEvict?.call(replaced);
    }
    _entries[key] = value;
    _currentBytes += sizeOf(value);
    _trim();
  }

  void removeWhere(bool Function(K key) predicate) {
    final keys = _entries.keys.where(predicate).toList(growable: false);
    for (final key in keys) {
      final value = _entries.remove(key);
      if (value != null) {
        _currentBytes -= sizeOf(value);
        onEvictEntry?.call(key, value);
        onEvict?.call(value);
      }
    }
  }

  void clear() {
    for (final entry in _entries.entries) {
      onEvictEntry?.call(entry.key, entry.value);
      onEvict?.call(entry.value);
    }
    _entries.clear();
    _currentBytes = 0;
  }

  void _trim() {
    while (_currentBytes > maxBytes && _entries.isNotEmpty) {
      final oldestKey = _entries.keys.first;
      final oldest = _entries.remove(oldestKey)!;
      _currentBytes -= sizeOf(oldest);
      onEvictEntry?.call(oldestKey, oldest);
      onEvict?.call(oldest);
    }
  }
}
