import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

enum AppCommandId {
  openDocument,
  search,
  commandPalette,
  goToPage,
  togglePanel,
  toggleBookmark,
  zoomIn,
  zoomOut,
  resetZoom,
  actualSize,
  fullScreen,
  toggleChrome,
  toggleImmersive,
  escape,
  nextPage,
  previousPage,
  firstPage,
  lastPage,
  bookMode,
  documentMode,
  showThumbnails,
  showOutline,
  showAnnotations,
  navigationBack,
  navigationForward,
  fitWidth,
  fitPage,
  changeReaderTheme,
  documentInfo,
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

  AppCommandDefinition withShortcut(String? value) => AppCommandDefinition(
    id: id,
    label: label,
    shortcutLabel: value,
    keywords: keywords,
  );
}

final class AppShortcutChoice {
  const AppShortcutChoice({
    required this.token,
    required this.label,
    required this.activator,
  });

  final String token;
  final String label;
  final SingleActivator activator;
}

abstract final class AppCommandRegistry {
  static Map<ShortcutActivator, VoidCallback> applicationShortcuts({
    required VoidCallback openDocument,
    String overrides = '{}',
  }) {
    final token = resolvedTokens(overrides)[AppCommandId.openDocument]!;
    final choice = shortcutChoices.firstWhere((item) => item.token == token);
    return {choice.activator: openDocument};
  }

  static const configurableIds = <AppCommandId>{
    AppCommandId.openDocument,
    AppCommandId.search,
    AppCommandId.commandPalette,
    AppCommandId.goToPage,
    AppCommandId.togglePanel,
    AppCommandId.toggleBookmark,
    AppCommandId.nextPage,
    AppCommandId.previousPage,
    AppCommandId.zoomIn,
    AppCommandId.zoomOut,
    AppCommandId.actualSize,
    AppCommandId.fitWidth,
    AppCommandId.fullScreen,
    AppCommandId.toggleChrome,
    AppCommandId.toggleImmersive,
  };

  /// These activators remain available to structural reader navigation.
  static const reservedShortcutTokens = <String>{'page-down', 'page-up'};

