import 'package:flutter/material.dart';

import '../../../core/i18n/app_localizations.dart';

enum SessionRecoveryChoice { restore, startClean }

final class SessionRecoveryDialog extends StatelessWidget {
  const SessionRecoveryDialog({super.key});

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      icon: const Icon(Icons.restore_page_outlined),
      title: Text(strings.get('recovery.sessionTitle')),
      content: Text(strings.get('recovery.sessionBody')),
      actions: [
        TextButton(
          onPressed: () =>
              Navigator.pop(context, SessionRecoveryChoice.startClean),
          child: Text(strings.get('recovery.startClean')),
        ),
        FilledButton.icon(
          onPressed: () =>
              Navigator.pop(context, SessionRecoveryChoice.restore),
          icon: const Icon(Icons.restore_rounded, size: 18),
          label: Text(strings.get('recovery.restoreSession')),
        ),
      ],
    );
  }
}
