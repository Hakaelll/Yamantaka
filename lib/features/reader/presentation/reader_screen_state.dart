part of 'reader_screen.dart';

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
    if (oldWidget.initialPanelOpen != widget.initialPanelOpen ||
        oldWidget.initialPanelSection != widget.initialPanelSection) {
      final nextPanel = widget.initialPanelOpen
          ? ReaderPanelSection.values
                .where((section) => section.name == widget.initialPanelSection)
                .firstOrNull
          : null;
      if (_panel != nextPanel) {
        _panel = nextPanel;
        if (nextPanel == null) {
          _chromeCoordinator.closePanel();
        } else {
          _chromeCoordinator.openPanel(
            nextPanel,
            presentation: _panelPinned
                ? ReaderPanelPresentation.pinned
                : ReaderPanelPresentation.overlay,
          );
        }
      }
    }
    if ((oldWidget.initialPanelWidth - widget.initialPanelWidth).abs() > 0.5) {
      _panelWidthValue = widget.initialPanelWidth
          .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
          .toDouble();
      _chromeCoordinator.setPanelWidth(_panelWidthValue);
    }
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
    final strings = YamantakaLocalizations.of(context);
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        if (widget.controller.status == ReaderStatus.loading) {
          return Scaffold(
            body: AppStatusView(
              icon: Icons.picture_as_pdf_outlined,
              title: strings.readerLoadingTitle,
              message: strings.readerLoadingBody,
              loading: true,
            ),
          );
        }
        if (widget.controller.status != ReaderStatus.ready ||
            widget.controller.openedDocument == null ||
            widget.controller.preferences == null) {
          final presentation = FailurePresentations.forFailure(
            widget.controller.error,
            strings: strings,
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
                final mediaSize = MediaQuery.sizeOf(context);
                final height = mediaSize.height;
                final compact =
                    AppBreakpoints.classify(mediaSize.width) ==
                    AppWindowClass.compact;
                final atRevealEdge =
                    event.localPosition.dy <= AppSizes.topRevealZone ||
                    event.localPosition.dy >= height - AppSpacing.space12;
                final inTopChrome =
                    event.localPosition.dy <=
                    AppSizes.readerTopBarHeight(compact: compact) +
                        AppSpacing.space8;
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
    final strings = YamantakaLocalizations.of(context);
    final controller = widget.controller;
    final opened = controller.openedDocument!;
    final preferences = controller.preferences!;
    final readerBackground = _readerCanvasColor(
      context,
      preferences.readingTheme,
    );
    final chromeTheme = _readerChromeTheme(context, preferences.readingTheme);
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
    final windowClass = AppBreakpoints.classify(constraints.maxWidth);
    final panelDocked =
        _panel != null && windowClass == AppWindowClass.expanded;
    final readerMargin =
        (preferences.horizontalMargin + preferences.verticalMargin) / 2;
    // The PDF layout already contains the editorial outer margin.  Keeping
    // this separate viewport margin at zero is what makes chrome genuinely
    // overlay-only instead of reserving a hidden toolbar strip.
    const viewportMargin = 0.0;
    _ensureReaderFocus(opened.documentId);

    return ColoredBox(
      color: readerBackground,
      child: Stack(
        fit: StackFit.expand,
        children: [
          Positioned(
            left: 0,
            top: 0,
            right: panelDocked ? panelWidth : 0,
            bottom: 0,
            child: ValueListenableBuilder<int>(
              valueListenable: controller.pageIndexListenable,
              builder: (context, _, child) => Semantics(
                label:
                    '${strings.get(_isBookMode ? 'reader.bookMode' : 'reader.documentMode')}, '
                    '${controller.readingUnitIndicator(direction: widget.readingDirection)}',
                child: child,
              ),
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
                      onInteractionChanged: widget.onViewportInteractionChanged,
                      onTextSelectionChanged: controller.updateTextSelection,
                    ),
                    initialPageIndex: controller.currentPageIndex,
                    onPageChanged: controller.onPageChanged,
                    onError: (error) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(
                          content: Text(strings.get('reader.renderPageFailed')),
                        ),
                      );
                    },
                  ),
                ),
              ),
            ),
          ),
          if (!immersive && controller.textSelection?.hasText == true)
            Positioned(
              left: AppSpacing.space12,
              right: panelDocked
                  ? panelWidth + AppSpacing.space12
                  : AppSpacing.space12,
              bottom: AppSizes.bottomControlHeight + AppSpacing.space24,
              child: Align(
                alignment: Alignment.bottomCenter,
                child: Theme(
                  data: chromeTheme,
                  child: _SelectionAnnotationToolbar(
                    text: controller.textSelection!.selectedText,
                    onCreate: _createSelectionAnnotation,
                    onDismiss: () => controller.updateTextSelection(null),
                  ),
                ),
              ),
            ),
          ValueListenableBuilder<ReaderDestinationCue?>(
            valueListenable: controller.destinationCue,
            builder: (context, cue, _) {
              if (cue == null) return const SizedBox.shrink();
              return Positioned(
                left: 0,
                top: 0,
                right: panelDocked ? panelWidth : 0,
                bottom: 0,
                child: IgnorePointer(
                  child: Semantics(
                    liveRegion: true,
                    label: cue.label,
                    child: TweenAnimationBuilder<double>(
                      key: ValueKey(cue.sequence),
                      duration: AppMotion.whenAllowed(
                        widget.reduceMotion ||
                            MediaQuery.disableAnimationsOf(context),
                        AppMotion.fast,
                      ),
                      tween: Tween(begin: 0, end: 1),
                      builder: (context, opacity, child) =>
                          Opacity(opacity: opacity, child: child),
                      child: DecoratedBox(
                        key: const Key('reader-destination-cue'),
                        decoration: BoxDecoration(
                          border: Border.all(
                            color: context.appColors.focusRing,
                            width: 2,
                          ),
                        ),
                        child: Align(
                          alignment: Alignment.topCenter,
                          child: Padding(
                            padding: EdgeInsets.only(
                              top:
                                  AppSizes.readerTopBarHeight(
                                    compact:
                                        windowClass == AppWindowClass.compact,
                                  ) +
                                  AppSpacing.space8,
                            ),
                            child: AppStatusLabel(
                              icon: Icons.center_focus_strong_rounded,
                              label: cue.label,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              );
            },
          ),
          if (_isBookMode && !immersive)
            Positioned.fill(
              child: ValueListenableBuilder<int>(
                valueListenable: controller.pageIndexListenable,
                builder: (context, _, _) => Stack(
                  children: _pageTurnZones(
                    controller: controller,
                    panelWidth: panelWidth,
                    panelVisible: panelDocked,
                  ),
                ),
              ),
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
              child: Theme(
                data: chromeTheme,
                child: AnimatedSwitcher(
                  duration: _chromeTransition(context, controlsShown),
                  child: ReaderBottomControls(
                    key: const Key('reader-bottom-controls'),
                    controller: controller,
                    compact: windowClass == AppWindowClass.compact,
                    onGoToPage: () => _showGoToPage(context),
                    onChooseTheme: () => _showThemeChooser(context),
                    readingDirection: widget.readingDirection,
                    active: controlsShown,
                    onActivate: _revealControls,
                    showProgress: widget.showProgress,
                    flatChrome:
                        _isElectronicInk || MediaQuery.highContrastOf(context),
                    shortcutOverrides: widget.shortcutOverrides,
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
                  color: context.appColors.nightBackground.withValues(
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
                  widget.reduceMotion ||
                      _isElectronicInk ||
                      MediaQuery.disableAnimationsOf(context),
                  AppMotion.panel,
                ),
                curve: Curves.easeOutCubic,
                tween: Tween(begin: 1, end: 0),
                builder: (context, offset, child) => Transform.translate(
                  offset: Offset(panelWidth * offset, 0),
                  child: child,
                ),
                child: Theme(
                  data: chromeTheme,
                  child: ReaderSidePanel(
                    section: _panel!,
                    controller: controller,
                    engine: widget.engine,
                    width: panelWidth,
                    pinned: _panelPinned,
                    docked: panelDocked,
                    allowPin: windowClass == AppWindowClass.expanded,
                    onPinnedChanged: _togglePanelPinned,
                    onSectionChanged: _showPanel,
                    flatChrome:
                        _isElectronicInk || MediaQuery.highContrastOf(context),
                    onClose: _closePanel,
                    onWidthChanged: _resizePanel,
                    onExportAnnotations: widget.onExportAnnotations,
                  ),
                ),
              ),
            ),
          if (!immersive &&
              widget.showProgress &&
              (!controller.isFullScreen || _controlsVisible))
            ValueListenableBuilder<int>(
              valueListenable: controller.pageIndexListenable,
              builder: (context, pageIndex, _) => Positioned(
                left: 0,
                right: panelDocked ? panelWidth : 0,
                bottom: 0,
                child: LinearProgressIndicator(
                  key: const Key('reader-progress'),
                  value: controller.pageCount == 0
                      ? 0
                      : ReadingProgress.calculate(
                          pageIndex: pageIndex,
                          pageCount: controller.pageCount,
                        ),
                  minHeight: 2,
                  backgroundColor: Colors.transparent,
                ),
              ),
            ),
          if (!immersive)
            ValueListenableBuilder<int>(
              valueListenable: controller.pageIndexListenable,
              builder: (context, pageIndex, _) {
                final hasAnnotations =
                    controller.annotations?.pageHasAnnotations(pageIndex) ??
                    false;
                if (!hasAnnotations) return const SizedBox.shrink();
                return Positioned(
                  left: AppSpacing.space12,
                  top: controlsShown
                      ? AppSizes.readerTopBarHeight(
                              compact: windowClass == AppWindowClass.compact,
                            ) +
                            AppSpacing.space8
                      : AppSpacing.space12,
                  child: Semantics(
                    label: strings.get(
                      'reader.pageHasAnnotations',
                      parameters: {'page': pageIndex + 1},
                    ),
                    child: DecoratedBox(
                      decoration: BoxDecoration(
                        color: context.appColors.surfacePrimary.withValues(
                          alpha: 0.92,
                        ),
                        borderRadius: BorderRadius.circular(AppRadii.capsule),
                        border: Border.all(
                          color: context.appColors.borderStrong,
                        ),
                      ),
                      child: const Padding(
                        padding: EdgeInsets.all(AppSpacing.space8),
                        child: Icon(Icons.rate_review_outlined, size: 16),
                      ),
                    ),
                  ),
                );
              },
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
    final strings = YamantakaLocalizations.of(context);
    final rtl = widget.readingDirection == ReadingDirection.rightToLeft;
    final readingTheme = widget.controller.preferences?.readingTheme;
    final chromeColors = _readerChromeTheme(
      context,
      readingTheme ?? ReaderTheme.ivory,
    ).extension<AppColorTokens>()!;
    Widget zone({
      required bool left,
      required String label,
      required VoidCallback? action,
    }) => Positioned(
      left: left ? 0 : null,
      right: left ? null : (panelVisible ? panelWidth : 0),
      top: AppSizes.readerTopBarHeight(
        compact:
            AppBreakpoints.classify(MediaQuery.sizeOf(context).width) ==
            AppWindowClass.compact,
      ),
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
            hoverColor: chromeColors.surfaceHover.withValues(alpha: 0.45),
            focusColor: chromeColors.focusRing.withValues(alpha: 0.18),
            child: Center(
              child: AnimatedOpacity(
                duration: _chromeTransition(context, true),
                opacity: action == null ? 0.32 : 1,
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: chromeColors.surfaceFloating.withValues(alpha: 0.82),
                    borderRadius: BorderRadius.circular(AppRadii.control),
                    border: Border.all(color: chromeColors.borderSubtle),
                    boxShadow: [
                      BoxShadow(
                        color: chromeColors.ambientShadow,
                        blurRadius: AppShadow.floatingControlBlur,
                        offset: const Offset(
                          0,
                          AppShadow.floatingControlOffset,
                        ),
                      ),
                    ],
                  ),
                  child: SizedBox.square(
                    dimension: 40,
                    child: Icon(
                      left
                          ? Icons.chevron_left_rounded
                          : Icons.chevron_right_rounded,
                      color: chromeColors.iconActive,
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
        label: strings.get(rtl ? 'reader.nextSpread' : 'reader.previousSpread'),
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
        label: strings.get(rtl ? 'reader.previousSpread' : 'reader.nextSpread'),
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
    return Color(theme.colorValue);
  }

  ThemeData _readerChromeTheme(BuildContext context, ReaderTheme theme) {
    final density =
        Theme.of(context).extension<AppDensityTokens>()?.interfaceDensity ??
        AppInterfaceDensity.comfortable;
    final highContrast = MediaQuery.highContrastOf(context);
    if (_isElectronicInk) {
      return AppTheme.light(density: density, highContrast: highContrast);
    }
    return switch (theme) {
      ReaderTheme.dark || ReaderTheme.black => AppTheme.dark(
        density: density,
        highContrast: highContrast,
      ),
      _ => AppTheme.light(density: density, highContrast: highContrast),
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
      offset: const Offset(0, 1),
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

  void _setPageLayout(PageLayoutMode layout) {
    widget.controller.setPageLayout(layout);
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
    _panelSaveTimer = Timer(AppMotion.persistenceDebounce, () {
      widget.onPanelWidthChanged?.call(_panelWidthValue);
    });
  }

  Map<AppCommandId, VoidCallback> _commandActions(BuildContext context) => {
    AppCommandId.openDocument: () => unawaited(widget.onOpenDocument()),
    AppCommandId.search: () => _showPanel(ReaderPanelSection.search),
    AppCommandId.commandPalette: () => _showCommandPalette(context),
    AppCommandId.goToPage: () => _showGoToPage(context),
    AppCommandId.documentNavigator: () =>
        unawaited(showDocumentNavigatorDialog(context, widget.controller)),
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
    AppCommandId.singlePageLayout: () => _setPageLayout(PageLayoutMode.single),
    AppCommandId.facingPageLayout: () => _setPageLayout(PageLayoutMode.facing),
    AppCommandId.continuousPageLayout: () =>
        _setPageLayout(PageLayoutMode.continuous),
    AppCommandId.showThumbnails: () =>
        _showPanel(ReaderPanelSection.thumbnails),
    AppCommandId.showOutline: () => _showPanel(ReaderPanelSection.outline),
    AppCommandId.showAnnotations: () =>
        _showPanel(ReaderPanelSection.annotations),
    AppCommandId.navigationBack: () =>
        unawaited(widget.controller.historyBack()),
    AppCommandId.navigationForward: () =>
        unawaited(widget.controller.historyForward()),
    AppCommandId.savePositionA: () =>
        _saveTemporaryPosition(ReaderTemporaryPositionSlot.a),
    AppCommandId.savePositionB: () =>
        _saveTemporaryPosition(ReaderTemporaryPositionSlot.b),
    AppCommandId.goToPositionA: () =>
        unawaited(_goToTemporaryPosition(ReaderTemporaryPositionSlot.a)),
    AppCommandId.goToPositionB: () =>
        unawaited(_goToTemporaryPosition(ReaderTemporaryPositionSlot.b)),
    AppCommandId.swapPositions: _swapTemporaryPositions,
    AppCommandId.undoAnnotation: _undoAnnotation,
    AppCommandId.redoAnnotation: _redoAnnotation,
    AppCommandId.fitWidth: () => unawaited(widget.controller.fitWidth()),
    AppCommandId.fitPage: () => unawaited(widget.controller.fitPage()),
    AppCommandId.changeReaderTheme: () => _showThemeChooser(context),
    AppCommandId.documentInfo:
        widget.onDocumentInfo ?? () => _showDocumentInfo(context),
    AppCommandId.settings: widget.onOpenSettings,
  };

  void _saveTemporaryPosition(ReaderTemporaryPositionSlot slot) {
    final strings = YamantakaLocalizations.of(context);
    final saved = widget.controller.saveTemporaryPosition(slot);
    _showTemporaryPositionNotice(
      saved == null
          ? strings.get('failure.none.title')
          : strings.get(
              'reader.positionSaved',
              parameters: {'slot': slot.name.toUpperCase()},
            ),
    );
  }

  Future<void> _goToTemporaryPosition(ReaderTemporaryPositionSlot slot) async {
    final strings = YamantakaLocalizations.of(context);
    final restored = await widget.controller.goToTemporaryPosition(slot);
    if (!mounted) return;
    _showTemporaryPositionNotice(
      restored
          ? strings.get(
              'reader.positionRestored',
              parameters: {'slot': slot.name.toUpperCase()},
            )
          : strings.get(
              'reader.positionNotSaved',
              parameters: {'slot': slot.name.toUpperCase()},
            ),
    );
  }

  void _swapTemporaryPositions() {
    final strings = YamantakaLocalizations.of(context);
    final swapped = widget.controller.swapTemporaryPositions();
    _showTemporaryPositionNotice(
      swapped
          ? strings.get('reader.positionsSwapped')
          : strings.get('reader.noTemporaryPositions'),
    );
  }

  void _showTemporaryPositionNotice(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Semantics(liveRegion: true, child: Text(message)),
        duration: const Duration(milliseconds: 1400),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  void _undoAnnotation() {
    final strings = YamantakaLocalizations.of(context);
    final changed = widget.controller.annotations?.undo() ?? false;
    _showTemporaryPositionNotice(
      changed
          ? strings.get('reader.annotationUndone')
          : strings.get('reader.noAnnotationUndo'),
    );
  }

  void _redoAnnotation() {
    final strings = YamantakaLocalizations.of(context);
    final changed = widget.controller.annotations?.redo() ?? false;
    _showTemporaryPositionNotice(
      changed
          ? strings.get('reader.annotationRedone')
          : strings.get('reader.noAnnotationRedo'),
    );
  }

  void _createSelectionAnnotation(AnnotationType type) {
    final result = widget.controller.createAnnotationFromSelection(type: type);
    if (result == null) {
      _showTemporaryPositionNotice('No se pudo crear la anotación.');
      return;
    }
    widget.controller.updateTextSelection(null);
    _showTemporaryPositionNotice(
      result.usedPreciseGeometry
          ? 'Anotación vinculada a la selección.'
          : 'La geometría no era fiable; se creó una nota de página.',
    );
  }

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
          SnackBar(
            content: Text(
              YamantakaLocalizations.of(
                context,
              ).get('reader.enterFullScreenFailed'),
            ),
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
          SnackBar(
            content: Text(
              YamantakaLocalizations.of(context).get('reader.fullScreenHint'),
            ),
            duration: const Duration(milliseconds: 1200),
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
          SnackBar(
            content: Text(
              YamantakaLocalizations.of(
                context,
              ).get('reader.exitFullScreenFailed'),
            ),
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
            ? AppMotion.fullscreenControlsAutoHide
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
    final strings = YamantakaLocalizations.of(context);
    final textController = TextEditingController(
      text: '${widget.controller.currentPageIndex + 1}',
    );
    var previewPage = widget.controller.currentPageIndex + 1;
    PageDestination? validation;
    final destination = await showDialog<Object>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) {
          void validate([String? raw]) {
            final parsed = PageDestination.parse(
              raw ?? textController.text,
              pageCount: widget.controller.pageCount,
            );
            setDialogState(() {
              validation = parsed;
              if (parsed.page != null) previewPage = parsed.page!;
            });
          }

          void submit() {
            final parsed = PageDestination.parse(
              textController.text,
              pageCount: widget.controller.pageCount,
            );
            if (parsed.isValid) {
              Navigator.pop(context, parsed.page);
            } else {
              setDialogState(() => validation = parsed);
            }
          }

          String? errorText() => switch (validation?.error) {
            PageDestinationError.invalidFormat || PageDestinationError.empty =>
              strings.get('reader.pageDestinationInvalid'),
            PageDestinationError.outOfRange => strings.get(
              'reader.pageDestinationOutOfRange',
              parameters: {'count': widget.controller.pageCount},
            ),
            null => null,
          };

          return AlertDialog(
            title: Text(strings.get('command.goToPage')),
            content: SizedBox(
              width: 390,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  TextField(
                    key: const Key('go-to-page-field'),
                    controller: textController,
                    autofocus: true,
                    textInputAction: TextInputAction.go,
                    decoration: InputDecoration(
                      labelText: strings.get('reader.pageDestination'),
                      helperText: strings.get('reader.pageDestinationHint'),
                      errorText: errorText(),
                    ),
                    onChanged: validate,
                    onSubmitted: (_) => submit(),
                  ),
                  const SizedBox(height: AppSpacing.space8),
                  Wrap(
                    spacing: AppSpacing.space8,
                    runSpacing: AppSpacing.space4,
                    children: [
                      ActionChip(
                        avatar: const Icon(Icons.first_page_rounded, size: 17),
                        label: Text(strings.get('reader.firstPage')),
                        onPressed: () {
                          textController.text = '1';
                          validate('1');
                        },
                      ),
                      ActionChip(
                        avatar: const Icon(Icons.last_page_rounded, size: 17),
                        label: Text(strings.get('reader.lastPage')),
                        onPressed: () {
                          final last = '${widget.controller.pageCount}';
                          textController.text = last;
                          validate(last);
                        },
                      ),
                      ActionChip(
                        avatar: const Icon(Icons.undo_rounded, size: 17),
                        label: Text(strings.get('command.navigationBack')),
                        onPressed:
                            widget.controller.navigationHistory?.canGoBack ==
                                true
                            ? () => Navigator.pop(
                                context,
                                _PagePickerHistory.back,
                              )
                            : null,
                      ),
                      ActionChip(
                        avatar: const Icon(Icons.redo_rounded, size: 17),
                        label: Text(strings.get('command.navigationForward')),
                        onPressed:
                            widget.controller.navigationHistory?.canGoForward ==
                                true
                            ? () => Navigator.pop(
                                context,
                                _PagePickerHistory.forward,
                              )
                            : null,
                      ),
                    ],
                  ),
                  if (widget.controller.pageCount >= 20) ...[
                    const SizedBox(height: AppSpacing.space8),
                    Semantics(
                      label: strings.get('reader.pageScrubber'),
                      value: strings.get(
                        'reader.pageIndicator',
                        parameters: {
                          'page': previewPage,
                          'count': widget.controller.pageCount,
                        },
                      ),
                      child: Slider(
                        key: const Key('reader-page-scrubber'),
                        min: 1,
                        max: widget.controller.pageCount.toDouble(),
                        divisions: math.min(
                          widget.controller.pageCount - 1,
                          500,
                        ),
                        value: previewPage.toDouble(),
                        label: '$previewPage',
                        onChanged: (value) {
                          final page = value.round();
                          textController.text = '$page';
                          validate('$page');
                        },
                      ),
                    ),
                    Text(
                      _outlineLabelForPage(previewPage - 1) ??
                          strings.get(
                            'reader.page',
                            parameters: {'page': previewPage},
                          ),
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
                child: Text(strings.get('action.cancel')),
              ),
              FilledButton(
                onPressed: submit,
                child: Text(strings.get('action.go')),
              ),
            ],
          );
        },
      ),
    );
    textController.dispose();
    if (!mounted) return;
    switch (destination) {
      case final int page:
        await widget.controller.goToPage(page);
      case _PagePickerHistory.back:
        await widget.controller.historyBack();
      case _PagePickerHistory.forward:
        await widget.controller.historyForward();
      case null:
        break;
      default:
        break;
    }
    if (mounted) _readerFocusNode.requestFocus();
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
    final strings = YamantakaLocalizations.of(context);
    final selected = await showDialog<ReaderTheme>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('reader.readingEnvironment')),
        content: SizedBox(
          width: 400,
          child: Wrap(
            spacing: AppSpacing.space8,
            runSpacing: AppSpacing.space8,
            children: [
              for (final theme in ReaderTheme.values)
                _ReaderThemeChoice(
                  theme: theme,
                  selected:
                      widget.controller.preferences?.readingTheme == theme,
                  onSelected: () => Navigator.pop(context, theme),
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.cancel')),
          ),
        ],
      ),
    );
    if (selected != null) widget.controller.setTheme(selected);
  }

  Future<void> _showDocumentInfo(BuildContext context) async {
    final strings = YamantakaLocalizations.of(context);
    final document = widget.controller.document;
    if (document == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.info_outline_rounded),
        title: Text(strings.get('command.documentInfo')),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: SelectionArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _DocumentInfoLine(
                  label: strings.get('reader.documentTitle'),
                  value: document.effectiveTitle,
                ),
                if (document.effectiveAuthor case final author?)
                  _DocumentInfoLine(
                    label: strings.get('reader.documentAuthor'),
                    value: author,
                  ),
                _DocumentInfoLine(
                  label: strings.get('reader.documentPages'),
                  value: '${document.pageCount}',
                ),
                _DocumentInfoLine(
                  label: strings.get('reader.documentSize'),
                  value: _formatFileSize(document.fileSize),
                ),
                _DocumentInfoLine(
                  label: strings.get('reader.documentModified'),
                  value: MaterialLocalizations.of(
                    context,
                  ).formatFullDate(document.fileModifiedAt.toLocal()),
                ),
                _DocumentInfoLine(
                  label: strings.get('reader.documentLocation'),
                  value: document.path,
                ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton.icon(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: document.path));
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text(strings.get('reader.pathCopied'))),
              );
            },
            icon: const Icon(Icons.copy_rounded, size: 18),
            label: Text(strings.get('reader.copyPath')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.close')),
          ),
        ],
      ),
    );
  }

  Future<void> _showMarginChooser(BuildContext context) async {
    final strings = YamantakaLocalizations.of(context);
    final preferences = widget.controller.preferences;
    if (preferences == null) return;
    var margin =
        (preferences.horizontalMargin + preferences.verticalMargin) / 2;
    final selected = await showDialog<double>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(strings.get('reader.readingMargins')),
          content: SizedBox(
            width: 360,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  strings.get('reader.readingMargins.description'),
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
              child: Text(strings.get('action.cancel')),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, margin),
              child: Text(strings.get('action.apply')),
            ),
          ],
        ),
      ),
    );
    if (selected != null) widget.controller.setMargins(selected);
  }
}

enum _PagePickerHistory { back, forward }
