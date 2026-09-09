part of 'settings_screen.dart';

final class _StorageSummary extends StatelessWidget {
  const _StorageSummary({
    required this.service,
    required this.onClearRebuildable,
    required this.onClearBackups,
  });

  final LocalStorageService service;
  final Future<void> Function() onClearRebuildable;
  final Future<void> Function() onClearBackups;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return FutureBuilder(
      future: service.measure(),
      builder: (context, snapshot) {
        final entries = snapshot.data;
        if (entries is! List<StorageEntry>) {
          return ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(strings.get('settings.localStorage')),
            subtitle: Text(strings.get('settings.measuringStorage')),
          );
        }
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: AppSpacing.space12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                strings.get('settings.localStorage'),
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: AppSpacing.space8),
              for (final entry in entries)
                ListTile(
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                  title: Text(
                    strings.get('storage.${entry.category.name}.title'),
                  ),
                  subtitle: Text(
                    strings.get('storage.${entry.category.name}.description'),
                  ),
                  trailing: Text(_formatStorageBytes(entry.bytes)),
                ),
              const SizedBox(height: AppSpacing.space4),
              Text(
                strings.get('settings.originalsNotDeleted'),
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: AppSpacing.space8),
              Wrap(
                spacing: AppSpacing.space8,
                runSpacing: AppSpacing.space8,
                children: [
                  OutlinedButton(
                    onPressed: onClearRebuildable,
                    child: Text(strings.get('settings.clearRebuildable')),
                  ),
                  OutlinedButton(
                    onPressed: onClearBackups,
                    child: Text(strings.get('settings.deleteBackups')),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}

enum SettingsSectionId {
  appearance(Icons.contrast_rounded),
  reading(Icons.menu_book_outlined),
  accessibility(Icons.accessibility_new_rounded),
  library(Icons.library_books_outlined),
  privacy(Icons.shield_outlined),
  storage(Icons.storage_outlined),
  recovery(Icons.restore_page_outlined),
  diagnostics(Icons.health_and_safety_outlined);

  const SettingsSectionId(this.icon);

  final IconData icon;

  String label(YamantakaLocalizations strings) =>
      strings.get('settings.section.$name');

  SettingsNavGroup get group => switch (this) {
    SettingsSectionId.appearance ||
    SettingsSectionId.reading ||
    SettingsSectionId.accessibility => SettingsNavGroup.preferences,
    SettingsSectionId.library => SettingsNavGroup.library,
    SettingsSectionId.privacy ||
    SettingsSectionId.storage ||
    SettingsSectionId.recovery => SettingsNavGroup.safety,
    SettingsSectionId.diagnostics => SettingsNavGroup.support,
  };
}

enum SettingsNavGroup { preferences, library, safety, support }

final class _SettingsSectionData {
  const _SettingsSectionData({
    required this.id,
    required this.description,
    required this.children,
    this.onReset,
  });

  final SettingsSectionId id;
  final String description;
  final List<Widget> children;
  final VoidCallback? onReset;
}

final class _SettingsSearchField extends StatelessWidget {
  const _SettingsSearchField({
    required this.controller,
    required this.query,
    required this.onChanged,
    required this.onClear,
  });

  final TextEditingController controller;
  final String query;
  final ValueChanged<String> onChanged;
  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.appColors.surfaceSecondary,
        border: Border(
          bottom: BorderSide(color: context.appColors.borderSubtle),
        ),
      ),
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 720),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: AppSpacing.space20,
              vertical: AppSpacing.space8,
            ),
            child: TextField(
              key: const Key('settings-search-field'),
              controller: controller,
              decoration: InputDecoration(
                prefixIcon: const Icon(Icons.search_rounded),
                hintText: strings.get('settings.search'),
                suffixIcon: query.isEmpty
                    ? null
                    : IconButton(
                        tooltip: strings.get('action.clear'),
                        onPressed: onClear,
                        icon: const Icon(Icons.clear_rounded),
                      ),
              ),
              textInputAction: TextInputAction.search,
              onChanged: onChanged,
            ),
          ),
        ),
      ),
    );
  }
}

