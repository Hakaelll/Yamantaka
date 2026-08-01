import 'dart:io';

import '../errors/app_failure.dart';

abstract final class LocalInputPolicy {
  static const allowedExternalSchemes = {'http', 'https', 'mailto'};

  static Uri? authorizedExternalUri(Uri? input) {
    if (input == null || !allowedExternalSchemes.contains(input.scheme)) {
      return null;
    }
    if (input.host.isEmpty && input.scheme != 'mailto') return null;
    return input;
  }

  static Uri requireAuthorizedExternalUri(Uri input) {
    final authorized = authorizedExternalUri(input);
    if (authorized == null) {
      throw const AppFailure(
        FailureKind.unauthorizedLink,
        'Este enlace no usa un esquema permitido.',
      );
    }
    return authorized;
  }

  static String boundedPassword(String value, {int maxLength = 512}) {
    if (value.isEmpty) {
      throw const AppFailure(
        FailureKind.invalidInput,
        'La contraseña no puede estar vacía.',
      );
    }
    if (value.length > maxLength) {
      throw const AppFailure(
        FailureKind.invalidInput,
        'La contraseña es demasiado larga.',
      );
    }
    return value;
  }

  static String boundedPath(String value, {int maxLength = 32768}) {
    final trimmed = value.trim().replaceAll('"', '');
    if (trimmed.isEmpty || trimmed.length > maxLength) {
      throw const AppFailure(
        FailureKind.invalidInput,
        'La ruta del archivo no es válida.',
      );
    }
    return File(trimmed).absolute.path;
  }
}
