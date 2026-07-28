import 'dart:io';

import 'package:window_manager/window_manager.dart';

abstract interface class WindowService {
  Future<bool> isFullScreen();
  Future<void> setFullScreen(bool value);
  Future<void> toggleFullScreen();
}

final class DesktopWindowService implements WindowService {
  const DesktopWindowService();

  bool get _supported =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  @override
  Future<bool> isFullScreen() async =>
      _supported ? windowManager.isFullScreen() : false;

  @override
  Future<void> setFullScreen(bool value) async {
    if (_supported) await windowManager.setFullScreen(value);
  }

  @override
  Future<void> toggleFullScreen() async {
    await setFullScreen(!await isFullScreen());
  }
}

final class FakeWindowService implements WindowService {
  bool fullScreen = false;

  @override
  Future<bool> isFullScreen() async => fullScreen;

  @override
  Future<void> setFullScreen(bool value) async => fullScreen = value;

  @override
  Future<void> toggleFullScreen() async => fullScreen = !fullScreen;
}