final class _DesktopSettings extends StatelessWidget {
  const _DesktopSettings({
    required this.sections,
    required this.settings,
    required this.selected,
    required this.onSelected,
  });

  final List<_SettingsSectionData> sections;
  final AppSettings settings;
  final SettingsSectionId selected;
  final ValueChanged<SettingsSectionId> onSelected;

  @override
  Widget build(BuildContext context) {
    final selectedSection = sections.firstWhere(
      (section) => section.id == selected,
    );
    final navigationSections = sections;
    final navigationWidth =
        (AppSizes.settingsNavigationWidth *
                MediaQuery.textScalerOf(context).scale(1).clamp(1, 1.35))
            .clamp(
              AppSizes.settingsNavigationWidth,
              AppSizes.settingsNavigationMaxWidth,
            )
            .toDouble();
    return Align(
      alignment: AlignmentDirectional.topStart,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1600),
        child: Row(
          children: [
            Container(
              width: navigationWidth,
              decoration: BoxDecoration(
                color: context.appColors.surfaceSecondary,
                border: BorderDirectional(
                  end: BorderSide(color: context.appColors.borderSubtle),
                ),
              ),
              child: ListView(
                padding: const EdgeInsets.fromLTRB(
                  AppSpacing.space16,
                  AppSpacing.space24,
                  AppSpacing.space16,
                  AppSpacing.space32,
                ),
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: AppSpacing.space12,
                    ),
                    child: Text(
                      YamantakaLocalizations.of(
                        context,
                      ).get('settings.localPreferences'),
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: context.appColors.textMuted,
                      ),
                    ),
                  ),
                  const SizedBox(height: AppSpacing.space12),
                  for (
                    var index = 0;
                    index < navigationSections.length;
                    index++
                  ) ...[
                    if (index == 0 ||
                        navigationSections[index - 1].id.group !=
                            navigationSections[index].id.group)
                      _SettingsNavGroupLabel(
                        group: navigationSections[index].id.group,
                      ),
                    _SettingsNavItem(
                      section: navigationSections[index].id,
                      selected: selected == navigationSections[index].id,
                      onPressed: () => onSelected(navigationSections[index].id),
                    ),
                  ],
                ],
              ),
            ),
            Expanded(
              child: _SettingsSectionView(
                section: selectedSection,
                padding: EdgeInsets.fromLTRB(
                  AppSpacing.pagePadding(MediaQuery.sizeOf(context).width),
                  AppSpacing.space32,
                  AppSpacing.pagePadding(MediaQuery.sizeOf(context).width),
                  AppSpacing.space64,
                ),
              ),
            ),
            Container(
              width: AppSizes.settingsPreviewWidth,
              decoration: BoxDecoration(
                color: context.appColors.surfaceSecondary,
                border: BorderDirectional(
                  start: BorderSide(color: context.appColors.borderSubtle),
                ),
              ),
              child: ListView(
                padding: const EdgeInsets.all(AppSpacing.space20),
                children: [
                  _AppearancePreview(settings: settings, inline: false),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _CompactSettings extends StatelessWidget {
  const _CompactSettings({required this.sections});

  final List<_SettingsSectionData> sections;

  @override
  Widget build(BuildContext context) =>
      _CompactSettingsNavigator(sections: sections);
}

final class _CompactSettingsNavigator extends StatefulWidget {
  const _CompactSettingsNavigator({required this.sections});

  final List<_SettingsSectionData> sections;

  @override
  State<_CompactSettingsNavigator> createState() =>
      _CompactSettingsNavigatorState();
}

final class _CompactSettingsNavigatorState
    extends State<_CompactSettingsNavigator> {
  SettingsSectionId? _selected;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final selected = _selected;
    if (selected != null) {
      final section = widget.sections.firstWhere((item) => item.id == selected);
      return ListView(
        padding: const EdgeInsets.fromLTRB(
          AppSpacing.space20,
          AppSpacing.space12,
          AppSpacing.space20,
          AppSpacing.space48,
        ),
        children: [
          TextButton.icon(
            onPressed: () => setState(() => _selected = null),
            icon: const Icon(
              Icons.arrow_back_rounded,
              size: AppIconSizes.standard,
            ),
            label: Text(strings.get('settings.allCategories')),
            style: TextButton.styleFrom(alignment: Alignment.centerLeft),
          ),
          const SizedBox(height: AppSpacing.space12),
          _SettingsSectionContent(section: section),
        ],
      );
    }

    return ListView(
      padding: const EdgeInsets.fromLTRB(
        AppSpacing.space20,
        AppSpacing.space24,
        AppSpacing.space20,
        AppSpacing.space48,
      ),
      children: [
        AppSectionHeader(
          title: strings.get('settings.localPreferences'),
          description: strings.get('settings.localPreferences.description'),
        ),
        const SizedBox(height: AppSpacing.space20),
        for (final section in widget.sections)
          Padding(
            padding: const EdgeInsets.only(bottom: AppSpacing.space8),
            child: _SettingsNavItem(
              section: section.id,
              selected: false,
              onPressed: () => setState(() => _selected = section.id),
            ),
          ),
      ],
    );
  }
}

final class _SettingsSectionView extends StatelessWidget {
  const _SettingsSectionView({required this.section, required this.padding});

  final _SettingsSectionData section;
  final EdgeInsets padding;

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.topCenter,
      child: ConstrainedBox(
        constraints: const BoxConstraints(
          maxWidth: AppSizes.settingsContentMaxWidth,
        ),
        child: ListView(
          padding: padding,
          children: [_SettingsSectionContent(section: section)],
        ),
      ),
    );
  }
}