  static const shortcutChoices = <AppShortcutChoice>[
    AppShortcutChoice(
      token: 'ctrl-o',
      label: 'Ctrl+O',
      activator: SingleActivator(LogicalKeyboardKey.keyO, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-o',
      label: 'Ctrl+Mayús+O',
      activator: SingleActivator(
        LogicalKeyboardKey.keyO,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-f',
      label: 'Ctrl+F',
      activator: SingleActivator(LogicalKeyboardKey.keyF, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-f',
      label: 'Ctrl+Mayús+F',
      activator: SingleActivator(
        LogicalKeyboardKey.keyF,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-k',
      label: 'Ctrl+K',
      activator: SingleActivator(LogicalKeyboardKey.keyK, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-k',
      label: 'Ctrl+Mayús+K',
      activator: SingleActivator(
        LogicalKeyboardKey.keyK,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-l',
      label: 'Ctrl+L',
      activator: SingleActivator(LogicalKeyboardKey.keyL, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-g',
      label: 'Ctrl+G',
      activator: SingleActivator(LogicalKeyboardKey.keyG, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-0',
      label: 'Ctrl+0',
      activator: SingleActivator(LogicalKeyboardKey.digit0, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-1',
      label: 'Ctrl+1',
      activator: SingleActivator(LogicalKeyboardKey.digit1, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-2',
      label: 'Ctrl+2',
      activator: SingleActivator(LogicalKeyboardKey.digit2, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-i',
      label: 'Ctrl+Mayús+I',
      activator: SingleActivator(
        LogicalKeyboardKey.keyI,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-j',
      label: 'Ctrl+J',
      activator: SingleActivator(LogicalKeyboardKey.keyJ, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-j',
      label: 'Ctrl+Mayús+J',
      activator: SingleActivator(
        LogicalKeyboardKey.keyJ,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-b',
      label: 'Ctrl+B',
      activator: SingleActivator(LogicalKeyboardKey.keyB, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-d',
      label: 'Ctrl+D',
      activator: SingleActivator(LogicalKeyboardKey.keyD, control: true),
    ),
    AppShortcutChoice(
      token: 'arrow-right',
      label: '→',
      activator: SingleActivator(LogicalKeyboardKey.arrowRight),
    ),
    AppShortcutChoice(
      token: 'page-down',
      label: 'Av Pág',
      activator: SingleActivator(LogicalKeyboardKey.pageDown),
    ),
    AppShortcutChoice(
      token: 'ctrl-right',
      label: 'Ctrl+→',
      activator: SingleActivator(LogicalKeyboardKey.arrowRight, control: true),
    ),
    AppShortcutChoice(
      token: 'arrow-left',
      label: '←',
      activator: SingleActivator(LogicalKeyboardKey.arrowLeft),
    ),
    AppShortcutChoice(
      token: 'page-up',
      label: 'Re Pág',
      activator: SingleActivator(LogicalKeyboardKey.pageUp),
    ),
    AppShortcutChoice(
      token: 'ctrl-left',
      label: 'Ctrl+←',
      activator: SingleActivator(LogicalKeyboardKey.arrowLeft, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-plus',
      label: 'Ctrl++',
      activator: SingleActivator(LogicalKeyboardKey.equal, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-plus',
      label: 'Ctrl+Mayús++',
      activator: SingleActivator(
        LogicalKeyboardKey.equal,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'ctrl-minus',
      label: 'Ctrl+-',
      activator: SingleActivator(LogicalKeyboardKey.minus, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-minus',
      label: 'Ctrl+Mayús+-',
      activator: SingleActivator(
        LogicalKeyboardKey.minus,
        control: true,
        shift: true,
      ),
    ),
    AppShortcutChoice(
      token: 'f11',
      label: 'F11',
      activator: SingleActivator(LogicalKeyboardKey.f11),
    ),
    AppShortcutChoice(
      token: 'ctrl-enter',
      label: 'Ctrl+Enter',
      activator: SingleActivator(LogicalKeyboardKey.enter, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-h',
      label: 'Ctrl+H',
      activator: SingleActivator(LogicalKeyboardKey.keyH, control: true),
    ),
    AppShortcutChoice(
      token: 'ctrl-shift-h',
      label: 'Ctrl+Mayús+H',
      activator: SingleActivator(
        LogicalKeyboardKey.keyH,
        control: true,
        shift: true,
      ),
    ),
  ];

  static const defaultTokens = <AppCommandId, String>{
    AppCommandId.openDocument: 'ctrl-o',
    AppCommandId.search: 'ctrl-f',
    AppCommandId.commandPalette: 'ctrl-k',
    AppCommandId.goToPage: 'ctrl-g',
    AppCommandId.togglePanel: 'ctrl-j',
    AppCommandId.toggleBookmark: 'ctrl-b',
    AppCommandId.nextPage: 'arrow-right',
    AppCommandId.previousPage: 'arrow-left',
    AppCommandId.zoomIn: 'ctrl-plus',
    AppCommandId.zoomOut: 'ctrl-minus',
    AppCommandId.actualSize: 'ctrl-1',
    AppCommandId.fitWidth: 'ctrl-2',
    AppCommandId.fullScreen: 'f11',
    AppCommandId.toggleChrome: 'ctrl-h',
    AppCommandId.toggleImmersive: 'ctrl-shift-i',
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
      id: AppCommandId.commandPalette,
      label: 'Abrir paleta de comandos',
      shortcutLabel: 'Ctrl+K',
      keywords: ['acciones', 'comandos'],
    ),
    AppCommandDefinition(
      id: AppCommandId.goToPage,
      label: 'Ir a una página',
      shortcutLabel: 'Ctrl+G',
    ),
    AppCommandDefinition(
      id: AppCommandId.nextPage,
      label: 'Página siguiente',
      shortcutLabel: '→',
    ),
    AppCommandDefinition(
      id: AppCommandId.previousPage,
      label: 'Página anterior',
      shortcutLabel: '←',
    ),
    AppCommandDefinition(
      id: AppCommandId.togglePanel,
      label: 'Abrir o cerrar panel',
      shortcutLabel: 'Ctrl+J',
    ),
    AppCommandDefinition(
      id: AppCommandId.toggleBookmark,
      label: 'Añadir o retirar marcador',
      shortcutLabel: 'Ctrl+B',
    ),
    AppCommandDefinition(
      id: AppCommandId.zoomIn,
      label: 'Aumentar zoom',
      shortcutLabel: 'Ctrl++',
    ),
    AppCommandDefinition(
      id: AppCommandId.zoomOut,
      label: 'Reducir zoom',
      shortcutLabel: 'Ctrl+-',
    ),
    AppCommandDefinition(
      id: AppCommandId.resetZoom,
      label: 'Restablecer zoom',
      shortcutLabel: 'Ctrl+0',
    ),
    AppCommandDefinition(
      id: AppCommandId.actualSize,
      label: 'Tamaño real (100 %)',
      shortcutLabel: 'Ctrl+1',
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
    AppCommandDefinition(
      id: AppCommandId.showAnnotations,
      label: 'Mostrar anotaciones',
    ),
    AppCommandDefinition(
      id: AppCommandId.navigationBack,
      label: 'Volver a la ubicación anterior',
      shortcutLabel: 'Alt+←',
    ),
    AppCommandDefinition(
      id: AppCommandId.navigationForward,
      label: 'Avanzar en el historial',
      shortcutLabel: 'Alt+→',
    ),
    AppCommandDefinition(
      id: AppCommandId.fitWidth,
      label: 'Ajustar al ancho',
      shortcutLabel: 'Ctrl+2',
    ),
    AppCommandDefinition(
      id: AppCommandId.fitPage,
      label: 'Ajustar a la página',
    ),
    AppCommandDefinition(
      id: AppCommandId.changeReaderTheme,
      label: 'Cambiar tema de lectura',
    ),
    AppCommandDefinition(
      id: AppCommandId.documentInfo,
      label: 'Información del documento',
      keywords: ['propiedades', 'archivo'],
    ),
    AppCommandDefinition(
      id: AppCommandId.fullScreen,
      label: 'Pantalla completa',
      shortcutLabel: 'F11',
    ),
    AppCommandDefinition(
      id: AppCommandId.toggleChrome,
      label: 'Mostrar u ocultar interfaz',
      shortcutLabel: 'Ctrl+H',
    ),
    AppCommandDefinition(
      id: AppCommandId.toggleImmersive,
      label: 'Alternar lectura inmersiva',
      shortcutLabel: 'Ctrl+Mayús+I',
    ),
    AppCommandDefinition(
      id: AppCommandId.settings,
      label: 'Abrir configuración',
    ),
  ];

  static Map<AppCommandId, String> decodeOverrides(String encoded) {
    try {
      final decoded = jsonDecode(encoded);
      if (decoded is! Map<String, dynamic>) return const {};
      final result = <AppCommandId, String>{};
      for (final entry in decoded.entries) {
        final id = AppCommandId.values
            .where((value) => value.name == entry.key)
            .firstOrNull;
        final token = entry.value;
        if (id != null &&
            configurableIds.contains(id) &&
            !reservedShortcutTokens.contains(token) &&
            token is String &&
            shortcutChoices.any((choice) => choice.token == token)) {
          result[id] = token;
        }
      }
      return result;
    } on FormatException {
      return const {};
    }
  }

  static String encodeOverrides(Map<AppCommandId, String> overrides) {
    final normalized = <String, String>{};
    for (final id in configurableIds) {
      final token = overrides[id];
      if (token != null &&
          token != defaultTokens[id] &&
          !reservedShortcutTokens.contains(token) &&
          shortcutChoices.any((choice) => choice.token == token)) {
        normalized[id.name] = token;
      }
    }
    return jsonEncode(normalized);
  }

  static Map<AppCommandId, String> resolvedTokens(String encoded) {
    final overrides = decodeOverrides(encoded);
    final result = <AppCommandId, String>{};
    final used = <String>{};
    for (final id in configurableIds) {
      final requested = overrides[id];
      final fallback = defaultTokens[id]!;
      final token = requested != null && !used.contains(requested)
          ? requested
          : fallback;
      if (used.add(token)) result[id] = token;
    }
    return result;
  }

  static String? assignmentError(
    AppCommandId id,
    String token,
    Map<AppCommandId, String> current,
  ) {
    if (!configurableIds.contains(id)) return 'Esta acción está reservada.';
    if (reservedShortcutTokens.contains(token)) {
      return 'Esta combinación está reservada para la navegación.';
    }
    if (!shortcutChoices.any((choice) => choice.token == token)) {
      return 'La combinación no está permitida.';
    }
    for (final entry in current.entries) {
      if (entry.key != id && entry.value == token) {
        return 'La combinación ya está asignada a otra acción.';
      }
    }
    return null;
  }

  static String shortcutLabel(AppCommandId id, String encoded) {
    final token = resolvedTokens(encoded)[id] ?? defaultTokens[id];
    return shortcutChoices
            .where((choice) => choice.token == token)
            .firstOrNull
            ?.label ??
        '';
  }

  static List<AppCommandDefinition> definitionsFor(String encoded) {
    return definitions
        .map(
          (definition) => configurableIds.contains(definition.id)
              ? definition.withShortcut(shortcutLabel(definition.id, encoded))
              : definition,
        )
        .toList(growable: false);
  }

  static Map<ShortcutActivator, VoidCallback> shortcuts(
    Map<AppCommandId, VoidCallback> actions, {
    String overrides = '{}',
  }) {
    VoidCallback action(AppCommandId id) => actions[id] ?? () {};
    final bindings = <ShortcutActivator, VoidCallback>{
      const SingleActivator(LogicalKeyboardKey.digit0, control: true): action(
        AppCommandId.resetZoom,
      ),
      const SingleActivator(LogicalKeyboardKey.digit1, control: true): action(
        AppCommandId.actualSize,
      ),
      const SingleActivator(LogicalKeyboardKey.digit2, control: true): action(
        AppCommandId.fitWidth,
      ),
      const SingleActivator(
        LogicalKeyboardKey.keyI,
        control: true,
        shift: true,
      ): action(AppCommandId.toggleImmersive),
      const SingleActivator(LogicalKeyboardKey.escape): action(
        AppCommandId.escape,
      ),
      const SingleActivator(LogicalKeyboardKey.pageDown): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.space): action(
        AppCommandId.nextPage,
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
      const SingleActivator(LogicalKeyboardKey.arrowLeft, alt: true): action(
        AppCommandId.navigationBack,
      ),
      const SingleActivator(LogicalKeyboardKey.arrowRight, alt: true): action(
        AppCommandId.navigationForward,
      ),
    };
    final tokens = resolvedTokens(overrides);
    for (final entry in tokens.entries) {
      final choice = shortcutChoices
          .where((item) => item.token == entry.value)
          .firstOrNull;
      if (choice != null) bindings[choice.activator] = action(entry.key);
    }
    // Keep the old go-to-page chord as a compatibility alias while Ctrl+G is
    // the documented binding.  Do not steal it when the user assigned it to
    // another configurable command.
    if (!tokens.values.contains('ctrl-l')) {
      bindings[const SingleActivator(LogicalKeyboardKey.keyL, control: true)] =
          action(AppCommandId.goToPage);
    }
    return bindings;
  }
}
