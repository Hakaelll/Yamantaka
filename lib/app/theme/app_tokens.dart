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
  static const double topBarHeight = 56;
  static const double readerCompactTopBarHeight = 52;
  static const double libraryHeaderHeight = 76;
  static const double bottomControlHeight = 48;
  static const double readerRestingDockHeight = 40;
  static const double readerPanelHeaderHeight = 52;
  static const double readerPanelRailWidth = 52;
  static const double sidePanelWidth = 344;
  static const double sidePanelMinWidth = 288;
  static const double sidePanelMaxWidth = 480;
  static const double compactSidePanelWidth = 360;
  static const double desktopControlExtent = 40;
  static const double touchControlExtent = 48;
  static const double dialogMaxWidth = 600;
  static const double dialogCompactWidth = 480;
  static const double dialogWideWidth = 680;
  static const double commandPaletteMaxHeight = 520;
  static const double readerEdgeNavigation = 64;
  static const double readerSpreadGutter = 2;
  static const double topRevealZone = 24;
  static const double readerPanelDefault = sidePanelWidth;
  static const double settingsNavigationWidth = 236;
  static const double settingsNavigationMaxWidth = 320;
  static const double settingsPreviewWidth = 320;
  static const double settingsContentMaxWidth = 780;
  static const double settingsControlWidth = 240;
  static const double commandPaletteRowHeight = 48;
  static const double commandPaletteExpandedWidth = 780;
  static const double commandPalettePreviewWidth = 228;
  static const double artifactSelectedScale = 1.035;
  static const double artifactHoverScale = 1.015;
  static const double artifactSelectedLift = 6;
  static const double artifactHoverLift = 2;

  static double readerTopBarHeight({required bool compact}) =>
      compact ? readerCompactTopBarHeight : topBarHeight;
}

/// Optical icon sizes are independent from their interactive target.
abstract final class AppIconSizes {
  static const double compact = 14;
  static const double small = 16;
  static const double standard = 18;
  static const double large = 20;
  static const double status = 22;
}

/// Shared control heights. Text scaling may increase these minimums but never
/// reduce the desktop or touch target.
abstract final class AppControlHeights {
  static const double dense = 34;
  static const double desktop = AppSizes.desktopControlExtent;
  static const double touch = AppSizes.touchControlExtent;
  static const double row = AppSizes.commandPaletteRowHeight;
}

abstract final class AppBorders {
  static const double hairline = 1;
  static const double focus = 2;
  static const double progress = 2;
}

abstract final class AppOpacity {
  static const double faint = 0.06;
  static const double subtle = 0.08;
  static const double toneSurface = 0.10;
  static const double borderEmphasis = 0.35;
  static const double strongBorder = 0.55;
  static const double hoverEmphasis = 0.65;
  static const double floatingSurface = 0.96;
}

/// Editorial roles used by [AppTheme]. Feature widgets select a semantic text
/// style instead of introducing local sizes. Essential information starts at
/// 12 logical pixels; density never scales this typography down.
abstract final class AppTypeScale {
  static const double screenTitle = 28;
  static const double sectionTitle = 19;
  static const double documentTitle = 17;
  static const double contextualHeading = 15;
  static const double body = 14;
  static const double control = 14;
  static const double metadata = 12;
  static const double auxiliary = 12;
  static const double figure = 12;
  static const double shortcut = 12;

  static const List<FontFeature> tabularFigures = [
    FontFeature.tabularFigures(),
  ];
}

abstract final class AppRadii {
  static const double indicator = 2;
  static const double page = 3;
  static const double control = 8;
  static const double field = 10;
  static const double floating = 12;
  static const double panel = 12;
  static const double dialog = 14;
  static const double touchSurface = 12;
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
  static const double readerPageBlur = 7;
  static const double readerGrayPageBlur = 4;
  static const double readerPanelBlur = 8;
  static const double floatingControlBlur = 6;
  static const double floatingControlOffset = 2;
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
  static const Duration dialog = Duration(milliseconds: 200);
  static const Duration inspector = Duration(milliseconds: 160);
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

abstract final class AppCurves {
  static const Curve feedback = Curves.easeOutCubic;
  static const Curve standard = Curves.easeInOutCubic;
  static const Curve panel = Curves.easeOutCubic;
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

  // Complete semantic interaction vocabulary. These aliases intentionally
  // resolve from the smaller maintained palette so themes cannot drift by
  // inventing one-off colors for a state.
  Color get surfaceSelected => selectionBackground;
  Color get surfaceFocused => accentSoft;
  Color get surfaceDisabled => surfaceSunken;
  Color get surfaceLoading => surfaceSecondary;
  Color get borderFocus => focusRing;
  Color get paperPdf => paperWhite;
  Color get overlayShadow => ambientShadow;

