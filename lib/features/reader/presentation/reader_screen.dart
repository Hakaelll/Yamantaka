import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_theme.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/errors/failure_presentation.dart';
import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../pdf_engine/presentation/pdf_viewer_adapter.dart';
import '../../../pdf_engine/presentation/reader_viewport_metrics.dart';
import '../../command_palette/presentation/command_palette.dart';
import '../../library/domain/library_document.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';
import 'reader_bottom_controls.dart';
import 'reader_side_panel.dart';
import 'reader_top_bar.dart';
import 'reader_ui_state.dart';

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
    this.showModeControls = true,
    this.showPanelControls = true,
    this.shortcutOverrides = '{}',
    this.recentDocuments = const [],
    this.onOpenPath,
    this.onPanelWidthChanged,
    this.onPanelChanged,
    this.onChromeVisibilityChanged,
    this.onDocumentInfo,
    this.onExportAnnotations,
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
  final bool showModeControls;
  final bool showPanelControls;
  final String shortcutOverrides;
  final List<LibraryDocument> recentDocuments;
  final Future<void> Function(String path)? onOpenPath;
  final ValueChanged<double>? onPanelWidthChanged;
  final ValueChanged<ReaderPanelSection?>? onPanelChanged;
  final ValueChanged<bool>? onChromeVisibilityChanged;
  final VoidCallback? onDocumentInfo;
  final Future<void> Function()? onExportAnnotations;

  @override
  State<ReaderScreen> createState() => _ReaderScreenState();
}

final class _ReaderScreenState extends State<ReaderScreen> {
  ReaderPanelSection? _panel;
  late bool _controlsVisible;
  late double _panelWidthValue;
  Timer? _controlsTimer;
  Timer? _panelSaveTimer;
  Timer? _cursorTimer;
  final FocusNode _readerFocusNode = FocusNode(debugLabel: 'PDF reader');
  final GlobalKey _viewerRegionKey = GlobalKey(debugLabel: 'PDF viewer region');
  FocusNode? _focusBeforePanel;
  String? _focusedDocumentId;
  Object? _viewerCacheKey;
  Widget? _viewerCache;
  late final ReaderChromeCoordinator _chromeCoordinator;
  bool _immersive = false;
  bool _cursorHidden = false;
  bool _panelPinned = false;
  Offset? _lastPointerPosition;

  bool get _isBookMode =>
      widget.controller.preferences?.viewMode == ReaderMode.book;

  bool get _isElectronicInk =>
      widget.visualProfile == ReaderVisualProfile.electronicInk;

