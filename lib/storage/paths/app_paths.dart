import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../../app/app_config.dart';

final class AppPaths {
  const AppPaths({
    required this.dataDirectory,
    required this.cacheDirectory,
    required this.thumbnailDirectory,
  });

  final Directory dataDirectory;
  final Directory cacheDirectory;
  final Directory thumbnailDirectory;

  String get databasePath =>
      p.join(dataDirectory.path, AppConfig.databaseFileName);

  static Future<AppPaths> create() async {
    final data = await getApplicationSupportDirectory();
    final systemCache = await getApplicationCacheDirectory();
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
    );
  }
}
