import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../../app/theme/app_tokens.dart';
import 'reader_side_panel.dart';

enum ReaderChromeVisibility { visible, autoHidden, userHidden }

enum ReaderPanelPresentation { overlay, pinned }

enum ReaderPointerRegion { canvas, topEdge, bottomEdge, panel }

@immutable
final class ReaderUiState {
  const ReaderUiState({
    this.chromeVisibility = ReaderChromeVisibility.visible,
    this.immersive = false,
    this.fullscreen = false,
    this.activePanel,
    this.panelPresentation = ReaderPanelPresentation.overlay,
    this.panelWidth = AppSizes.readerPanelDefault,
    this.activePopover,
    this.controlsHaveFocus = false,
    this.pointerRegion = ReaderPointerRegion.canvas,
  });

  final ReaderChromeVisibility chromeVisibility;
  final bool immersive;
  final bool fullscreen;
  final ReaderPanelSection? activePanel;
  final ReaderPanelPresentation panelPresentation;
  final double panelWidth;
  final String? activePopover;
  final bool controlsHaveFocus;
  final ReaderPointerRegion pointerRegion;

  bool get chromeVisible =>
      chromeVisibility == ReaderChromeVisibility.visible && !immersive;

  ReaderUiState copyWith({
    ReaderChromeVisibility? chromeVisibility,
    bool? immersive,
    bool? fullscreen,
    ReaderPanelSection? activePanel,
    bool clearPanel = false,
    ReaderPanelPresentation? panelPresentation,
    double? panelWidth,
    String? activePopover,
    bool clearPopover = false,
    bool? controlsHaveFocus,
    ReaderPointerRegion? pointerRegion,
  }) => ReaderUiState(
    chromeVisibility: chromeVisibility ?? this.chromeVisibility,
    immersive: immersive ?? this.immersive,
    fullscreen: fullscreen ?? this.fullscreen,
    activePanel: clearPanel ? null : activePanel ?? this.activePanel,
    panelPresentation: panelPresentation ?? this.panelPresentation,
    panelWidth: panelWidth ?? this.panelWidth,
    activePopover: clearPopover ? null : activePopover ?? this.activePopover,
    controlsHaveFocus: controlsHaveFocus ?? this.controlsHaveFocus,
    pointerRegion: pointerRegion ?? this.pointerRegion,
  );

  @override
  bool operator ==(Object other) =>
      other is ReaderUiState &&
      other.chromeVisibility == chromeVisibility &&
      other.immersive == immersive &&
      other.fullscreen == fullscreen &&
      other.activePanel == activePanel &&
      other.panelPresentation == panelPresentation &&
      other.panelWidth == panelWidth &&
      other.activePopover == activePopover &&
      other.controlsHaveFocus == controlsHaveFocus &&
      other.pointerRegion == pointerRegion;

  @override
  int get hashCode => Object.hash(
    chromeVisibility,
    immersive,
    fullscreen,
    activePanel,
    panelPresentation,
    panelWidth,
    activePopover,
    controlsHaveFocus,
    pointerRegion,
  );
}

/// Testable coordinator for reveal/auto-hide rules.  The widget owns the
/// actual focus nodes and calls this class with semantic pointer events.
final class ReaderChromeCoordinator extends ChangeNotifier {
  ReaderChromeCoordinator({
    ReaderUiState initialState = const ReaderUiState(),
    this._autoHide = true,
    this._reduceMotion = false,
    this._disableAnimations = false,
  }) : _state = initialState;

  ReaderUiState _state;
  bool _autoHide;
  bool _reduceMotion;
  bool _disableAnimations;
  Timer? _timer;

  ReaderUiState get state => _state;
  bool get reduceMotion => _reduceMotion;
  bool get disableAnimations => _disableAnimations;
  Duration get revealDuration => _disableAnimations
      ? Duration.zero
      : _reduceMotion
      ? const Duration(milliseconds: 80)
      : AppMotion.chromeReveal;

  Duration get _autoHideDuration => _state.fullscreen
      ? const Duration(milliseconds: 2500)
      : AppMotion.controlsAutoHide;

  void configure({
    bool? autoHide,
    bool? reduceMotion,
    bool? disableAnimations,
  }) {
    _autoHide = autoHide ?? _autoHide;
    _reduceMotion = reduceMotion ?? _reduceMotion;
    _disableAnimations = disableAnimations ?? _disableAnimations;
    if (!_autoHide) _timer?.cancel();
  }

  void reveal() {
    _set(_state.copyWith(chromeVisibility: ReaderChromeVisibility.visible));
    _timer?.cancel();
    if (_autoHide &&
        !_state.immersive &&
        !_state.controlsHaveFocus &&
        _state.activePanel == null) {
      _timer = Timer(_autoHideDuration, hideAutomatically);
    }
  }

  void hideAutomatically() {
    if (_state.controlsHaveFocus || _state.activePanel != null) return;
    _set(_state.copyWith(chromeVisibility: ReaderChromeVisibility.autoHidden));
  }

  void toggleUserChrome() {
    _timer?.cancel();
    final visible =
        _state.chromeVisibility == ReaderChromeVisibility.userHidden;
    _set(
      _state.copyWith(
        chromeVisibility: visible
            ? ReaderChromeVisibility.visible
            : ReaderChromeVisibility.userHidden,
      ),
    );
  }

  void setImmersive(bool value) {
    _timer?.cancel();
    _set(_state.copyWith(immersive: value));
    if (!value) reveal();
  }

  void setFullscreen(bool value) => _set(_state.copyWith(fullscreen: value));

  void openPanel(
    ReaderPanelSection section, {
    ReaderPanelPresentation presentation = ReaderPanelPresentation.overlay,
  }) {
    _set(
      _state.copyWith(
        activePanel: section,
        panelPresentation: presentation,
        chromeVisibility: ReaderChromeVisibility.visible,
      ),
    );
    _timer?.cancel();
  }

  void closePanel() => _set(_state.copyWith(clearPanel: true));

  void setPanelWidth(double width) => _set(
    _state.copyWith(
      panelWidth: width
          .clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth)
          .toDouble(),
    ),
  );

  void setControlsFocus(bool value) {
    _set(_state.copyWith(controlsHaveFocus: value));
    if (value) {
      _timer?.cancel();
    } else {
      reveal();
    }
  }

  void setPointerRegion(ReaderPointerRegion region) =>
      _set(_state.copyWith(pointerRegion: region));

  void setPopover(String? value) {
    if (value == null) {
      _set(_state.copyWith(clearPopover: true));
    } else {
      _set(_state.copyWith(activePopover: value));
    }
  }

  void _set(ReaderUiState next) {
    if (next == _state) return;
    _state = next;
    notifyListeners();
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}
