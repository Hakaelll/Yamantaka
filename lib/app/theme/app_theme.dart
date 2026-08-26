import 'package:flutter/material.dart';

import 'app_density.dart';
import 'app_tokens.dart';

abstract final class AppTheme {
  static ThemeData light({
    AppInterfaceDensity density = AppInterfaceDensity.comfortable,
    bool highContrast = false,
  }) => _build(
    brightness: Brightness.light,
    tokens: AppColorTokens.light,
    density: density,
    highContrast: highContrast,
  );

  static ThemeData dark({
    AppInterfaceDensity density = AppInterfaceDensity.comfortable,
    bool highContrast = false,
  }) => _build(
    brightness: Brightness.dark,
    tokens: AppColorTokens.dark,
    density: density,
    highContrast: highContrast,
  );

  static ThemeData library({
    AppInterfaceDensity density = AppInterfaceDensity.comfortable,
    bool highContrast = false,
  }) => _build(
    brightness: Brightness.dark,
    tokens: AppColorTokens.library,
    density: density,
    highContrast: highContrast,
  );

  static ThemeData _build({
    required Brightness brightness,
    required AppColorTokens tokens,
    required AppInterfaceDensity density,
    required bool highContrast,
  }) {
    final isDark = brightness == Brightness.dark;
    if (highContrast) {
      final flatSurface = isDark ? Colors.black : Colors.white;
      tokens = tokens.copyWith(
        appBackground: flatSurface,
        surfacePrimary: flatSurface,
        surfaceSecondary: flatSurface,
        surfaceElevated: flatSurface,
        surfaceSunken: flatSurface,
        textPrimary: isDark ? Colors.white : Colors.black,
        textSecondary: isDark ? Colors.white : const Color(0xFF202020),
        textMuted: isDark ? const Color(0xFFE0E0E0) : const Color(0xFF303030),
        borderSubtle: isDark ? Colors.white : Colors.black,
        borderStrong: isDark ? Colors.white : Colors.black,
        focusRing: isDark ? const Color(0xFFFFFF00) : const Color(0xFF0000A0),
        readerBackground: flatSurface,
        libraryBackground: flatSurface,
        settingsBackground: flatSurface,
        surfaceFloating: flatSurface,
        surfaceActive: flatSurface,
        surfaceHover: flatSurface,
        surfacePressed: flatSurface,
        iconNormal: isDark ? Colors.white : Colors.black,
        iconActive: isDark ? Colors.white : Colors.black,
        iconDisabled: isDark
            ? const Color(0xFFB0B0B0)
            : const Color(0xFF606060),
        separatorExceptional: isDark ? Colors.white : Colors.black,
        ambientShadow: Colors.transparent,
        pageShadow: Colors.transparent,
      );
    }
    final densityTokens = switch (density) {
      AppInterfaceDensity.comfortable => AppDensityTokens.comfortable,
      AppInterfaceDensity.compact => AppDensityTokens.compact,
    };
    final scheme = ColorScheme(
      brightness: brightness,
      primary: tokens.accentPrimary,
      onPrimary: isDark ? tokens.appBackground : tokens.paperWhite,
      secondary: tokens.accentPrimary,
      onSecondary: isDark ? tokens.appBackground : tokens.paperWhite,
      error: tokens.danger,
      onError: isDark ? tokens.appBackground : tokens.paperWhite,
      surface: tokens.surfacePrimary,
      onSurface: tokens.textPrimary,
      surfaceContainerLowest: tokens.appBackground,
      surfaceContainerLow: tokens.surfaceSecondary,
      surfaceContainer: tokens.surfacePrimary,
      surfaceContainerHigh: tokens.surfaceElevated,
      surfaceContainerHighest: tokens.surfaceSunken,
      outline: tokens.borderStrong,
      outlineVariant: tokens.borderSubtle,
    );
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: tokens.appBackground,
      canvasColor: tokens.appBackground,
      disabledColor: tokens.textMuted.withValues(alpha: 0.55),
      fontFamily: 'Segoe UI Variable',
      fontFamilyFallback: const ['Segoe UI', 'Roboto', 'sans-serif'],
      visualDensity: density == AppInterfaceDensity.compact
          ? const VisualDensity(horizontal: -1, vertical: -1)
          : VisualDensity.standard,
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      extensions: [tokens, densityTokens],
    );
    final textTheme = base.textTheme.copyWith(
      displaySmall: base.textTheme.displaySmall?.copyWith(
        fontSize: 30,
        height: 1.15,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.35,
      ),
      headlineSmall: base.textTheme.headlineSmall?.copyWith(
        fontSize: AppTypeScale.screenTitle,
        height: 1.2,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.2,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontSize: AppTypeScale.sectionTitle,
        height: 1.25,
        fontWeight: FontWeight.w600,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: AppTypeScale.documentTitle,
        height: 1.3,
        fontWeight: FontWeight.w600,
      ),
      titleSmall: base.textTheme.titleSmall?.copyWith(
        fontSize: AppTypeScale.contextualHeading,
        height: 1.3,
        fontWeight: FontWeight.w600,
      ),
      bodyLarge: base.textTheme.bodyLarge?.copyWith(
        fontSize: AppTypeScale.documentTitle,
        height: 1.35,
        fontWeight: FontWeight.w400,
      ),
      bodyMedium: base.textTheme.bodyMedium?.copyWith(
        fontSize: AppTypeScale.body,
        height: 1.35,
        fontWeight: FontWeight.w400,
      ),
      bodySmall: base.textTheme.bodySmall?.copyWith(
        color: tokens.textSecondary,
        fontSize: AppTypeScale.metadata,
        height: 1.35,
        fontWeight: FontWeight.w400,
      ),
      labelLarge: base.textTheme.labelLarge?.copyWith(
        fontSize: AppTypeScale.control,
        height: 1.3,
        fontWeight: FontWeight.w500,
      ),
      labelMedium: base.textTheme.labelMedium?.copyWith(
        color: tokens.textSecondary,
        fontSize: AppTypeScale.figure,
        height: 1.3,
        fontWeight: FontWeight.w500,
        fontFeatures: AppTypeScale.tabularFigures,
      ),
      labelSmall: base.textTheme.labelSmall?.copyWith(
        color: tokens.textMuted,
        fontSize: AppTypeScale.auxiliary,
        height: 1.35,
        fontWeight: FontWeight.w500,
        fontFeatures: AppTypeScale.tabularFigures,
      ),
    );
    final focusSide = WidgetStateProperty.resolveWith<BorderSide>((states) {
      if (states.contains(WidgetState.focused)) {
        return BorderSide(color: tokens.focusRing, width: 2);
      }
      return BorderSide(color: Colors.transparent);
    });
    final overlay = WidgetStateProperty.resolveWith<Color?>((states) {
      if (states.contains(WidgetState.pressed)) {
        return tokens.accentPressed.withValues(alpha: 0.14);
      }
      if (states.contains(WidgetState.hovered)) {
        return tokens.accentHover.withValues(alpha: 0.10);
      }
      if (states.contains(WidgetState.focused)) {
        return tokens.accentSoft.withValues(alpha: 0.8);
      }
      return null;
    });
    final controlShape = WidgetStatePropertyAll(
      RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadii.control),
      ),
    );
    const iconShape = WidgetStatePropertyAll(CircleBorder());

    return base.copyWith(
      textTheme: textTheme,
      primaryTextTheme: textTheme,
      appBarTheme: AppBarTheme(
        backgroundColor: tokens.surfacePrimary,
        surfaceTintColor: Colors.transparent,
        foregroundColor: tokens.textPrimary,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        toolbarHeight: AppSizes.topBarHeight,
        shape: Border(bottom: BorderSide(color: tokens.borderSubtle)),
        titleTextStyle: textTheme.titleSmall?.copyWith(
          color: tokens.textPrimary,
        ),
      ),
      dividerColor: tokens.borderSubtle,
      dividerTheme: DividerThemeData(
        color: tokens.borderSubtle,
        thickness: 1,
        space: 1,
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        color: tokens.surfacePrimary,
        surfaceTintColor: Colors.transparent,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: tokens.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
      ),
      dialogTheme: DialogThemeData(
        backgroundColor: tokens.surfaceElevated,
        surfaceTintColor: Colors.transparent,
        elevation: AppElevation.popup,
        shadowColor: tokens.ambientShadow,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: tokens.borderStrong),
          borderRadius: BorderRadius.circular(AppRadii.dialog),
        ),
        titleTextStyle: textTheme.titleLarge?.copyWith(
          color: tokens.textPrimary,
        ),
        contentTextStyle: textTheme.bodyMedium?.copyWith(
          color: tokens.textSecondary,
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: tokens.surfaceElevated,
        isDense: true,
        contentPadding: EdgeInsets.symmetric(
          horizontal: AppSpacing.space12,
          vertical: density == AppInterfaceDensity.compact
              ? AppSpacing.space8
              : AppSpacing.space12,
        ),
        hintStyle: textTheme.bodyMedium?.copyWith(color: tokens.textMuted),
        labelStyle: textTheme.bodyMedium?.copyWith(color: tokens.textSecondary),
        border: _inputBorder(tokens.borderStrong),
        enabledBorder: _inputBorder(tokens.borderSubtle),
        focusedBorder: _inputBorder(tokens.focusRing, width: 2),
        errorBorder: _inputBorder(tokens.danger),
        focusedErrorBorder: _inputBorder(tokens.danger, width: 2),
      ),
      focusColor: tokens.accentSoft,
      hoverColor: tokens.accentSoft.withValues(alpha: 0.65),
      splashFactory: InkRipple.splashFactory,
      tooltipTheme: TooltipThemeData(
        waitDuration: const Duration(milliseconds: 450),
        decoration: BoxDecoration(
          color: tokens.textPrimary,
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
        textStyle: textTheme.labelMedium?.copyWith(color: tokens.appBackground),
      ),
      progressIndicatorTheme: ProgressIndicatorThemeData(
        color: tokens.accentPrimary,
        linearTrackColor: tokens.surfaceSunken,
      ),
      iconTheme: IconThemeData(color: tokens.textSecondary, size: 20),
      iconButtonTheme: IconButtonThemeData(
        style: ButtonStyle(
          minimumSize: const WidgetStatePropertyAll(
            Size.square(AppSizes.desktopControlExtent),
          ),
          maximumSize: const WidgetStatePropertyAll(
            Size.square(AppSizes.touchControlExtent),
          ),
          foregroundColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.disabled)) {
              return tokens.textMuted.withValues(alpha: 0.55);
            }
            if (states.contains(WidgetState.hovered)) {
              return tokens.textPrimary;
            }
            return tokens.textSecondary;
          }),
          overlayColor: overlay,
          side: focusSide,
          shape: iconShape,
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: ButtonStyle(
          minimumSize: const WidgetStatePropertyAll(
            Size(40, AppSizes.desktopControlExtent),
          ),
          foregroundColor: WidgetStatePropertyAll(tokens.accentPrimary),
          overlayColor: overlay,
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(AppRadii.control),
            ),
          ),
          side: focusSide,
          textStyle: WidgetStatePropertyAll(textTheme.labelLarge),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: ButtonStyle(
          minimumSize: const WidgetStatePropertyAll(
            Size(40, AppSizes.desktopControlExtent),
          ),
          backgroundColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.disabled)) {
              return tokens.surfaceSunken;
            }
            if (states.contains(WidgetState.pressed)) {
              return tokens.accentPressed;
            }
            if (states.contains(WidgetState.hovered)) return tokens.accentHover;
            return tokens.accentPrimary;
          }),
          foregroundColor: WidgetStatePropertyAll(
            isDark ? tokens.appBackground : tokens.paperWhite,
          ),
          shape: controlShape,
          side: focusSide,
          textStyle: WidgetStatePropertyAll(textTheme.labelLarge),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: ButtonStyle(
          minimumSize: const WidgetStatePropertyAll(
            Size(40, AppSizes.desktopControlExtent),
          ),
          foregroundColor: WidgetStatePropertyAll(tokens.textPrimary),
          overlayColor: overlay,
          side: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.focused)) {
              return BorderSide(color: tokens.focusRing, width: 2);
            }
            return BorderSide(color: tokens.borderStrong);
          }),
          shape: controlShape,
          textStyle: WidgetStatePropertyAll(textTheme.labelLarge),
        ),
      ),
      listTileTheme: ListTileThemeData(
        minTileHeight: 48,
        contentPadding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space12,
        ),
        iconColor: tokens.textSecondary,
        textColor: tokens.textPrimary,
        selectedColor: tokens.textPrimary,
        selectedTileColor: tokens.selectionBackground,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
      ),
      popupMenuTheme: PopupMenuThemeData(
        color: tokens.surfaceElevated,
        surfaceTintColor: Colors.transparent,
        elevation: AppElevation.overlay,
        menuPadding: const EdgeInsets.symmetric(vertical: AppSpacing.space6),
        shadowColor: tokens.ambientShadow,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: tokens.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.field),
        ),
        textStyle: textTheme.bodyMedium,
        iconColor: tokens.iconNormal,
      ),
      menuTheme: MenuThemeData(
        style: MenuStyle(
          backgroundColor: WidgetStatePropertyAll(tokens.surfaceElevated),
          surfaceTintColor: const WidgetStatePropertyAll(Colors.transparent),
          elevation: const WidgetStatePropertyAll(AppElevation.overlay),
          shadowColor: WidgetStatePropertyAll(tokens.ambientShadow),
          side: WidgetStatePropertyAll(BorderSide(color: tokens.borderSubtle)),
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(AppRadii.field),
            ),
          ),
          padding: const WidgetStatePropertyAll(
            EdgeInsets.symmetric(vertical: AppSpacing.space6),
          ),
        ),
      ),
      dropdownMenuTheme: DropdownMenuThemeData(
        textStyle: textTheme.bodyMedium,
        menuStyle: MenuStyle(
          backgroundColor: WidgetStatePropertyAll(tokens.surfaceElevated),
          surfaceTintColor: const WidgetStatePropertyAll(Colors.transparent),
          side: WidgetStatePropertyAll(BorderSide(color: tokens.borderSubtle)),
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(AppRadii.field),
            ),
          ),
        ),
      ),
      bottomSheetTheme: BottomSheetThemeData(
        backgroundColor: tokens.surfaceElevated,
        modalBackgroundColor: tokens.surfaceElevated,
        surfaceTintColor: Colors.transparent,
        shadowColor: tokens.ambientShadow,
        elevation: AppElevation.popup,
        modalElevation: AppElevation.popup,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: tokens.borderSubtle),
          borderRadius: const BorderRadius.vertical(
            top: Radius.circular(AppRadii.dialog),
          ),
        ),
      ),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: tokens.surfaceSecondary,
        indicatorColor: tokens.selectionBackground,
        indicatorShape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
        selectedIconTheme: IconThemeData(color: tokens.textPrimary, size: 20),
        unselectedIconTheme: IconThemeData(
          color: tokens.textSecondary,
          size: 20,
        ),
        selectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: tokens.textPrimary,
        ),
        unselectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: tokens.textSecondary,
        ),
      ),
      chipTheme: base.chipTheme.copyWith(
        backgroundColor: tokens.surfaceSecondary,
        selectedColor: tokens.selectionBackground,
        disabledColor: tokens.surfaceSunken,
        side: BorderSide(color: tokens.borderSubtle),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
        labelStyle: textTheme.labelMedium,
        secondaryLabelStyle: textTheme.labelMedium?.copyWith(
          color: tokens.textPrimary,
        ),
        padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space6),
      ),
      checkboxTheme: CheckboxThemeData(
        fillColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return tokens.accentPrimary;
          }
          return Colors.transparent;
        }),
        checkColor: WidgetStatePropertyAll(
          isDark ? tokens.appBackground : tokens.paperWhite,
        ),
        side: BorderSide(color: tokens.borderStrong),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppRadii.indicator),
        ),
      ),
      radioTheme: RadioThemeData(
        fillColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected)
              ? tokens.accentPrimary
              : tokens.textSecondary,
        ),
      ),
      scrollbarTheme: ScrollbarThemeData(
        thumbColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.hovered)
              ? tokens.textSecondary
              : tokens.borderStrong,
        ),
        trackColor: WidgetStatePropertyAll(
          tokens.surfaceSunken.withValues(alpha: 0.45),
        ),
        thickness: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.hovered) ? 8 : 6,
        ),
        radius: const Radius.circular(AppRadii.indicator),
        crossAxisMargin: AppSpacing.space4,
        mainAxisMargin: AppSpacing.space8,
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        backgroundColor: tokens.textPrimary,
        contentTextStyle: textTheme.bodyMedium?.copyWith(
          color: tokens.appBackground,
        ),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppRadii.field),
        ),
      ),
      sliderTheme: base.sliderTheme.copyWith(
        activeTrackColor: tokens.accentPrimary,
        inactiveTrackColor: tokens.surfaceSunken,
        thumbColor: tokens.accentPrimary,
        overlayColor: tokens.accentSoft,
        trackHeight: 3,
      ),
      switchTheme: SwitchThemeData(
        thumbColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return isDark ? tokens.appBackground : Colors.white;
          }
          return tokens.textMuted;
        }),
        trackColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return tokens.accentPrimary;
          }
          return tokens.surfaceSunken;
        }),
        trackOutlineColor: WidgetStatePropertyAll(tokens.borderStrong),
      ),
      segmentedButtonTheme: SegmentedButtonThemeData(
        style: ButtonStyle(
          minimumSize: const WidgetStatePropertyAll(Size(38, 34)),
          backgroundColor: WidgetStateProperty.resolveWith(
            (states) => states.contains(WidgetState.selected)
                ? tokens.accentSoft
                : Colors.transparent,
          ),
          foregroundColor: WidgetStateProperty.resolveWith(
            (states) => states.contains(WidgetState.selected)
                ? tokens.textPrimary
                : tokens.textSecondary,
          ),
          side: WidgetStatePropertyAll(BorderSide(color: tokens.borderSubtle)),
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(AppRadii.capsule),
            ),
          ),
        ),
      ),
    );
  }

  static OutlineInputBorder _inputBorder(Color color, {double width = 1}) =>
      OutlineInputBorder(
        borderRadius: BorderRadius.circular(AppRadii.field),
        borderSide: BorderSide(color: color, width: width),
      );
}
