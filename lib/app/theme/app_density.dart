import 'package:flutter/material.dart';

enum AppInterfaceDensity { comfortable, compact }

@immutable
final class AppDensityTokens extends ThemeExtension<AppDensityTokens> {
  const AppDensityTokens({
    required this.interfaceDensity,
    required this.listRowVerticalPadding,
    required this.settingsRowVerticalPadding,
    required this.sectionSpacing,
    required this.recentCoverWidth,
    required this.recentCoverHeight,
  });

  static const comfortable = AppDensityTokens(
    interfaceDensity: AppInterfaceDensity.comfortable,
    listRowVerticalPadding: 12,
    settingsRowVerticalPadding: 16,
    sectionSpacing: 40,
    recentCoverWidth: 44,
    recentCoverHeight: 60,
  );

  static const compact = AppDensityTokens(
    interfaceDensity: AppInterfaceDensity.compact,
    listRowVerticalPadding: 8,
    settingsRowVerticalPadding: 10,
    sectionSpacing: 28,
    recentCoverWidth: 38,
    recentCoverHeight: 52,
  );

  final AppInterfaceDensity interfaceDensity;
  final double listRowVerticalPadding;
  final double settingsRowVerticalPadding;
  final double sectionSpacing;
  final double recentCoverWidth;
  final double recentCoverHeight;

  @override
  AppDensityTokens copyWith({
    AppInterfaceDensity? interfaceDensity,
    double? listRowVerticalPadding,
    double? settingsRowVerticalPadding,
    double? sectionSpacing,
    double? recentCoverWidth,
    double? recentCoverHeight,
  }) => AppDensityTokens(
    interfaceDensity: interfaceDensity ?? this.interfaceDensity,
    listRowVerticalPadding:
        listRowVerticalPadding ?? this.listRowVerticalPadding,
    settingsRowVerticalPadding:
        settingsRowVerticalPadding ?? this.settingsRowVerticalPadding,
    sectionSpacing: sectionSpacing ?? this.sectionSpacing,
    recentCoverWidth: recentCoverWidth ?? this.recentCoverWidth,
    recentCoverHeight: recentCoverHeight ?? this.recentCoverHeight,
  );

  @override
  AppDensityTokens lerp(AppDensityTokens? other, double t) {
    if (other == null || t < 0.5) return this;
    return other;
  }
}

extension AppDensityContext on BuildContext {
  AppDensityTokens get appDensity =>
      Theme.of(this).extension<AppDensityTokens>() ??
      AppDensityTokens.comfortable;
}
