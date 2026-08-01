import 'package:flutter/material.dart';

import '../../../core/i18n/app_localizations.dart';

enum WelcomeAction { open, help, dismiss }

final class WelcomeDialog extends StatelessWidget {
  const WelcomeDialog({super.key});

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      icon: const Icon(Icons.menu_book_outlined),
      title: Text(strings.get('welcome.title')),
      content: Text(strings.get('welcome.body')),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context, WelcomeAction.help),
          child: Text(strings.get('welcome.help')),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context, WelcomeAction.dismiss),
          child: Text(strings.get('action.close')),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, WelcomeAction.open),
          child: Text(strings.get('welcome.open')),
        ),
      ],
    );
  }
}
