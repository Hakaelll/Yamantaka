import 'dart:async';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import '../../../app/app_services.dart';
import '../../../app/shortcuts/app_commands.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_density.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../storage/repositories/app_settings_repository.dart';
import '../../../storage/backup/local_backup_service.dart';
import '../../reader/domain/reader_models.dart';
import '../../diagnostics/presentation/diagnostics_dialog.dart';

final class SettingsScreen extends StatefulWidget {
  const SettingsScreen({
    super.key,
    required this.services,
    required this.onBack,
  });

  final AppServices services;
  final VoidCallback onBack;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

final class _SettingsScreenState extends State<SettingsScreen> {
  _SettingsSectionId _selectedSection = _SettingsSectionId.appearance;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.services.settings,
      builder: (context, _) {
        final settings = widget.services.settings.settings;
        return Scaffold(
          appBar: AppBar(
            leading: IconButton(
              tooltip: 'Volver',
              onPressed: widget.onBack,
              icon: const Icon(Icons.arrow_back_rounded),
            ),
            title: const Text('Ajustes'),
          ),
          body: LayoutBuilder(
            builder: (context, constraints) {
              final windowClass = AppBreakpoints.classify(constraints.maxWidth);
              final sections = _buildSections(settings);
              if (windowClass == AppWindowClass.compact) {
                return _CompactSettings(sections: sections);
              }
              return _DesktopSettings(
                sections: sections,
                selected: _selectedSection,
                onSelected: (section) {
                  setState(() => _selectedSection = section);
                },
              );
            },
          ),
        );
      },
    );
  }

  List<_SettingsSectionData> _buildSections(AppSettings settings) {
    return [
      _SettingsSectionData(
        id: _SettingsSectionId.appearance,
        description: 'Tema, controles y perfil de pantalla.',
        children: [
          _AppearancePreview(settings: settings),
          _SwitchSetting(
            label: 'Tema oscuro de la aplicación',
            description: 'Afecta a la biblioteca, el lector y los ajustes.',
            value: settings.darkMode,
            onChanged: (value) => _update(settings.copyWith(darkMode: value)),
          ),
          _DropdownSetting(
            label: 'Densidad de la interfaz',
            description:
                'Compacta muestra más contenido sin cambiar el tamaño del PDF.',
            value: settings.interfaceDensity.name,
            values: const {'comfortable': 'Cómoda', 'compact': 'Compacta'},
            onChanged: (value) => _update(
              settings.copyWith(
                interfaceDensity: AppInterfaceDensity.values.byName(value),
              ),
            ),
          ),
          _DropdownSetting(
            label: 'Tema de lectura predeterminado',
            description: 'Color del lienzo al abrir un documento nuevo.',
            value: settings.defaultReaderTheme,
            values: const {
              'light': 'Claro',
              'ivory': 'Marfil',
              'sepia': 'Sepia',
              'gray': 'Gris',
              'dark': 'Oscuro',
              'black': 'Negro',
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultReaderTheme: value)),
          ),
          _SwitchSetting(
            label: 'Ocultar controles automáticamente',
            description: 'Mantiene el documento despejado durante la lectura.',
            value: settings.autoHideControls,
            onChanged: (value) =>
                _update(settings.copyWith(autoHideControls: value)),
          ),
          _SwitchSetting(
            label: 'Mostrar barra de progreso',
            value: settings.showProgress,
            onChanged: (value) =>
                _update(settings.copyWith(showProgress: value)),
          ),
          _SwitchSetting(
            label: 'Reducir animaciones',
            description: 'Acorta las transiciones sin eliminarlas.',
            value: settings.reduceMotion,
            onChanged: (value) =>
                _update(settings.copyWith(reduceMotion: value)),
          ),
          _SwitchSetting(
            label: 'Desactivar animaciones',
            description: 'Elimina transiciones y movimiento decorativo.',
            value: settings.disableAnimations,
            onChanged: (value) =>
                _update(settings.copyWith(disableAnimations: value)),
          ),
          _DropdownSetting(
            label: 'Perfil visual',
            description:
                'Tinta electrónica fuerza alto contraste, una página y cero '
                'movimiento.',
            value: settings.visualProfile.name,
            values: const {
              'standard': 'Estándar',
              'electronicInk': 'Tinta electrónica',
            },
            onChanged: (value) {
              final profile = ReaderVisualProfile.values.byName(value);
              _update(
                settings.copyWith(
                  visualProfile: profile,
                  reduceMotion: profile == ReaderVisualProfile.electronicInk
                      ? true
                      : settings.reduceMotion,
                  disableAnimations:
                      profile == ReaderVisualProfile.electronicInk
                      ? true
                      : settings.disableAnimations,
                  defaultReaderTheme:
                      profile == ReaderVisualProfile.electronicInk
                      ? 'light'
                      : settings.defaultReaderTheme,
                  defaultPageLayout:
                      profile == ReaderVisualProfile.electronicInk
                      ? 'single'
                      : settings.defaultPageLayout,
                ),
              );
            },
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.reading,
        description: 'Comportamiento inicial y navegación.',
        children: [
          _DropdownSetting(
            label: 'Perfil de lectura',
            description:
                'Aplica un conjunto coherente sin mezclar densidad de interfaz '
                'y zoom del contenido.',
            value: widget.services.readingProfiles.defaultProfileId,
            values: const {
              'builtin-reading': 'Lectura',
              'builtin-study': 'Estudio',
              'builtin-presentation': 'Presentación',
              'builtin-night': 'Noche',
              'builtin-eink': 'Tinta electrónica',
              'builtin-comparison': 'Comparación',
            },
            onChanged: (value) {
              widget.services.readingProfiles
                ..setDefault(value)
                ..apply(value);
              setState(() {});
            },
          ),
          _DropdownSetting(
            label: 'Vista predeterminada',
            value: settings.defaultViewMode,
            values: const {'document': 'Documento', 'book': 'Libro'},
            onChanged: (value) =>
                _update(settings.copyWith(defaultViewMode: value)),
          ),
          _DropdownSetting(
            label: 'Disposición de páginas',
            description:
                'La vista enfrentada solo se activa si ambas páginas son '
                'legibles.',
            value: settings.defaultPageLayout,
            values: const {
              'single': 'Una página',
              'facing': 'Dos páginas',
              'continuous': 'Continua',
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultPageLayout: value)),
          ),
          _DropdownSetting(
            label: 'Ajuste de zoom',
            value: settings.defaultZoomMode,
            values: const {
              'fitWidth': 'Ajustar al ancho',
              'fitPage': 'Ajustar a la página',
            },
            onChanged: (value) =>
                _update(settings.copyWith(defaultZoomMode: value)),
          ),
          _SliderSetting(
            label: 'Márgenes',
            description: 'Espacio alrededor del documento.',
            value: settings.defaultMargin,
            onChanged: (value) =>
                _update(settings.copyWith(defaultMargin: value)),
          ),
          _SwitchSetting(
            label: 'Restaurar la última posición',
            value: settings.restoreLastPosition,
            onChanged: (value) =>
                _update(settings.copyWith(restoreLastPosition: value)),
          ),
          _DropdownSetting(
            label: 'Dirección de lectura',
            value: settings.readingDirection.name,
            values: const {
              'leftToRight': 'Izquierda a derecha',
              'rightToLeft': 'Derecha a izquierda',
            },
            onChanged: (value) => _update(
              settings.copyWith(
                readingDirection: ReadingDirection.values.byName(value),
              ),
            ),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.readerBar,
        description: 'Jerarquía y espacio de las herramientas del lector.',
        children: [
          _SwitchSetting(
            label: 'Mostrar selector Documento / Libro',
            description:
                'La acción seguirá disponible en Más opciones y en la paleta.',
            value: settings.showReaderModeControls,
            onChanged: (value) =>
                _update(settings.copyWith(showReaderModeControls: value)),
          ),
          _SwitchSetting(
            label: 'Mostrar accesos al panel',
            description:
                'Miniaturas, índice y marcadores seguirán en Más opciones.',
            value: settings.showReaderPanelControls,
            onChanged: (value) =>
                _update(settings.copyWith(showReaderPanelControls: value)),
          ),
          _SliderSetting(
            label: 'Ancho del panel lateral',
            description:
                'Se conserva entre sesiones y también puede ajustarse '
                'arrastrando su borde.',
            value: settings.readerPanelWidth,
            min: AppSizes.sidePanelMinWidth,
            max: AppSizes.sidePanelMaxWidth,
            divisions: 10,
            onChanged: (value) =>
                _update(settings.copyWith(readerPanelWidth: value)),
          ),
          _SwitchSetting(
            label: 'Recuperar la sesión al iniciar',
            description:
                'Reabre de forma segura el último documento y su contexto.',
            value: settings.restoreLastSession,
            onChanged: (value) =>
                _update(settings.copyWith(restoreLastSession: value)),
          ),
          _SwitchSetting(
            label: 'Iniciar siempre en la biblioteca',
            description:
                'Tiene prioridad sobre la recuperación automática de sesión.',
            value: settings.startInLibrary,
            onChanged: (value) =>
                _update(settings.copyWith(startInLibrary: value)),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.shortcuts,
        description: 'Un conjunto limitado, seguro y restaurable.',
        children: [
          for (final id in AppCommandRegistry.configurableIds)
            _ShortcutSetting(
              commandId: id,
              value: AppCommandRegistry.resolvedTokens(
                settings.shortcutOverrides,
              )[id]!,
              onChanged: (token) =>
                  _assignShortcut(context, settings, id, token),
            ),
          _ActionSetting(
            label: 'Restaurar atajos',
            description:
                'Recupera las combinaciones predeterminadas. Escape, Inicio, '
                'Fin y las flechas básicas permanecen reservadas.',
            actionLabel: 'Restaurar',
            onPressed: () =>
                _update(settings.copyWith(shortcutOverrides: '{}')),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.library,
        description: 'Recientes, portadas y almacenamiento temporal.',
        children: [
          _SwitchSetting(
            label: 'Mostrar documentos recientes',
            value: settings.showRecents,
            onChanged: (value) =>
                _update(settings.copyWith(showRecents: value)),
          ),
          _SwitchSetting(
            label: 'Generar portadas',
            description:
                'Elige localmente la mejor de las seis primeras páginas y '
                'guarda miniaturas nítidas.',
            value: settings.generateCovers,
            onChanged: (value) =>
                unawaited(_setCoverGeneration(settings, value)),
          ),
          _DropdownSetting(
            label: 'Tamaño máximo de caché',
            value: '${settings.maxCacheMegabytes}',
            values: const {
              '64': '64 MB',
              '128': '128 MB',
              '256': '256 MB',
              '512': '512 MB',
            },
            onChanged: (value) =>
                _update(settings.copyWith(maxCacheMegabytes: int.parse(value))),
          ),
          _ActionSetting(
            label: 'Limpiar caché',
            description: 'Elimina portadas y renderizados temporales.',
            actionLabel: 'Limpiar',
            onPressed: () => _runWithNotice(
              context,
              widget.services.clearCache,
              'Caché eliminada.',
            ),
          ),
          _ActionSetting(
            label: 'Eliminar historial',
            description:
                'Reinicia progreso, preferencias por documento, marcadores y '
                'sesiones. Conserva biblioteca, anotaciones y colecciones.',
            actionLabel: 'Eliminar',
            destructive: true,
            onPressed: () => _confirmAndRun(
              context,
              title: '¿Eliminar historial?',
              body:
                  'Se reiniciarán progreso, preferencias por documento, '
                  'marcadores y sesiones. Se conservarán documentos, '
                  'anotaciones, etiquetas y colecciones.',
              action: widget.services.deleteHistory,
            ),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.history,
        description: 'Actividad local agrupada por día y bajo tu control.',
        children: [
          _SwitchSetting(
            label: 'Registrar sesiones de lectura',
            description: 'Solo tiempo activo aproximado, guardado localmente.',
            value: settings.recordReadingSessions,
            onChanged: (value) =>
                _update(settings.copyWith(recordReadingSessions: value)),
          ),
          _DropdownSetting(
            label: 'Límite del historial',
            description:
                'Las sesiones más antiguas se descartan automáticamente.',
            value: '${settings.readingHistoryLimit}',
            values: const {
              '50': '50 sesiones',
              '100': '100 sesiones',
              '250': '250 sesiones',
              '500': '500 sesiones',
              '1000': '1000 sesiones',
            },
            onChanged: (value) {
              final limit = int.parse(value);
              widget.services.readingSessions.trimToLimit(limit);
              _update(settings.copyWith(readingHistoryLimit: limit));
            },
          ),
          _ActionSetting(
            label: 'Ver historial de lectura',
            description:
                'Consulta páginas, duración y fecha; elimina entradas '
                'individuales o todas.',
            actionLabel: 'Ver historial',
            onPressed: () => _showReadingHistory(context, settings),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.backup,
        description: 'Copia local versionada, validada y sin archivos PDF.',
        children: [
          _ActionSetting(
            label: 'Exportar copia local',
            description:
                'Incluye biblioteca, progreso, marcadores, historial y '
                'ajustes; nunca incluye el contenido de los PDF.',
            actionLabel: 'Exportar',
            onPressed: () => _exportBackup(context),
          ),
          _ActionSetting(
            label: 'Importar copia local',
            description:
                'Muestra un resumen y pide confirmación. Antes crea una copia '
                'de seguridad automática del estado actual.',
            actionLabel: 'Importar',
            onPressed: () => _importBackup(context),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.advanced,
        description: 'Diagnóstico, integridad y recuperación local.',
        children: [
          _ActionSetting(
            label: 'Centro de diagnóstico',
            description:
                'Versiones, esquema, tamaños, recursos e integridad sin rutas, '
                'títulos, consultas ni contenido.',
            actionLabel: 'Abrir',
            onPressed: () => showDialog<void>(
              context: context,
              builder: (context) =>
                  DiagnosticsDialog(services: widget.services),
            ),
          ),
          _ActionSetting(
            label: 'Ayuda contextual',
            description:
                'Resumen local de búsqueda, anotaciones, espacios y atajos.',
            actionLabel: 'Ver ayuda',
            onPressed: () => _showContextualHelp(context),
          ),
        ],
      ),
      _SettingsSectionData(
        id: _SettingsSectionId.privacy,
        description: 'Datos locales y sesiones de lectura.',
        children: [
          _ActionSetting(
            label: 'Abrir ubicación de datos',
            description: widget.services.paths.dataDirectory.path,
            actionLabel: 'Abrir',
            onPressed: widget.services.openDataLocation,
          ),
          _ActionSetting(
            label: 'Eliminar todos los datos locales',
            description: 'No elimina los archivos PDF originales.',
            actionLabel: 'Eliminar',
            destructive: true,
            onPressed: () => _confirmAndRun(
              context,
              title: '¿Eliminar todos los datos?',
              body:
                  'Se borrarán biblioteca, progreso, marcadores, sesiones, '
                  'ajustes y caché. Los PDF originales no se modificarán.',
              action: widget.services.deleteAllLocalData,
            ),
          ),
        ],
      ),
    ];
  }

  Future<void> _showContextualHelp(BuildContext context) => showDialog<void>(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('Ayuda de Yamantaka'),
      content: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 520),
        child: const SingleChildScrollView(
          child: Text(
            'Anotaciones: abre el panel de notas en el lector; se guardan '
            'fuera del PDF y pueden exportarse en Markdown o JSON.\n\n'
            'Búsqueda global: Ctrl+Mayús+F desde la biblioteca. La indexación '
            'es opcional, cancelable y permanece en este dispositivo.\n\n'
            'Espacios de trabajo: guarda documentos, posiciones y el panel para '
            'restaurar una sesión. Los PDF nunca se copian.\n\n'
            'Teclado: Ctrl+K abre comandos, Ctrl+F busca en la superficie '
            'actual, F11 entra o sale de pantalla completa y Escape cierra la '
            'superficie temporal activa.',
          ),
        ),
      ),
      actions: [
        FilledButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Entendido'),
        ),
      ],
    ),
  );

  Future<void> _assignShortcut(
    BuildContext context,
    AppSettings settings,
    AppCommandId id,
    String token,
  ) async {
    final current = AppCommandRegistry.resolvedTokens(
      settings.shortcutOverrides,
    );
    final error = AppCommandRegistry.assignmentError(id, token, current);
    if (error != null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(error)));
      return;
    }
    current[id] = token;
    await _update(
      settings.copyWith(
        shortcutOverrides: AppCommandRegistry.encodeOverrides(current),
      ),
    );
  }

  Future<void> _showReadingHistory(
    BuildContext context,
    AppSettings settings,
  ) async {
    var entries = widget.services.readingSessions.history(
      limit: settings.readingHistoryLimit,
    );
    await showDialog<void>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) {
          final children = <Widget>[];
          String? previousDay;
          for (final entry in entries) {
            final local = entry.session.startedAt.toLocal();
            final day = '${local.year}-${local.month}-${local.day}';
            if (day != previousDay) {
              previousDay = day;
              children.add(
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    AppSpacing.space16,
                    AppSpacing.space16,
                    AppSpacing.space8,
                  ),
                  child: Text(
                    MaterialLocalizations.of(context).formatFullDate(local),
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                ),
              );
            }
            children.add(
              ListTile(
                leading: const Icon(Icons.menu_book_outlined),
                title: Text(
                  entry.documentTitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                subtitle: Text(
                  'Páginas ${entry.session.startPage + 1}–'
                  '${entry.session.endPage + 1} · '
                  '${_formatDuration(entry.session.activeSeconds)}',
                ),
                trailing: IconButton(
                  tooltip: 'Eliminar esta sesión',
                  onPressed: () {
                    widget.services.readingSessions.delete(entry.session.id);
                    setDialogState(() {
                      entries = entries
                          .where((item) => item.session.id != entry.session.id)
                          .toList(growable: false);
                    });
                  },
                  icon: const Icon(Icons.delete_outline_rounded),
                ),
              ),
            );
          }
          return AlertDialog(
            icon: const Icon(Icons.history_rounded),
            title: const Text('Historial de lectura'),
            content: SizedBox(
              width: 620,
              height: 460,
              child: entries.isEmpty
                  ? const AppStatusView(
                      icon: Icons.history_toggle_off_rounded,
                      title: 'Sin sesiones guardadas',
                      message:
                          'El historial aparecerá aquí cuando cierres una '
                          'sesión de lectura.',
                    )
                  : ListView(children: children),
            ),
            actions: [
              if (entries.isNotEmpty)
                TextButton(
                  onPressed: () {
                    widget.services.readingSessions.clear();
                    setDialogState(() => entries = const []);
                  },
                  child: const Text('Eliminar todo'),
                ),
              FilledButton(
                onPressed: () => Navigator.pop(dialogContext),
                child: const Text('Cerrar'),
              ),
            ],
          );
        },
      ),
    );
  }

  Future<void> _exportBackup(BuildContext context) async {
    const type = XTypeGroup(
      label: 'Copia local de Yamantaka',
      extensions: ['json'],
      mimeTypes: ['application/json'],
    );
    final stamp = DateTime.now().toIso8601String().split('T').first;
    final location = await getSaveLocation(
      suggestedName: 'yamantaka-backup-$stamp.json',
      acceptedTypeGroups: const [type],
    );
    if (location == null) return;
    try {
      await widget.services.backups.exportTo(location.path);
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('Copia local exportada.')));
      }
    } on Object catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('No se pudo exportar: $error')));
      }
    }
  }

  Future<void> _importBackup(BuildContext context) async {
    const type = XTypeGroup(
      label: 'Copia local de Yamantaka',
      extensions: ['json'],
      mimeTypes: ['application/json'],
    );
    final selected = await openFile(acceptedTypeGroups: const [type]);
    if (selected == null || !context.mounted) return;
    try {
      final preview = await widget.services.backups.preview(selected.path);
      if (!context.mounted) return;
      final confirmed = await _confirmBackupImport(context, preview);
      if (!confirmed) return;
      await widget.services.reader.closeDocument();
      final safetyPath = await widget.services.backups.importFrom(
        selected.path,
      );
      await widget.services.clearCache();
      await widget.services.library.load();
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Copia importada. Respaldo previo guardado en $safetyPath',
            ),
          ),
        );
      }
    } on FormatException catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(error.message)));
      }
    } on Object {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text(
              'No se pudo importar. Los datos actuales se conservaron.',
            ),
          ),
        );
      }
    }
  }

  Future<bool> _confirmBackupImport(
    BuildContext context,
    BackupPreview preview,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.inventory_2_outlined),
        title: const Text('Importar copia local'),
        content: Text(
          'Versión ${preview.version} · ${preview.documentCount} documentos · '
          '${preview.bookmarkCount} marcadores · '
          '${preview.annotationCount} anotaciones · '
          '${preview.sessionCount} sesiones.\n\n'
          'Se sustituirán los datos locales actuales. Antes se guardará una '
          'copia de seguridad automática. Los PDF no se modifican.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Importar'),
          ),
        ],
      ),
    );
    return confirmed ?? false;
  }

  Future<void> _update(AppSettings settings) =>
      widget.services.settings.update(settings);

  Future<void> _setCoverGeneration(AppSettings settings, bool enabled) async {
    await _update(settings.copyWith(generateCovers: enabled));
    if (!enabled) return;
    for (final document in widget.services.library.documents) {
      if (!document.isMissing) {
        widget.services.thumbnails.enqueueCover(document);
      }
    }
  }

  Future<void> _runWithNotice(
    BuildContext context,
    Future<void> Function() action,
    String message,
  ) async {
    await action();
    if (context.mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(message)));
    }
  }

  Future<void> _confirmAndRun(
    BuildContext context, {
    required String title,
    required String body,
    required Future<void> Function() action,
  }) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        icon: Icon(
          Icons.delete_outline_rounded,
          color: dialogContext.appColors.danger,
        ),
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: dialogContext.appColors.danger,
            ),
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('Eliminar'),
          ),
        ],
      ),
    );
    if (confirmed == true) await action();
  }
}

