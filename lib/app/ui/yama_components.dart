import 'package:flutter/material.dart';

import '../theme/app_tokens.dart';

/// Small, deliberately boring building blocks for the desktop design system.
/// They own hit targets, focus treatment and semantics; feature widgets only
/// decide where a control belongs.
final class YamaIconButton extends StatelessWidget {
  const YamaIconButton({
    super.key,
    required this.icon,
    required this.semanticLabel,
    required this.onPressed,
    this.tooltip,
    this.selected = false,
    this.badge,
  });

  final IconData icon;
  final String semanticLabel;
  final VoidCallback? onPressed;
  final String? tooltip;
  final bool selected;
  final Widget? badge;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final button = Semantics(
      button: true,
      selected: selected,
      label: semanticLabel,
      child: IconButton(
        tooltip: tooltip ?? semanticLabel,
        onPressed: onPressed,
        style: IconButton.styleFrom(
          minimumSize: const Size.square(AppSizes.desktopControlExtent),
          maximumSize: const Size.square(AppSizes.touchControlExtent),
          padding: EdgeInsets.zero,
          backgroundColor: selected ? colors.surfaceActive : null,
          foregroundColor: selected ? colors.iconActive : colors.iconNormal,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
        ),
        icon: Icon(icon, size: 18),
      ),
    );
    if (badge == null) return button;
    return Stack(
      clipBehavior: Clip.none,
      children: [
        button,
        Positioned(right: 1, top: 1, child: badge!),
      ],
    );
  }
}

final class YamaToolbarGroup extends StatelessWidget {
  const YamaToolbarGroup({
    super.key,
    required this.children,
    this.label,
    this.padding = const EdgeInsets.all(AppSpacing.space4),
    this.flat = false,
  });

  final List<Widget> children;
  final String? label;
  final EdgeInsets padding;
  final bool flat;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final flatSurface = flat || MediaQuery.highContrastOf(context);
    final group = DecoratedBox(
      decoration: BoxDecoration(
        color: flatSurface
            ? colors.surfaceFloating
            : colors.surfaceFloating.withValues(alpha: 0.96),
        border: Border.all(color: colors.borderSubtle),
        borderRadius: BorderRadius.circular(AppRadii.floating),
        boxShadow: flatSurface
            ? null
            : [
                BoxShadow(
                  color: colors.ambientShadow,
                  blurRadius: 14,
                  offset: const Offset(0, 4),
                ),
              ],
      ),
      child: Padding(
        padding: padding,
        child: Row(mainAxisSize: MainAxisSize.min, children: children),
      ),
    );
    if (label == null) return group;
    return Semantics(container: true, label: label, child: group);
  }
}

final class YamaSegment<T> {
  const YamaSegment({required this.value, required this.label, this.icon});

  final T value;
  final String label;
  final IconData? icon;
}

final class YamaSegmentedControl<T> extends StatelessWidget {
  const YamaSegmentedControl({
    super.key,
    required this.value,
    required this.segments,
    required this.onChanged,
    this.semanticLabel,
  });

  final T value;
  final List<YamaSegment<T>> segments;
  final ValueChanged<T> onChanged;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    return Semantics(
      container: true,
      label: semanticLabel,
      explicitChildNodes: true,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var index = 0; index < segments.length; index++) ...[
            if (index > 0) const SizedBox(width: AppSpacing.space16),
            _YamaSegmentButton<T>(
              segment: segments[index],
              selected: segments[index].value == value,
              colors: colors,
              onPressed: () => onChanged(segments[index].value),
            ),
          ],
        ],
      ),
    );
  }
}

final class _YamaSegmentButton<T> extends StatelessWidget {
  const _YamaSegmentButton({
    required this.segment,
    required this.selected,
    required this.colors,
    required this.onPressed,
  });

