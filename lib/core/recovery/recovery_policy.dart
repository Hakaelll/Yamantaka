enum RecoveryCause {
  cleanStartup,
  uncleanShutdown,
  migrationFailure,
  integrityFailure,
  diskSpace,
  nativeFailure,
}

final class RecoveryDecision {
  const RecoveryDecision({
    required this.safeMode,
    required this.actions,
    required this.reason,
  });

  final bool safeMode;
  final List<RecoveryAction> actions;
  final RecoveryCause reason;

  Map<String, Object> toJson() => {
    'safe_mode': safeMode,
    'reason': reason.name,
    'actions': [for (final action in actions) action.name],
  };
}

enum RecoveryAction {
  integrityCheck,
  createBackup,
  rebuildSearchIndex,
  retryFeature,
  openLibrary,
}

abstract final class RecoveryPolicy {
  static RecoveryDecision decide(RecoveryCause cause) => switch (cause) {
    RecoveryCause.cleanStartup => const RecoveryDecision(
      safeMode: false,
      reason: RecoveryCause.cleanStartup,
      actions: [RecoveryAction.openLibrary],
    ),
    RecoveryCause.uncleanShutdown => const RecoveryDecision(
      safeMode: true,
      reason: RecoveryCause.uncleanShutdown,
      actions: [
        RecoveryAction.integrityCheck,
        RecoveryAction.createBackup,
        RecoveryAction.openLibrary,
      ],
    ),
    RecoveryCause.migrationFailure => const RecoveryDecision(
      safeMode: true,
      reason: RecoveryCause.migrationFailure,
      actions: [
        RecoveryAction.integrityCheck,
        RecoveryAction.createBackup,
        RecoveryAction.openLibrary,
      ],
    ),
    RecoveryCause.integrityFailure => const RecoveryDecision(
      safeMode: true,
      reason: RecoveryCause.integrityFailure,
      actions: [
        RecoveryAction.integrityCheck,
        RecoveryAction.createBackup,
        RecoveryAction.openLibrary,
      ],
    ),
    RecoveryCause.diskSpace => const RecoveryDecision(
      safeMode: true,
      reason: RecoveryCause.diskSpace,
      actions: [RecoveryAction.retryFeature, RecoveryAction.openLibrary],
    ),
    RecoveryCause.nativeFailure => const RecoveryDecision(
      safeMode: true,
      reason: RecoveryCause.nativeFailure,
      actions: [RecoveryAction.retryFeature, RecoveryAction.openLibrary],
    ),
  };
}
