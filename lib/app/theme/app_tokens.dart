import 'dart:math' as math;

import 'package:flutter/material.dart';

enum AppWindowClass { compact, medium, expanded }

abstract final class AppBreakpoints {
  static const double medium = 680;
  static const double expanded = 1100;

  static AppWindowClass classify(double width) {
    if (width < medium) return AppWindowClass.compact;
    if (width < expanded) return AppWindowClass.medium;
    return AppWindowClass.expanded;
  }
}

abstract final class AppSpacing {
  static const double space2 = 2;
  static const double space4 = 4;
  static const double space6 = 6;
  static const double space8 = 8;
  static const double space12 = 12;
  static const double space16 = 16;
  static const double space20 = 20;
  static const double space24 = 24;
  static const double space32 = 32;
  static const double space40 = 40;
  static const double space48 = 48;
  static const double space64 = 64;

  static double pagePadding(double width) =>
      switch (AppBreakpoints.classify(width)) {
        AppWindowClass.compact => space20,
        AppWindowClass.medium => space28,
        AppWindowClass.expanded => space40,
      };

  static const double space28 = 28;
}

abstract final class AppSizes {
  static const double topBarHeight = 46;
  static const double bottomControlHeight = 44;
  static const double sidePanelWidth = 320;
  static const double sidePanelMinWidth = 280;
  static const double sidePanelMaxWidth = 480;
  static const double compactSidePanelWidth = 304;
  static const double desktopControlExtent = 40;
  static const double touchControlExtent = 48;
  static const double dialogMaxWidth = 600;
  static const double commandPaletteMaxHeight = 520;
  static const double contentMaxWidth = 1000;
  static const double readerMinimumPageWidth = 340;
  static const double readerMaximumPageWidth = 920;
  static const double readerEdgeNavigation = 64;
  static const double readerSpreadGutter = 16;
}

abstract final class AppRadii {
  static const double indicator = 2;
  static const double control = 4;
  static const double field = 6;
  static const double dialog = 8;
  static const double touchSurface = 10;
}

abstract final class AppMotion {
  static const Duration instant = Duration.zero;
  static const Duration feedback = Duration(milliseconds: 80);
  static const Duration fast = Duration(milliseconds: 120);
  static const Duration standard = Duration(milliseconds: 160);
  static const Duration panel = Duration(milliseconds: 200);
  static const Duration controlsAutoHide = Duration(seconds: 3);

  static Duration whenAllowed(bool reduceMotion, Duration duration) =>
      reduceMotion ? instant : duration;
}

abstract final class ReaderLayoutPolicy {
  static bool shouldUseFacing({
    required double viewportWidth,
    required double viewportHeight,
    required double pageAspectRatio,
    double occupiedPanelWidth = 0,
    double visualMargin = AppSpacing.space32,
  }) {
    if (AppBreakpoints.classify(viewportWidth) != AppWindowClass.expanded) {
      return false;
    }
    final usableWidth = math.max(
      0,
      viewportWidth -
          occupiedPanelWidth -
          visualMargin * 2 -
          AppSizes.readerSpreadGutter,
    );
    final widthPerPage = usableWidth / 2;
    final usableHeight = math.max(0, viewportHeight - visualMargin * 2);
    final widthWhenFitToHeight = usableHeight * pageAspectRatio.clamp(0.2, 5.0);
    final legibleWidth = math.min(widthPerPage, widthWhenFitToHeight);
    return legibleWidth >= AppSizes.readerMinimumPageWidth;
  }
}

@immutable
final class AppColorTokens extends ThemeExtension<AppColorTokens> {
  const AppColorTokens({
    required this.appBackground,
    required this.surfacePrimary,
    required this.surfaceSecondary,
    required this.surfaceElevated,
    required this.surfaceSunken,
    required this.textPrimary,
    required this.textSecondary,
    required this.textMuted,
    required this.borderSubtle,
    required this.borderStrong,
    required this.accentPrimary,
    required this.accentHover,
    required this.accentPressed,
    required this.accentSoft,
    required this.selectionBackground,
    required this.focusRing,
    required this.danger,
    required this.warning,
    required this.success,
    required this.readerPageShadow,
  });

  final Color appBackground;
  final Color surfacePrimary;
  final Color surfaceSecondary;
  final Color surfaceElevated;
  final Color surfaceSunken;
  final Color textPrimary;
  final Color textSecondary;
  final Color textMuted;
  final Color borderSubtle;
  final Color borderStrong;
  final Color accentPrimary;
  final Color accentHover;
  final Color accentPressed;
  final Color accentSoft;
  final Color selectionBackground;
  final Color focusRing;
  final Color danger;
  final Color warning;
  final Color success;
  final Color readerPageShadow;

  static const light = AppColorTokens(
    appBackground: Color(0xFFF1F0EC),
    surfacePrimary: Color(0xFFF8F7F4),
    surfaceSecondary: Color(0xFFF4F2ED),
    surfaceElevated: Color(0xFFFDFCFA),
    surfaceSunken: Color(0xFFEAE8E2),
    textPrimary: Color(0xFF20211F),
    textSecondary: Color(0xFF62635E),
    textMuted: Color(0xFF85857E),
    borderSubtle: Color(0xFFDAD8D1),
    borderStrong: Color(0xFFC3C0B7),
    accentPrimary: Color(0xFF655D4C),
    accentHover: Color(0xFF574F40),
    accentPressed: Color(0xFF484236),
    accentSoft: Color(0xFFE7E2D7),
    selectionBackground: Color(0xFFE7E2D7),
    focusRing: Color(0xFF756B55),
    danger: Color(0xFFA23F38),
    warning: Color(0xFF8A6428),
    success: Color(0xFF4F7058),
    readerPageShadow: Color(0x24000000),
  );

