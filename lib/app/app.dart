import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter/services.dart';
import 'package:window_manager/window_manager.dart';

import '../core/errors/app_failure.dart';
import '../core/errors/failure_presentation.dart';
import '../core/i18n/app_localizations.dart';
import '../core/performance/startup_timeline.dart';
import '../core/platform/activation_channel.dart';
import '../features/library/domain/library_document.dart';
import '../features/library/domain/document_enrichment.dart';
import '../features/library/domain/library_selection.dart';
import '../features/library/domain/library_query.dart';
import '../features/library/domain/metadata_editor.dart';
import '../features/annotations/domain/annotation.dart';
import '../features/annotations/presentation/global_annotations_dialog.dart';
import '../features/automations/domain/local_automation.dart';
import '../features/automations/application/local_automation_engine.dart';
import '../features/automations/presentation/automation_manager_dialog.dart';
import '../features/library/presentation/library_screen.dart';
import '../features/library/presentation/document_cover_picker.dart';
import '../features/library/presentation/folder_import_dialog.dart';
import '../features/help/presentation/help_center_dialog.dart';
import '../features/help/presentation/welcome_dialog.dart';
import '../features/navigation/application/navigation_history_controller.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/domain/reader_models.dart';
import '../features/reader/domain/reading_session.dart';
import '../features/reader/presentation/reader_screen.dart';
import '../features/reader/presentation/document_comparison_dialog.dart';
import '../features/relations/presentation/document_relations_dialog.dart';
import '../features/recovery/domain/recovery_models.dart';
import '../features/recovery/presentation/problems_screen.dart';
import '../features/settings/presentation/settings_screen.dart';
import '../features/search/domain/global_search_models.dart';
import '../features/search/presentation/global_search_dialog.dart';
import '../features/workspaces/domain/workspace_models.dart';
import '../features/workspaces/presentation/document_tab_switcher.dart';
import '../features/workspaces/presentation/workspace_dialog.dart';
import '../storage/repositories/app_settings_repository.dart';
import 'app_config.dart';
import 'app_services.dart';
import 'shortcuts/app_commands.dart';
import 'theme/app_theme.dart';
import 'theme/app_tokens.dart';
import 'theme/app_components.dart';

part 'app_shell.dart';

@visibleForTesting
ThemeMode resolveAppThemeMode(AppSettings settings, {required bool safeMode}) {
  // Safe mode protects restoration and indexing; changing palette is a
  // presentation-only operation and remains available.
  return settings.darkMode ? ThemeMode.dark : ThemeMode.light;
}

final class YamantakaApp extends StatefulWidget {
  const YamantakaApp({
    super.key,
    required this.services,
    this.initialPath,
    this.initialPaths = const [],
  });

  final AppServices services;
  final String? initialPath;
  final List<String> initialPaths;

  @override
  State<YamantakaApp> createState() => _YamantakaAppState();
}

enum _AppPage { library, reader, settings, problems }

enum _DocumentInfoAction {
  editMetadata,
  acceptSuggestion,
  dismissSuggestion,
  reanalyze,
  copyPath,
  reveal,
  resetProgress,
  remove,
  relocate,
  open,
  relations,
}

final class _MetadataEditResult {
  const _MetadataEditResult({required this.metadata, required this.topics});

  final MetadataEditState metadata;
  final Set<LibraryTopic> topics;
}

enum _AnnotationExportFormat { markdown, json }

enum _RecoveryStartupAction { restore, startClean }

final class _DocumentMetadataLine extends StatelessWidget {
  const _DocumentMetadataLine({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: AppSpacing.space6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 132,
          child: Text(label, style: Theme.of(context).textTheme.labelMedium),
        ),
        Expanded(child: Text(value)),
      ],
    ),
  );
}

String _formatBytes(int bytes) {
  if (bytes < 1024) return '$bytes B';
  final kib = bytes / 1024;
  if (kib < 1024) return '${kib.toStringAsFixed(1)} KiB';
  return '${(kib / 1024).toStringAsFixed(1)} MiB';
}

String _formatLocalDate(DateTime value) {
  final local = value.toLocal();
  String two(int number) => number.toString().padLeft(2, '0');
  return '${two(local.day)}/${two(local.month)}/${local.year} '
      '${two(local.hour)}:${two(local.minute)}';
}

bool shouldRestoreReaderSession(
  AppSettings settings, {
  required bool pathExists,
  bool initialPathProvided = false,
}) {
  return !initialPathProvided &&
      settings.restoreLastSession &&
      !settings.startInLibrary &&
      settings.lastDocumentPath.isNotEmpty &&
      pathExists;
}
