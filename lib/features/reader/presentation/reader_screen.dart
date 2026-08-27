import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_theme.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../../command_palette/presentation/command_palette.dart';
import '../../library/domain/library_document.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';
import '../domain/page_destination.dart';
import 'reader_bottom_controls.dart';
import 'reader_side_panel.dart';
import 'reader_top_bar.dart';
import 'reader_ui_state.dart';

part 'reader_screen_state.dart';

final class ReaderScreen extends StatefulWidget {
  const ReaderScreen({
    super.key,
    required this.controller,
    required this.engine,
    required this.viewerAdapter,
    required this.onBack,
    required this.onOpenDocument,
    required this.onOpenSettings,
    required this.reduceMotion,
    required this.showProgress,
    required this.readingDirection,
    this.visualProfile = ReaderVisualProfile.standard,
    this.controlsAutoHide,
    this.initialPanelWidth = AppSizes.sidePanelWidth,
    this.initialPanelSection = '',
    this.initialPanelOpen = false,
    this.initialPanelPinned = false,
    this.initialChromeVisible = true,
    this.showPanelControls = true,
    this.shortcutOverrides = '{}',
    this.recentDocuments = const [],
    this.onOpenPath,
    this.onPanelWidthChanged,
    this.onPanelChanged,
    this.onChromeVisibilityChanged,
    this.onDocumentInfo,
    this.onExportAnnotations,
    this.onViewportInteractionChanged,
  });

  final ReaderController controller;
  final PdfDocumentEngine engine;
  final PdfViewerAdapter viewerAdapter;
  final Future<void> Function() onBack;
  final Future<void> Function() onOpenDocument;
  final VoidCallback onOpenSettings;
  final bool reduceMotion;
  final bool showProgress;
  final ReadingDirection readingDirection;
  final ReaderVisualProfile visualProfile;

  /// Overrides the document preference when the global setting changes while
  /// a reader is already open.
  final bool? controlsAutoHide;
  final double initialPanelWidth;
  final String initialPanelSection;
  final bool initialPanelOpen;
  final bool initialPanelPinned;
  final bool initialChromeVisible;
  final bool showPanelControls;
  final String shortcutOverrides;
  final List<LibraryDocument> recentDocuments;
  final Future<void> Function(String path)? onOpenPath;
  final ValueChanged<double>? onPanelWidthChanged;
  final ValueChanged<ReaderPanelSection?>? onPanelChanged;
  final ValueChanged<bool>? onChromeVisibilityChanged;
  final VoidCallback? onDocumentInfo;
  final Future<void> Function()? onExportAnnotations;
  final ValueChanged<bool>? onViewportInteractionChanged;

  @override
  State<ReaderScreen> createState() => _ReaderScreenState();
}

final class _ReaderThemeChoice extends StatelessWidget {
  const _ReaderThemeChoice({
    required this.theme,
    required this.selected,
    required this.onSelected,
  });

  final ReaderTheme theme;
  final bool selected;
  final VoidCallback onSelected;

  @override
  Widget build(BuildContext context) {
    final colors = context.appColors;
    final strings = YamantakaLocalizations.of(context);
    final label = strings.get('settings.value.${theme.name}');
    return Semantics(
      button: true,
      selected: selected,
      label: strings.get(
        'reader.environmentChoice',
        parameters: {'theme': label},
      ),
      child: InkWell(
        key: ValueKey('reader-theme-${theme.name}'),
        onTap: onSelected,
        borderRadius: BorderRadius.circular(AppRadii.control),
        child: AnimatedContainer(
          duration: AppMotion.whenAllowed(
            MediaQuery.disableAnimationsOf(context),
            AppMotion.fast,
          ),
          constraints: const BoxConstraints(
            minWidth: 124,
            minHeight: AppSizes.touchControlExtent,
          ),
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.space12,
            vertical: AppSpacing.space8,
          ),
          decoration: BoxDecoration(
            color: selected ? colors.surfaceActive : Colors.transparent,
            border: Border.all(
              color: selected ? colors.focusRing : colors.borderSubtle,
              width: selected ? 2 : 1,
            ),
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 34,
                height: 30,
                padding: const EdgeInsets.all(4),
                decoration: BoxDecoration(
                  color: Color(theme.colorValue),
                  border: Border.all(color: colors.borderStrong),
                  borderRadius: BorderRadius.circular(AppRadii.indicator),
                ),
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: Colors.white,
                    border: Border.all(color: const Color(0x22000000)),
                  ),
                  child: Align(
                    alignment: Alignment.bottomCenter,
                    child: Container(
                      height: 3,
                      margin: const EdgeInsets.all(3),
                      color:
                          theme == ReaderTheme.dark ||
                              theme == ReaderTheme.black
                          ? Colors.white70
                          : Colors.black54,
                    ),
                  ),
                ),
              ),
              const SizedBox(width: AppSpacing.space8),
              Text(label),
              if (selected) ...[
                const SizedBox(width: AppSpacing.space6),
                Icon(Icons.check_rounded, size: 16, color: colors.iconActive),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

final class _DocumentInfoLine extends StatelessWidget {
  const _DocumentInfoLine({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpacing.space12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 96,
            child: Text(label, style: Theme.of(context).textTheme.labelMedium),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}

String _formatFileSize(int bytes) {
  if (bytes < 1024) return '$bytes B';
  final kib = bytes / 1024;
  if (kib < 1024) return '${kib.toStringAsFixed(1)} KiB';
  return '${(kib / 1024).toStringAsFixed(1)} MiB';
}

final class _ReaderErrorState extends StatelessWidget {
  const _ReaderErrorState({
    required this.title,
    required this.message,
    required this.onBack,
  });

  final String title;
  final String message;
  final VoidCallback onBack;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: AppStatusView(
        icon: Icons.error_outline_rounded,
        title: title,
        message: message,
        action: onBack,
        actionLabel: YamantakaLocalizations.of(
          context,
        ).get('reader.backToLibrary'),
        tone: AppStatusTone.error,
      ),
    );
  }
}
