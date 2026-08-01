import 'dart:io';

import '../../storage/paths/app_paths.dart';
import 'storage_catalog.dart';

final class LocalStorageService {
  const LocalStorageService(this._paths);

  final AppPaths _paths;

  Future<List<StorageEntry>> measure() async => [
    StorageCatalog.describe(
      StorageCategory.database,
      await _fileBytes(File(_paths.databasePath)),
    ),
    StorageCatalog.describe(
      StorageCategory.covers,
      await _directoryBytes(_paths.thumbnailDirectory),
    ),
    StorageCatalog.describe(
      StorageCategory.renderCache,
      await _directoryBytes(_paths.cacheDirectory),
    ),
    StorageCatalog.describe(StorageCategory.searchIndex, 0),
    StorageCatalog.describe(StorageCategory.logs, 0),
    StorageCatalog.describe(
      StorageCategory.backups,
      await _directoryBytes(Directory('${_paths.dataDirectory.path}/backups')),
    ),
    StorageCatalog.describe(StorageCategory.preferences, 0),
  ];

  Future<void> clearRebuildable() async {
    await _deleteFiles(_paths.cacheDirectory);
    await _deleteFiles(_paths.thumbnailDirectory);
  }

  Future<void> clearBackups() =>
      _deleteFiles(Directory('${_paths.dataDirectory.path}/backups'));

  static Future<int> _fileBytes(File file) async {
    try {
      return await file.exists() ? file.length() : 0;
    } on FileSystemException {
      return 0;
    }
  }

  static Future<int> _directoryBytes(Directory directory) async {
    if (!await directory.exists()) return 0;
    var total = 0;
    await for (final entity in directory.list(
      recursive: true,
      followLinks: false,
    )) {
      if (entity is File) total += await _fileBytes(entity);
    }
    return total;
  }

  static Future<void> _deleteFiles(Directory directory) async {
    if (!await directory.exists()) return;
    await for (final entity in directory.list(
      recursive: true,
      followLinks: false,
    )) {
      if (entity is File && !entity.path.toLowerCase().endsWith('.pdf')) {
        try {
          await entity.delete();
        } on FileSystemException {
          // A locked cache entry is reported by the next measurement.
        }
      }
    }
  }
}
