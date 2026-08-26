import 'package:flutter/material.dart';

import 'app_tokens.dart';

enum AppStatusTone { neutral, success, warning, error }

final class AppSectionHeader extends StatelessWidget {
  const AppSectionHeader({
    super.key,
    required this.title,
    this.description,
    this.trailing,
  });

  final String title;
  final String? description;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleLarge),
              if (description case final value?) ...[
                const SizedBox(height: AppSpacing.space4),
                Text(value, style: Theme.of(context).textTheme.bodySmall),
              ],
            ],
          ),
        ),
        if (trailing case final value?) ...[
          const SizedBox(width: AppSpacing.space16),
          value,
        ],
      ],
    );
  }
}

final class AppStatusView extends StatelessWidget {
  const AppStatusView({
    super.key,
    required this.icon,
    required this.title,
    required this.message,
    this.action,
    this.actionLabel,
    this.actionKey,
    this.tone = AppStatusTone.neutral,
    this.loading = false,
  }) : assert((action == null) == (actionLabel == null));

  final IconData icon;
  final String title;
  final String message;
  final VoidCallback? action;
  final String? actionLabel;
  final Key? actionKey;
  final AppStatusTone tone;
  final bool loading;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final toneColor = switch (tone) {
      AppStatusTone.neutral => colors.accentPrimary,
      AppStatusTone.success => colors.success,
      AppStatusTone.warning => colors.warning,
      AppStatusTone.error => colors.danger,
    };
    return LayoutBuilder(
      builder: (context, constraints) => SingleChildScrollView(
        child: ConstrainedBox(
          constraints: BoxConstraints(minHeight: constraints.maxHeight),
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 440),
              child: Padding(
                padding: const EdgeInsets.all(AppSpacing.space32),
                child: Semantics(
                  liveRegion: loading || tone == AppStatusTone.error,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Container(
                        width: 40,
                        height: 40,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(
                          color: toneColor.withValues(alpha: 0.10),
                          borderRadius: BorderRadius.circular(AppRadii.control),
                        ),
                        child: Icon(icon, color: toneColor, size: 22),
                      ),
                      const SizedBox(height: AppSpacing.space20),
                      Text(
                        title,
                        style: Theme.of(context).textTheme.headlineSmall,
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: AppSpacing.space8),
                      Text(
                        message,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: colors.textSecondary,
                        ),
                        textAlign: TextAlign.center,
                      ),
                      if (loading) ...[
                        const SizedBox(height: AppSpacing.space20),
                        const SizedBox(
                          width: 180,
                          child: LinearProgressIndicator(minHeight: 2),
                        ),
                      ],
                      if (action case final callback?) ...[
                        const SizedBox(height: AppSpacing.space24),
                        FilledButton(
                          key: actionKey,
                          onPressed: callback,
                          child: Text(actionLabel!),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class AppShortcutHint extends StatelessWidget {
  const AppShortcutHint(this.label, {super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: AppSpacing.space6,
        vertical: AppSpacing.space2,
      ),
      decoration: BoxDecoration(
        color: context.appColors.surfaceSunken,
        border: Border.all(color: context.appColors.borderSubtle),
        borderRadius: BorderRadius.circular(AppRadii.indicator),
      ),
      child: Text(
        label,
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
          fontFeatures: const [FontFeature.tabularFigures()],
        ),
      ),
    );
  }
}

final class AppStatusLabel extends StatelessWidget {
  const AppStatusLabel({
    super.key,
    required this.icon,
    required this.label,
    this.tone = AppStatusTone.neutral,
  });

  final IconData icon;
  final String label;
  final AppStatusTone tone;

  @override
  Widget build(BuildContext context) {
    final color = switch (tone) {
      AppStatusTone.neutral => context.appColors.textSecondary,
      AppStatusTone.success => context.appColors.success,
      AppStatusTone.warning => context.appColors.warning,
      AppStatusTone.error => context.appColors.danger,
    };
    return Semantics(
      label: label,
      child: Container(
        padding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space8,
          vertical: AppSpacing.space4,
        ),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.08),
          border: Border.all(color: color.withValues(alpha: 0.35)),
          borderRadius: BorderRadius.circular(AppRadii.indicator),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 14, color: color),
            const SizedBox(width: AppSpacing.space4),
            Flexible(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Theme.of(
                  context,
                ).textTheme.labelSmall?.copyWith(color: color),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Consistent content for popup menus and context menus on desktop.
///
/// The popup owns activation and keyboard behavior; this widget owns visual
/// hierarchy so checked, destructive and shortcut states do not drift between
/// features.
final class AppMenuItemContent extends StatelessWidget {
  const AppMenuItemContent({
    super.key,
    required this.icon,
    required this.label,
    this.shortcut,
    this.checked = false,
    this.destructive = false,
  });

  final IconData icon;
  final String label;
  final String? shortcut;
  final bool checked;
  final bool destructive;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final foreground = destructive ? colors.danger : colors.textPrimary;
    return Row(
      children: [
        SizedBox(
          width: 24,
          child: Icon(
            checked ? Icons.check_rounded : icon,
            size: 17,
            color: foreground,
          ),
        ),
        const SizedBox(width: AppSpacing.space8),
        Expanded(
          child: Text(
            label,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(color: foreground),
          ),
        ),
        if (shortcut case final value?) ...[
          const SizedBox(width: AppSpacing.space20),
          AppShortcutHint(value),
        ],
      ],
    );
  }
}

/// Desktop dialog heading with a stable icon, title and explanatory text.
final class AppDialogHeader extends StatelessWidget {
  const AppDialogHeader({
    super.key,
    required this.icon,
    required this.title,
    this.description,
    this.trailing,
  });

  final IconData icon;
  final String title;
  final String? description;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 36,
          height: 36,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: context.appColors.surfaceActive,
            border: Border.all(color: context.appColors.borderSubtle),
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
          child: Icon(icon, size: 19),
        ),
        const SizedBox(width: AppSpacing.space12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleLarge),
              if (description case final value?) ...[
                const SizedBox(height: AppSpacing.space4),
                Text(
                  value,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: context.appColors.textSecondary,
                  ),
                ),
              ],
            ],
          ),
        ),
        if (trailing case final value?) ...[
          const SizedBox(width: AppSpacing.space12),
          value,
        ],
      ],
    );
  }
}

/// Visually isolates a destructive or privacy-sensitive operation from normal
/// settings and dialog actions.
final class AppDangerZone extends StatelessWidget {
  const AppDangerZone({
    super.key,
    required this.title,
    required this.description,
    required this.child,
  });

  final String title;
  final String description;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final danger = context.appColors.danger;
    final details = Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(Icons.warning_amber_rounded, size: 20, color: danger),
        const SizedBox(width: AppSpacing.space12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: Theme.of(
                  context,
                ).textTheme.titleSmall?.copyWith(color: danger),
              ),
              const SizedBox(height: AppSpacing.space4),
              Text(description, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ),
      ],
    );
    return LayoutBuilder(
      builder: (context, constraints) => Container(
        padding: const EdgeInsets.all(AppSpacing.space16),
        decoration: BoxDecoration(
          color: danger.withValues(alpha: 0.06),
          border: Border.all(color: danger.withValues(alpha: 0.55)),
          borderRadius: BorderRadius.circular(AppRadii.panel),
        ),
        child: constraints.maxWidth < 520
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  details,
                  const SizedBox(height: AppSpacing.space12),
                  Align(alignment: Alignment.centerRight, child: child),
                ],
              )
            : Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(child: details),
                  const SizedBox(width: AppSpacing.space16),
                  child,
                ],
              ),
      ),
    );
  }
}
