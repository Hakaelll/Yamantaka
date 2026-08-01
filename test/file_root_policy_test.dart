import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/file_root_policy.dart';

void main() {
  test('authorized root does not accept a sibling with a shared prefix', () {
    final root = Directory.systemTemp.createTempSync('yamantaka-root-');
    addTearDown(() => root.deleteSync(recursive: true));
    final policy = AuthorizedFileRoot(root.path);
    expect(
      policy.contains('${root.path}${Platform.pathSeparator}book.pdf'),
      isTrue,
    );
    expect(policy.contains('${root.path}-other/book.pdf'), isFalse);
  });
}
