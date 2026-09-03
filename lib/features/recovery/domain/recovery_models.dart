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
    this.canRollback = false,
    this.canDiscard = true,
    this.phase,
    this.interruptionReason,
  });

  final String id;
  final RecoveryProblemKind kind;
  final String code;
  final DateTime createdAt;
  final bool canResume;
  final bool canRollback;
  final bool canDiscard;
  final String? phase;
  final String? interruptionReason;
}

final class AtomicSessionSnapshot {
  const AtomicSessionSnapshot({
    required this.workspaceId,
    required this.activeDocumentId,
    required this.savedAt,
    required this.clean,
    this.activeDocumentFingerprint,
    this.pageIndex,
    this.pageCount,
    this.quarantinedDocumentId,
    this.lifecycleState,
  });

  final String? workspaceId;
  final String? activeDocumentId;
  final String? activeDocumentFingerprint;
  final int? pageIndex;
  final int? pageCount;
  final String? quarantinedDocumentId;
  final String? lifecycleState;
  final DateTime savedAt;
  final bool clean;

  Map<String, Object?> toJson() => {
    'workspaceId': workspaceId,
    'activeDocumentId': activeDocumentId,
    'activeDocumentFingerprint': activeDocumentFingerprint,
    'pageIndex': pageIndex,
    'pageCount': pageCount,
    'quarantinedDocumentId': quarantinedDocumentId,
    'lifecycleState': lifecycleState,
    'savedAt': savedAt.toUtc().toIso8601String(),
    'clean': clean,
  };

  factory AtomicSessionSnapshot.fromJson(Map<String, Object?> json) =>
      AtomicSessionSnapshot(
        workspaceId: json['workspaceId'] as String?,
        activeDocumentId: json['activeDocumentId'] as String?,
        activeDocumentFingerprint: json['activeDocumentFingerprint'] as String?,
        pageIndex: json['pageIndex'] as int?,
        pageCount: json['pageCount'] as int?,
        quarantinedDocumentId: json['quarantinedDocumentId'] as String?,
        lifecycleState: json['lifecycleState'] as String?,
        savedAt: DateTime.parse(json['savedAt'] as String),
        clean: json['clean'] as bool? ?? false,
      );
}
