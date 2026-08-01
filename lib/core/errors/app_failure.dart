enum FailureKind {
  fileNotFound,
  accessDenied,
  corruptPdf,
  emptyPdf,
  passwordRequired,
  wrongPassword,
  unsupportedFormat,
  renderFailed,
  outOfMemory,
  databaseFailed,
  thumbnailFailed,
  restoreFailed,
  cancelled,
  invalidInput,
  unauthorizedLink,
  unknown,
}

/// Safe, typed error that can cross an application boundary.
final class AppFailure implements Exception {
  const AppFailure(this.kind, this.userMessage, {this.debugMessage});

  final FailureKind kind;
  final String userMessage;
  final String? debugMessage;

  @override
  String toString() => 'AppFailure(${kind.name})';
}
