import 'dart:developer' as developer;

enum LogLevel { debug, info, warning, error }

/// Optional observer used by tests and future diagnostic exporters.
typedef AppLogSink =
    void Function(
      LogLevel level,
      String message,
      Object? error,
      StackTrace? stackTrace,
    );

/// Small local logger. Callers must never pass document text or passwords.
final class AppLogger {
  AppLogger({this.minimumLevel = LogLevel.info, this.sink});

  final LogLevel minimumLevel;
  final AppLogSink? sink;

  void debug(String message) => _write(LogLevel.debug, message);
  void info(String message) => _write(LogLevel.info, message);
  void warning(String message, [Object? error]) =>
      _write(LogLevel.warning, message, error);
  void error(String message, [Object? error, StackTrace? stackTrace]) =>
      _write(LogLevel.error, message, error, stackTrace);

  void _write(
    LogLevel level,
    String message, [
    Object? error,
    StackTrace? stackTrace,
  ]) {
    if (level.index < minimumLevel.index) return;
    sink?.call(level, message, error, stackTrace);
    developer.log(
      message,
      name: 'yamantaka',
      level: _developerLevel(level),
      error: error,
      stackTrace: stackTrace,
    );
  }

  int _developerLevel(LogLevel level) => switch (level) {
    LogLevel.debug => 500,
    LogLevel.info => 800,
    LogLevel.warning => 900,
    LogLevel.error => 1000,
  };
}
