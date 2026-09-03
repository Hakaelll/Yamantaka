import '../../../core/errors/app_failure.dart';

/// One authoritative lifecycle for document opening and ownership.
///
/// Paths and passwords are deliberately excluded so the state can be included
/// in diagnostics and crash recovery without exposing private input.
enum DocumentLifecycleState {
  empty,
  opening,
  ready,
  cancelled,
  fileMissing,
  accessDenied,
  passwordRequired,
  wrongPassword,
  damaged,
  recoverableFailure,
  closed,
}

final class DocumentLifecycleSnapshot {
  const DocumentLifecycleSnapshot({
    required this.state,
    required this.generation,
    this.documentId,
    this.failureKind,
    this.passwordAttempt = 0,
  });

  const DocumentLifecycleSnapshot.empty()
    : state = DocumentLifecycleState.empty,
      generation = 0,
      documentId = null,
      failureKind = null,
      passwordAttempt = 0;

  final DocumentLifecycleState state;
  final int generation;
  final String? documentId;
  final FailureKind? failureKind;
  final int passwordAttempt;

  bool get isOpening => state == DocumentLifecycleState.opening;
  bool get canRetry => switch (state) {
    DocumentLifecycleState.fileMissing ||
    DocumentLifecycleState.accessDenied ||
    DocumentLifecycleState.passwordRequired ||
    DocumentLifecycleState.wrongPassword ||
    DocumentLifecycleState.damaged ||
    DocumentLifecycleState.recoverableFailure => true,
    _ => false,
  };

  factory DocumentLifecycleSnapshot.failure({
    required int generation,
    required AppFailure failure,
    String? documentId,
    int passwordAttempt = 0,
  }) => DocumentLifecycleSnapshot(
    state: switch (failure.kind) {
      FailureKind.fileNotFound => DocumentLifecycleState.fileMissing,
      FailureKind.accessDenied => DocumentLifecycleState.accessDenied,
      FailureKind.passwordRequired => DocumentLifecycleState.passwordRequired,
      FailureKind.wrongPassword => DocumentLifecycleState.wrongPassword,
      FailureKind.corruptPdf ||
      FailureKind.emptyPdf ||
      FailureKind.unsupportedFormat => DocumentLifecycleState.damaged,
      FailureKind.cancelled => DocumentLifecycleState.cancelled,
      _ => DocumentLifecycleState.recoverableFailure,
    },
    generation: generation,
    documentId: documentId,
    failureKind: failure.kind,
    passwordAttempt: passwordAttempt,
  );
}
