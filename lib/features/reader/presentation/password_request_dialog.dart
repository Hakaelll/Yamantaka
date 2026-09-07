import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';

/// Requests a PDF password without exposing it beyond the dialog result.
///
/// The controller is owned and disposed by this route; callers must not log or
/// persist the returned value.
final class PasswordRequestDialog extends StatefulWidget {
  const PasswordRequestDialog({super.key, required this.wrongPassword});

  final bool wrongPassword;

  @override
  State<PasswordRequestDialog> createState() => _PasswordRequestDialogState();
}

final class _PasswordRequestDialogState extends State<PasswordRequestDialog> {
  final TextEditingController _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AppDialogFrame(
      icon: Icons.lock_outline_rounded,
      title: strings.get(
        widget.wrongPassword
            ? 'failure.wrongPassword.title'
            : 'failure.passwordRequired.title',
      ),
      description: strings.get('reader.passwordNotStored'),
      maxWidth: AppSizes.dialogCompactWidth,
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(strings.get('action.cancel')),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, _controller.text),
          child: Text(strings.get('action.open')),
        ),
      ],
      content: TextField(
        key: const Key('pdf-password-field'),
        controller: _controller,
        autofocus: true,
        obscureText: true,
        enableSuggestions: false,
        autocorrect: false,
        decoration: InputDecoration(
          labelText: strings.get('reader.password'),
          errorText: widget.wrongPassword
              ? strings.get('reader.wrongPasswordRetry')
              : null,
          errorMaxLines: 3,
        ),
        onSubmitted: (value) => Navigator.pop(context, value),
      ),
    );
  }
}
