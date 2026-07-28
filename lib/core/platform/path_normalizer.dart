import 'dart:io';

import 'package:path/path.dart' as p;

abstract final class PathNormalizer {
  static String normalize(String input, {bool? windows}) {
    final isWindows = windows ?? Platform.isWindows;
    var normalized = p.normalize(p.absolute(input.trim()));
    if (isWindows) {
      normalized = normalized.replaceAll('/', r'\').toLowerCase();
    }
    if (normalized.length > 1 &&
        (normalized.endsWith('/') || normalized.endsWith(r'\'))) {
      normalized = normalized.substring(0, normalized.length - 1);
    }
    return normalized;
  }
}
