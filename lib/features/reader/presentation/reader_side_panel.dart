import 'dart:collection';

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import '../../../app/app_config.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../app/ui/yama_components.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../annotations/domain/annotation.dart';
import '../../bookmarks/domain/bookmark.dart';
import '../../navigation/application/navigation_history_controller.dart';
import '../application/reader_controller.dart';

part 'reader_side_panel_components.dart';

enum ReaderPanelSection { thumbnails, outline, bookmarks, annotations, search }

final class ReaderSidePanel extends StatefulWidget {
  const ReaderSidePanel({
    super.key,
    required this.section,
    required this.controller,
    required this.engine,
    required this.width,
    required this.onClose,
    required this.onWidthChanged,
    required this.onSectionChanged,
    this.pinned = false,
    this.docked = false,
    this.allowPin = false,
    this.onPinnedChanged,
    this.flatChrome = false,
    this.onExportAnnotations,
  });

  final ReaderPanelSection section;
  final ReaderController controller;
  final PdfDocumentEngine engine;
  final double width;
  final VoidCallback onClose;
  final ValueChanged<double> onWidthChanged;
  final ValueChanged<ReaderPanelSection> onSectionChanged;
  final bool pinned;
  final bool docked;
  final bool allowPin;
  final ValueChanged<bool>? onPinnedChanged;
  final bool flatChrome;
  final Future<void> Function()? onExportAnnotations;

  @override
  State<ReaderSidePanel> createState() => _ReaderSidePanelState();
}

final class _ReaderSidePanelState extends State<ReaderSidePanel> {
  final TextEditingController _searchTextController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _searchTextController.text = widget.controller.search.query;
  }

  @override
  void dispose() {
    _searchTextController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final title = _title(strings);
    return Semantics(
      container: true,
      label: strings.get('reader.panel', parameters: {'title': title}),
      child: FocusTraversalGroup(
        child: Material(
          key: const Key('reader-side-panel'),
          color: context.appColors.surfaceElevated,
          elevation: 0,
          borderRadius: BorderRadius.horizontal(
            left: Radius.circular(widget.docked ? 0 : AppRadii.panel),
          ),
          clipBehavior: Clip.antiAlias,
          child: Container(
            width: widget.width,
            decoration: BoxDecoration(
              border: Border(
                left: BorderSide(color: context.appColors.borderStrong),
              ),
              boxShadow: widget.flatChrome
                  ? null
                  : [
                      BoxShadow(
                        color: context.appColors.ambientShadow,
                        blurRadius: AppShadow.readerPanelBlur,
                        offset: const Offset(-2, 0),
                      ),
                    ],
            ),
            child: Stack(
              children: [
                Column(
                  children: [
                    SizedBox(
                      height: AppSizes.readerPanelHeaderHeight,
                      child: Row(
                        children: [
                          const SizedBox(width: AppSpacing.space12),
                          Container(
                            width: 3,
                            height: 20,
                            decoration: BoxDecoration(
                              color: context.appColors.accentPrimary,
                              borderRadius: BorderRadius.circular(
                                AppRadii.indicator,
                              ),
                            ),
                          ),
                          const SizedBox(width: AppSpacing.space8),
                          Icon(
                            _panelSectionIcon(widget.section),
                            size: 18,
                            color: context.appColors.textSecondary,
                          ),
                          const SizedBox(width: AppSpacing.space8),
                          Expanded(
                            child: Text(
                              title,
                              style: Theme.of(context).textTheme.titleSmall
                                  ?.copyWith(fontWeight: FontWeight.w600),
                            ),
                          ),
                          if (widget.allowPin)
                            IconButton(
                              tooltip: widget.pinned
                                  ? strings.get('reader.unpinPanel')
                                  : strings.get('reader.pinPanel'),
                              onPressed: widget.onPinnedChanged == null
                                  ? null
                                  : () =>
                                        widget.onPinnedChanged!(!widget.pinned),
                              icon: Icon(
                                widget.pinned
                                    ? Icons.push_pin_rounded
                                    : Icons.push_pin_outlined,
                              ),
                            ),
                          IconButton(
                            autofocus:
                                widget.section != ReaderPanelSection.search,
                            tooltip: strings.get('reader.closePanel'),
                            onPressed: widget.onClose,
                            icon: const Icon(Icons.close_rounded),
                          ),
                          const SizedBox(width: AppSpacing.space4),
                        ],
                      ),
                    ),
                    const Divider(height: 1),
                    Expanded(
                      child: widget.docked
                          ? Row(
                              children: [
                                _PanelSectionRail(
                                  selected: widget.section,
                                  onSelected: widget.onSectionChanged,
                                ),
                                const VerticalDivider(width: 1),
                                Expanded(child: _content()),
                              ],
                            )
                          : Column(
                              children: [
                                _PanelSectionBar(
                                  selected: widget.section,
                                  onSelected: widget.onSectionChanged,
                                ),
                                const Divider(height: 1),
                                Expanded(child: _content()),
                              ],
                            ),
                    ),
                  ],
                ),
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  child: _PanelResizeHandle(
                    width: widget.width,
                    onWidthChanged: widget.onWidthChanged,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _title(YamantakaLocalizations strings) => switch (widget.section) {
    ReaderPanelSection.thumbnails => strings.get('reader.thumbnails'),
    ReaderPanelSection.outline => strings.get('reader.outline'),
    ReaderPanelSection.bookmarks => strings.get('reader.bookmarks'),
    ReaderPanelSection.annotations => strings.get('reader.annotations'),
    ReaderPanelSection.search => strings.get('reader.search'),
  };

  Widget _content() => switch (widget.section) {
    ReaderPanelSection.thumbnails => _ThumbnailPanel(
      controller: widget.controller,
      engine: widget.engine,
      panelWidth: widget.width,
    ),
    ReaderPanelSection.outline => _OutlinePanel(controller: widget.controller),
    ReaderPanelSection.bookmarks => _BookmarksPanel(
      controller: widget.controller,
    ),
    ReaderPanelSection.annotations => _AnnotationsPanel(
      controller: widget.controller,
      onExport: widget.onExportAnnotations,
    ),
    ReaderPanelSection.search => _SearchPanel(
      controller: widget.controller,
      textController: _searchTextController,
    ),
  };
}
