import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/app.dart';
import 'package:yamantaka/app/theme/app_density.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';

void main() {
  group('responsive window classes', () {
    test('use the documented non-overlapping breakpoints', () {
      expect(AppBreakpoints.classify(0), AppWindowClass.compact);
      expect(AppBreakpoints.classify(679), AppWindowClass.compact);
      expect(AppBreakpoints.classify(680), AppWindowClass.medium);
      expect(AppBreakpoints.classify(1099), AppWindowClass.medium);
      expect(AppBreakpoints.classify(1100), AppWindowClass.expanded);
      expect(AppBreakpoints.classify(1920), AppWindowClass.expanded);
    });

    test('page padding grows deliberately with the window class', () {
      expect(AppSpacing.pagePadding(679), AppSpacing.space20);
      expect(AppSpacing.pagePadding(680), AppSpacing.space28);
      expect(AppSpacing.pagePadding(1100), AppSpacing.space40);
    });
  });

  test('light and dark themes expose semantic color tokens', () {
    final light = AppTheme.light().extension<AppColorTokens>()!;
    final dark = AppTheme.dark().extension<AppColorTokens>()!;

    expect(light.appBackground, const Color(0xFFECEAE4));
    expect(light.accentPrimary, const Color(0xFF393B37));
    expect(dark.appBackground, const Color(0xFF111310));
    expect(dark.textPrimary, const Color(0xFFF0F1EB));
  });

  test('library theme keeps its spatial identity across app theme modes', () {
    final library = AppTheme.library().extension<AppColorTokens>()!;

    expect(library.libraryBackground, const Color(0xFF080808));
    expect(library.accentPrimary, const Color(0xFFF5F5F5));
    expect(library.librarySelectionAccent, const Color(0xFFB58AFF));
    expect(AppTheme.library().brightness, Brightness.dark);
  });

  test('editorial interaction tokens remain restrained and readable', () {
    expect(AppSizes.artifactSelectedScale, lessThanOrEqualTo(1.04));
    expect(
      AppSizes.artifactHoverScale,
      lessThan(AppSizes.artifactSelectedScale),
    );
    expect(AppShadow.artifactGlowSpread, 0);
    expect(AppSizes.settingsControlWidth, 240);
    expect(AppSizes.readerPanelDefault, AppSizes.sidePanelWidth);
  });

  test(
    'dark mode remains active even while safe mode is protecting startup',
    () {
      expect(
        resolveAppThemeMode(const AppSettings(darkMode: true), safeMode: true),
        ThemeMode.dark,
      );
      expect(
        resolveAppThemeMode(
          const AppSettings(darkMode: false),
          safeMode: false,
        ),
        ThemeMode.light,
      );
    },
  );

  test('the semantic style reaches every global application surface', () {
    final theme = AppTheme.light();
    final colors = theme.extension<AppColorTokens>()!;

    expect(theme.scaffoldBackgroundColor, colors.appBackground);
    expect(theme.appBarTheme.backgroundColor, colors.surfacePrimary);
    expect(theme.dialogTheme.backgroundColor, colors.surfaceElevated);
    expect(theme.popupMenuTheme.color, colors.surfaceElevated);
    expect(theme.bottomSheetTheme.backgroundColor, colors.surfaceElevated);
    expect(
      theme.menuTheme.style?.backgroundColor?.resolve({}),
      colors.surfaceElevated,
    );
    expect(theme.navigationRailTheme.backgroundColor, colors.surfaceSecondary);
    expect(theme.scrollbarTheme.thumbColor?.resolve({}), colors.borderStrong);
  });

  test(
    'compact density preserves control targets and tightens shared tokens',
    () {
      final theme = AppTheme.light(density: AppInterfaceDensity.compact);
      final density = theme.extension<AppDensityTokens>()!;
      final iconMinimum = theme.iconButtonTheme.style!.minimumSize!.resolve({});

      expect(theme.visualDensity.vertical, -1);
      expect(density.interfaceDensity, AppInterfaceDensity.compact);
      expect(
        density.listRowVerticalPadding,
        lessThan(AppDensityTokens.comfortable.listRowVerticalPadding),
      );
      expect(iconMinimum, const Size.square(AppSizes.desktopControlExtent));
    },
  );
}