enum _SettingsSectionId {
  appearance('Apariencia', Icons.contrast_rounded),
  reading('Lectura', Icons.menu_book_outlined),
  readerBar('Barra del lector', Icons.space_dashboard_outlined),
  shortcuts('Atajos', Icons.keyboard_outlined),
  library('Biblioteca', Icons.library_books_outlined),
  history('Historial', Icons.history_rounded),
  backup('Copias locales', Icons.inventory_2_outlined),
  advanced('Avanzado', Icons.health_and_safety_outlined),
  privacy('Privacidad', Icons.shield_outlined);

  const _SettingsSectionId(this.label, this.icon);

  final String label;
  final IconData icon;
}

final class _SettingsSectionData {
  const _SettingsSectionData({
    required this.id,
    required this.description,
    required this.children,
  });

  final _SettingsSectionId id;
  final String description;
  final List<Widget> children;
}

final class _DesktopSettings extends StatelessWidget {
  const _DesktopSettings({
    required this.sections,
    required this.selected,
    required this.onSelected,
  });

  final List<_SettingsSectionData> sections;
  final _SettingsSectionId selected;
  final ValueChanged<_SettingsSectionId> onSelected;

  @override
  Widget build(BuildContext context) {
    final selectedSection = sections.firstWhere(
      (section) => section.id == selected,
    );
    return Row(
      children: [
        Container(
          width: 220,
          decoration: BoxDecoration(
            color: context.appColors.surfaceSecondary,
            border: Border(
              right: BorderSide(color: context.appColors.borderSubtle),
            ),
          ),
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppSpacing.space16,
              AppSpacing.space24,
              AppSpacing.space16,
              AppSpacing.space32,
            ),
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppSpacing.space12,
                ),
                child: Text(
                  'Preferencias locales',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: context.appColors.textMuted,
                  ),
                ),
              ),
              const SizedBox(height: AppSpacing.space12),
              for (final section in sections)
                _SettingsNavItem(
                  section: section.id,
                  selected: selected == section.id,
                  onPressed: () => onSelected(section.id),
                ),
            ],
          ),
        ),
        Expanded(
          child: _SettingsSectionView(
            section: selectedSection,
            padding: EdgeInsets.fromLTRB(
              AppSpacing.pagePadding(MediaQuery.sizeOf(context).width),
              AppSpacing.space32,
              AppSpacing.pagePadding(MediaQuery.sizeOf(context).width),
              AppSpacing.space64,
            ),
          ),
        ),
      ],
    );
  }
}

