import 'package:flutter/material.dart';

enum AppWindowClass { compact, medium, expanded }

/// The three visual motion profiles exposed by the redesign.  The legacy
/// boolean preferences are still read and written by the settings repository,
/// but new UI surfaces consume this single semantic value.
enum AppMotionPreference { full, reduced, none }

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
  static const double topBarHeight = 64;
  static const double libraryHeaderHeight = 72;
  static const double bottomControlHeight = 48;
  static const double sidePanelWidth = 344;
  static const double sidePanelMinWidth = 288;
  static const double sidePanelMaxWidth = 480;
  static const double compactSidePanelWidth = 360;
  static const double desktopControlExtent = 40;
  static const double touchControlExtent = 48;
  static const double dialogMaxWidth = 600;
  static const double commandPaletteMaxHeight = 520;
  static const double readerEdgeNavigation = 64;
  static const double readerSpreadGutter = 2;
  static const double topRevealZone = 24;
  static const double readerPanelDefault = sidePanelWidth;
  static const double settingsNavigationWidth = 220;
  static const double settingsPreviewWidth = 300;
  static const double settingsContentMaxWidth = 760;
  static const double settingsControlWidth = 240;
  static const double commandPaletteRowHeight = 52;
  static const double artifactSelectedScale = 1.035;
  static const double artifactHoverScale = 1.015;
  static const double artifactSelectedLift = 6;
  static const double artifactHoverLift = 2;
}

abstract final class AppRadii {
  static const double indicator = 2;
  static const double page = 3;
  static const double control = 10;
  static const double field = 12;
  static const double floating = 16;
  static const double panel = 18;
  static const double dialog = 20;
  static const double touchSurface = 14;
  static const double capsule = 999;
}

abstract final class AppElevation {
  static const double flat = 0;
  static const double control = 2;
  static const double raised = 3;
  static const double overlay = 6;
  static const double popup = 8;
}

abstract final class AppShadow {
  static const double readerPageBlur = 10;
  static const double readerGrayPageBlur = 6;
  static const double readerPanelBlur = 10;
  static const double floatingControlBlur = 10;
  static const double floatingControlOffset = 3;
  static const double artifactRestBlur = 8;
  static const double artifactSelectedBlur = 14;
  static const double artifactRestOffset = 4;
  static const double artifactSelectedOffset = 7;
  static const double artifactGlowBlur = 18;
  static const double artifactGlowSpread = 0;
}

abstract final class AppMotion {
  static const Duration instant = Duration.zero;
  static const Duration feedback = Duration(milliseconds: 80);
  static const Duration fast = Duration(milliseconds: 120);
  static const Duration standard = Duration(milliseconds: 160);
  static const Duration panel = Duration(milliseconds: 200);
  static const Duration controlsAutoHide = Duration(seconds: 3);
  static const Duration chromeReveal = Duration(milliseconds: 120);
  static const Duration chromeHide = Duration(milliseconds: 100);
  static const Duration searchDebounce = standard;
  static const Duration persistenceDebounce = panel;
  static const Duration fullscreenControlsAutoHide = Duration(
    milliseconds: 2500,
  );

  static bool isDisabled(AppMotionPreference preference) =>
      preference == AppMotionPreference.none;

  static bool isReduced(AppMotionPreference preference) =>
      preference != AppMotionPreference.full;

  static Duration whenAllowed(bool reduceMotion, Duration duration) =>
      reduceMotion ? instant : duration;
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
    required this.librarySelectionAccent,
    this.readerBackground = const Color(0xFFE7E4DC),
    this.libraryBackground = const Color(0xFFEDEBE5),
    this.settingsBackground = const Color(0xFFF1EFE9),
    this.surfaceFloating = const Color(0xFFFBF9F3),
    this.surfaceActive = const Color(0xFFDCDAD3),
    this.surfaceHover = const Color(0xFFE4E2DB),
    this.surfacePressed = const Color(0xFFD3D1CA),
    this.textTertiary = const Color(0xFF777B72),
    this.textDisabled = const Color(0xFF9B9E97),
    this.iconNormal = const Color(0xFF62665E),
    this.iconActive = const Color(0xFF20221E),
    this.iconDisabled = const Color(0xFFA4A79F),
    this.separatorExceptional = const Color(0xFFBBB8AF),
    this.paperWhite = const Color(0xFFF8F7F2),
    this.ivory = const Color(0xFFEDE7D8),
    this.nightBackground = const Color(0xFF0B0D0B),
    this.ambientShadow = const Color(0x1F000000),
    this.pageShadow = const Color(0x2E000000),
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
  final Color librarySelectionAccent;
  final Color readerBackground;
  final Color libraryBackground;
  final Color settingsBackground;
  final Color surfaceFloating;
  final Color surfaceActive;
  final Color surfaceHover;
  final Color surfacePressed;
  final Color textTertiary;
  final Color textDisabled;
  final Color iconNormal;
  final Color iconActive;
  final Color iconDisabled;
  final Color separatorExceptional;
  final Color paperWhite;
  final Color ivory;
  final Color nightBackground;
  final Color ambientShadow;
  final Color pageShadow;

