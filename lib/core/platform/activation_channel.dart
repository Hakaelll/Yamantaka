import 'package:flutter/services.dart';

import 'single_instance_protocol.dart';

abstract final class ActivationChannel {
  static const MethodChannel _channel = MethodChannel('yamantaka/activation');

  static Future<void> install(
    Future<void> Function(List<String> paths) onPaths,
  ) async {
    _channel.setMethodCallHandler((call) async {
      if (call.method != 'open') return;
      final message = call.arguments is String ? call.arguments as String : '';
      final request = SingleInstanceProtocol.decode(message);
      if (request.pdfPaths.isNotEmpty) await onPaths(request.pdfPaths);
    });
  }

  static Future<void> uninstall() async {
    _channel.setMethodCallHandler(null);
  }
}
