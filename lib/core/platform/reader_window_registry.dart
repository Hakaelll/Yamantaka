final class ReaderWindowHandle {
  const ReaderWindowHandle({required this.id, required this.documentId});

  final String id;
  final String documentId;
}

final class ReaderWindowRegistry {
  ReaderWindowRegistry({this.maxWindows = 2})
    : assert(maxWindows > 0),
      _nextId = 0;

  final int maxWindows;
  final Map<String, ReaderWindowHandle> _windows = {};
  int _nextId;

  List<ReaderWindowHandle> get windows => List.unmodifiable(_windows.values);

  ReaderWindowHandle? open(String documentId) {
    final existing = _windows.values
        .where((window) => window.documentId == documentId)
        .firstOrNull;
    if (existing != null) return existing;
    if (_windows.length >= maxWindows) return null;
    final handle = ReaderWindowHandle(
      id: 'reader-window-${++_nextId}',
      documentId: documentId,
    );
    _windows[handle.id] = handle;
    return handle;
  }

  void close(String id) => _windows.remove(id);
}