  static const light = AppColorTokens(
    appBackground: Color(0xFFECEAE4),
    surfacePrimary: Color(0xFFF4F2EC),
    surfaceSecondary: Color(0xFFF0EEE8),
    surfaceElevated: Color(0xFFF7F5EF),
    surfaceSunken: Color(0xFFE2E0D8),
    textPrimary: Color(0xFF1C1E1A),
    textSecondary: Color(0xFF5D6159),
    textMuted: Color(0xFF8C8D87),
    borderSubtle: Color(0xFFD5D2CA),
    borderStrong: Color(0xFFBBB8AF),
    accentPrimary: Color(0xFF393B37),
    accentHover: Color(0xFF2D2F2B),
    accentPressed: Color(0xFF20211F),
    accentSoft: Color(0xFFE4E3DE),
    selectionBackground: Color(0xFFD8D7CF),
    focusRing: Color(0xFF50594E),
    danger: Color(0xFF9C4943),
    warning: Color(0xFF85652F),
    success: Color(0xFF4E7058),
    readerPageShadow: Color(0x2E000000),
    librarySelectionAccent: Color(0xFF7652BC),
    readerBackground: Color(0xFFE7E4DC),
    libraryBackground: Color(0xFFEDEBE5),
    settingsBackground: Color(0xFFF1EFE9),
    surfaceFloating: Color(0xFFFBF9F3),
    surfaceActive: Color(0xFFDCDAD3),
    surfaceHover: Color(0xFFE4E2DB),
    surfacePressed: Color(0xFFD3D1CA),
    textTertiary: Color(0xFF777B72),
    textDisabled: Color(0xFF9B9E97),
    iconNormal: Color(0xFF62665E),
    iconActive: Color(0xFF20221E),
    iconDisabled: Color(0xFFA4A79F),
    separatorExceptional: Color(0xFFBBB8AF),
    paperWhite: Color(0xFFF8F7F2),
    ivory: Color(0xFFEDE7D8),
    nightBackground: Color(0xFF0B0D0B),
    ambientShadow: Color(0x1F000000),
    pageShadow: Color(0x2E000000),
  );

  static const dark = AppColorTokens(
    appBackground: Color(0xFF111310),
    surfacePrimary: Color(0xFF181B17),
    surfaceSecondary: Color(0xFF151815),
    surfaceElevated: Color(0xFF1C1F1B),
    surfaceSunken: Color(0xFF0B0C0A),
    textPrimary: Color(0xFFF0F1EB),
    textSecondary: Color(0xFFB7BBB1),
    textMuted: Color(0xFF777A72),
    borderSubtle: Color(0xFF2B2F29),
    borderStrong: Color(0xFF3B4038),
    accentPrimary: Color(0xFFD0D1CA),
    accentHover: Color(0xFFE0E1DA),
    accentPressed: Color(0xFFF1F1EA),
    accentSoft: Color(0xFF2B2D28),
    selectionBackground: Color(0xFF2B2D28),
    focusRing: Color(0xFFD8DED3),
    danger: Color(0xFFD9847C),
    warning: Color(0xFFD0AD70),
    success: Color(0xFF8EB397),
    readerPageShadow: Color(0x85000000),
    librarySelectionAccent: Color(0xFFB58AFF),
    readerBackground: Color(0xFF141613),
    libraryBackground: Color(0xFF10120F),
    settingsBackground: Color(0xFF151714),
    surfaceFloating: Color(0xFF232620),
    surfaceActive: Color(0xFF30342D),
    surfaceHover: Color(0xFF242720),
    surfacePressed: Color(0xFF34382F),
    textTertiary: Color(0xFF8F948A),
    textDisabled: Color(0xFF666B62),
    iconNormal: Color(0xFFA9ADA4),
    iconActive: Color(0xFFF3F4EE),
    iconDisabled: Color(0xFF5E635A),
    separatorExceptional: Color(0xFF3B4038),
    paperWhite: Color(0xFFF8F7F2),
    ivory: Color(0xFFEDE7D8),
    nightBackground: Color(0xFF0B0D0B),
    ambientShadow: Color(0x66000000),
    pageShadow: Color(0x85000000),
  );