  static const light = AppColorTokens(
    appBackground: Color(0xFFF7F6F2),
    surfacePrimary: Color(0xFFFFFFFF),
    surfaceSecondary: Color(0xFFF0EFEB),
    surfaceElevated: Color(0xFFFFFFFF),
    surfaceSunken: Color(0xFFE9E7E2),
    textPrimary: Color(0xFF1C1E1A),
    textSecondary: Color(0xFF5D6159),
    textMuted: Color(0xFF696B73),
    borderSubtle: Color(0xFFD5D2CA),
    borderStrong: Color(0xFFBBB8AF),
    accentPrimary: Color(0xFF7055B5),
    accentHover: Color(0xFF60469F),
    accentPressed: Color(0xFF51398B),
    accentSoft: Color(0xFFECE6F7),
    selectionBackground: Color(0xFFE9E1F5),
    focusRing: Color(0xFF7055B5),
    danger: Color(0xFF9C4943),
    warning: Color(0xFF85652F),
    success: Color(0xFF4E7058),
    readerPageShadow: Color(0x2E000000),
    librarySelectionAccent: Color(0xFF7652BC),
    readerBackground: Color(0xFFE7E4DC),
    libraryBackground: Color(0xFFF7F6F2),
    settingsBackground: Color(0xFFF7F6F2),
    surfaceFloating: Color(0xFFFFFFFF),
    surfaceActive: Color(0xFFE9E1F5),
    surfaceHover: Color(0xFFEDEBE6),
    surfacePressed: Color(0xFFD3D1CA),
    textTertiary: Color(0xFF696B73),
    textDisabled: Color(0xFF9B9E97),
    iconNormal: Color(0xFF62665E),
    iconActive: Color(0xFF20221E),
    iconDisabled: Color(0xFFA4A79F),
    separatorExceptional: Color(0xFFBBB8AF),
    paperWhite: Color(0xFFF8F7F2),
    ivory: Color(0xFFEDE7D8),
    nightBackground: Color(0xFF0B0D0B),
    ambientShadow: Color(0x180F1420),
    pageShadow: Color(0x2E000000),
  );

  static const dark = AppColorTokens(
    appBackground: Color(0xFF181A20),
    surfacePrimary: Color(0xFF20232B),
    surfaceSecondary: Color(0xFF1C1F26),
    surfaceElevated: Color(0xFF292D36),
    surfaceSunken: Color(0xFF14161C),
    textPrimary: Color(0xFFF0F1EB),
    textSecondary: Color(0xFFB9BEC9),
    textMuted: Color(0xFF9A9FAE),
    borderSubtle: Color(0xFF373B46),
    borderStrong: Color(0xFF555B69),
    accentPrimary: Color(0xFFC4AFF4),
    accentHover: Color(0xFFD9C9FC),
    accentPressed: Color(0xFFAF95E5),
    accentSoft: Color(0xFF363044),
    selectionBackground: Color(0xFF393148),
    focusRing: Color(0xFFD9C9FC),
    danger: Color(0xFFD9847C),
    warning: Color(0xFFD0AD70),
    success: Color(0xFF8EB397),
    readerPageShadow: Color(0x85000000),
    librarySelectionAccent: Color(0xFFB58AFF),
    readerBackground: Color(0xFF141613),
    libraryBackground: Color(0xFF181A20),
    settingsBackground: Color(0xFF181A20),
    surfaceFloating: Color(0xFF292D36),
    surfaceActive: Color(0xFF393148),
    surfaceHover: Color(0xFF2C303A),
    surfacePressed: Color(0xFF3A3E49),
    textTertiary: Color(0xFFA5AAB8),
    textDisabled: Color(0xFF666B62),
    iconNormal: Color(0xFFB9BEC9),
    iconActive: Color(0xFFF3F4EE),
    iconDisabled: Color(0xFF5E635A),
    separatorExceptional: Color(0xFF3B4038),
    paperWhite: Color(0xFFF8F7F2),
    ivory: Color(0xFFEDE7D8),
    nightBackground: Color(0xFF0B0D0B),
    ambientShadow: Color(0x66000000),
    pageShadow: Color(0x85000000),
  );

  /// Compatibility alias; the library now uses the shared application theme.
  static const library = dark;
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
    if (other == null || identical(this, other)) return this;
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