final class _CompactSettings extends StatelessWidget {
  const _CompactSettings({required this.sections});

  final List<_SettingsSectionData> sections;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(
        AppSpacing.space20,
        AppSpacing.space24,
        AppSpacing.space20,
        AppSpacing.space48,
      ),
      children: [
        const AppSectionHeader(
          title: 'Preferencias locales',
          description: 'Los cambios reversibles se guardan al instante.',
        ),
        const SizedBox(height: AppSpacing.space32),
        for (var index = 0; index < sections.length; index++) ...[
          _SettingsSectionContent(section: sections[index]),
          if (index + 1 < sections.length)
            SizedBox(height: context.appDensity.sectionSpacing),
        ],
      ],
    );
  }
}

final class _SettingsSectionView extends StatelessWidget {
  const _SettingsSectionView({required this.section, required this.padding});

  final _SettingsSectionData section;
  final EdgeInsets padding;

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.topCenter,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 840),
        child: ListView(
          padding: padding,
          children: [_SettingsSectionContent(section: section)],
        ),
      ),
    );
  }
}

final class _SettingsSectionContent extends StatelessWidget {
  const _SettingsSectionContent({required this.section});

  final _SettingsSectionData section;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AppSectionHeader(
          title: section.id.label,
          description: section.description,
        ),
        const SizedBox(height: AppSpacing.space20),
        Divider(color: context.appColors.borderStrong),
        for (var index = 0; index < section.children.length; index++) ...[
          section.children[index],
          if (index + 1 < section.children.length)
            Divider(color: context.appColors.borderSubtle),
        ],
      ],
    );
  }
}

