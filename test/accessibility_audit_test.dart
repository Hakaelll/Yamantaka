import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_components.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/app/ui/yama_components.dart';

void main() {
  test('semantic text token pairs meet WCAG AA contrast', () {
    for (final tokens in [
      AppColorTokens.light,
      AppColorTokens.dark,
      AppColorTokens.library,
    ]) {
      expect(
        _contrast(tokens.textPrimary, tokens.appBackground),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(tokens.textSecondary, tokens.appBackground),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(tokens.textPrimary, tokens.surfacePrimary),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(tokens.accentPrimary, tokens.surfacePrimary),
        greaterThanOrEqualTo(3),
      );
      expect(
        _contrast(tokens.danger, tokens.surfacePrimary),
        greaterThanOrEqualTo(3),
      );
    }
  });

  testWidgets('core status surface has labels, contrast and keyboard focus', (
    tester,
  ) async {
    var activated = false;
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: Scaffold(
          body: AppStatusView(
            icon: Icons.menu_book_outlined,
            title: 'Biblioteca vacía',
            message: 'Abre un documento local para comenzar.',
            action: () => activated = true,
            actionLabel: 'Abrir PDF',
          ),
        ),
      ),
    );

    expect(tester, meetsGuideline(textContrastGuideline));
    expect(tester, meetsGuideline(labeledTapTargetGuideline));
    await tester.sendKeyEvent(LogicalKeyboardKey.tab);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    expect(activated, isTrue);
  });

  testWidgets('desktop icon states preserve the semantic priority order', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: Scaffold(
          body: YamaIconButton(
            icon: Icons.grid_view_rounded,
            semanticLabel: 'Vista de cuadrícula',
            selected: true,
            onPressed: () {},
          ),
        ),
      ),
    );
    final colors = AppColorTokens.light;
    final button = tester.widget<IconButton>(find.byType(IconButton));
    final style = button.style!;

    expect(
      style.backgroundColor!.resolve({WidgetState.hovered}),
      colors.surfaceActive,
      reason: 'selection remains explicit while the pointer hovers',
    );
    expect(
      style.backgroundColor!.resolve({WidgetState.pressed}),
      colors.surfacePressed,
      reason: 'pressed feedback takes priority over selection',
    );
    expect(
      style.side!.resolve({WidgetState.focused}),
      BorderSide(color: colors.focusRing, width: 2),
      reason: 'keyboard focus always gets the strongest outline',
    );
  });
}

double _contrast(Color foreground, Color background) {
  final lighter = foreground.computeLuminance() > background.computeLuminance()
      ? foreground
      : background;
  final darker = identical(lighter, foreground) ? background : foreground;
  return (lighter.computeLuminance() + 0.05) /
      (darker.computeLuminance() + 0.05);
}
