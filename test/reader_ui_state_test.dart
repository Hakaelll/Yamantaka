import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/reader/presentation/reader_ui_state.dart';
import 'package:yamantaka/features/reader/presentation/reader_side_panel.dart';

void main() {
  test('chrome, immersive mode and fullscreen remain independent states', () {
    final coordinator = ReaderChromeCoordinator(autoHide: false);
    addTearDown(coordinator.dispose);

    coordinator.setFullscreen(true);
    coordinator.setImmersive(true);

    expect(coordinator.state.fullscreen, isTrue);
    expect(coordinator.state.immersive, isTrue);
    expect(coordinator.state.chromeVisible, isFalse);

    coordinator.setImmersive(false);
    expect(coordinator.state.fullscreen, isTrue);
    expect(coordinator.state.chromeVisible, isTrue);
  });

  test('panel state is overlay by default and restores after closing', () {
    final coordinator = ReaderChromeCoordinator(autoHide: false);
    addTearDown(coordinator.dispose);

    coordinator.openPanel(ReaderPanelSection.search);
    expect(coordinator.state.activePanel, ReaderPanelSection.search);
    expect(
      coordinator.state.panelPresentation,
      ReaderPanelPresentation.overlay,
    );

    coordinator.closePanel();
    expect(coordinator.state.activePanel, isNull);
  });

  test('user-hidden chrome is distinct from automatic hiding', () {
    final coordinator = ReaderChromeCoordinator(autoHide: false);
    addTearDown(coordinator.dispose);

    coordinator.toggleUserChrome();
    expect(
      coordinator.state.chromeVisibility,
      ReaderChromeVisibility.userHidden,
    );
    coordinator.hideAutomatically();
    expect(
      coordinator.state.chromeVisibility,
      ReaderChromeVisibility.autoHidden,
    );
  });
}