final class _SettingsNavItem extends StatelessWidget {
  const _SettingsNavItem({
    required this.section,
    required this.selected,
    required this.onPressed,
  });

  final _SettingsSectionId section;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      selected: selected,
      label: section.label,
      child: Padding(
        padding: const EdgeInsets.only(bottom: AppSpacing.space4),
        child: Material(
          color: selected
              ? context.appColors.selectionBackground
              : Colors.transparent,
          borderRadius: BorderRadius.circular(AppRadii.control),
          child: InkWell(
            borderRadius: BorderRadius.circular(AppRadii.control),
            onTap: onPressed,
            child: Container(
              height: AppSizes.desktopControlExtent,
              decoration: BoxDecoration(
                border: Border(
                  left: BorderSide(
                    color: selected
                        ? context.appColors.accentPrimary
                        : Colors.transparent,
                    width: 2,
                  ),
                ),
              ),
              padding: const EdgeInsets.symmetric(
                horizontal: AppSpacing.space12,
              ),
              child: Row(
                children: [
                  Icon(section.icon, size: 18),
                  const SizedBox(width: AppSpacing.space12),
                  Expanded(child: Text(section.label)),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

final class _AppearancePreview extends StatelessWidget {
  const _AppearancePreview({required this.settings});

  final AppSettings settings;

  @override
  Widget build(BuildContext context) {
    final readerTheme = ReaderTheme.values
        .where((theme) => theme.name == settings.defaultReaderTheme)
        .firstOrNull;
    return Semantics(
      label:
          'Vista previa: interfaz ${settings.darkMode ? 'oscura' : 'clara'}, '
          'densidad ${settings.interfaceDensity.name}, '
          'lienzo ${readerTheme?.label ?? 'Marfil'}',
      child: Container(
        key: const Key('appearance-preview'),
        padding: const EdgeInsets.all(AppSpacing.space16),
        decoration: BoxDecoration(
          color: context.appColors.surfaceSecondary,
          border: Border.all(color: context.appColors.borderSubtle),
          borderRadius: BorderRadius.circular(AppRadii.field),
        ),
        child: Row(
          children: [
            Container(
              width: 52,
              height: 68,
              decoration: BoxDecoration(
                color: Color(
                  readerTheme?.colorValue ?? ReaderTheme.ivory.colorValue,
                ),
                border: Border.all(color: context.appColors.borderStrong),
                boxShadow:
                    settings.visualProfile == ReaderVisualProfile.electronicInk
                    ? null
                    : [
                        BoxShadow(
                          color: context.appColors.readerPageShadow,
                          blurRadius: AppShadow.readerPageBlur,
                          offset: const Offset(0, 1),
                        ),
                      ],
              ),
            ),
            const SizedBox(width: AppSpacing.space16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Vista previa',
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: AppSpacing.space4),
                  Text(
                    '${settings.interfaceDensity == AppInterfaceDensity.compact ? 'Compacta' : 'Cómoda'} · '
                    '${readerTheme?.label ?? 'Marfil'} · '
                    '${settings.visualProfile == ReaderVisualProfile.electronicInk ? 'Tinta electrónica' : 'Estándar'}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _SettingRow extends StatelessWidget {
  const _SettingRow({
    required this.label,
    required this.control,
    this.description,
  });

  final String label;
  final String? description;
  final Widget control;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final stacked = constraints.maxWidth < 560;
        final labelBlock = Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: Theme.of(context).textTheme.bodyLarge),
            if (description case final value?) ...[
              const SizedBox(height: AppSpacing.space4),
              Text(value, style: Theme.of(context).textTheme.bodySmall),
            ],
          ],
        );
        return Padding(
          padding: EdgeInsets.symmetric(
            vertical: context.appDensity.settingsRowVerticalPadding,
          ),
          child: stacked
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    labelBlock,
                    const SizedBox(height: AppSpacing.space12),
                    Align(alignment: Alignment.centerLeft, child: control),
                  ],
                )
              : Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    Expanded(child: labelBlock),
                    const SizedBox(width: AppSpacing.space24),
                    control,
                  ],
                ),
        );
      },
    );
  }
}

final class _SwitchSetting extends StatelessWidget {
  const _SwitchSetting({
    required this.label,
    required this.value,
    required this.onChanged,
    this.description,
  });

  final String label;
  final String? description;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: Semantics(
        label: label,
        child: Switch(value: value, onChanged: onChanged),
      ),
    );
  }
}

final class _DropdownSetting extends StatelessWidget {
  const _DropdownSetting({
    required this.label,
    required this.value,
    required this.values,
    required this.onChanged,
    this.description,
  });

