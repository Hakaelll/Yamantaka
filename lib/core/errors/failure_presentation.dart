import '../i18n/app_localizations.dart';
import 'app_failure.dart';

enum FailureAction { retry, chooseAnotherFile, enterPassword, goBack, dismiss }

final class FailurePresentation {
  const FailurePresentation({
    required this.title,
    required this.message,
    required this.action,
    this.isCancellation = false,
  });

  final String title;
  final String message;
  final FailureAction action;
  final bool isCancellation;
}

/// Complete, private-safe copy for each typed failure. The UI can choose the
/// action without exposing native exceptions or paths in a banner.
abstract final class FailurePresentations {
  static FailurePresentation forFailure(
    AppFailure? failure, {
    YamantakaLocalizations strings = const YamantakaLocalizations(
      AppLanguage.spanish,
    ),
  }) {
    final kind = failure?.kind;
    final key = kind?.name ?? 'none';
    final action = switch (kind) {
      null => FailureAction.goBack,
      FailureKind.fileNotFound ||
      FailureKind.corruptPdf ||
      FailureKind.emptyPdf ||
      FailureKind.unsupportedFormat => FailureAction.chooseAnotherFile,
      FailureKind.passwordRequired ||
      FailureKind.wrongPassword => FailureAction.enterPassword,
      FailureKind.thumbnailFailed ||
      FailureKind.cancelled ||
      FailureKind.unauthorizedLink => FailureAction.dismiss,
      FailureKind.restoreFailed => FailureAction.goBack,
      _ => FailureAction.retry,
    };
    return FailurePresentation(
      title: strings.get('failure.$key.title'),
      message: strings.get('failure.$key.message'),
      action: action,
      isCancellation: kind == FailureKind.cancelled,
    );
  }
}
