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
    this.targetExtent = 40,
    this.settingRowMinHeight = 56,
    this.compactMenuRowHeight = 34,
  });

  static const comfortable = AppDensityTokens(
    interfaceDensity: AppInterfaceDensity.comfortable,
    listRowVerticalPadding: 10,
    settingsRowVerticalPadding: 12,
    sectionSpacing: 32,
    recentCoverWidth: 44,
    recentCoverHeight: 60,
    targetExtent: 40,
    settingRowMinHeight: 52,
    compactMenuRowHeight: 40,
  );

  static const compact = AppDensityTokens(
    interfaceDensity: AppInterfaceDensity.compact,
    listRowVerticalPadding: 6,
    settingsRowVerticalPadding: 8,
    sectionSpacing: 24,
    recentCoverWidth: 38,
    recentCoverHeight: 52,
    targetExtent: 40,
    settingRowMinHeight: 44,
    compactMenuRowHeight: 40,
  );

  final AppInterfaceDensity interfaceDensity;
  final double listRowVerticalPadding;
  final double settingsRowVerticalPadding;
  final double sectionSpacing;
  final double recentCoverWidth;
  final double recentCoverHeight;
  final double targetExtent;
  final double settingRowMinHeight;
  final double compactMenuRowHeight;

  @override
  AppDensityTokens copyWith({
    AppInterfaceDensity? interfaceDensity,
    double? listRowVerticalPadding,
    double? settingsRowVerticalPadding,
    double? sectionSpacing,
    double? recentCoverWidth,
    double? recentCoverHeight,
    double? targetExtent,
    double? settingRowMinHeight,
    double? compactMenuRowHeight,
  }) => AppDensityTokens(
    interfaceDensity: interfaceDensity ?? this.interfaceDensity,
    listRowVerticalPadding:
        listRowVerticalPadding ?? this.listRowVerticalPadding,
    settingsRowVerticalPadding:
        settingsRowVerticalPadding ?? this.settingsRowVerticalPadding,
    sectionSpacing: sectionSpacing ?? this.sectionSpacing,
    recentCoverWidth: recentCoverWidth ?? this.recentCoverWidth,
    recentCoverHeight: recentCoverHeight ?? this.recentCoverHeight,
    targetExtent: targetExtent ?? this.targetExtent,
    settingRowMinHeight: settingRowMinHeight ?? this.settingRowMinHeight,
    compactMenuRowHeight: compactMenuRowHeight ?? this.compactMenuRowHeight,
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