  final String label;
  final String? description;
  final String value;
  final Map<String, String> values;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: SizedBox(
        width: 224,
        child: DropdownButtonFormField<String>(
          initialValue: value,
          isExpanded: true,
          decoration: const InputDecoration(
            contentPadding: EdgeInsets.symmetric(
              horizontal: AppSpacing.space12,
              vertical: AppSpacing.space8,
            ),
          ),
          items: values.entries
              .map(
                (entry) => DropdownMenuItem(
                  value: entry.key,
                  child: Text(entry.value),
                ),
              )
              .toList(growable: false),
          onChanged: (next) {
            if (next != null) onChanged(next);
          },
        ),
      ),
    );
  }
}

final class _SliderSetting extends StatelessWidget {
  const _SliderSetting({
    required this.label,
    required this.description,
    required this.value,
    required this.onChanged,
    this.min = 8,
    this.max = 72,
    this.divisions = 8,
  });

  final String label;
  final String description;
  final double value;
  final ValueChanged<double> onChanged;
  final double min;
  final double max;
  final int divisions;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: SizedBox(
        width: 224,
        child: Row(
          children: [
            Expanded(
              child: Slider(
                value: value,
                min: min,
                max: max,
                divisions: divisions,
                label: '${value.round()} px',
                onChanged: onChanged,
              ),
            ),
            SizedBox(
              width: 48,
              child: Text(
                '${value.round()} px',
                textAlign: TextAlign.end,
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

final class _ShortcutSetting extends StatelessWidget {
  const _ShortcutSetting({
    required this.commandId,
    required this.value,
    required this.onChanged,
  });

  final AppCommandId commandId;
  final String value;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    final definition = AppCommandRegistry.definitions.firstWhere(
      (item) => item.id == commandId,
    );
    return _SettingRow(
      label: definition.label,
      description: 'Solo combinaciones prevalidadas; se rechazan conflictos.',
      control: SizedBox(
        width: 180,
        child: DropdownButtonFormField<String>(
          initialValue: value,
          decoration: const InputDecoration(isDense: true),
          items: [
            for (final choice in AppCommandRegistry.shortcutChoices)
              DropdownMenuItem(value: choice.token, child: Text(choice.label)),
          ],
          onChanged: (next) {
            if (next != null) onChanged(next);
          },
        ),
      ),
    );
  }
}

final class _ActionSetting extends StatelessWidget {
  const _ActionSetting({
    required this.label,
    required this.description,
    required this.actionLabel,
    required this.onPressed,
    this.destructive = false,
  });

  final String label;
  final String description;
  final String actionLabel;
  final VoidCallback onPressed;
  final bool destructive;

  @override
  Widget build(BuildContext context) {
    return _SettingRow(
      label: label,
      description: description,
      control: destructive
          ? OutlinedButton(
              style: OutlinedButton.styleFrom(
                foregroundColor: context.appColors.danger,
                side: BorderSide(color: context.appColors.danger),
              ),
              onPressed: onPressed,
              child: Text(actionLabel),
            )
          : OutlinedButton(onPressed: onPressed, child: Text(actionLabel)),
    );
  }
}

String _formatDuration(int seconds) {
  if (seconds < 60) return '$seconds s';
  final minutes = seconds ~/ 60;
  if (minutes < 60) return '$minutes min';
  final hours = minutes ~/ 60;
  final remainder = minutes % 60;
  return '$hours h ${remainder.toString().padLeft(2, '0')} min';
}
