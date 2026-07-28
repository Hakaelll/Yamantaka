import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/app_config.dart';
import '../../../app/shortcuts/app_commands.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../../command_palette/presentation/command_palette.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';
import 'reader_bottom_controls.dart';
import 'reader_side_panel.dart';
import 'reader_top_bar.dart';

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

  @override
  State<ReaderScreen> createState() => _ReaderScreenState();
}

final class _ReaderScreenState extends State<ReaderScreen> {
  ReaderPanelSection? _panel;
  bool _controlsVisible = true;
  Timer? _controlsTimer;

  bool get _isBookMode =>
      widget.controller.preferences?.viewMode == ReaderMode.book;

  Duration get _animationDuration =>
      widget.reduceMotion ? Duration.zero : const Duration(milliseconds: 140);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _revealControls());
  }

  @override
  void dispose() {
    _controlsTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final combined = Listenable.merge([
      widget.controller,
      widget.controller.viewController,
      widget.controller.bookmarks,
    ]);
    return ListenableBuilder(
      listenable: combined,
      builder: (context, _) {
        if (widget.controller.status == ReaderStatus.loading) {
          return const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          );
        }
        if (widget.controller.status != ReaderStatus.ready ||
            widget.controller.openedDocument == null ||
            widget.controller.preferences == null) {
          return _ReaderErrorState(
            message:
                widget.controller.error?.userMessage ??
                'No hay un documento abierto.',
            onBack: widget.onBack,
          );
        }
        return CallbackShortcuts(
          bindings: AppCommandRegistry.shortcuts(_commandActions(context)),
          child: Focus(
            autofocus: true,
            onKeyEvent: (_, event) {
              if (event is KeyDownEvent) {
                widget.controller.registerActivity();
                _revealControls();
              }
              return KeyEventResult.ignored;
            },
            child: MouseRegion(
              onHover: (_) => _revealControls(),
              child: Listener(
                onPointerDown: (_) {
                  widget.controller.registerActivity();
                  _revealControls();
                },
                child: Scaffold(
                  body: LayoutBuilder(
                    builder: (context, constraints) =>
                        _buildReader(context, constraints.maxWidth),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildReader(BuildContext context, double availableWidth) {
    final controller = widget.controller;
    final opened = controller.openedDocument!;
    final preferences = controller.preferences!;
    final readerBackground = Color(preferences.readingTheme.colorValue);
    final requestedFlow = switch (preferences.pageLayout) {
      PageLayoutMode.continuous => PdfViewFlow.continuous,
      PageLayoutMode.single => PdfViewFlow.single,
      PageLayoutMode.facing =>
        availableWidth >= 900 ? PdfViewFlow.facing : PdfViewFlow.single,
    };
    final controlsShown = !_isBookMode || _controlsVisible;
    return ColoredBox(
      color: readerBackground,
      child: Stack(
        children: [
          Positioned.fill(
            child: Semantics(
              label:
                  'Documento PDF, página ${controller.currentPageIndex + 1} '
                  'de ${controller.pageCount}',
              child: widget.viewerAdapter.buildViewer(
                document: opened,
                controller: controller.viewController,
                configuration: PdfViewConfiguration(
                  backgroundColor: readerBackground,
                  margin:
                      (preferences.horizontalMargin +
                          preferences.verticalMargin) /
                      2,
                  flow: requestedFlow,
                  rotation: preferences.pageRotation,
                  reduceMotion: widget.reduceMotion,
                  initialZoom: preferences.zoomLevel,
                  initialZoomMode: switch (preferences.zoomMode) {
                    ZoomMode.custom => PdfViewZoomMode.custom,
                    ZoomMode.fitWidth => PdfViewZoomMode.fitWidth,
                    ZoomMode.fitPage => PdfViewZoomMode.fitPage,
                  },
                  initialScrollOffset: preferences.scrollOffset,
                  readingDirection:
                      widget.readingDirection == ReadingDirection.rightToLeft
                      ? PdfViewReadingDirection.rightToLeft
                      : PdfViewReadingDirection.leftToRight,
                ),
                initialPageIndex: controller.currentPageIndex,
                onPageChanged: controller.onPageChanged,
                onError: (error) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('No se pudo renderizar la página.'),
                    ),
                  );
                },
              ),
            ),
          ),
          if (_isBookMode) ...[
            Positioned(
              left: 0,
              top: 64,
              bottom: 64,
              width: 64,
              child: Semantics(
                button: true,
                label: 'Página anterior',
                child: GestureDetector(
                  behavior: HitTestBehavior.translucent,
                  onTap: controller.previousPage,
                ),
              ),
            ),
            Positioned(
              right: _panel == null ? 0 : 304,
              top: 64,
              bottom: 64,
              width: 64,
              child: Semantics(
                button: true,
                label: 'Página siguiente',
                child: GestureDetector(
                  behavior: HitTestBehavior.translucent,
                  onTap: controller.nextPage,
                ),
              ),
            ),
          ],
          AnimatedSlide(
            duration: _animationDuration,
            offset: controlsShown ? Offset.zero : const Offset(0, -1),
            child: IgnorePointer(
              ignoring: !controlsShown,
              child: ReaderTopBar(
                controller: controller,
                onBack: widget.onBack,
                onSearch: () => _showPanel(ReaderPanelSection.search),
                onCommandPalette: () => _showCommandPalette(context),
                onShowThumbnails: () =>
                    _showPanel(ReaderPanelSection.thumbnails),
                onShowOutline: () => _showPanel(ReaderPanelSection.outline),
                onShowBookmarks: () => _showPanel(ReaderPanelSection.bookmarks),
                onAdjustMargins: () => _showMarginChooser(context),
                onOpenSettings: widget.onOpenSettings,
              ),
            ),
          ),
          Align(
            alignment: Alignment.bottomCenter,
            child: AnimatedSlide(
              duration: _animationDuration,
              offset: controlsShown ? Offset.zero : const Offset(0, 1),
              child: IgnorePointer(
                ignoring: !controlsShown,
                child: ReaderBottomControls(
                  controller: controller,
                  onGoToPage: () => _showGoToPage(context),
                  onChooseTheme: () => _showThemeChooser(context),
                ),
              ),
            ),
          ),
          if (_panel != null)
            Positioned(
              top: 0,
              bottom: 0,
              right: 0,
              child: ReaderSidePanel(
                section: _panel!,
                controller: controller,
                engine: widget.engine,
                onClose: () => setState(() => _panel = null),
              ),
            ),
          if (widget.showProgress)
            Positioned(
              left: 0,
              right: _panel == null ? 0 : 304,
              bottom: 0,
              child: LinearProgressIndicator(
                key: const Key('reader-progress'),
                value: controller.pageCount == 0
                    ? 0
                    : ReadingProgress.calculate(
                        pageIndex: controller.currentPageIndex,
                        pageCount: controller.pageCount,
                      ),
                minHeight: 3,
                backgroundColor: Colors.transparent,
              ),
            ),
        ],
      ),
    );
  }

  Map<AppCommandId, VoidCallback> _commandActions(BuildContext context) => {
    AppCommandId.openDocument: () => unawaited(widget.onOpenDocument()),
    AppCommandId.search: () => _showPanel(ReaderPanelSection.search),
    AppCommandId.commandPalette: () => _showCommandPalette(context),
    AppCommandId.goToPage: () => _showGoToPage(context),
    AppCommandId.toggleBookmark: widget.controller.toggleBookmark,
    AppCommandId.zoomIn: () => unawaited(widget.controller.zoomBy(1.18)),
    AppCommandId.zoomOut: () => unawaited(widget.controller.zoomBy(0.85)),
    AppCommandId.resetZoom: () => unawaited(widget.controller.fitPage()),
    AppCommandId.fullScreen: () =>
        unawaited(widget.controller.toggleFullScreen()),
    AppCommandId.escape: _escape,
    AppCommandId.nextPage: () => unawaited(widget.controller.nextPage()),
    AppCommandId.previousPage: () =>
        unawaited(widget.controller.previousPage()),
    AppCommandId.firstPage: () => unawaited(widget.controller.firstPage()),
    AppCommandId.lastPage: () => unawaited(widget.controller.lastPage()),
    AppCommandId.bookMode: () =>
        widget.controller.setReaderMode(ReaderMode.book),
    AppCommandId.documentMode: () =>
        widget.controller.setReaderMode(ReaderMode.document),
    AppCommandId.showThumbnails: () =>
        _showPanel(ReaderPanelSection.thumbnails),
    AppCommandId.showOutline: () => _showPanel(ReaderPanelSection.outline),
    AppCommandId.fitWidth: () => unawaited(widget.controller.fitWidth()),
    AppCommandId.fitPage: () => unawaited(widget.controller.fitPage()),
    AppCommandId.changeReaderTheme: () => _showThemeChooser(context),
    AppCommandId.settings: widget.onOpenSettings,
  };

  void _showPanel(ReaderPanelSection section) {
    setState(() {
      _panel = _panel == section ? null : section;
      _controlsVisible = true;
    });
  }

  void _escape() {
    if (_panel != null) {
      setState(() => _panel = null);
    } else {
      unawaited(widget.controller.leaveFullScreen());
    }
  }

  void _revealControls() {
    if (!mounted) return;
    if (!_controlsVisible) setState(() => _controlsVisible = true);
    _controlsTimer?.cancel();
    final preferences = widget.controller.preferences;
    if (_isBookMode && (preferences?.controlsAutoHide ?? true)) {
      _controlsTimer = Timer(AppConfig.controlsAutoHideDelay, () {
        if (mounted && _panel == null) {
          setState(() => _controlsVisible = false);
        }
      });
    }
  }

  Future<void> _showCommandPalette(BuildContext context) async {
    _controlsTimer?.cancel();
    await showDialog<void>(
      context: context,
      builder: (_) => CommandPalette(actions: _commandActions(context)),
    );
    _revealControls();
  }

  Future<void> _showGoToPage(BuildContext context) async {
    final textController = TextEditingController(
      text: '${widget.controller.currentPageIndex + 1}',
    );
    final page = await showDialog<int>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Ir a página'),
        content: TextField(
          key: const Key('go-to-page-field'),
          controller: textController,
          autofocus: true,
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            helperText: 'Entre 1 y ${widget.controller.pageCount}',
          ),
          onSubmitted: (value) => Navigator.pop(context, int.tryParse(value)),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () =>
                Navigator.pop(context, int.tryParse(textController.text)),
            child: const Text('Ir'),
          ),
        ],
      ),
    );
    textController.dispose();
    if (page != null) await widget.controller.goToPage(page);
  }

  Future<void> _showThemeChooser(BuildContext context) async {
    final selected = await showDialog<ReaderTheme>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Tema de lectura'),
        children: [
          RadioGroup<ReaderTheme>(
            groupValue: widget.controller.preferences?.readingTheme,
            onChanged: (value) => Navigator.pop(context, value),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: ReaderTheme.values
                  .map(
                    (theme) => RadioListTile<ReaderTheme>(
                      value: theme,
                      title: Text(theme.label),
                      secondary: Container(
                        width: 24,
                        height: 24,
                        decoration: BoxDecoration(
                          color: Color(theme.colorValue),
                          border: Border.all(
                            color: Theme.of(context).dividerColor,
                          ),
                          shape: BoxShape.circle,
                        ),
                      ),
                    ),
                  )
                  .toList(growable: false),
            ),
          ),
        ],
      ),
    );
    if (selected != null) widget.controller.setTheme(selected);
  }

  Future<void> _showMarginChooser(BuildContext context) async {
    final preferences = widget.controller.preferences;
    if (preferences == null) return;
    var margin =
        (preferences.horizontalMargin + preferences.verticalMargin) / 2;
    final selected = await showDialog<double>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('Márgenes de lectura'),
          content: SizedBox(
            width: 360,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Slider(
                  value: margin.clamp(8, 72),
                  min: 8,
                  max: 72,
                  divisions: 8,
                  label: '${margin.round()} px',
                  onChanged: (value) => setDialogState(() => margin = value),
                ),
                Text('${margin.round()} px'),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancelar'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, margin),
              child: const Text('Aplicar'),
            ),
          ],
        ),
      ),
    );
    if (selected != null) widget.controller.setMargins(selected);
  }
}

final class _ReaderErrorState extends StatelessWidget {
  const _ReaderErrorState({required this.message, required this.onBack});

  final String message;
  final VoidCallback onBack;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline_rounded, size: 48),
            const SizedBox(height: 16),
            Text(message),
            const SizedBox(height: 20),
            OutlinedButton(
              onPressed: onBack,
              child: const Text('Volver a la biblioteca'),
            ),
          ],
        ),
      ),
    );
  }
}
