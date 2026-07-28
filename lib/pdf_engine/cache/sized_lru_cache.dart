import 'dart:collection';

final class SizedLruCache<K, V extends Object> {
  SizedLruCache({required this.maxBytes, required this.sizeOf, this.onEvict})
    : assert(maxBytes >= 0);

  final int maxBytes;
  final int Function(V value) sizeOf;
  final void Function(V value)? onEvict;
  final LinkedHashMap<K, V> _entries = LinkedHashMap<K, V>();
  int _currentBytes = 0;

  int get currentBytes => _currentBytes;
  int get length => _entries.length;

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
        onEvict?.call(value);
      }
    }
  }

  void clear() {
    if (onEvict != null) {
      for (final value in _entries.values) {
        onEvict!(value);
      }
    }
    _entries.clear();
    _currentBytes = 0;
  }

  void _trim() {
    while (_currentBytes > maxBytes && _entries.isNotEmpty) {
      final oldestKey = _entries.keys.first;
      final oldest = _entries.remove(oldestKey)!;
      _currentBytes -= sizeOf(oldest);
      onEvict?.call(oldest);
    }
  }
}
