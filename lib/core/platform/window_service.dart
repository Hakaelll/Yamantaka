import 'dart:io';

import 'package:window_manager/window_manager.dart';

abstract interface class WindowService {
  Future<bool> isFullScreen();
  Future<void> setFullScreen(bool value);
  Future<void> toggleFullScreen();
}

final class DesktopWindowService implements WindowService {
  bool _restoreMaximizedOnExit = false;

  bool get _supported =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  @override
  Future<bool> isFullScreen() async =>
      _supported ? windowManager.isFullScreen() : false;

  @override
  Future<void> setFullScreen(bool value) async {
    if (!_supported || await windowManager.isFullScreen() == value) return;
    if (Platform.isWindows && value) {
      // window_manager 0.5.2 keeps the native title bar when fullscreen is
      // requested from a maximized window. Enter from a restored state so the
      // plugin snapshots/removes the complete frame, then restore maximized
      // state after leaving fullscreen.
      _restoreMaximizedOnExit = await windowManager.isMaximized();
      if (_restoreMaximizedOnExit) await windowManager.unmaximize();
      await windowManager.setFullScreen(true);
      return;
    }
    await windowManager.setFullScreen(value);
    if (Platform.isWindows && !value && _restoreMaximizedOnExit) {
      _restoreMaximizedOnExit = false;
      await windowManager.maximize();
    }
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
