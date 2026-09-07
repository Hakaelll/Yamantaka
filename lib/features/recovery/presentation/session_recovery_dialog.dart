import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';

enum SessionRecoveryChoice { restore, startClean }

final class SessionRecoveryDialog extends StatelessWidget {
  const SessionRecoveryDialog({super.key});

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AppDialogFrame(
      icon: Icons.restore_page_outlined,
      title: strings.get('recovery.sessionTitle'),
      description: strings.get('recovery.sessionBody'),
      maxWidth: AppSizes.dialogCompactWidth,
      actions: [
        TextButton(
          onPressed: () =>
              Navigator.pop(context, SessionRecoveryChoice.startClean),
          child: Text(strings.get('recovery.startClean')),
        ),
        FilledButton.icon(
          onPressed: () =>
              Navigator.pop(context, SessionRecoveryChoice.restore),
          icon: const Icon(Icons.restore_rounded, size: AppIconSizes.standard),
          label: Text(strings.get('recovery.restoreSession')),
        ),
      ],
    );
  }
}
