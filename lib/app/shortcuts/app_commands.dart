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
  fullScreen,
  toggleChrome,
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
    AppCommandId.fullScreen,
    AppCommandId.toggleChrome,
  };

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
    AppCommandId.goToPage: 'ctrl-l',
    AppCommandId.togglePanel: 'ctrl-j',
    AppCommandId.toggleBookmark: 'ctrl-b',
    AppCommandId.nextPage: 'page-down',
    AppCommandId.previousPage: 'page-up',
    AppCommandId.zoomIn: 'ctrl-plus',
    AppCommandId.zoomOut: 'ctrl-minus',
    AppCommandId.fullScreen: 'f11',
    AppCommandId.toggleChrome: 'ctrl-h',
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
      shortcutLabel: 'Ctrl+L',
    ),
    AppCommandDefinition(
      id: AppCommandId.nextPage,
      label: 'Página siguiente',
      shortcutLabel: 'Av Pág',
    ),
    AppCommandDefinition(
      id: AppCommandId.previousPage,
      label: 'Página anterior',
      shortcutLabel: 'Re Pág',
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
      const SingleActivator(LogicalKeyboardKey.escape): action(
        AppCommandId.escape,
      ),
      const SingleActivator(LogicalKeyboardKey.arrowRight): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.space): action(
        AppCommandId.nextPage,
      ),
      const SingleActivator(LogicalKeyboardKey.arrowLeft): action(
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
    final tokens = resolvedTokens(overrides);
    for (final entry in tokens.entries) {
      final choice = shortcutChoices
          .where((item) => item.token == entry.value)
          .firstOrNull;
      if (choice != null) bindings[choice.activator] = action(entry.key);
    }
    return bindings;
  }
}
