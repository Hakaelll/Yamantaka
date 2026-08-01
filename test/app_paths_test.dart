import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';

void main() {
  test('portable marker keeps data beside the executable', () async {
    final root = await Directory.systemTemp.createTemp('yamantaka-portable-');
    addTearDown(() => root.delete(recursive: true));
    final executable = File('${root.path}/Yamantaka.exe')..createSync();
    File('${root.path}/.yamantaka-portable').writeAsStringSync('1');
    final paths = await AppPaths.create(executablePath: executable.path);
    expect(paths.isPortable, isTrue);
    expect(
      paths.dataDirectory.path,
      '${root.path}${Platform.pathSeparator}data',
    );
  });
}
