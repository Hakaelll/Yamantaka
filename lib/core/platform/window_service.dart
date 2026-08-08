import 'dart:io';

import 'package:flutter/services.dart';
import 'package:window_manager/window_manager.dart';

abstract interface class WindowService {
  Future<bool> isFullScreen();
  Future<void> setFullScreen(bool value);
  Future<void> toggleFullScreen();
}

final class DesktopWindowService implements WindowService {
  static const _windowsChannel = MethodChannel('yamantaka/window');

  bool get _supported =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  @override
  Future<bool> isFullScreen() async {
    if (!_supported) return false;
    if (Platform.isWindows) {
      return await _windowsChannel.invokeMethod<bool>('isFullScreen') ?? false;
    }
    return windowManager.isFullScreen();
  }

  @override
  Future<void> setFullScreen(bool value) async {
    if (!_supported || await isFullScreen() == value) return;
    if (Platform.isWindows) {
      final applied = await _windowsChannel.invokeMethod<bool>(
        'setFullScreen',
        {'value': value},
      );
      if (applied != value) {
        throw StateError('Windows rejected the fullscreen transition.');
      }
      return;
    }
    await windowManager.setFullScreen(value);
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