  /// Monochrome palette reserved for Home/Library. Violet is deliberately
  /// isolated in [librarySelectionAccent] and only identifies selected files.
  /// Reader and Settings continue to use [light] and [dark].
  static const library = AppColorTokens(
    appBackground: Color(0xFF080808),
    surfacePrimary: Color(0xFF0E0E0F),
    surfaceSecondary: Color(0xFF131314),
    surfaceElevated: Color(0xFF19191A),
    surfaceSunken: Color(0xFF050505),
    textPrimary: Color(0xFFF5F5F5),
    textSecondary: Color(0xFFBDBDC1),
    textMuted: Color(0xFF7B7B80),
    borderSubtle: Color(0xFF262628),
    borderStrong: Color(0xFF404043),
    accentPrimary: Color(0xFFF5F5F5),
    accentHover: Color(0xFFFFFFFF),
    accentPressed: Color(0xFFD7D7DA),
    accentSoft: Color(0xFF202022),
    selectionBackground: Color(0xFF29292C),
    focusRing: Color(0xFFFFFFFF),
    danger: Color(0xFFE4E4E6),
    warning: Color(0xFFBEBEC2),
    success: Color(0xFFF5F5F5),
    readerPageShadow: Color(0xB8000000),
    librarySelectionAccent: Color(0xFFB58AFF),
    readerBackground: Color(0xFF0E0E0F),
    libraryBackground: Color(0xFF080808),
    settingsBackground: Color(0xFF0E0E0F),
    surfaceFloating: Color(0xF2141415),
    surfaceActive: Color(0xFF2A2A2D),
    surfaceHover: Color(0xFF1D1D1F),
    surfacePressed: Color(0xFF323235),
    textTertiary: Color(0xFF98989D),
    textDisabled: Color(0xFF606065),
    iconNormal: Color(0xFFBDBDC1),
    iconActive: Color(0xFFF5F5F5),
    iconDisabled: Color(0xFF5A5A5F),
    separatorExceptional: Color(0xFF404043),
    paperWhite: Color(0xFFF8F7F2),
    ivory: Color(0xFFEDE7D8),
    nightBackground: Color(0xFF020202),
    ambientShadow: Color(0x8A000000),
    pageShadow: Color(0xA8000000),
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
    Color? librarySelectionAccent,
    Color? readerBackground,
    Color? libraryBackground,
    Color? settingsBackground,
    Color? surfaceFloating,
    Color? surfaceActive,
    Color? surfaceHover,
    Color? surfacePressed,
    Color? textTertiary,
    Color? textDisabled,
    Color? iconNormal,
    Color? iconActive,
    Color? iconDisabled,
    Color? separatorExceptional,
    Color? paperWhite,
    Color? ivory,
    Color? nightBackground,
    Color? ambientShadow,
    Color? pageShadow,
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
    librarySelectionAccent:
        librarySelectionAccent ?? this.librarySelectionAccent,
    readerBackground: readerBackground ?? this.readerBackground,
    libraryBackground: libraryBackground ?? this.libraryBackground,
    settingsBackground: settingsBackground ?? this.settingsBackground,
    surfaceFloating: surfaceFloating ?? this.surfaceFloating,
    surfaceActive: surfaceActive ?? this.surfaceActive,
    surfaceHover: surfaceHover ?? this.surfaceHover,
    surfacePressed: surfacePressed ?? this.surfacePressed,
    textTertiary: textTertiary ?? this.textTertiary,
    textDisabled: textDisabled ?? this.textDisabled,
    iconNormal: iconNormal ?? this.iconNormal,
    iconActive: iconActive ?? this.iconActive,
    iconDisabled: iconDisabled ?? this.iconDisabled,
    separatorExceptional: separatorExceptional ?? this.separatorExceptional,
    paperWhite: paperWhite ?? this.paperWhite,
    ivory: ivory ?? this.ivory,
    nightBackground: nightBackground ?? this.nightBackground,
    ambientShadow: ambientShadow ?? this.ambientShadow,
    pageShadow: pageShadow ?? this.pageShadow,
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
      librarySelectionAccent: Color.lerp(
        librarySelectionAccent,
        other.librarySelectionAccent,
        t,
      )!,
      readerBackground: Color.lerp(
        readerBackground,
        other.readerBackground,
        t,
      )!,
      libraryBackground: Color.lerp(
        libraryBackground,
        other.libraryBackground,
        t,
      )!,
      settingsBackground: Color.lerp(
        settingsBackground,
        other.settingsBackground,
        t,
      )!,
      surfaceFloating: Color.lerp(surfaceFloating, other.surfaceFloating, t)!,
      surfaceActive: Color.lerp(surfaceActive, other.surfaceActive, t)!,
      surfaceHover: Color.lerp(surfaceHover, other.surfaceHover, t)!,
      surfacePressed: Color.lerp(surfacePressed, other.surfacePressed, t)!,
      textTertiary: Color.lerp(textTertiary, other.textTertiary, t)!,
      textDisabled: Color.lerp(textDisabled, other.textDisabled, t)!,
      iconNormal: Color.lerp(iconNormal, other.iconNormal, t)!,
      iconActive: Color.lerp(iconActive, other.iconActive, t)!,
      iconDisabled: Color.lerp(iconDisabled, other.iconDisabled, t)!,
      separatorExceptional: Color.lerp(
        separatorExceptional,
        other.separatorExceptional,
        t,
      )!,
      paperWhite: Color.lerp(paperWhite, other.paperWhite, t)!,
      ivory: Color.lerp(ivory, other.ivory, t)!,
      nightBackground: Color.lerp(nightBackground, other.nightBackground, t)!,
      ambientShadow: Color.lerp(ambientShadow, other.ambientShadow, t)!,
      pageShadow: Color.lerp(pageShadow, other.pageShadow, t)!,
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
