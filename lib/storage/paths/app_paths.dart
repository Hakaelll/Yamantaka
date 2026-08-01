import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../../app/app_config.dart';

enum AppStorageMode { installed, portable }

final class AppPaths {
  const AppPaths({
    required this.dataDirectory,
    required this.cacheDirectory,
    required this.thumbnailDirectory,
    this.mode = AppStorageMode.installed,
  });

  final Directory dataDirectory;
  final Directory cacheDirectory;
  final Directory thumbnailDirectory;
  final AppStorageMode mode;

  bool get isPortable => mode == AppStorageMode.portable;

  String get databasePath =>
      p.join(dataDirectory.path, AppConfig.databaseFileName);

  static Future<AppPaths> create({String? executablePath}) async {
    final executable = executablePath ?? Platform.resolvedExecutable;
    final executableDirectory = File(executable).parent;
    final portableMarker = File(
      p.join(executableDirectory.path, '.yamantaka-portable'),
    );
    final portable = await portableMarker.exists();
    final data = portable
        ? Directory(p.join(executableDirectory.path, 'data'))
        : await getApplicationSupportDirectory();
    final systemCache = portable
        ? Directory(p.join(executableDirectory.path, 'cache'))
        : await getApplicationCacheDirectory();
    final cache = Directory(
      p.join(systemCache.path, AppConfig.cacheDirectoryName),
    );
    final thumbnails = Directory(
      p.join(systemCache.path, AppConfig.thumbnailDirectoryName),
    );
    await Future.wait([
      data.create(recursive: true),
      cache.create(recursive: true),
      thumbnails.create(recursive: true),
    ]);
    return AppPaths(
      dataDirectory: data,
      cacheDirectory: cache,
      thumbnailDirectory: thumbnails,
      mode: portable ? AppStorageMode.portable : AppStorageMode.installed,
    );
  }
}