  final YamaSegment<T> segment;
  final bool selected;
  final AppColorTokens colors;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final foreground = selected ? colors.textPrimary : colors.textSecondary;
    return Semantics(
      button: true,
      selected: selected,
      label: segment.label,
      child: InkWell(
        onTap: onPressed,
        borderRadius: BorderRadius.circular(AppRadii.control),
        child: Container(
          constraints: const BoxConstraints(minHeight: 40),
          padding: const EdgeInsets.fromLTRB(4, 4, 4, 2),
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: selected ? colors.textPrimary : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (segment.icon != null) ...[
                Icon(segment.icon, size: 16, color: foreground),
                const SizedBox(width: AppSpacing.space6),
              ],
              Text(
                segment.label,
                style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: foreground,
                  fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

final class YamaSettingRow extends StatelessWidget {
  const YamaSettingRow({
    super.key,
    required this.label,
    required this.control,
    this.description,
    this.enabled = true,
    this.dirty = false,
  });

  final String label;
  final String? description;
  final Widget control;
  final bool enabled;
  final bool dirty;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    return LayoutBuilder(
      builder: (context, constraints) {
        final stacked = constraints.maxWidth < 560;
        final labelWidget = Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    label,
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w500,
                      color: enabled ? colors.textPrimary : colors.textDisabled,
                    ),
                  ),
                  if (description != null) ...[
                    const SizedBox(height: AppSpacing.space4),
                    Text(
                      description!,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: enabled
                            ? colors.textSecondary
                            : colors.textDisabled,
                      ),
                    ),
                  ],
                ],
              ),
            ),
            if (dirty)
              Padding(
                padding: const EdgeInsets.only(left: AppSpacing.space8),
                child: Tooltip(
                  message: 'Distinto del valor predeterminado',
                  child: Icon(Icons.circle, size: 7, color: colors.accentPrimary),
                ),
              ),
          ],
        );
        return ConstrainedBox(
          constraints: const BoxConstraints(minHeight: 56),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: AppSpacing.space8),
            child: stacked
                ? Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      labelWidget,
                      const SizedBox(height: AppSpacing.space8),
                      Align(alignment: Alignment.centerLeft, child: control),
                    ],
                  )
                : Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: [
                      Expanded(child: labelWidget),
                      const SizedBox(width: AppSpacing.space24),
                      control,
                    ],
                  ),
          ),
        );
      },
    );
  }
}

final class YamaSection extends StatelessWidget {
  const YamaSection({
    super.key,
    required this.title,
    this.description,
    required this.child,
  });

  final String title;
  final String? description;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleLarge),
        if (description != null) ...[
          const SizedBox(height: AppSpacing.space4),
          Text(description!, style: Theme.of(context).textTheme.bodySmall),
        ],
        const SizedBox(height: AppSpacing.space12),
        DecoratedBox(
          decoration: BoxDecoration(
            color: colors.surfaceElevated,
            border: Border.all(color: colors.borderSubtle),
            borderRadius: BorderRadius.circular(AppRadii.floating),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space16),
            child: child,
          ),
        ),
      ],
    );
  }
}

final class YamaPanelSurface extends StatelessWidget {
  const YamaPanelSurface({super.key, required this.child, this.label});

  final Widget child;
  final String? label;

  @override
  Widget build(BuildContext context) {
    final surface = DecoratedBox(
      decoration: BoxDecoration(
        color: context.appColors.surfaceElevated,
        border: Border(left: BorderSide(color: context.appColors.borderStrong)),
        boxShadow: [
          BoxShadow(
            color: context.appColors.ambientShadow,
            blurRadius: 14,
            offset: const Offset(-2, 0),
          ),
        ],
      ),
      child: child,
    );
    return label == null
        ? surface
        : Semantics(container: true, label: label, child: surface);
  }
}

final class YamaStatusSurface extends StatelessWidget {
  const YamaStatusSurface({
    super.key,
    required this.title,
    required this.message,
    this.icon = Icons.info_outline_rounded,
    this.action,
    this.actionLabel,
  });

  final String title;
  final String message;
  final IconData icon;
  final VoidCallback? action;
  final String? actionLabel;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 440),
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.space24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 24, color: context.appColors.iconNormal),
              const SizedBox(height: AppSpacing.space12),
              Text(title, style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: AppSpacing.space8),
              Text(message, textAlign: TextAlign.center),
              if (action != null && actionLabel != null) ...[
                const SizedBox(height: AppSpacing.space16),
                FilledButton(onPressed: action, child: Text(actionLabel!)),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
