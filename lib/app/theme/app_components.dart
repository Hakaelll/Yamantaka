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
    this.secondaryAction,
    this.secondaryActionLabel,
    this.tone = AppStatusTone.neutral,
    this.loading = false,
  }) : assert((action == null) == (actionLabel == null)),
       assert((secondaryAction == null) == (secondaryActionLabel == null));

  final IconData icon;
  final String title;
  final String message;
  final VoidCallback? action;
  final String? actionLabel;
  final Key? actionKey;
  final VoidCallback? secondaryAction;
  final String? secondaryActionLabel;
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
                          color: toneColor.withValues(
                            alpha: AppOpacity.toneSurface,
                          ),
                          borderRadius: BorderRadius.circular(AppRadii.control),
                        ),
                        child: Icon(
                          icon,
                          color: toneColor,
                          size: AppIconSizes.status,
                        ),
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
                          child: LinearProgressIndicator(
                            minHeight: AppBorders.progress,
                          ),
                        ),
                      ],
                      if (action case final callback?) ...[
                        const SizedBox(height: AppSpacing.space24),
                        Wrap(
                          alignment: WrapAlignment.center,
                          spacing: AppSpacing.space8,
                          runSpacing: AppSpacing.space8,
                          children: [
                            FilledButton(
                              key: actionKey,
                              onPressed: callback,
                              child: Text(actionLabel!),
                            ),
                            if (secondaryAction case final secondary?)
                              TextButton(
                                onPressed: secondary,
                                child: Text(secondaryActionLabel!),
                              ),
                          ],
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
          color: color.withValues(alpha: AppOpacity.subtle),
          border: Border.all(
            color: color.withValues(alpha: AppOpacity.borderEmphasis),
          ),
          borderRadius: BorderRadius.circular(AppRadii.indicator),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: AppIconSizes.compact, color: color),
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
    final stackedShortcut = MediaQuery.textScalerOf(context).scale(14) > 18;
    return Row(
      children: [
        SizedBox(
          width: 24,
          child: Icon(
            checked ? Icons.check_rounded : icon,
            size: AppIconSizes.standard,
            color: foreground,
          ),
        ),
        const SizedBox(width: AppSpacing.space8),
        Expanded(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label, style: TextStyle(color: foreground)),
              if (stackedShortcut && shortcut != null)
                AppShortcutHint(shortcut!),
            ],
          ),
        ),
        if (!stackedShortcut && shortcut != null) ...[
          const SizedBox(width: AppSpacing.space20),
          AppShortcutHint(shortcut!),
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
          width: 32,
          height: 32,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: context.appColors.surfaceActive,
            border: Border.all(color: context.appColors.borderSubtle),
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
          child: Icon(icon, size: AppIconSizes.standard),
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
        Icon(
          Icons.warning_amber_rounded,
          size: AppIconSizes.large,
          color: danger,
        ),
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
          color: danger.withValues(alpha: AppOpacity.faint),
          border: Border.all(
            color: danger.withValues(alpha: AppOpacity.strongBorder),
          ),
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

/// Shared adaptive shell for critical desktop dialogs.
///
/// [AlertDialog.scrollable] keeps the title and actions reachable at 200 %
/// text scaling, while its overflow bar stacks actions when the viewport is
/// narrow. Routes retain Flutter's Escape and focus-restoration behavior.
final class AppDialogFrame extends StatelessWidget {
  const AppDialogFrame({
    super.key,
    required this.icon,
    required this.title,
    required this.actions,
    this.description,
    this.content,
    this.maxWidth = AppSizes.dialogMaxWidth,
    this.semanticLabel,
  });

  final IconData icon;
  final String title;
  final String? description;
  final Widget? content;
  final List<Widget> actions;
  final double maxWidth;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) => AlertDialog(
    scrollable: true,
    semanticLabel: semanticLabel ?? title,
    insetPadding: const EdgeInsets.all(AppSpacing.space16),
    titlePadding: const EdgeInsets.fromLTRB(
      AppSpacing.space24,
      AppSpacing.space24,
      AppSpacing.space24,
      AppSpacing.space12,
    ),
    contentPadding: content == null
        ? EdgeInsets.zero
        : const EdgeInsets.fromLTRB(
            AppSpacing.space24,
            AppSpacing.space4,
            AppSpacing.space24,
            AppSpacing.space16,
          ),
    actionsPadding: const EdgeInsets.fromLTRB(
      AppSpacing.space16,
      AppSpacing.space8,
      AppSpacing.space16,
      AppSpacing.space16,
    ),
    actionsOverflowButtonSpacing: AppSpacing.space8,
    title: AppDialogHeader(icon: icon, title: title, description: description),
    content: content == null
        ? null
        : ConstrainedBox(
            constraints: BoxConstraints(maxWidth: maxWidth),
            child: content,
          ),
    actions: actions,
  );
}

/// Common language for long-running local work with a named phase, real
/// progress when available, and a reachable cancellation action.
final class AppOperationStatus extends StatelessWidget {
  const AppOperationStatus({
    super.key,
    required this.label,
    required this.phase,
    this.value,
    this.onCancel,
    this.cancelLabel,
    this.icon = Icons.sync_rounded,
  }) : assert(value == null || (value >= 0 && value <= 1)),
       assert((onCancel == null) == (cancelLabel == null));

  final String label;
  final String phase;
  final double? value;
  final VoidCallback? onCancel;
  final String? cancelLabel;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final percent = value == null ? null : (value! * 100).round();
    final semantics = StringBuffer('$label. $phase');
    if (percent != null) semantics.write('. $percent %');
    return Semantics(
      container: true,
      liveRegion: true,
      label: semantics.toString(),
      value: percent == null ? null : '$percent %',
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: colors.surfaceSecondary,
          border: Border.all(color: colors.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.panel),
        ),
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.space16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Icon(icon, size: AppIconSizes.large),
                  const SizedBox(width: AppSpacing.space12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          label,
                          style: Theme.of(context).textTheme.titleSmall,
                        ),
                        const SizedBox(height: AppSpacing.space2),
                        Text(
                          phase,
                          style: Theme.of(context).textTheme.bodySmall
                              ?.copyWith(color: colors.textSecondary),
                        ),
                      ],
                    ),
                  ),
                  if (percent != null)
                    Text(
                      '$percent %',
                      style: Theme.of(context).textTheme.labelMedium?.copyWith(
                        fontFeatures: AppTypeScale.tabularFigures,
                      ),
                    ),
                  if (onCancel != null) ...[
                    const SizedBox(width: AppSpacing.space8),
                    TextButton.icon(
                      onPressed: onCancel,
                      icon: const Icon(
                        Icons.stop_circle_outlined,
                        size: AppIconSizes.standard,
                      ),
                      label: Text(cancelLabel!),
                    ),
                  ],
                ],
              ),
              const SizedBox(height: AppSpacing.space12),
              LinearProgressIndicator(
                value: value,
                minHeight: AppBorders.progress,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
