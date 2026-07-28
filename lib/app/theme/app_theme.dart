import 'package:flutter/material.dart';

abstract final class AppTheme {
  static const Color accent = Color(0xFF675E4A);

  static ThemeData light() => _build(
    brightness: Brightness.light,
    surface: const Color(0xFFF4F3EF),
    elevated: const Color(0xFFFCFBF8),
    text: const Color(0xFF252522),
    secondary: const Color(0xFF66645E),
    border: const Color(0xFFD8D5CE),
  );

  static ThemeData dark() => _build(
    brightness: Brightness.dark,
    surface: const Color(0xFF1D1D1B),
    elevated: const Color(0xFF262624),
    text: const Color(0xFFECEAE4),
    secondary: const Color(0xFFAAA79F),
    border: const Color(0xFF4A4944),
  );

  static ThemeData _build({
    required Brightness brightness,
    required Color surface,
    required Color elevated,
    required Color text,
    required Color secondary,
    required Color border,
  }) {
    final scheme = ColorScheme(
      brightness: brightness,
      primary: accent,
      onPrimary: Colors.white,
      secondary: accent,
      onSecondary: Colors.white,
      error: const Color(0xFFA33A32),
      onError: Colors.white,
      surface: elevated,
      onSurface: text,
    );
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: surface,
      visualDensity: VisualDensity.standard,
    );
    return base.copyWith(
      appBarTheme: AppBarTheme(
        backgroundColor: elevated,
        foregroundColor: text,
        elevation: 0,
        centerTitle: false,
        toolbarHeight: 48,
        shape: Border(bottom: BorderSide(color: border)),
        titleTextStyle: base.textTheme.titleMedium?.copyWith(
          color: text,
          fontWeight: FontWeight.w600,
        ),
      ),
      dividerColor: border,
      cardTheme: CardThemeData(
        elevation: 0,
        color: elevated,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: border),
          borderRadius: BorderRadius.circular(4),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: elevated,
        isDense: true,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(4),
          borderSide: BorderSide(color: border),
        ),
      ),
      focusColor: accent.withValues(alpha: 0.14),
      splashFactory: InkSparkle.splashFactory,
      tooltipTheme: const TooltipThemeData(
        waitDuration: Duration(milliseconds: 500),
      ),
      progressIndicatorTheme: const ProgressIndicatorThemeData(color: accent),
      iconButtonTheme: IconButtonThemeData(
        style: IconButton.styleFrom(minimumSize: const Size(40, 40)),
      ),
      listTileTheme: const ListTileThemeData(
        minTileHeight: 48,
        contentPadding: EdgeInsets.symmetric(horizontal: 12),
      ),
      textTheme: base.textTheme.apply(
        bodyColor: text,
        displayColor: text,
        decorationColor: secondary,
      ),
    );
  }
}
