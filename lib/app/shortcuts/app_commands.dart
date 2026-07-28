import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

enum AppCommandId {
  openDocument,
  search,
  commandPalette,
  goToPage,
  toggleBookmark,
  zoomIn,
  zoomOut,
  resetZoom,
  fullScreen,
  escape,
  nextPage,
  previousPage,
  firstPage,
  lastPage,
  bookMode,
  documentMode,
  showThumbnails,
  showOutline,
  fitWidth,
  fitPage,
  changeReaderTheme,
  settings,
}

final class AppCommandDefinition {
  const AppCommandDefinition({
    required this.id,
    required this.label,
    this.shortcutLabel,
    this.keywords = const [],
  });

  final AppCommandId id;
  final String label;
  final String? shortcutLabel;
  final List<String> keywords;
}

abstract final class AppCommandRegistry {
  static Map<ShortcutActivator, VoidCallback> applicationShortcuts({
    required VoidCallback openDocument,
  }) => {
    const SingleActivator(LogicalKeyboardKey.keyO, control: true): openDocument,
  };

  static const definitions = <AppCommandDefinition>[
    AppCommandDefinition(
      id: AppCommandId.openDocument,
      label: 'Abrir documento',
      shortcutLabel: 'Ctrl+O',
      keywords: ['pdf', 'archivo'],
    ),
    AppCommandDefinition(
      id: AppCommandId.search,
      label: 'Buscar en el documento',
      shortcutLabel: 'Ctrl+F',
    ),
    AppCommandDefinition(
      id: AppCommandId.goToPage,
      label: 'Ir a una página',
      shortcutLabel: 'Ctrl+L',
    ),
    AppCommandDefinition(
      id: AppCommandId.toggleBookmark,
      label: 'Añadir o retirar marcador',
      shortcutLabel: 'Ctrl+B',
    ),
    AppCommandDefinition(
      id: AppCommandId.bookMode,
      label: 'Activar modo libro',
    ),
    AppCommandDefinition(
      id: AppCommandId.documentMode,
      label: 'Activar modo documento',
    ),
    AppCommandDefinition(
      id: AppCommandId.showThumbnails,
      label: 'Mostrar miniaturas',
    ),
    AppCommandDefinition(id: AppCommandId.showOutline, label: 'Mostrar índice'),
    AppCommandDefinition(id: AppCommandId.fitWidth, label: 'Ajustar al ancho'),
    AppCommandDefinition(
      id: AppCommandId.fitPage,
      label: 'Ajustar a la página',
    ),
    AppCommandDefinition(
      id: AppCommandId.changeReaderTheme,
      label: 'Cambiar tema de lectura',
    ),
    AppCommandDefinition(
      id: AppCommandId.fullScreen,
      label: 'Pantalla completa',
      shortcutLabel: 'F11',
    ),
    AppCommandDefinition(
      id: AppCommandId.settings,
      label: 'Abrir configuración',
    ),
  ];

  static Map<ShortcutActivator, VoidCallback> shortcuts(
    Map<AppCommandId, VoidCallback> actions,
  ) {
    VoidCallback action(AppCommandId id) => actions[id] ?? () {};
    return <ShortcutActivator, VoidCallback>{
      const SingleActivator(LogicalKeyboardKey.keyO, control: true): action(
        AppCommandId.openDocument,
      ),
      const SingleActivator(LogicalKeyboardKey.keyF, control: true): action(
        AppCommandId.search,
      ),
      const SingleActivator(LogicalKeyboardKey.keyK, control: true): action(
        AppCommandId.commandPalette,
      ),
      const SingleActivator(LogicalKeyboardKey.keyL, control: true): action(
        AppCommandId.goToPage,
      ),
      const SingleActivator(LogicalKeyboardKey.keyB, control: true): action(
        AppCommandId.toggleBookmark,
      ),
      const SingleActivator(LogicalKeyboardKey.equal, control: true): action(
        AppCommandId.zoomIn,
      ),
      const SingleActivator(
        LogicalKeyboardKey.equal,
        control: true,
        shift: true,
      ): action(
        AppCommandId.zoomIn,
      ),
      const SingleActivator(LogicalKeyboardKey.minus, control: true): action(
        AppCommandId.zoomOut,
      ),
      const SingleActivator(LogicalKeyboardKey.digit0, control: true): action(
        AppCommandId.resetZoom,
      ),
      const SingleActivator(LogicalKeyboardKey.f11): action(
        AppCommandId.fullScreen,
      ),
      const SingleActivator(LogicalKeyboardKey.escape): action(
        AppCommandId.escape,
      ),
      const SingleActivator(LogicalKeyboardKey.arrowRight): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.pageDown): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.space): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.arrowLeft): action(
        AppCommandId.previousPage,
      ),
      const SingleActivator(LogicalKeyboardKey.pageUp): action(
        AppCommandId.previousPage,
      ),
      const SingleActivator(LogicalKeyboardKey.space, shift: true): action(
        AppCommandId.previousPage,
      ),
      const SingleActivator(LogicalKeyboardKey.home): action(
        AppCommandId.firstPage,
      ),
      const SingleActivator(LogicalKeyboardKey.end): action(
        AppCommandId.lastPage,
      ),
    };
  }
}