final class _SettingsSectionContent extends StatelessWidget {
  const _SettingsSectionContent({required this.section});

  final _SettingsSectionData section;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Semantics(
          header: true,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                section.id.label(YamantakaLocalizations.of(context)),
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: AppSpacing.space4),
              Text(
                section.description,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ],
          ),
        ),
        const SizedBox(height: AppSpacing.space12),
        Text(
          YamantakaLocalizations.of(context).get('settings.globalScope'),
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: context.appColors.textSecondary,
          ),
        ),
        const SizedBox(height: AppSpacing.space24),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: AppSpacing.space4),
          child: Column(
            children: [for (final child in section.children) child],
          ),
        ),
        const SizedBox(height: AppSpacing.space24),
        if (section.onReset != null) ...[
          const SizedBox(height: AppSpacing.space8),
          TextButton.icon(
            key: ValueKey('settings-reset-${section.id.name}'),
            onPressed: section.onReset,
            icon: const Icon(
              Icons.restart_alt_rounded,
              size: AppIconSizes.standard,
            ),
            label: Text(
              YamantakaLocalizations.of(context).get('settings.resetSection'),
            ),
          ),
        ],
      ],
    );
  }
}

final class _SettingsNavItem extends StatelessWidget {
  const _SettingsNavItem({
    required this.section,
    required this.selected,
    required this.onPressed,
  });

