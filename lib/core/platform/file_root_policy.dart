import 'dart:io';

final class AuthorizedFileRoot {
  AuthorizedFileRoot(String path) : path = _normalized(path);

  final String path;

  bool contains(String candidate) {
    final root = _normalized(path);
    final value = _normalized(candidate);
    if (Platform.isWindows) {
      final left = root.toLowerCase();
      final right = value.toLowerCase();
      return right == left ||
          right.startsWith('$left${Platform.pathSeparator}');
    }
    return value == root ||
        value.startsWith(
          root.endsWith(Platform.pathSeparator)
              ? root
              : '$root${Platform.pathSeparator}',
        );
  }

  static String _normalized(String value) {
    final absolute = File(value).absolute.path;
    final separator = Platform.pathSeparator;
    if (absolute.length <= separator.length) return absolute;
    return absolute.endsWith(separator)
        ? absolute.substring(0, absolute.length - separator.length)
        : absolute;
  }
}

abstract final class FileRootPolicy {
  static bool mayRead(Iterable<AuthorizedFileRoot> roots, String path) =>
      roots.any((root) => root.contains(path));
}
