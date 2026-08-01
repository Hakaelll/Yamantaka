import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/privacy/local_storage_service.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';

void main() {
  test('measures and clears rebuildable files without touching PDFs', () async {
    final root = await Directory.systemTemp.createTemp('yamantaka-storage-');
    addTearDown(() => root.delete(recursive: true));
    final cache = Directory('${root.path}/cache')..createSync();
    final covers = Directory('${root.path}/covers')..createSync();
    final backups = Directory('${root.path}/backups')..createSync();
    File('${cache.path}/render.bin').writeAsBytesSync([1, 2, 3]);
    File('${covers.path}/cover.bin').writeAsBytesSync([4, 5]);
    File('${backups.path}/old.json').writeAsBytesSync([6]);
    final pdf = File('${root.path}/book.pdf')..writeAsBytesSync([9]);
    final backupPdf = File('${backups.path}/user.pdf')..writeAsBytesSync([8]);
    final service = LocalStorageService(
      AppPaths(
        dataDirectory: root,
        cacheDirectory: cache,
        thumbnailDirectory: covers,
      ),
    );
    final entries = await service.measure();
    expect(
      entries.firstWhere((entry) => entry.category.name == 'renderCache').bytes,
      3,
    );
    await service.clearRebuildable();
    expect(pdf.existsSync(), isTrue);
    expect(File('${cache.path}/render.bin').existsSync(), isFalse);
    await service.clearBackups();
    expect(backupPdf.existsSync(), isTrue);
    expect(File('${backups.path}/old.json').existsSync(), isFalse);
  });
}