  final SettingsSectionId section;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final label = section.label(strings);
    final foreground = selected
        ? context.appColors.textPrimary
        : context.appColors.textSecondary;
    return Semantics(
      button: true,
      selected: selected,
      label: label,
      child: Padding(
        padding: const EdgeInsets.only(bottom: AppSpacing.space4),
        child: Material(
          color: selected
              ? context.appColors.selectionBackground
              : Colors.transparent,
          borderRadius: BorderRadius.circular(AppRadii.control),
          child: InkWell(
            borderRadius: BorderRadius.circular(AppRadii.control),
            onTap: onPressed,
            child: Container(
              constraints: const BoxConstraints(
                minHeight: AppSizes.desktopControlExtent,
              ),
              padding: const EdgeInsets.symmetric(
                horizontal: AppSpacing.space12,
                vertical: AppSpacing.space8,
              ),
              child: Row(
                children: [
                  AnimatedContainer(
                    duration: AppMotion.whenAllowed(
                      MediaQuery.disableAnimationsOf(context),
                      AppMotion.fast,
                    ),
                    width: 3,
                    height: selected ? 18 : 3,
                    decoration: BoxDecoration(
                      color: selected
                          ? context.appColors.accentPrimary
                          : Colors.transparent,
                      borderRadius: BorderRadius.circular(AppRadii.indicator),
                    ),
                  ),
                  const SizedBox(width: AppSpacing.space8),
                  Icon(
                    section.icon,
                    size: AppIconSizes.standard,
                    color: foreground,
                  ),
                  const SizedBox(width: AppSpacing.space12),
                  Expanded(
                    child: Text(
                      label,
                      style: Theme.of(
                        context,
                      ).textTheme.bodyMedium?.copyWith(color: foreground),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _SettingsNavGroupLabel extends StatelessWidget {
  const _SettingsNavGroupLabel({required this.group});

  final SettingsNavGroup group;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        AppSpacing.space12,
        AppSpacing.space12,
        AppSpacing.space12,
        AppSpacing.space6,
      ),
      child: Text(
        YamantakaLocalizations.of(
          context,
        ).get('settings.navigationGroup.${group.name}'),
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
          color: context.appColors.textMuted,
          letterSpacing: 0.5,
        ),
      ),
    );
  }
}

enum _PreviewMode { resting, active, panel, electronicInk }

final class _AppearancePreview extends StatefulWidget {
  const _AppearancePreview({required this.settings, this.inline = true});

  final AppSettings settings;
  final bool inline;

  @override
  State<_AppearancePreview> createState() => _AppearancePreviewState();
}

final class _AppearancePreviewState extends State<_AppearancePreview> {
  _PreviewMode _mode = _PreviewMode.resting;

  @override
  Widget build(BuildContext context) {
    if (widget.inline && context.windowClass == AppWindowClass.expanded) {
      return const SizedBox.shrink();
    }
    final strings = YamantakaLocalizations.of(context);
    final readerTheme = ReaderTheme.values
        .where((theme) => theme.name == widget.settings.defaultReaderTheme)
        .firstOrNull;
    return Semantics(
      label: strings.get(
        'settings.appearancePreview',
        parameters: {
          'theme': strings.get(
            widget.settings.darkMode
                ? 'settings.value.dark'
                : 'settings.value.light',
          ),
          'density': strings.get(
            'settings.value.${widget.settings.interfaceDensity.name}',
          ),
          'canvas': strings.get(
            'settings.value.${readerTheme?.name ?? 'ivory'}',
          ),
        },
      ),
      child: Container(
        key: const Key('appearance-preview'),
        padding: EdgeInsets.all(
          widget.settings.interfaceDensity == AppInterfaceDensity.compact
              ? AppSpacing.space8
              : AppSpacing.space12,
        ),
        decoration: BoxDecoration(
          color: context.appColors.surfaceSecondary,
          border: Border.all(color: context.appColors.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.control),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              strings.get('settings.preview'),
              style: Theme.of(context).textTheme.titleSmall,
            ),
            const SizedBox(height: AppSpacing.space4),
            Text(
              strings.get('settings.preview.description'),
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: AppSpacing.space12),
            _PreviewModeSelector(
              value: _mode,
              onChanged: (value) => setState(() => _mode = value),
            ),
            const SizedBox(height: AppSpacing.space8),
            ExcludeSemantics(
              child: _LiveReaderPreview(
                settings: widget.settings,
                readerTheme: readerTheme ?? ReaderTheme.ivory,
                mode: _mode,
              ),
            ),
            const SizedBox(height: AppSpacing.space12),
            Wrap(
              spacing: AppSpacing.space6,
              runSpacing: AppSpacing.space4,
              children: [
                _PreviewLabel(
                  label: strings.get(
                    'settings.value.${widget.settings.interfaceDensity.name}',
                  ),
                ),
                _PreviewLabel(
                  label: strings.get(
                    'settings.value.${readerTheme?.name ?? 'ivory'}',
                  ),
                ),
                _PreviewLabel(
                  label: strings.get(
                    'settings.value.${widget.settings.visualProfile.name}',
                  ),
                ),
                _PreviewLabel(
                  label: strings.get(
                    'settings.value.${widget.settings.resolvedMotionPreference.name == 'full' ? 'motionFull' : 'motionReduced'}',
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

final class _PreviewModeSelector extends StatelessWidget {
  const _PreviewModeSelector({required this.value, required this.onChanged});

  final _PreviewMode value;
  final ValueChanged<_PreviewMode> onChanged;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    String label(_PreviewMode mode) =>
        strings.get('settings.previewMode.${mode.name}');
    IconData icon(_PreviewMode mode) => switch (mode) {
      _PreviewMode.resting => Icons.horizontal_rule_rounded,
      _PreviewMode.active => Icons.tune_rounded,
      _PreviewMode.panel => Icons.view_sidebar_outlined,
      _PreviewMode.electronicInk => Icons.draw_outlined,
    };
    return Semantics(
      container: true,
      label: strings.get('settings.previewMode'),
      child: Row(
        children: [
          for (final mode in _PreviewMode.values)
            Expanded(
              child: YamaIconButton(
                key: ValueKey('settings-preview-mode-${mode.name}'),
                icon: icon(mode),
                semanticLabel: label(mode),
                tooltip: label(mode),
                selected: value == mode,
                onPressed: () => onChanged(mode),
              ),
            ),
        ],
      ),
    );
  }
}

final class _LiveReaderPreview extends StatelessWidget {
  const _LiveReaderPreview({
    required this.settings,
    required this.readerTheme,
    required this.mode,
  });

  final AppSettings settings;
  final ReaderTheme readerTheme;
  final _PreviewMode mode;

  @override
  Widget build(BuildContext context) {
    final eInk =
        mode == _PreviewMode.electronicInk ||
        settings.visualProfile == ReaderVisualProfile.electronicInk;
    final compact = settings.interfaceDensity == AppInterfaceDensity.compact;
    final canvas = eInk ? Colors.white : Color(readerTheme.colorValue);
    final showActiveChrome =
        mode == _PreviewMode.active || mode == _PreviewMode.panel;
    final showPanel = mode == _PreviewMode.panel;
    final highContrast = MediaQuery.highContrastOf(context);
    return Container(
      key: const Key('appearance-live-reader-preview'),
      height: inlinePreviewHeight,
      width: double.infinity,
      decoration: BoxDecoration(
        color: canvas,
        border: Border.all(
          color: highContrast
              ? context.appColors.focusRing
              : context.appColors.borderStrong,
          width: highContrast ? 2 : 1,
        ),
        borderRadius: BorderRadius.circular(AppRadii.control),
      ),
      child: Stack(
        alignment: Alignment.center,
        children: [
          Container(
            width: compact ? 132 : 144,
            height: compact ? 174 : 190,
            padding: const EdgeInsets.all(AppSpacing.space12),
            decoration: BoxDecoration(
              color: Colors.white,
              border: Border.all(color: const Color(0x22000000)),
              boxShadow: eInk
                  ? null
                  : [
                      BoxShadow(
                        color: context.appColors.readerPageShadow,
                        blurRadius: AppShadow.readerPageBlur,
                        offset: const Offset(0, 1),
                      ),
                    ],
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Yamantaka',
                  textScaler: TextScaler.noScaling,
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                    color: Colors.black87,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: AppSpacing.space12),
                for (final width in [96.0, 108.0, 104.0, 80.0, 96.0, 68.0]) ...[
                  Container(width: width, height: 2, color: Colors.black38),
                  const SizedBox(height: AppSpacing.space4),
                ],
              ],
            ),
          ),
          if (showPanel)
            Positioned(
              top: AppSpacing.space8,
              right: AppSpacing.space8,
              bottom: AppSpacing.space40,
              child: Container(
                key: const Key('appearance-preview-panel'),
                width: 64,
                padding: const EdgeInsets.all(AppSpacing.space6),
                decoration: BoxDecoration(
                  color: context.appColors.surfacePrimary,
                  border: Border.all(color: context.appColors.borderStrong),
                  borderRadius: BorderRadius.circular(AppRadii.control),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Container(
                      width: 22,
                      height: 3,
                      color: context.appColors.accentPrimary,
                    ),
                    const SizedBox(height: AppSpacing.space8),
                    for (final width in [42.0, 34.0, 46.0]) ...[
                      Container(
                        width: width,
                        height: 2,
                        color: context.appColors.textMuted,
                      ),
                      const SizedBox(height: AppSpacing.space6),
                    ],
                  ],
                ),
              ),
            ),
          Positioned(
            left: AppSpacing.space8,
            right: AppSpacing.space8,
            bottom: AppSpacing.space8,
            child: IgnorePointer(
              child: !showActiveChrome
                  ? Container(
                      height: 24,
                      padding: const EdgeInsets.symmetric(
                        horizontal: AppSpacing.space8,
                      ),
                      decoration: BoxDecoration(
                        color: context.appColors.surfaceFloating,
                        border: Border.all(
                          color: context.appColors.borderSubtle,
                        ),
                        borderRadius: BorderRadius.circular(AppRadii.floating),
                      ),
                      child: Row(
                        children: [
                          if (settings.showProgress)
                            Expanded(
                              child: LinearProgressIndicator(
                                value: 0.42,
                                minHeight: 2,
                                backgroundColor: context.appColors.borderSubtle,
                              ),
                            ),
                          if (settings.showProgress)
                            const SizedBox(width: AppSpacing.space8),
                          SizedBox(
                            width: 30,
                            child: FittedBox(
                              fit: BoxFit.scaleDown,
                              child: Text(
                                '42 %',
                                style: Theme.of(context).textTheme.labelSmall,
                              ),
                            ),
                          ),
                        ],
                      ),
                    )
                  : FittedBox(
                      alignment: Alignment.center,
                      child: YamaToolbarGroup(
                        label: 'Preview controls',
                        flat: eInk,
                        children: [
                          YamaIconButton(
                            icon: Icons.chevron_left_rounded,
                            semanticLabel: 'Previous',
                            onPressed: () {},
                          ),
                          Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: AppSpacing.space8,
                            ),
                            child: Text(
                              '4 / 12',
                              style: Theme.of(context).textTheme.labelSmall,
                            ),
                          ),
                          YamaIconButton(
                            icon: Icons.chevron_right_rounded,
                            semanticLabel: 'Next',
                            onPressed: () {},
                          ),
                        ],
                      ),
                    ),
            ),
          ),
        ],
      ),
    );
  }

  static const double inlinePreviewHeight = 248;
}

final class _PreviewLabel extends StatelessWidget {
  const _PreviewLabel({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.appColors.surfaceSunken,
        borderRadius: BorderRadius.circular(AppRadii.indicator),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space6,
          vertical: AppSpacing.space2,
        ),
        child: Text(label, style: Theme.of(context).textTheme.labelSmall),
      ),
    );
  }
}

final class _SettingRow extends StatelessWidget {
  const _SettingRow({
    required this.label,
    required this.control,
    this.description,
  });

  final String label;
  final String? description;
  final Widget control;

  @override
  Widget build(BuildContext context) {
    return YamaSettingRow(
      label: label,
      description: description,
      control: control,
    );
  }
}

final class _SwitchSetting extends StatelessWidget {
  const _SwitchSetting({
    required this.label,
    required this.value,
    required this.onChanged,
    this.description,
  });

  final String label;
  final String? description;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: Semantics(
        label: label,
        child: Switch(value: value, onChanged: onChanged),
      ),
    );
  }
}

final class _DropdownSetting extends StatelessWidget {
  const _DropdownSetting({
    required this.label,
    required this.value,
    required this.values,
    required this.onChanged,
    this.description,
  });

  final String label;
  final String? description;
  final String value;
  final Map<String, String> values;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: SizedBox(
        width: 224,
        child: DropdownButtonFormField<String>(
          style: Theme.of(context).textTheme.bodyMedium,
          initialValue: value,
          isExpanded: true,
          decoration: const InputDecoration(
            contentPadding: EdgeInsets.symmetric(
              horizontal: AppSpacing.space12,
              vertical: AppSpacing.space8,
            ),
          ),
          items: values.entries
              .map(
                (entry) => DropdownMenuItem(
                  value: entry.key,
                  child: Text(entry.value),
                ),
              )
              .toList(growable: false),
          onChanged: (next) {
            if (next != null) onChanged(next);
          },
        ),
      ),
    );
  }
}

final class _SliderSetting extends StatelessWidget {
  const _SliderSetting({
    required this.label,
    required this.description,
    required this.value,
    required this.onChanged,
    this.min = 8,
    this.max = 72,
    this.divisions = 8,
  });

  final String label;
  final String description;
  final double value;
  final ValueChanged<double> onChanged;
  final double min;
  final double max;
  final int divisions;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: SizedBox(
        width: 224,
        child: Row(
          children: [
            Expanded(
              child: Slider(
                value: value,
                min: min,
                max: max,
                divisions: divisions,
                label: '${value.round()} px',
                onChanged: onChanged,
              ),
            ),
            SizedBox(
              width: 48,
              child: Text(
                '${value.round()} px',
                textAlign: TextAlign.end,
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _ShortcutSetting extends StatelessWidget {
  const _ShortcutSetting({
    required this.commandId,
    required this.value,
    required this.onChanged,
  });

  final AppCommandId commandId;
  final String value;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    final definition = AppCommandRegistry.definitions.firstWhere(
      (item) => item.id == commandId,
    );
    return _SettingRow(
      label: YamantakaLocalizations.of(
        context,
      ).get('command.${definition.id.name}'),
      description: YamantakaLocalizations.of(
        context,
      ).get('settings.shortcut.description'),
      control: SizedBox(
        width: 180,
        child: DropdownButtonFormField<String>(
          style: Theme.of(context).textTheme.bodyMedium,
          initialValue: value,
          decoration: const InputDecoration(isDense: true),
          items: [
            for (final choice in AppCommandRegistry.shortcutChoices)
              DropdownMenuItem(
                value: choice.token,
                child: Text(
                  AppCommandRegistry.shortcutChoiceLabel(
                    choice,
                    strings: YamantakaLocalizations.of(context),
                  ),
                ),
              ),
          ],
          onChanged: (next) {
            if (next != null) onChanged(next);
          },
        ),
      ),
    );
  }
}

final class _ActionSetting extends StatelessWidget {
  const _ActionSetting({
    required this.label,
    required this.description,
    required this.actionLabel,
    required this.onPressed,
    this.destructive = false,
  });

  final String label;
  final String description;
  final String actionLabel;
  final VoidCallback onPressed;
  final bool destructive;

  @override
  Widget build(BuildContext context) {
    if (destructive) {
      return Padding(
        padding: const EdgeInsets.only(top: AppSpacing.space16),
        child: AppDangerZone(
          title: label,
          description: description,
          child: OutlinedButton(
            style: OutlinedButton.styleFrom(
              foregroundColor: context.appColors.danger,
              side: BorderSide(color: context.appColors.danger),
            ),
            onPressed: onPressed,
            child: Text(actionLabel),
          ),
        ),
      );
    }
    final row = _SettingRow(
      label: label,
      description: description,
      control: OutlinedButton(onPressed: onPressed, child: Text(actionLabel)),
    );
    return row;
  }
}

String _formatStorageBytes(int bytes) {
  if (bytes < 1024) return '$bytes B';
  if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
  if (bytes < 1024 * 1024 * 1024) {
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
  return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
}

String _formatDuration(int seconds) {
  if (seconds < 60) return '$seconds s';
  final minutes = seconds ~/ 60;
  if (minutes < 60) return '$minutes min';
  final hours = minutes ~/ 60;
  final remainder = minutes % 60;
  return '$hours h ${remainder.toString().padLeft(2, '0')} min';
}