  static const dark = AppColorTokens(
    appBackground: Color(0xFF171816),
    surfacePrimary: Color(0xFF1E1F1D),
    surfaceSecondary: Color(0xFF222320),
    surfaceElevated: Color(0xFF272825),
    surfaceSunken: Color(0xFF121311),
    textPrimary: Color(0xFFECECE7),
    textSecondary: Color(0xFFB5B5AE),
    textMuted: Color(0xFF85867F),
    borderSubtle: Color(0xFF343631),
    borderStrong: Color(0xFF4A4C46),
    accentPrimary: Color(0xFFB5AA8E),
    accentHover: Color(0xFFC4B99C),
    accentPressed: Color(0xFFD2C6A8),
    accentSoft: Color(0xFF343127),
    selectionBackground: Color(0xFF343127),
    focusRing: Color(0xFFC6B998),
    danger: Color(0xFFDF8077),
    warning: Color(0xFFD4AA69),
    success: Color(0xFF8BB293),
    readerPageShadow: Color(0x52000000),
  );

  @override
  AppColorTokens copyWith({
    Color? appBackground,
    Color? surfacePrimary,
    Color? surfaceSecondary,
    Color? surfaceElevated,
    Color? surfaceSunken,
    Color? textPrimary,
    Color? textSecondary,
    Color? textMuted,
    Color? borderSubtle,
    Color? borderStrong,
    Color? accentPrimary,
    Color? accentHover,
    Color? accentPressed,
    Color? accentSoft,
    Color? selectionBackground,
    Color? focusRing,
    Color? danger,
    Color? warning,
    Color? success,
    Color? readerPageShadow,
  }) => AppColorTokens(
    appBackground: appBackground ?? this.appBackground,
    surfacePrimary: surfacePrimary ?? this.surfacePrimary,
    surfaceSecondary: surfaceSecondary ?? this.surfaceSecondary,
    surfaceElevated: surfaceElevated ?? this.surfaceElevated,
    surfaceSunken: surfaceSunken ?? this.surfaceSunken,
    textPrimary: textPrimary ?? this.textPrimary,
    textSecondary: textSecondary ?? this.textSecondary,
    textMuted: textMuted ?? this.textMuted,
    borderSubtle: borderSubtle ?? this.borderSubtle,
    borderStrong: borderStrong ?? this.borderStrong,
    accentPrimary: accentPrimary ?? this.accentPrimary,
    accentHover: accentHover ?? this.accentHover,
    accentPressed: accentPressed ?? this.accentPressed,
    accentSoft: accentSoft ?? this.accentSoft,
    selectionBackground: selectionBackground ?? this.selectionBackground,
    focusRing: focusRing ?? this.focusRing,
    danger: danger ?? this.danger,
    warning: warning ?? this.warning,
    success: success ?? this.success,
    readerPageShadow: readerPageShadow ?? this.readerPageShadow,
  );

  @override
  AppColorTokens lerp(AppColorTokens? other, double t) {
    if (other == null) return this;
    return AppColorTokens(
      appBackground: Color.lerp(appBackground, other.appBackground, t)!,
      surfacePrimary: Color.lerp(surfacePrimary, other.surfacePrimary, t)!,
      surfaceSecondary: Color.lerp(
        surfaceSecondary,
        other.surfaceSecondary,
        t,
      )!,
      surfaceElevated: Color.lerp(surfaceElevated, other.surfaceElevated, t)!,
      surfaceSunken: Color.lerp(surfaceSunken, other.surfaceSunken, t)!,
      textPrimary: Color.lerp(textPrimary, other.textPrimary, t)!,
      textSecondary: Color.lerp(textSecondary, other.textSecondary, t)!,
      textMuted: Color.lerp(textMuted, other.textMuted, t)!,
      borderSubtle: Color.lerp(borderSubtle, other.borderSubtle, t)!,
      borderStrong: Color.lerp(borderStrong, other.borderStrong, t)!,
      accentPrimary: Color.lerp(accentPrimary, other.accentPrimary, t)!,
      accentHover: Color.lerp(accentHover, other.accentHover, t)!,
      accentPressed: Color.lerp(accentPressed, other.accentPressed, t)!,
      accentSoft: Color.lerp(accentSoft, other.accentSoft, t)!,
      selectionBackground: Color.lerp(
        selectionBackground,
        other.selectionBackground,
        t,
      )!,
      focusRing: Color.lerp(focusRing, other.focusRing, t)!,
      danger: Color.lerp(danger, other.danger, t)!,
      warning: Color.lerp(warning, other.warning, t)!,
      success: Color.lerp(success, other.success, t)!,
      readerPageShadow: Color.lerp(
        readerPageShadow,
        other.readerPageShadow,
        t,
      )!,
    );
  }
}

extension AppThemeContext on BuildContext {
  AppColorTokens get appColors {
    final theme = Theme.of(this);
    return theme.extension<AppColorTokens>() ??
        (theme.brightness == Brightness.dark
            ? AppColorTokens.dark
            : AppColorTokens.light);
  }

  AppWindowClass get windowClass =>
      AppBreakpoints.classify(MediaQuery.sizeOf(this).width);
}
