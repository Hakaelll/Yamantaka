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
          side: selected ? BorderSide(color: colors.borderStrong) : null,
          shape: const CircleBorder(),
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
        borderRadius: BorderRadius.circular(AppRadii.capsule),
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
  const YamaSegment({
    required this.value,
    required this.label,
    this.icon,
    this.tooltip,
  });

  final T value;
  final String label;
  final IconData? icon;
  final String? tooltip;
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
    final control = Semantics(
      container: true,
      label: semanticLabel,
      explicitChildNodes: true,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: colors.surfaceSecondary,
          border: Border.all(color: colors.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.capsule),
        ),
        child: Padding(
          padding: const EdgeInsets.all(AppSpacing.space4),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (var index = 0; index < segments.length; index++) ...[
                if (index > 0) const SizedBox(width: AppSpacing.space2),
                _YamaSegmentButton<T>(
                  segment: segments[index],
                  selected: segments[index].value == value,
                  colors: colors,
                  onPressed: () => onChanged(segments[index].value),
                ),
              ],
            ],
          ),
        ),
      ),
    );
    return control;
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
    final duration = AppMotion.whenAllowed(
      MediaQuery.disableAnimationsOf(context),
      AppMotion.fast,
    );
    final control = Semantics(
      button: true,
      selected: selected,
      label: segment.label,
      child: InkWell(
        onTap: onPressed,
        borderRadius: BorderRadius.circular(AppRadii.capsule),
        child: AnimatedContainer(
          duration: duration,
          curve: Curves.easeOutCubic,
          constraints: const BoxConstraints(minHeight: 34),
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.space12,
            vertical: AppSpacing.space6,
          ),
          decoration: BoxDecoration(
            color: selected ? colors.surfaceActive : Colors.transparent,
            borderRadius: BorderRadius.circular(AppRadii.capsule),
            border: Border.all(
              color: selected ? colors.borderStrong : Colors.transparent,
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
    final tooltip = segment.tooltip;
    return tooltip == null
        ? control
        : Tooltip(message: tooltip, child: control);
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
                  child: Icon(
                    Icons.circle,
                    size: 7,
                    color: colors.accentPrimary,
                  ),
                ),
              ),
          ],
        );
        return DecoratedBox(
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: colors.borderSubtle)),
          ),
          child: ConstrainedBox(
            constraints: const BoxConstraints(minHeight: 60),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: AppSpacing.space12),
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
          ),
        );
      },
    );
  }
}
