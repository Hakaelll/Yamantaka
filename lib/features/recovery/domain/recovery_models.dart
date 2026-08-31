enum RecoveryProblemKind {
  interruptedOperation,
  inaccessibleDocument,
  integrityFailure,
  importFailure,
}

final class RecoveryProblem {
  const RecoveryProblem({
    required this.id,
    required this.kind,
    required this.code,
    required this.createdAt,
    this.canResume = false,
    this.canDiscard = true,
  });

  final String id;
  final RecoveryProblemKind kind;
  final String code;
  final DateTime createdAt;
  final bool canResume;
  final bool canDiscard;
}

final class AtomicSessionSnapshot {
  const AtomicSessionSnapshot({
    required this.workspaceId,
    required this.activeDocumentId,
    required this.savedAt,
    required this.clean,
  });

  final String? workspaceId;
  final String? activeDocumentId;
  final DateTime savedAt;
  final bool clean;

  Map<String, Object?> toJson() => {
    'workspaceId': workspaceId,
    'activeDocumentId': activeDocumentId,
    'savedAt': savedAt.toUtc().toIso8601String(),
    'clean': clean,
  };

  factory AtomicSessionSnapshot.fromJson(Map<String, Object?> json) =>
      AtomicSessionSnapshot(
        workspaceId: json['workspaceId'] as String?,
        activeDocumentId: json['activeDocumentId'] as String?,
        savedAt: DateTime.parse(json['savedAt'] as String),
        clean: json['clean'] as bool? ?? false,
      );
}
