import 'dart:async';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import '../../../app/app_services.dart';
import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../app/ui/yama_components.dart';
import '../../../core/privacy/storage_catalog.dart';
import '../../../core/privacy/local_storage_service.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../storage/repositories/app_settings_repository.dart';
import '../../../storage/backup/local_backup_service.dart';
import '../../reader/domain/reader_models.dart';
import '../../diagnostics/presentation/diagnostics_dialog.dart';
import '../../help/presentation/help_center_dialog.dart';
import 'backup_restore_preview_dialog.dart';

part 'settings_screen_components.dart';

final class SettingsScreen extends StatefulWidget {
  const SettingsScreen({
    super.key,
    required this.services,
    required this.onBack,
  });

  final AppServices services;
  final VoidCallback onBack;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

final class _SettingsScreenState extends State<SettingsScreen> {
  SettingsSectionId _selectedSection = SettingsSectionId.appearance;
  final TextEditingController _searchController = TextEditingController();
  String _searchQuery = '';

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.services.settings,
      builder: (context, _) {
        final strings = YamantakaLocalizations.of(context);
        final settings = widget.services.settings.settings;
        return Scaffold(
          backgroundColor: context.appColors.settingsBackground,
          appBar: SettingsAppBar(onBack: widget.onBack),
          body: Align(
            alignment: AlignmentDirectional.topStart,
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1600),
              child: Column(
                children: [
                  _SettingsSearchField(
                    controller: _searchController,
                    query: _searchQuery,
                    onChanged: (value) => setState(() => _searchQuery = value),
                    onClear: () {
                      _searchController.clear();
                      setState(() => _searchQuery = '');
                    },
                  ),
                  Expanded(
                    child: LayoutBuilder(
                      builder: (context, constraints) {
                        final allSections = _buildSections(settings, strings);
                        final sections = _filteredSections(
                          allSections,
                          strings,
                        );
                        if (sections.isEmpty) {
                          return AppStatusView(
                            icon: Icons.search_off_rounded,
                            title: strings.get('settings.searchNoResults'),
                            message: strings.get(
                              'settings.searchNoResults.description',
                            ),
                            action: () {
                              _searchController.clear();
                              setState(() => _searchQuery = '');
                            },
                            actionLabel: strings.get('action.clear'),
                          );
                        }
                        final selected =
                            sections.any(
                              (section) => section.id == _selectedSection,
                            )
                            ? _selectedSection
                            : sections.first.id;
                        if (AppBreakpoints.classify(constraints.maxWidth) !=
                            AppWindowClass.expanded) {
                          return _CompactSettings(sections: sections);
                        }
                        return _DesktopSettings(
                          sections: sections,
                          settings: settings,
                          selected: selected,
                          onSelected: (section) {
                            setState(() => _selectedSection = section);
                          },
                        );
                      },
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

  List<_SettingsSectionData> _filteredSections(
    List<_SettingsSectionData> sections,
    YamantakaLocalizations strings,
  ) {
    final terms = _searchQuery
        .trim()
        .toLowerCase()
        .split(RegExp(r'\s+'))
        .where((term) => term.isNotEmpty)
        .toList(growable: false);
    if (terms.isEmpty) return sections;
    return sections
        .where((section) {
          final text = _settingsSearchText(section, strings).toLowerCase();
          return terms.every(text.contains);
        })
        .toList(growable: false);
  }

  String _settingsSearchText(
    _SettingsSectionData section,
    YamantakaLocalizations strings,
  ) {
    final values = <String>[section.id.label(strings), section.description];
    for (final child in section.children) {
      switch (child) {
        case final _SwitchSetting setting:
          values.addAll([setting.label, ?setting.description]);
        case final _DropdownSetting setting:
          values.addAll([
            setting.label,
            ?setting.description,
            ...setting.values.values,
          ]);
        case final _SliderSetting setting:
          values.addAll([setting.label, setting.description]);
        case final _ActionSetting setting:
          values.addAll([setting.label, setting.description]);
        case final _ShortcutSetting setting:
          final definition = AppCommandRegistry.definitions.firstWhere(
            (item) => item.id == setting.commandId,
          );
          values.add(strings.get('command.${definition.id.name}'));
        case final _SettingRow setting:
          values.addAll([setting.label, ?setting.description]);
        default:
          break;
      }
    }
    return values.join(' ');
  }

  List<_SettingsSectionData> _buildSections(
    AppSettings settings,
    YamantakaLocalizations strings,
  ) {
    final sections = [
      _SettingsSectionData(
        id: SettingsSectionId.appearance,
        description: strings.get('settings.appearance.description'),
        children: [
          _AppearancePreview(settings: settings),
          _DropdownSetting(
            label: strings.get('settings.language'),
            description: strings.get('settings.language.description'),
            value: settings.languageCode,
            values: {
              'es': strings.get('settings.language.spanish'),
              'en': strings.get('settings.language.english'),
            },
            onChanged: (value) =>
                _update(settings.copyWith(languageCode: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.darkMode'),
            description: strings.get('settings.darkMode.description'),
            value: settings.darkMode,
            onChanged: (value) => _update(settings.copyWith(darkMode: value)),
          ),
          _DropdownSetting(
            label: strings.get('settings.density'),
            description: strings.get('settings.density.description'),
            value: settings.interfaceDensity.name,
            values: {
              'comfortable': strings.get('settings.value.comfortable'),
              'compact': strings.get('settings.value.compact'),
            },
            onChanged: (value) => _update(
              settings.copyWith(
                interfaceDensity: AppInterfaceDensity.values.byName(value),
              ),
            ),
          ),
          _DropdownSetting(
            label: strings.get('settings.defaultReaderTheme'),
            description: strings.get('settings.defaultReaderTheme.description'),
            value: settings.defaultReaderTheme,
            values: {
              'light': strings.get('settings.value.light'),
              'ivory': strings.get('settings.value.ivory'),
              'sepia': strings.get('settings.value.sepia'),
              'gray': strings.get('settings.value.gray'),
              'dark': strings.get('settings.value.dark'),
              'black': strings.get('settings.value.black'),
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultReaderTheme: value)),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.reading,
        description: strings.get('settings.reading.description'),
        children: [
          _DropdownSetting(
            label: strings.get('settings.readingProfile'),
            description: strings.get('settings.readingProfile.description'),
            value: widget.services.readingProfiles.defaultProfileId,
            values: {
              'builtin-reading': strings.get('settings.value.reading'),
              'builtin-study': strings.get('settings.value.study'),
              'builtin-presentation': strings.get(
                'settings.value.presentation',
              ),
              'builtin-night': strings.get('settings.value.night'),
              'builtin-eink': strings.get('settings.value.electronicInk'),
              'builtin-comparison': strings.get('settings.value.comparison'),
            },
            onChanged: (value) {
              widget.services.readingProfiles
                ..setDefault(value)
                ..apply(value);
              setState(() {});
            },
          ),
          _DropdownSetting(
            label: strings.get('settings.defaultView'),
            value: settings.defaultViewMode,
            values: {
              'document': strings.get('settings.value.document'),
              'book': strings.get('settings.value.book'),
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultViewMode: value)),
          ),
          _DropdownSetting(
            label: strings.get('settings.pageLayout'),
            description: strings.get('settings.pageLayout.description'),
            value: settings.defaultPageLayout,
            values: {
              'single': strings.get('settings.value.singlePage'),
              'facing': strings.get('settings.value.facingPages'),
              'continuous': strings.get('settings.value.continuous'),
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultPageLayout: value)),
          ),
          _DropdownSetting(
            label: strings.get('settings.zoomMode'),
            value: settings.defaultZoomMode,
            values: {
              'fitWidth': strings.get('settings.value.fitWidth'),
              'fitPage': strings.get('settings.value.fitPage'),
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultZoomMode: value)),
          ),
          _SliderSetting(
            label: strings.get('settings.margins'),
            description: strings.get('settings.margins.description'),
            value: settings.defaultMargin,
            onChanged: (value) =>
                _update(settings.copyWith(defaultMargin: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.restorePosition'),
            value: settings.restoreLastPosition,
            onChanged: (value) =>
                _update(settings.copyWith(restoreLastPosition: value)),
          ),
          _DropdownSetting(
            label: strings.get('settings.readingDirection'),
            value: settings.readingDirection.name,
            values: {
              'leftToRight': strings.get('settings.value.leftToRight'),
              'rightToLeft': strings.get('settings.value.rightToLeft'),
            },
            onChanged: (value) => _update(
              settings.copyWith(
                readingDirection: ReadingDirection.values.byName(value),
              ),
            ),
          ),
          _SwitchSetting(
            label: strings.get('settings.autoHideControls'),
            description: strings.get('settings.autoHideControls.description'),
            value: settings.autoHideControls,
            onChanged: (value) =>
                _update(settings.copyWith(autoHideControls: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.showProgress'),
            value: settings.showProgress,
            onChanged: (value) =>
                _update(settings.copyWith(showProgress: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.showPanelControls'),
            description: strings.get('settings.showPanelControls.description'),
            value: settings.showReaderPanelControls,
            onChanged: (value) =>
                _update(settings.copyWith(showReaderPanelControls: value)),
          ),
          _SliderSetting(
            label: strings.get('settings.panelWidth'),
            description: strings.get('settings.panelWidth.description'),
            value: settings.readerPanelWidth,
            min: AppSizes.sidePanelMinWidth,
            max: AppSizes.sidePanelMaxWidth,
            divisions: 10,
            onChanged: (value) =>
                _update(settings.copyWith(readerPanelWidth: value)),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.accessibility,
        description: strings.get('settings.accessibility.description'),
        children: [
          _DropdownSetting(
            label: strings.get('settings.motion'),
            description: strings.get('settings.motion.description'),
            value: settings.resolvedMotionPreference == AppMotionPreference.full
                ? AppMotionPreference.full.name
                : AppMotionPreference.reduced.name,
            values: {
              'full': strings.get('settings.value.motionFull'),
              'reduced': strings.get('settings.value.motionReduced'),
            },
            onChanged: (value) {
              final preference = AppMotionPreference.values.byName(value);
              _update(
                settings.copyWith(
                  motionPreference: preference,
                  reduceMotion: preference != AppMotionPreference.full,
                  disableAnimations: preference != AppMotionPreference.full,
                ),
              );
            },
          ),
          _DropdownSetting(
            label: strings.get('settings.visualProfile'),
            description: strings.get('settings.visualProfile.description'),
            value: settings.visualProfile.name,
            values: {
              'standard': strings.get('settings.value.standard'),
              'electronicInk': strings.get('settings.value.electronicInk'),
            },
            onChanged: (value) {
              final profile = ReaderVisualProfile.values.byName(value);
              _update(
                settings.copyWith(
                  visualProfile: profile,
                  reduceMotion: profile == ReaderVisualProfile.electronicInk
                      ? true
                      : settings.reduceMotion,
                  disableAnimations:
                      profile == ReaderVisualProfile.electronicInk
                      ? true
                      : settings.disableAnimations,
                  motionPreference: profile == ReaderVisualProfile.electronicInk
                      ? AppMotionPreference.none
                      : settings.motionPreference,
                  defaultReaderTheme:
                      profile == ReaderVisualProfile.electronicInk
                      ? 'light'
                      : settings.defaultReaderTheme,
                  defaultPageLayout:
                      profile == ReaderVisualProfile.electronicInk
                      ? 'single'
                      : settings.defaultPageLayout,
                ),
              );
            },
          ),
          for (final id in AppCommandRegistry.configurableIds)
            _ShortcutSetting(
              commandId: id,
              value: AppCommandRegistry.resolvedTokens(
                settings.shortcutOverrides,
              )[id]!,
              onChanged: (token) =>
                  _assignShortcut(context, settings, id, token),
            ),
          _ActionSetting(
            label: strings.get('settings.restoreShortcuts'),
            description: strings.get('settings.restoreShortcuts.description'),
            actionLabel: strings.get('action.reset'),
            onPressed: () =>
                _update(settings.copyWith(shortcutOverrides: '{}')),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.library,
        description: strings.get('settings.library.description'),
        children: [
          _SwitchSetting(
            label: strings.get('settings.showRecents'),
            value: settings.showRecents,
            onChanged: (value) =>
                _update(settings.copyWith(showRecents: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.generateCovers'),
            description: strings.get('settings.generateCovers.description'),
            value: settings.generateCovers,
            onChanged: (value) =>
                unawaited(_setCoverGeneration(settings, value)),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.privacy,
        description: strings.get('settings.privacy.description'),
        children: [
          _ActionSetting(
            label: strings.get('settings.deleteAllData'),
            description: strings.get('settings.deleteAllData.description'),
            actionLabel: strings.get('action.delete'),
            destructive: true,
            onPressed: () => _confirmAndRun(
              context,
              title: strings.get('settings.deleteAllData.confirmTitle'),
              body: strings.get('settings.deleteAllData.confirmBody'),
              action: widget.services.deleteAllLocalData,
            ),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.storage,
        description: strings.get('settings.storage.description'),
        children: [
          _DropdownSetting(
            label: strings.get('settings.maxCache'),
            value: '${settings.maxCacheMegabytes}',
            values: const {
              '64': '64 MB',
              '128': '128 MB',
              '256': '256 MB',
              '512': '512 MB',
            },
            onChanged: (value) =>
                _update(settings.copyWith(maxCacheMegabytes: int.parse(value))),
          ),
          _ActionSetting(
            label: strings.get('settings.clearCache'),
            description: strings.get('settings.clearCache.description'),
            actionLabel: strings.get('action.clear'),
            onPressed: () => _runWithNotice(
              context,
              widget.services.clearCache,
              strings.get('settings.clearCache.done'),
            ),
          ),
          _StorageSummary(
            service: widget.services.storage,
            onClearRebuildable: () async {
              await _runWithNotice(
                context,
                widget.services.clearCache,
                strings.get('settings.rebuildableCleared'),
              );
              if (mounted) setState(() {});
            },
            onClearBackups: () async {
              await _confirmAndRun(
                context,
                title: strings.get('settings.deleteBackups.confirmTitle'),
                body: strings.get('settings.deleteBackups.confirmBody'),
                action: widget.services.storage.clearBackups,
              );
              if (mounted) setState(() {});
            },
          ),
          _ActionSetting(
            label: strings.get('settings.openDataLocation'),
            description: widget.services.paths.dataDirectory.path,
            actionLabel: strings.get('action.open'),
            onPressed: widget.services.openDataLocation,
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.recovery,
        description: strings.get('settings.recovery.description'),
        children: [
          _SwitchSetting(
            label: strings.get('settings.restoreSession'),
            description: strings.get('settings.restoreSession.description'),
            value: settings.restoreLastSession,
            onChanged: (value) =>
                _update(settings.copyWith(restoreLastSession: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.startInLibrary'),
            description: strings.get('settings.startInLibrary.description'),
            value: settings.startInLibrary,
            onChanged: (value) =>
                _update(settings.copyWith(startInLibrary: value)),
          ),
          _SwitchSetting(
            label: strings.get('settings.recordSessions'),
            description: strings.get('settings.recordSessions.description'),
            value: settings.recordReadingSessions,
            onChanged: (value) =>
                _update(settings.copyWith(recordReadingSessions: value)),
          ),
          _DropdownSetting(
            label: strings.get('settings.historyLimit'),
            description: strings.get('settings.historyLimit.description'),
            value: '${settings.readingHistoryLimit}',
            values: {
              for (final count in const [50, 100, 250, 500, 1000])
                '$count': strings.count('count.sessions', count),
            },
            onChanged: (value) {
              final limit = int.parse(value);
              widget.services.readingSessions.trimToLimit(limit);
              _update(settings.copyWith(readingHistoryLimit: limit));
            },
          ),
          _ActionSetting(
            label: strings.get('settings.viewHistory'),
            description: strings.get('settings.viewHistory.description'),
            actionLabel: strings.get('settings.viewHistory.action'),
            onPressed: () => _showReadingHistory(context, settings),
          ),
          _ActionSetting(
            label: strings.get('settings.exportBackup'),
            description: strings.get('settings.exportBackup.description'),
            actionLabel: strings.get('action.export'),
            onPressed: () => _exportBackup(context),
          ),
          _ActionSetting(
            label: strings.get('settings.importBackup'),
            description: strings.get('settings.importBackup.description'),
            actionLabel: strings.get('action.import'),
            onPressed: () => _importBackup(context),
          ),
          _ActionSetting(
            label: strings.get('settings.deleteHistory'),
            description: strings.get('settings.deleteHistory.description'),
            actionLabel: strings.get('action.delete'),
            destructive: true,
            onPressed: () => _confirmAndRun(
              context,
              title: strings.get('settings.deleteHistory.confirmTitle'),
              body: strings.get('settings.deleteHistory.confirmBody'),
              action: widget.services.deleteHistory,
            ),
          ),
        ],
      ),
      _SettingsSectionData(
        id: SettingsSectionId.diagnostics,
        description: strings.get('settings.diagnosticsSection.description'),
        children: [
          _ActionSetting(
            label: strings.get('settings.diagnostics'),
            description: strings.get('settings.diagnostics.description'),
            actionLabel: strings.get('action.open'),
            onPressed: () => showDialog<void>(
              context: context,
              builder: (context) =>
                  DiagnosticsDialog(services: widget.services),
            ),
          ),
          _ActionSetting(
            label: strings.get('settings.contextualHelp'),
            description: strings.get('settings.contextualHelp.description'),
            actionLabel: strings.get('welcome.help'),
            onPressed: () => _showContextualHelp(context),
          ),
        ],
      ),
    ];
    return [
      for (final section in sections)
        _SettingsSectionData(
          id: section.id,
          description: section.description,
          children: section.children,
          onReset: _isResettableSection(section.id)
              ? () => _resetSection(section.id, settings)
              : null,
        ),
    ];
  }

  bool _isResettableSection(SettingsSectionId id) => switch (id) {
    SettingsSectionId.appearance ||
    SettingsSectionId.reading ||
    SettingsSectionId.accessibility ||
    SettingsSectionId.library ||
    SettingsSectionId.recovery => true,
    SettingsSectionId.privacy ||
    SettingsSectionId.storage ||
    SettingsSectionId.diagnostics => false,
  };

  void _resetSection(SettingsSectionId id, AppSettings settings) {
    const defaults = AppSettings();
    final next = switch (id) {
      SettingsSectionId.appearance => settings.copyWith(
        darkMode: defaults.darkMode,
        defaultReaderTheme: defaults.defaultReaderTheme,
        interfaceDensity: defaults.interfaceDensity,
        languageCode: defaults.languageCode,
      ),
      SettingsSectionId.reading => settings.copyWith(
        defaultViewMode: defaults.defaultViewMode,
        defaultPageLayout: defaults.defaultPageLayout,
        defaultZoomMode: defaults.defaultZoomMode,
        readingDirection: defaults.readingDirection,
        defaultMargin: defaults.defaultMargin,
        restoreLastPosition: defaults.restoreLastPosition,
        autoHideControls: defaults.autoHideControls,
        showProgress: defaults.showProgress,
        showReaderPanelControls: defaults.showReaderPanelControls,
        readerPanelWidth: defaults.readerPanelWidth,
      ),
      SettingsSectionId.accessibility => settings.copyWith(
        reduceMotion: defaults.reduceMotion,
        disableAnimations: defaults.disableAnimations,
        motionPreference: AppMotionPreference.full,
        visualProfile: defaults.visualProfile,
        shortcutOverrides: defaults.shortcutOverrides,
      ),
      SettingsSectionId.library => settings.copyWith(
        showRecents: defaults.showRecents,
        generateCovers: defaults.generateCovers,
        libraryReadingFilter: defaults.libraryReadingFilter,
      ),
      SettingsSectionId.recovery => settings.copyWith(
        recordReadingSessions: defaults.recordReadingSessions,
        readingHistoryLimit: defaults.readingHistoryLimit,
        restoreLastSession: defaults.restoreLastSession,
        startInLibrary: defaults.startInLibrary,
      ),
      SettingsSectionId.privacy ||
      SettingsSectionId.storage ||
      SettingsSectionId.diagnostics => settings,
    };
    if (id == SettingsSectionId.reading) {
      widget.services.readingProfiles
        ..setDefault('builtin-reading')
        ..apply('builtin-reading');
    }
    unawaited(_update(next));
  }

  Future<void> _showContextualHelp(BuildContext context) => showDialog<void>(
    context: context,
    builder: (context) =>
        HelpCenterDialog(language: YamantakaLocalizations.of(context).language),
  );

  Future<void> _assignShortcut(
    BuildContext context,
    AppSettings settings,
    AppCommandId id,
    String token,
  ) async {
    final current = AppCommandRegistry.resolvedTokens(
      settings.shortcutOverrides,
    );
    final error = AppCommandRegistry.assignmentError(
      id,
      token,
      current,
      localize: YamantakaLocalizations.of(context).get,
    );
    if (error != null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(error)));
      return;
    }
    current[id] = token;
    await _update(
      settings.copyWith(
        shortcutOverrides: AppCommandRegistry.encodeOverrides(current),
      ),
    );
  }

  Future<void> _showReadingHistory(
    BuildContext context,
    AppSettings settings,
  ) async {
    final strings = YamantakaLocalizations.of(context);
    var entries = widget.services.readingSessions.history(
      limit: settings.readingHistoryLimit,
    );
    await showDialog<void>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) {
          final children = <Widget>[];
          String? previousDay;
          for (final entry in entries) {
            final local = entry.session.startedAt.toLocal();
            final day = '${local.year}-${local.month}-${local.day}';
            if (day != previousDay) {
              previousDay = day;
              children.add(
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    AppSpacing.space16,
                    AppSpacing.space16,
                    AppSpacing.space8,
                  ),
                  child: Text(
                    MaterialLocalizations.of(context).formatFullDate(local),
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                ),
              );
            }
            children.add(
              ListTile(
                leading: const Icon(Icons.menu_book_outlined),
                title: Text(
                  entry.documentTitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                subtitle: Text(
                  strings.get(
                    'settings.historyEntry',
                    parameters: {
                      'start': entry.session.startPage + 1,
                      'end': entry.session.endPage + 1,
                      'duration': _formatDuration(entry.session.activeSeconds),
                    },
                  ),
                ),
                trailing: IconButton(
                  tooltip: strings.get('settings.deleteSession'),
                  onPressed: () {
                    widget.services.readingSessions.delete(entry.session.id);
                    setDialogState(() {
                      entries = entries
                          .where((item) => item.session.id != entry.session.id)
                          .toList(growable: false);
                    });
                  },
                  icon: const Icon(Icons.delete_outline_rounded),
                ),
              ),
            );
          }
          return AlertDialog(
            icon: const Icon(Icons.history_rounded),
            title: Text(strings.get('settings.historyTitle')),
            content: SizedBox(
              width: 620,
              height: 460,
              child: entries.isEmpty
                  ? AppStatusView(
                      icon: Icons.history_toggle_off_rounded,
                      title: strings.get('settings.noSessions'),
                      message: strings.get('settings.noSessions.description'),
                    )
                  : ListView(children: children),
            ),
            actions: [
              if (entries.isNotEmpty)
                TextButton(
                  onPressed: () {
                    widget.services.readingSessions.clear();
                    setDialogState(() => entries = const []);
                  },
                  child: Text(strings.get('action.deleteAll')),
                ),
              FilledButton(
                onPressed: () => Navigator.pop(dialogContext),
                child: Text(strings.get('action.close')),
              ),
            ],
          );
        },
      ),
    );
  }

  Future<void> _exportBackup(BuildContext context) async {
    final strings = YamantakaLocalizations.of(context);
    final type = XTypeGroup(
      label: strings.get('backup.fileType'),
      extensions: ['json'],
      mimeTypes: ['application/json'],
    );
    final stamp = DateTime.now().toIso8601String().split('T').first;
    final location = await getSaveLocation(
      suggestedName: 'yamantaka-backup-$stamp.json',
      acceptedTypeGroups: [type],
    );
    if (location == null) return;
    try {
      await widget.services.backups.exportTo(location.path);
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(strings.get('backup.exported'))));
      }
    } on Object catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              strings.get('backup.exportFailed', parameters: {'error': error}),
            ),
          ),
        );
      }
    }
  }

  Future<void> _importBackup(BuildContext context) async {
    final strings = YamantakaLocalizations.of(context);
    final type = XTypeGroup(
      label: strings.get('backup.fileType'),
      extensions: ['json'],
      mimeTypes: ['application/json'],
    );
    final selected = await openFile(acceptedTypeGroups: [type]);
    if (selected == null || !context.mounted) return;
    try {
      final preview = await widget.services.backups.preview(selected.path);
      if (!context.mounted) return;
      final confirmed = await _confirmBackupImport(context, preview);
      if (!confirmed) return;
      await widget.services.reader.closeDocument();
      final safetyPath = await widget.services.backups.importFrom(
        selected.path,
      );
      await widget.services.clearCache();
      await widget.services.library.load();
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              strings.get('backup.imported', parameters: {'path': safetyPath}),
            ),
          ),
        );
      }
    } on FormatException catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(error.message)));
      }
    } on Object {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(strings.get('backup.importFailed'))),
        );
      }
    }
  }

  Future<bool> _confirmBackupImport(
    BuildContext context,
    BackupPreview preview,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => BackupRestorePreviewDialog(preview: preview),
    );
    return confirmed ?? false;
  }

  Future<void> _update(AppSettings settings) async {
    final strings = YamantakaLocalizations.of(context);
    try {
      await widget.services.settings.updateStrict(settings);
      if (!mounted) return;
      final messenger = ScaffoldMessenger.of(context);
      messenger.hideCurrentSnackBar();
      messenger.showSnackBar(
        SnackBar(
          content: Text(strings.get('settings.updated')),
          duration: const Duration(seconds: 4),
          action: SnackBarAction(
            label: strings.get('action.undo'),
            onPressed: () async {
              await widget.services.settings.undoLastUpdate();
              if (mounted) setState(() {});
            },
          ),
        ),
      );
    } on Object catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(strings.get('settings.saveFailed')),
          action: SnackBarAction(
            label: strings.get('action.retry'),
            onPressed: () => unawaited(_update(settings)),
          ),
        ),
      );
    }
  }

  Future<void> _setCoverGeneration(AppSettings settings, bool enabled) async {
    await _update(settings.copyWith(generateCovers: enabled));
    if (!enabled) return;
    for (final document in widget.services.library.documents) {
      if (!document.isMissing) {
        widget.services.thumbnails.enqueueCover(document);
      }
    }
  }

  Future<void> _runWithNotice(
    BuildContext context,
    Future<void> Function() action,
    String message,
  ) async {
    await action();
    if (context.mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(message)));
    }
  }

  Future<void> _confirmAndRun(
    BuildContext context, {
    required String title,
    required String body,
    required Future<void> Function() action,
  }) async {
    final strings = YamantakaLocalizations.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        icon: Icon(
          Icons.delete_outline_rounded,
          color: dialogContext.appColors.danger,
        ),
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: dialogContext.appColors.danger,
            ),
            onPressed: () => Navigator.pop(dialogContext, true),
            child: Text(strings.get('action.delete')),
          ),
        ],
      ),
    );
    if (confirmed == true) await action();
  }
}

/// Lightweight, independently testable settings chrome. Keeping it separate
/// also prevents locale-only changes from rebuilding the settings sections.
final class SettingsAppBar extends StatelessWidget
    implements PreferredSizeWidget {
  const SettingsAppBar({super.key, required this.onBack});

  final VoidCallback onBack;

  @override
  Size get preferredSize => const Size.fromHeight(AppSizes.topBarHeight);

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AppBar(
      leading: IconButton(
        tooltip: strings.settingsBack,
        onPressed: onBack,
        icon: const Icon(Icons.arrow_back_rounded),
      ),
      title: Text(strings.settingsTitle),
    );
  }
}
