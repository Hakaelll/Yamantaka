import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/recovery/recovery_policy.dart';

void main() {
  test('unclean shutdown enters reversible safe mode', () {
    final decision = RecoveryPolicy.decide(RecoveryCause.uncleanShutdown);
    expect(decision.safeMode, isTrue);
    expect(decision.actions, contains(RecoveryAction.createBackup));
    expect(decision.actions, contains(RecoveryAction.openLibrary));
  });

  test('migration failures keep their cause in diagnostics', () {
    expect(
      RecoveryPolicy.decide(RecoveryCause.migrationFailure).reason,
      RecoveryCause.migrationFailure,
    );
  });

  test('disk-space failures remain distinguishable from native failures', () {
    expect(
      RecoveryPolicy.decide(RecoveryCause.diskSpace).reason,
      RecoveryCause.diskSpace,
    );
    expect(
      RecoveryPolicy.decide(RecoveryCause.nativeFailure).reason,
      RecoveryCause.nativeFailure,
    );
  });
}
