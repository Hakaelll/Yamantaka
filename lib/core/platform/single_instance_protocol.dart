import 'dart:convert';
import 'dart:io';

final class ActivationRequest {
  const ActivationRequest(this.pdfPaths);

  final List<String> pdfPaths;

  Map<String, Object> toJson() => {'pdf_paths': pdfPaths};
}

abstract final class SingleInstanceProtocol {
  static const maxPaths = 12;
  static const maxMessageBytes = 64 * 1024;

  static ActivationRequest fromArguments(Iterable<String> arguments) {
    final paths = <String>[];
    for (final raw in arguments) {
      final value = raw.replaceAll('"', '').trim();
      if (value.isEmpty || !value.toLowerCase().endsWith('.pdf')) continue;
      final path = File(value).absolute.path;
      if (!paths.any((existing) => _samePath(existing, path))) paths.add(path);
      if (paths.length == maxPaths) break;
    }
    return ActivationRequest(List.unmodifiable(paths));
  }

  static String encode(ActivationRequest request) {
    final value = request.pdfPaths.join('\n');
    if (utf8.encode(value).length > maxMessageBytes) {
      throw StateError('Activation request exceeds the local message limit.');
    }
    return value;
  }

  static ActivationRequest decode(String message) =>
      fromArguments(message.split('\n'));

  static bool _samePath(String left, String right) => Platform.isWindows
      ? left.toLowerCase() == right.toLowerCase()
      : left == right;
}
