import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';

/// Paints the book's editorial channel without adding physical space to the
/// PDF layout.  Keeping this as an ignored overlay preserves link hit testing
/// and makes the fit rectangle a property of the spread, not of its chrome.
final class ReaderBookSpine extends StatelessWidget {
  const ReaderBookSpine({super.key, this.width = 14});

  final double width;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final dark = Theme.of(context).brightness == Brightness.dark;
    return IgnorePointer(
      child: Center(
        child: SizedBox(
          width: width,
          height: double.infinity,
          child: DecoratedBox(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.centerLeft,
                end: Alignment.centerRight,
                colors: [
                  Colors.transparent,
                  (dark ? colors.bookSpineChannel : colors.bookSpineChannel)
                      .withValues(alpha: dark ? 0.18 : 0.10),
                  Colors.transparent,
                ],
                stops: const [0, 0.5, 1],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
