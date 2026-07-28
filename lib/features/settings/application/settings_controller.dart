import 'package:flutter/foundation.dart';

import '../../../core/logging/app_logger.dart';
import '../../../storage/repositories/app_settings_repository.dart';

final class SettingsController extends ChangeNotifier {
  SettingsController(this._repository, this._logger);

  final AppSettingsRepository _repository;
  final AppLogger _logger;
  AppSettings settings = const AppSettings();
  bool isLoaded = false;

  Future<void> load() async {
    settings = await _repository.load();
    isLoaded = true;
    notifyListeners();
  }

  Future<void> update(AppSettings next) async {
    settings = next;
    notifyListeners();
    try {
      await _repository.save(next);
    } on Object catch (error, stackTrace) {
      _logger.error('Global settings save failed.', error, stackTrace);
    }
  }

  Future<void> clear() async {
    await _repository.clear();
    settings = const AppSettings();
    notifyListeners();
  }

  Future<void> saveWindowBounds({
    required double x,
    required double y,
    required double width,
    required double height,
  }) => update(
    settings.copyWith(
      windowX: x,
      windowY: y,
      windowWidth: width,
      windowHeight: height,
    ),
  );
}
