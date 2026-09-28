import 'package:flutter/material.dart';

import '../theme/app_density.dart';
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
    this.statesController,
  });

  final IconData icon;
  final String semanticLabel;
  final VoidCallback? onPressed;
  final String? tooltip;
  final bool selected;
  final Widget? badge;
  final WidgetStatesController? statesController;

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
        statesController: statesController,
        style: ButtonStyle(
          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          minimumSize: const WidgetStatePropertyAll(
            Size.square(AppSizes.desktopControlExtent),
          ),
          maximumSize: const WidgetStatePropertyAll(
            Size.square(AppSizes.touchControlExtent),
          ),
          padding: const WidgetStatePropertyAll(EdgeInsets.zero),
          backgroundColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.pressed)) {
              return colors.surfacePressed;
            }
            if (selected) return colors.surfaceActive;
            if (states.contains(WidgetState.hovered)) {
              return colors.surfaceHover;
            }
            return Colors.transparent;
          }),
          foregroundColor: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.disabled)) {
              return colors.iconDisabled;
            }
            return selected ? colors.iconActive : colors.iconNormal;
          }),
          side: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.focused)) {
              return BorderSide(
                color: colors.focusRing,
                width: AppBorders.focus,
              );
            }
            return BorderSide(
              color: selected ? colors.borderStrong : Colors.transparent,
            );
          }),
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(AppRadii.control),
            ),
          ),
        ),
        icon: Icon(icon, size: AppIconSizes.standard),
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
    this.mainAxisSize = MainAxisSize.min,
    this.radius = AppRadii.floating,
    this.bare = false,
  });

  final List<Widget> children;
  final String? label;
  final EdgeInsets padding;
  final bool flat;
  final MainAxisSize mainAxisSize;
  final double radius;
  final bool bare;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final flatSurface = flat || MediaQuery.highContrastOf(context);
    final row = Padding(
      padding: padding,
      child: Row(mainAxisSize: mainAxisSize, children: children),
    );
    final group = bare
        ? row
        : DecoratedBox(
            decoration: BoxDecoration(
              color: flatSurface
                  ? colors.surfaceFloating
                  : colors.surfaceFloating.withValues(
                      alpha: AppOpacity.floatingSurface,
                    ),
              border: Border.all(color: colors.borderSubtle),
              borderRadius: BorderRadius.circular(radius),
              boxShadow: flatSurface
                  ? null
                  : [
                      BoxShadow(
                        color: colors.ambientShadow,
                        blurRadius: AppShadow.floatingControlBlur,
                        offset: const Offset(
                          0,
                          AppShadow.floatingControlOffset,
                        ),
                      ),
                    ],
            ),
            child: row,
          );
    if (label == null) return group;
    return Semantics(container: true, label: label, child: group);
  }
}

final class YamaToolbarDivider extends StatelessWidget {
  const YamaToolbarDivider({super.key, this.height = AppSpacing.space24});

  final double height;

  @override
  Widget build(BuildContext context) => ExcludeSemantics(
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space6),
      child: SizedBox(
        height: height,
        child: VerticalDivider(
          width: AppBorders.hairline,
          thickness: AppBorders.hairline,
          color: context.appColors.borderSubtle,
        ),
      ),
    ),
  );
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
          borderRadius: BorderRadius.circular(AppRadii.control),
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
        borderRadius: BorderRadius.circular(AppRadii.control),
        child: AnimatedContainer(
          duration: duration,
          curve: AppCurves.feedback,
          constraints: const BoxConstraints(minHeight: AppControlHeights.dense),
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.space12,
            vertical: AppSpacing.space6,
          ),
          decoration: BoxDecoration(
            color: selected ? colors.surfaceActive : Colors.transparent,
            borderRadius: BorderRadius.circular(AppRadii.control),
            border: Border.all(
              color: selected ? colors.borderStrong : Colors.transparent,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (segment.icon != null) ...[
                Icon(segment.icon, size: AppIconSizes.small, color: foreground),
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
        final density = context.appDensity;
        return DecoratedBox(
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: colors.borderSubtle)),
          ),
          child: ConstrainedBox(
            constraints: BoxConstraints(minHeight: density.settingRowMinHeight),
            child: Padding(
              padding: EdgeInsets.symmetric(
                vertical: density.settingsRowVerticalPadding,
              ),
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
                        SizedBox(
                          width: AppSizes.settingsControlWidth,
                          child: Align(
                            alignment: Alignment.centerRight,
                            child: control,
                          ),
                        ),
                      ],
                    ),
            ),
          ),
        );
      },
    );
  }
}