  Duration _chromeTransition(BuildContext context, bool visible) =>
      AppMotion.whenAllowed(
        widget.reduceMotion ||
            _isElectronicInk ||
            MediaQuery.disableAnimationsOf(context),
        visible ? AppMotion.chromeReveal : AppMotion.chromeHide,
      );

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_handleHardwareNavigation);
    _chromeCoordinator = ReaderChromeCoordinator(
      autoHide: widget.controlsAutoHide ?? true,
      reduceMotion: widget.reduceMotion,
    );
    _controlsVisible = widget.initialChromeVisible;
    _panelPinned = widget.initialPanelPinned;
    _panelWidthValue = widget.initialPanelWidth
        .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
        .toDouble();
    _chromeCoordinator.setPanelWidth(_panelWidthValue);
    if (widget.initialPanelOpen) {
      _panel = ReaderPanelSection.values
          .where((section) => section.name == widget.initialPanelSection)
          .firstOrNull;
      if (_panel case final section?) {
        _chromeCoordinator.openPanel(
          section,
          presentation: _panelPinned
              ? ReaderPanelPresentation.pinned
              : ReaderPanelPresentation.overlay,
        );
      }
    }
    if (_controlsVisible) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _revealControls());
    }
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_handleHardwareNavigation);
    _controlsTimer?.cancel();
    _panelSaveTimer?.cancel();
    _cursorTimer?.cancel();
    _chromeCoordinator.dispose();
    _readerFocusNode.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant ReaderScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controlsAutoHide == widget.controlsAutoHide &&
        oldWidget.reduceMotion == widget.reduceMotion) {
      return;
    }
    _controlsTimer?.cancel();
    _chromeCoordinator.configure(
      autoHide: widget.controlsAutoHide,
      reduceMotion: widget.reduceMotion,
    );
    if (widget.controlsAutoHide == false) {
      if (!_controlsVisible) {
        setState(() => _controlsVisible = true);
        widget.onChromeVisibilityChanged?.call(true);
      }
    } else {
      _revealControls();
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        if (widget.controller.status == ReaderStatus.loading) {
          return const Scaffold(
            body: AppStatusView(
              icon: Icons.picture_as_pdf_outlined,
              title: 'Abriendo documento',
              message:
                  'Preparando páginas, posición y preferencias de lectura.',
              loading: true,
            ),
          );
        }
        if (widget.controller.status != ReaderStatus.ready ||
            widget.controller.openedDocument == null ||
            widget.controller.preferences == null) {
          final presentation = FailurePresentations.forFailure(
            widget.controller.error,
          );
          return _ReaderErrorState(
            title: presentation.title,
            message: presentation.message,
            onBack: widget.onBack,
          );
        }
        return CallbackShortcuts(
          bindings: _shortcutBindings(context),
          child: Focus(
            focusNode: _readerFocusNode,
            autofocus: true,
            onKeyEvent: (_, event) {
              if (event is KeyDownEvent) {
                widget.controller.registerActivity();
                _revealControls();
              }
              return KeyEventResult.ignored;
            },
            child: MouseRegion(
              cursor: _cursorHidden
                  ? SystemMouseCursors.none
                  : SystemMouseCursors.basic,
              onHover: (event) {
                final previous = _lastPointerPosition;
                _lastPointerPosition = event.position;
                _showCursor();
                _scheduleCursorHide();
                final height = MediaQuery.sizeOf(context).height;
                final atRevealEdge =
                    event.localPosition.dy <= AppSizes.topRevealZone ||
                    event.localPosition.dy >= height - AppSpacing.space12;
                final inTopChrome =
                    event.localPosition.dy <=
                    AppSizes.topBarHeight + AppSpacing.space8;
                final inBottomChrome =
                    event.localPosition.dy >=
                    height - AppSizes.bottomControlHeight - AppSpacing.space24;
                _chromeCoordinator.setPointerRegion(
                  inTopChrome
                      ? ReaderPointerRegion.topEdge
                      : inBottomChrome
                      ? ReaderPointerRegion.bottomEdge
                      : ReaderPointerRegion.canvas,
                );
                if (atRevealEdge &&
                    (previous == null ||
                        (event.position - previous).distance > 2)) {
                  _revealControls();
                }
              },
              child: Listener(
                onPointerDown: (_) {
                  widget.controller.registerActivity();
                  _showCursor();
                  _scheduleCursorHide();
                  _revealControls();
                },
                child: Scaffold(
                  body: LayoutBuilder(
                    builder: (context, constraints) =>
                        _buildReader(context, constraints),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildReader(BuildContext context, BoxConstraints constraints) {
    final controller = widget.controller;
    final opened = controller.openedDocument!;
    final preferences = controller.preferences!;
    final readerBackground = _readerCanvasColor(
      context,
      preferences.readingTheme,
    );
    final chromeTheme = _isBookMode && !_isElectronicInk
        ? AppTheme.dark(highContrast: MediaQuery.highContrastOf(context))
        : Theme.of(context);
    final panelWidth = _panelWidth(constraints.maxWidth);
    final immersive = _immersive;
    final requestedFlow = _isElectronicInk
        ? PdfViewFlow.single
        : _isBookMode
        ? PdfViewFlow.facing
        : preferences.pageLayout == PageLayoutMode.continuous
        ? PdfViewFlow.continuous
        : PdfViewFlow.single;
    final controlsShown = _controlsVisible && !immersive;
    // Panels are overlays by default. Their visibility must not alter PDF
    // constraints, which is the core continuity invariant of the redesign.
    final windowClass = AppBreakpoints.classify(constraints.maxWidth);
    final panelDocked =
        _panel != null &&
        _panelPinned &&
        windowClass == AppWindowClass.expanded;
    final viewerSize = Size(constraints.maxWidth, constraints.maxHeight);
    final readerMargin = ReaderViewportMetrics.adaptiveMargin(
      viewport: viewerSize,
      preferred:
          (preferences.horizontalMargin + preferences.verticalMargin) / 2,
    );
    // The PDF layout already contains the editorial outer margin.  Keeping
    // this separate viewport margin at zero is what makes chrome genuinely
    // overlay-only instead of reserving a hidden toolbar strip.
    const viewportMargin = 0.0;
    _ensureReaderFocus(opened.documentId);

    return ColoredBox(
      color: readerBackground,
      child: Stack(
        children: [
          Positioned(
            left: 0,
            top: 0,
            right: panelDocked ? panelWidth : 0,
            bottom: 0,
            child: Semantics(
              label:
                  '${_isBookMode ? 'Modo Libro' : 'Modo Documento'}, '
                  '${controller.readingUnitIndicator(direction: widget.readingDirection)}',
              child: Listener(
                key: _viewerRegionKey,
                behavior: HitTestBehavior.translucent,
                onPointerDown: (_) => _requestReaderFocus(),
                child: TweenAnimationBuilder<double>(
                  key: ValueKey((opened.documentId, requestedFlow)),
                  duration: AppMotion.whenAllowed(
                    widget.reduceMotion ||
                        _isElectronicInk ||
                        MediaQuery.disableAnimationsOf(context),
                    AppMotion.standard,
                  ),
                  curve: Curves.easeOutCubic,
                  tween: Tween(begin: 0.94, end: 1),
                  builder: (context, opacity, child) =>
                      Opacity(opacity: opacity, child: child),
                  child: _cachedViewer(
                    document: opened,
                    controller: controller.viewController,
                    configuration: PdfViewConfiguration(
                      backgroundColor: readerBackground,
                      margin: readerMargin,
                      viewportMargin: viewportMargin,
                      coverPolicy: preferences.coverPolicy,
                      initialSide: preferences.initialSide,
                      flow: requestedFlow,
                      rotation: preferences.pageRotation,
                      reduceMotion: widget.reduceMotion || _isElectronicInk,
                      initialZoom: preferences.zoomLevel,
                      initialZoomMode: switch (preferences.zoomMode) {
                        ZoomMode.custom => PdfViewZoomMode.custom,
                        ZoomMode.fitWidth => PdfViewZoomMode.fitWidth,
                        ZoomMode.fitPage => PdfViewZoomMode.fitPage,
                      },
                      initialScrollOffset: preferences.scrollOffset,
                      readingDirection:
                          widget.readingDirection ==
                              ReadingDirection.rightToLeft
                          ? PdfViewReadingDirection.rightToLeft
                          : PdfViewReadingDirection.leftToRight,
                      pageShadow: _pageShadow(
                        context,
                        preferences.readingTheme,
                      ),
                    ),
                    initialPageIndex: controller.currentPageIndex,
                    onPageChanged: controller.onPageChanged,
                    onError: (error) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text(
                            'No se pudo renderizar esta página. '
                            'Prueba a cambiar de página o reabrir el documento.',
                          ),
                        ),
                      );
                    },
                  ),
                ),
              ),
            ),
          ),
          if (_isBookMode && !immersive)
            ..._pageTurnZones(
              controller: controller,
              panelWidth: panelWidth,
              panelVisible: panelDocked,
            ),
          if (!immersive)
            Positioned(
              left: 0,
              top: 0,
              right: panelDocked ? panelWidth : 0,
              child: AnimatedSlide(
                duration: _chromeTransition(context, controlsShown),
                curve: Curves.easeOutCubic,
                offset: controlsShown ? Offset.zero : const Offset(0, -1),
                child: AnimatedOpacity(
                  duration: _chromeTransition(context, controlsShown),
                  opacity: controlsShown ? 1 : 0,
                  child: IgnorePointer(
                    ignoring: !controlsShown,
                    child: Theme(
                      data: chromeTheme,
                      child: ReaderTopBar(
                        key: const Key('reader-top-bar'),
                        controller: controller,
                        canvasColor: readerBackground,
                        flatChrome:
                            _isElectronicInk ||
                            MediaQuery.highContrastOf(context),
                        compact: windowClass == AppWindowClass.compact,
                        expanded: windowClass == AppWindowClass.expanded,
                        activePanel: _panel,
                        showModeControls: widget.showModeControls,
                        showPanelControls: widget.showPanelControls,
                        shortcutOverrides: widget.shortcutOverrides,
                        onBack: widget.onBack,
                        onOpenDocument: () =>
                            unawaited(widget.onOpenDocument()),
                        onSearch: () => _showPanel(ReaderPanelSection.search),
                        onCommandPalette: () => _showCommandPalette(context),
                        onShowThumbnails: () =>
                            _showPanel(ReaderPanelSection.thumbnails),
                        onShowOutline: () =>
                            _showPanel(ReaderPanelSection.outline),
                        onShowBookmarks: () =>
                            _showPanel(ReaderPanelSection.bookmarks),
                        onShowAnnotations: () =>
                            _showPanel(ReaderPanelSection.annotations),
                        onAdjustMargins: () => _showMarginChooser(context),
                        onModeChanged: _setReaderMode,
                        onDocumentInfo:
                            widget.onDocumentInfo ??
                            () => _showDocumentInfo(context),
                        onToggleFullScreen: _toggleFullScreen,
                        onToggleChrome: _toggleChrome,
                        onToggleImmersive: _toggleImmersive,
                        onOpenSettings: widget.onOpenSettings,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          if (!immersive)
            Positioned(
              left: AppSpacing.space12,
              right: panelDocked
                  ? panelWidth + AppSpacing.space12
                  : AppSpacing.space12,
              bottom: AppSpacing.space12,
              child: AnimatedSlide(
                duration: _chromeTransition(context, controlsShown),
                curve: Curves.easeOutCubic,
                offset: controlsShown ? Offset.zero : const Offset(0, 1.3),
                child: AnimatedOpacity(
                  duration: _chromeTransition(context, controlsShown),
                  opacity: controlsShown ? 1 : 0,
                  child: IgnorePointer(
                    ignoring: !controlsShown,
                    child: Theme(
                      data: chromeTheme,
                      child: ReaderBottomControls(
                        key: const Key('reader-bottom-controls'),
                        controller: controller,
                        compact: windowClass == AppWindowClass.compact,
                        onGoToPage: () => _showGoToPage(context),
                        onChooseTheme: () => _showThemeChooser(context),
                        readingDirection: widget.readingDirection,
                        flatChrome:
                            _isElectronicInk ||
                            MediaQuery.highContrastOf(context),
                        shortcutOverrides: widget.shortcutOverrides,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          if (!immersive &&
              _panel != null &&
              !panelDocked &&
              !_isElectronicInk &&
              !MediaQuery.highContrastOf(context))
            Positioned.fill(
              child: GestureDetector(
                key: const Key('reader-panel-scrim'),
                behavior: HitTestBehavior.opaque,
                onTap: _closePanel,
                child: ColoredBox(
                  color: Colors.black.withValues(
                    alpha: Theme.of(context).brightness == Brightness.dark
                        ? 0.12
                        : 0.08,
                  ),
                ),
              ),
            ),
          if (!immersive && _panel != null)
            Positioned(
              top: 0,
              bottom: 0,
              right: 0,
              width: panelWidth,
              child: TweenAnimationBuilder<double>(
                key: ValueKey(_panel),
                duration: AppMotion.whenAllowed(
                  widget.reduceMotion,
                  AppMotion.panel,
                ),
                curve: Curves.easeOutCubic,
                tween: Tween(begin: 1, end: 0),
                builder: (context, offset, child) => Transform.translate(
                  offset: Offset(panelWidth * offset, 0),
                  child: child,
                ),
                child: ReaderSidePanel(
                  section: _panel!,
                  controller: controller,
                  engine: widget.engine,
                  width: panelWidth,
                  pinned: _panelPinned,
                  allowPin: windowClass == AppWindowClass.expanded,
                  onPinnedChanged: _togglePanelPinned,
                  flatChrome:
                      _isElectronicInk || MediaQuery.highContrastOf(context),
                  onClose: _closePanel,
                  onWidthChanged: _resizePanel,
                  onExportAnnotations: widget.onExportAnnotations,
                ),
              ),
            ),
          if (!immersive &&
              widget.showProgress &&
              (!controller.isFullScreen || _controlsVisible))
            Positioned(
              left: 0,
              right: panelDocked ? panelWidth : 0,
              bottom: 0,
              child: LinearProgressIndicator(
                key: const Key('reader-progress'),
                value: controller.pageCount == 0
                    ? 0
                    : ReadingProgress.calculate(
                        pageIndex: controller.currentPageIndex,
                        pageCount: controller.pageCount,
                      ),
                minHeight: 2,
                backgroundColor: Colors.transparent,
              ),
            ),
          if (!immersive &&
              (controller.annotations?.pageHasAnnotations(
                    controller.currentPageIndex,
                  ) ??
                  false))
            Positioned(
              left: AppSpacing.space12,
              top: controlsShown
                  ? AppSizes.topBarHeight + AppSpacing.space8
                  : AppSpacing.space12,
              child: Semantics(
                label:
                    'La página ${controller.currentPageIndex + 1} tiene anotaciones',
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: context.appColors.surfacePrimary.withValues(
                      alpha: 0.92,
                    ),
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(color: context.appColors.borderStrong),
                  ),
                  child: const Padding(
                    padding: EdgeInsets.all(AppSpacing.space8),
                    child: Icon(Icons.rate_review_outlined, size: 16),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  List<Widget> _pageTurnZones({
    required ReaderController controller,
    required double panelWidth,
    required bool panelVisible,
  }) {
    final rtl = widget.readingDirection == ReadingDirection.rightToLeft;
    Widget zone({
      required bool left,
      required String label,
      required VoidCallback? action,
    }) => Positioned(
      left: left ? 0 : null,
      right: left ? null : (panelVisible ? panelWidth : 0),
      top: AppSizes.topBarHeight,
      bottom: AppSizes.bottomControlHeight + AppSpacing.space24,
      width: AppSizes.readerEdgeNavigation,
      child: Semantics(
        button: true,
        enabled: action != null,
        label: label,
        child: Tooltip(
          message: label,
          child: InkWell(
            key: Key(left ? 'reader-left-zone' : 'reader-right-zone'),
            onTap: action,
            hoverColor: Colors.white.withValues(alpha: 0.035),
            focusColor: Colors.white.withValues(alpha: 0.06),
            child: Center(
              child: AnimatedOpacity(
                duration: _chromeTransition(context, true),
                opacity: action == null ? 0.32 : 1,
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: const Color(0xE6222522),
                    borderRadius: BorderRadius.circular(AppRadii.control),
                    border: Border.all(color: const Color(0x2EFFFFFF)),
                    boxShadow: const [
                      BoxShadow(
                        color: Color(0x52000000),
                        blurRadius: 12,
                        offset: Offset(0, 3),
                      ),
                    ],
                  ),
                  child: SizedBox.square(
                    dimension: 40,
                    child: Icon(
                      left
                          ? Icons.chevron_left_rounded
                          : Icons.chevron_right_rounded,
                      color: Colors.white,
                      size: 22,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    return [
      zone(
        left: true,
        label: rtl ? 'Página siguiente' : 'Página anterior',
        action: rtl
            ? controller.canGoToNextReadingUnit
                  ? controller.goToNextReadingUnit
                  : null
            : controller.canGoToPreviousReadingUnit
            ? controller.goToPreviousReadingUnit
            : null,
      ),
      zone(
        left: false,
        label: rtl ? 'Página anterior' : 'Página siguiente',
        action: rtl
            ? controller.canGoToPreviousReadingUnit
                  ? controller.goToPreviousReadingUnit
                  : null
            : controller.canGoToNextReadingUnit
            ? controller.goToNextReadingUnit
            : null,
      ),
    ];
  }

  Color _readerCanvasColor(BuildContext context, ReaderTheme theme) {
    if (_isElectronicInk || MediaQuery.highContrastOf(context)) {
      return Colors.white;
    }
    if (_isBookMode) {
      return AppColorTokens.dark.readerBackground;
    }
    final colors = context.appColors;
    return switch (theme) {
      ReaderTheme.light => colors.paperWhite,
      ReaderTheme.ivory => colors.ivory,
      ReaderTheme.sepia => const Color(0xFFE2D5BE),
      ReaderTheme.gray => const Color(0xFFB8BAB5),
      ReaderTheme.dark => const Color(0xFF242722),
      // Pure black remains readable for old persisted documents, but the
      // normal preset is the semantic charcoal token from the redesign.
      ReaderTheme.black => colors.nightBackground,
    };
  }

  BoxShadow? _pageShadow(BuildContext context, ReaderTheme theme) {
    if (_isElectronicInk ||
        MediaQuery.highContrastOf(context) ||
        theme == ReaderTheme.dark ||
        theme == ReaderTheme.black) {
      return null;
    }
    return BoxShadow(
      color: context.appColors.pageShadow,
      blurRadius: theme == ReaderTheme.gray
          ? AppShadow.readerGrayPageBlur
          : AppShadow.readerPageBlur,
      spreadRadius: 0,
      offset: const Offset(0, 2),
    );
  }

  double _panelWidth(double viewportWidth) {
    if (AppBreakpoints.classify(viewportWidth) == AppWindowClass.compact) {
      final availableWidth = math.max(0.0, viewportWidth - AppSpacing.space24);
      return math.min(
        _panelWidthValue,
        math.min(availableWidth, viewportWidth * 0.88),
      );
    }
    return _panelWidthValue;
  }

  Widget _cachedViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  }) {
    final key = (
      document.documentId,
      configuration.backgroundColor,
      configuration.margin,
      configuration.viewportMargin,
      configuration.coverPolicy,
      configuration.initialSide,
      configuration.flow,
      configuration.rotation,
      configuration.reduceMotion,
      configuration.readingDirection,
    );
    if (_viewerCacheKey != key || _viewerCache == null) {
      _viewerCacheKey = key;
      _viewerCache = RepaintBoundary(
        key: ValueKey('reader-viewer-${document.documentId}'),
        child: widget.viewerAdapter.buildViewer(
          document: document,
          controller: controller,
          configuration: configuration,
          initialPageIndex: initialPageIndex,
          onPageChanged: onPageChanged,
          onError: onError,
        ),
      );
    }
    return _viewerCache!;
  }

  void _ensureReaderFocus(String documentId) {
    if (_focusedDocumentId == documentId) return;
    _focusedDocumentId = documentId;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _requestReaderFocus();
    });
  }

  void _requestReaderFocus() {
    if (!mounted || !_readerFocusNode.canRequestFocus) return;
    _readerFocusNode.requestFocus();
  }

  bool get _textInputHasFocus {
    final focusContext = FocusManager.instance.primaryFocus?.context;
    if (focusContext == null) return false;
    return focusContext.widget is EditableText ||
        focusContext.findAncestorWidgetOfExactType<EditableText>() != null;
  }

  bool get _readerCanvasHasFocus {
    final primaryFocus = FocusManager.instance.primaryFocus;
    if (identical(primaryFocus, _readerFocusNode)) return true;
    final focusContext = primaryFocus?.context;
    final viewerContext = _viewerRegionKey.currentContext;
    if (focusContext == null || viewerContext == null) return false;
    if (identical(focusContext, viewerContext)) return true;
    var isInsideViewer = false;
    focusContext.visitAncestorElements((element) {
      if (identical(element, viewerContext)) {
        isInsideViewer = true;
        return false;
      }
      return true;
    });
    return isInsideViewer;
  }

  bool _handleHardwareNavigation(KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) return false;
    if (!mounted ||
        ModalRoute.of(context)?.isCurrent != true ||
        widget.controller.status != ReaderStatus.ready ||
        _textInputHasFocus ||
        !_readerCanvasHasFocus) {
      return false;
    }
    final keyboard = HardwareKeyboard.instance;
    if (keyboard.isControlPressed ||
        keyboard.isAltPressed ||
        keyboard.isMetaPressed ||
        keyboard.isShiftPressed) {
      return false;
    }
    final shortcuts = AppCommandRegistry.resolvedTokens(
      widget.shortcutOverrides,
    );
    Future<void> Function()? navigation;
    if (event.logicalKey == LogicalKeyboardKey.arrowRight &&
        shortcuts[AppCommandId.nextPage] == 'arrow-right') {
      navigation = widget.controller.goToNextReadingUnit;
    } else if (event.logicalKey == LogicalKeyboardKey.arrowLeft &&
        shortcuts[AppCommandId.previousPage] == 'arrow-left') {
      navigation = widget.controller.goToPreviousReadingUnit;
    }
    if (navigation == null) return false;
    widget.controller.registerActivity();
    _revealControls();
    unawaited(navigation());
    return true;
  }

  Map<ShortcutActivator, VoidCallback> _shortcutBindings(BuildContext context) {
    final bindings = AppCommandRegistry.shortcuts(
      _commandActions(context),
      overrides: widget.shortcutOverrides,
    );
    final shortcuts = AppCommandRegistry.resolvedTokens(
      widget.shortcutOverrides,
    );
    if (shortcuts[AppCommandId.nextPage] == 'arrow-right') {
      bindings.remove(const SingleActivator(LogicalKeyboardKey.arrowRight));
    }
    if (shortcuts[AppCommandId.previousPage] == 'arrow-left') {
      bindings.remove(const SingleActivator(LogicalKeyboardKey.arrowLeft));
    }
    return bindings;
  }

  void _runReaderNavigation(Future<void> Function() action) {
    if (_textInputHasFocus) return;
    unawaited(action());
  }

  void _setReaderMode(ReaderMode mode) {
    widget.controller.setReaderMode(mode);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _requestReaderFocus();
    });
  }

  void _resizePanel(double width) {
    final next = width
        .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
        .toDouble();
    if ((next - _panelWidthValue).abs() < 0.5) return;
    setState(() => _panelWidthValue = next);
    _chromeCoordinator.setPanelWidth(next);
    _panelSaveTimer?.cancel();
    _panelSaveTimer = Timer(const Duration(milliseconds: 240), () {
      widget.onPanelWidthChanged?.call(_panelWidthValue);
    });
  }

  Map<AppCommandId, VoidCallback> _commandActions(BuildContext context) => {
    AppCommandId.openDocument: () => unawaited(widget.onOpenDocument()),
    AppCommandId.search: () => _showPanel(ReaderPanelSection.search),
    AppCommandId.commandPalette: () => _showCommandPalette(context),
    AppCommandId.goToPage: () => _showGoToPage(context),
    AppCommandId.togglePanel: () => _panel == null
        ? _showPanel(ReaderPanelSection.thumbnails)
        : _closePanel(),
    AppCommandId.toggleBookmark: widget.controller.toggleBookmark,
    AppCommandId.zoomIn: () => unawaited(widget.controller.zoomBy(1.18)),
    AppCommandId.zoomOut: () => unawaited(widget.controller.zoomBy(0.85)),
    AppCommandId.resetZoom: () => unawaited(widget.controller.fitPage()),
    AppCommandId.actualSize: () =>
        unawaited(widget.controller.viewController.actualSize()),
    AppCommandId.fullScreen: _toggleFullScreen,
    AppCommandId.toggleChrome: _toggleChrome,
    AppCommandId.toggleImmersive: _toggleImmersive,
    AppCommandId.escape: _escape,
    AppCommandId.nextPage: () =>
        _runReaderNavigation(widget.controller.goToNextReadingUnit),
    AppCommandId.previousPage: () =>
        _runReaderNavigation(widget.controller.goToPreviousReadingUnit),
    AppCommandId.firstPage: () =>
        _runReaderNavigation(widget.controller.goToFirstReadingUnit),
    AppCommandId.lastPage: () =>
        _runReaderNavigation(widget.controller.goToLastReadingUnit),
    AppCommandId.bookMode: () => _setReaderMode(ReaderMode.book),
    AppCommandId.documentMode: () => _setReaderMode(ReaderMode.document),
    AppCommandId.showThumbnails: () =>
        _showPanel(ReaderPanelSection.thumbnails),
    AppCommandId.showOutline: () => _showPanel(ReaderPanelSection.outline),
    AppCommandId.showAnnotations: () =>
        _showPanel(ReaderPanelSection.annotations),
    AppCommandId.navigationBack: () =>
        unawaited(widget.controller.historyBack()),
    AppCommandId.navigationForward: () =>
        unawaited(widget.controller.historyForward()),
    AppCommandId.fitWidth: () => unawaited(widget.controller.fitWidth()),
    AppCommandId.fitPage: () => unawaited(widget.controller.fitPage()),
    AppCommandId.changeReaderTheme: () => _showThemeChooser(context),
    AppCommandId.documentInfo:
        widget.onDocumentInfo ?? () => _showDocumentInfo(context),
    AppCommandId.settings: widget.onOpenSettings,
  };

  void _showPanel(ReaderPanelSection section) {
    if (_panel == section) {
      _closePanel();
      return;
    }
    _focusBeforePanel ??= FocusManager.instance.primaryFocus;
    setState(() {
      _panel = section;
      _controlsVisible = true;
    });
    _chromeCoordinator.openPanel(
      section,
      presentation: _panelPinned
          ? ReaderPanelPresentation.pinned
          : ReaderPanelPresentation.overlay,
    );
    widget.onPanelChanged?.call(section);
    widget.onChromeVisibilityChanged?.call(true);
  }

  void _closePanel() {
    if (_panel == null) return;
    setState(() => _panel = null);
    _chromeCoordinator.closePanel();
    widget.onPanelChanged?.call(null);
    final previousFocus = _focusBeforePanel;
    _focusBeforePanel = null;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (previousFocus?.canRequestFocus ?? false) {
        previousFocus?.requestFocus();
      }
    });
  }

  void _togglePanelPinned(bool pinned) {
    if (_panel == null ||
        AppBreakpoints.classify(MediaQuery.sizeOf(context).width) !=
            AppWindowClass.expanded) {
      return;
    }
    setState(() => _panelPinned = pinned);
    _chromeCoordinator.openPanel(
      _panel!,
      presentation: pinned
          ? ReaderPanelPresentation.pinned
          : ReaderPanelPresentation.overlay,
    );
  }

  void _escape() {
    if (_panel != null) {
      _closePanel();
    } else if (_immersive) {
      _setImmersive(false);
    } else if (widget.controller.isFullScreen) {
      unawaited(_leaveFullScreen());
    }
  }

  void _toggleFullScreen() => unawaited(_toggleFullScreenNow());

  Future<void> _toggleFullScreenNow() async {
    _controlsTimer?.cancel();
    final controlsWereVisible = _controlsVisible;
    try {
      await widget.controller.toggleFullScreen();
    } on Object {
      // Window services can fail during monitor/DPI transitions. Keep the
      // reader usable and reconcile the presentation state with the window.
      _chromeCoordinator.setFullscreen(widget.controller.isFullScreen);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('No se pudo activar pantalla completa'),
            behavior: SnackBarBehavior.floating,
          ),
        );
        _revealControls();
      }
      return;
    }
    _chromeCoordinator.setFullscreen(widget.controller.isFullScreen);
    if (widget.controller.isFullScreen) {
      // Fullscreen is orthogonal to immersive mode: preserve the user's
      // chrome choice and only change the window bounds.
      if (mounted) setState(() {});
      if (controlsWereVisible) _revealControls();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Pantalla completa · F11 para salir'),
            duration: Duration(milliseconds: 1200),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
    } else {
      if (mounted) setState(() {});
      _revealControls();
    }
  }

  Future<void> _leaveFullScreen() async {
    try {
      await widget.controller.leaveFullScreen();
    } on Object {
      _chromeCoordinator.setFullscreen(widget.controller.isFullScreen);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('No se pudo salir de pantalla completa'),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
      return;
    }
    _chromeCoordinator.setFullscreen(false);
    if (mounted) setState(() {});
    _revealControls();
  }

  void _toggleChrome() {
    final show = !_controlsVisible;
    setState(() {
      _controlsVisible = show;
    });
    if (show) {
      _chromeCoordinator.reveal();
    } else {
      _chromeCoordinator.toggleUserChrome();
    }
    widget.onChromeVisibilityChanged?.call(show);
  }

  void _toggleImmersive() => _setImmersive(!_immersive);

  void _setImmersive(bool value) {
    _controlsTimer?.cancel();
    _cursorTimer?.cancel();
    if (value && _panel != null) {
      // Do not leave an interactive panel mounted behind an immersive canvas.
      setState(() => _panel = null);
      _chromeCoordinator.closePanel();
      widget.onPanelChanged?.call(null);
    }
    setState(() {
      _immersive = value;
      if (value) _controlsVisible = false;
      if (!value) _controlsVisible = true;
    });
    if (!value) {
      _showCursor();
    } else {
      _scheduleCursorHide();
    }
    _chromeCoordinator.setImmersive(value);
    widget.onChromeVisibilityChanged?.call(!value);
    if (!value) _revealControls();
  }

  void _showCursor() {
    if (!_cursorHidden) return;
    if (mounted) setState(() => _cursorHidden = false);
  }

  void _scheduleCursorHide() {
    _cursorTimer?.cancel();
    if (!_immersive || _chromeCoordinator.state.controlsHaveFocus) return;
    _cursorTimer = Timer(const Duration(seconds: 2), () {
      if (mounted &&
          _immersive &&
          !_chromeCoordinator.state.controlsHaveFocus) {
        setState(() => _cursorHidden = true);
      }
    });
  }

  void _revealControls() {
    if (!mounted || _immersive) return;
    if (!_controlsVisible) {
      setState(() => _controlsVisible = true);
      widget.onChromeVisibilityChanged?.call(true);
    }
    _chromeCoordinator.reveal();
    _controlsTimer?.cancel();
    final preferences = widget.controller.preferences;
    if (widget.controlsAutoHide ?? preferences?.controlsAutoHide ?? true) {
      _controlsTimer = Timer(
        widget.controller.isFullScreen
            ? const Duration(milliseconds: 2500)
            : AppMotion.controlsAutoHide,
        () {
          final region = _chromeCoordinator.state.pointerRegion;
          final pointerOverChrome =
              region == ReaderPointerRegion.topEdge ||
              region == ReaderPointerRegion.bottomEdge;
          if (mounted && _panel == null && !pointerOverChrome) {
            setState(() => _controlsVisible = false);
            _chromeCoordinator.hideAutomatically();
            widget.onChromeVisibilityChanged?.call(false);
          }
        },
      );
    }
  }

  Future<void> _showCommandPalette(BuildContext context) async {
    _controlsTimer?.cancel();
    await showDialog<void>(
      context: context,
      builder: (_) => CommandPalette(
        actions: _commandActions(context),
        reduceMotion: widget.reduceMotion,
        shortcutOverrides: widget.shortcutOverrides,
        recentDocuments: widget.recentDocuments,
        onOpenRecent: widget.onOpenPath == null
            ? null
            : (document) => unawaited(widget.onOpenPath!(document.path)),
      ),
    );
    _revealControls();
  }

  Future<void> _showGoToPage(BuildContext context) async {
    final textController = TextEditingController(
      text: '${widget.controller.currentPageIndex + 1}',
    );
    var previewPage = widget.controller.currentPageIndex + 1;
    final page = await showDialog<int>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('Ir a página'),
          content: SizedBox(
            width: 360,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  key: const Key('go-to-page-field'),
                  controller: textController,
                  autofocus: true,
                  keyboardType: TextInputType.number,
                  decoration: InputDecoration(
                    labelText: 'Número de página',
                    helperText: 'Entre 1 y ${widget.controller.pageCount}',
                  ),
                  onChanged: (value) {
                    final parsed = int.tryParse(value);
                    if (parsed != null) {
                      setDialogState(
                        () => previewPage = parsed.clamp(
                          1,
                          widget.controller.pageCount,
                        ),
                      );
                    }
                  },
                  onSubmitted: (value) =>
                      Navigator.pop(context, int.tryParse(value)),
                ),
                if (widget.controller.pageCount >= 20) ...[
                  const SizedBox(height: AppSpacing.space12),
                  Semantics(
                    label: 'Recorrido rápido de páginas',
                    value:
                        'Página $previewPage de ${widget.controller.pageCount}',
                    child: Slider(
                      key: const Key('reader-page-scrubber'),
                      min: 1,
                      max: widget.controller.pageCount.toDouble(),
                      divisions: math.min(widget.controller.pageCount - 1, 500),
                      value: previewPage.toDouble(),
                      label: '$previewPage',
                      onChanged: (value) {
                        setDialogState(() {
                          previewPage = value.round();
                          textController.text = '$previewPage';
                        });
                      },
                    ),
                  ),
                  Text(
                    _outlineLabelForPage(previewPage - 1) ??
                        'Página $previewPage',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.labelMedium,
                  ),
                ],
              ],
            ),
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
      ),
    );
    textController.dispose();
    if (page != null) await widget.controller.goToPage(page);
  }

  String? _outlineLabelForPage(int pageIndex) {
    String? label;
    void visit(List<PdfOutlineItem> items) {
      for (final item in items) {
        final itemPage = item.pageIndex;
        if (itemPage != null && itemPage <= pageIndex) label = item.title;
        visit(item.children);
      }
    }

    visit(widget.controller.outline);
    return label;
  }

  Future<void> _showThemeChooser(BuildContext context) async {
    final selected = await showDialog<ReaderTheme>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Entorno de lectura'),
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
                            color: context.appColors.borderStrong,
                          ),
                          borderRadius: BorderRadius.circular(
                            AppRadii.indicator,
                          ),
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

  Future<void> _showDocumentInfo(BuildContext context) async {
    final document = widget.controller.document;
    if (document == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.info_outline_rounded),
        title: const Text('Información del documento'),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: SelectionArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _DocumentInfoLine(
                  label: 'Título',
                  value: document.effectiveTitle,
                ),
                if (document.effectiveAuthor case final author?)
                  _DocumentInfoLine(label: 'Autor', value: author),
                _DocumentInfoLine(
                  label: 'Páginas',
                  value: '${document.pageCount}',
                ),
                _DocumentInfoLine(
                  label: 'Tamaño',
                  value: _formatFileSize(document.fileSize),
                ),
                _DocumentInfoLine(
                  label: 'Modificado',
                  value: MaterialLocalizations.of(
                    context,
                  ).formatFullDate(document.fileModifiedAt.toLocal()),
                ),
                _DocumentInfoLine(label: 'Ubicación', value: document.path),
              ],
            ),
          ),
        ),
        actions: [
          TextButton.icon(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: document.path));
              ScaffoldMessenger.of(
                context,
              ).showSnackBar(const SnackBar(content: Text('Ruta copiada.')));
            },
            icon: const Icon(Icons.copy_rounded, size: 18),
            label: const Text('Copiar ruta'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cerrar'),
          ),
        ],
      ),
    );
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
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Separa la página del borde sin modificar el PDF.',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                const SizedBox(height: AppSpacing.space16),
                Slider(
                  value: margin.clamp(8, 72),
                  min: 8,
                  max: 72,
                  divisions: 8,
                  label: '${margin.round()} px',
                  onChanged: (value) => setDialogState(() => margin = value),
                ),
                Align(
                  alignment: Alignment.centerRight,
                  child: Text(
                    '${margin.round()} px',
                    style: Theme.of(context).textTheme.labelMedium,
                  ),
                ),
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
        actionLabel: 'Volver a la biblioteca',
        tone: AppStatusTone.error,
      ),
    );
  }
}
