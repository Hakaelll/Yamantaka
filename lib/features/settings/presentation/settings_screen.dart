import 'package:flutter/material.dart';

import '../../../app/app_services.dart';
import '../../../storage/repositories/app_settings_repository.dart';
import '../../reader/domain/reader_models.dart';

final class SettingsScreen extends StatelessWidget {
  const SettingsScreen({
    super.key,
    required this.services,
    required this.onBack,
  });

  final AppServices services;
  final VoidCallback onBack;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: services.settings,
      builder: (context, _) {
        final settings = services.settings.settings;
        return Scaffold(
          appBar: AppBar(
            leading: IconButton(
              tooltip: 'Volver',
              onPressed: onBack,
              icon: const Icon(Icons.arrow_back_rounded),
            ),
            title: const Text('Configuración'),
          ),
          body: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 760),
              child: ListView(
                padding: const EdgeInsets.fromLTRB(24, 28, 24, 48),
                children: [
                  _SettingsSection(
                    title: 'Apariencia',
                    children: [
                      SwitchListTile(
                        title: const Text('Tema oscuro de la aplicación'),
                        value: settings.darkMode,
                        onChanged: (value) =>
                            _update(settings.copyWith(darkMode: value)),
                      ),
                      _DropdownSetting(
                        label: 'Tema de lectura predeterminado',
                        value: settings.defaultReaderTheme,
                        values: const {
                          'light': 'Claro',
                          'ivory': 'Marfil',
                          'sepia': 'Sepia',
                          'gray': 'Gris',
                          'dark': 'Oscuro',
                          'black': 'Negro',
                        },
                        onChanged: (value) => _update(
                          settings.copyWith(defaultReaderTheme: value),
                        ),
                      ),
                      SwitchListTile(
                        title: const Text('Ocultar controles automáticamente'),
                        value: settings.autoHideControls,
                        onChanged: (value) =>
                            _update(settings.copyWith(autoHideControls: value)),
                      ),
                      SwitchListTile(
                        title: const Text('Mostrar barra de progreso'),
                        value: settings.showProgress,
                        onChanged: (value) =>
                            _update(settings.copyWith(showProgress: value)),
                      ),
                      SwitchListTile(
                        title: const Text('Reducir animaciones'),
                        value: settings.reduceMotion,
                        onChanged: (value) =>
                            _update(settings.copyWith(reduceMotion: value)),
                      ),
                      SwitchListTile(
                        title: const Text('Desactivar animaciones'),
                        value: settings.disableAnimations,
                        onChanged: (value) => _update(
                          settings.copyWith(disableAnimations: value),
                        ),
                      ),
                      _DropdownSetting(
                        label: 'Perfil visual',
                        value: settings.visualProfile.name,
                        values: const {
                          'standard': 'Estándar',
                          'electronicInk': 'Tinta electrónica',
                        },
                        onChanged: (value) {
                          final profile = ReaderVisualProfile.values.byName(
                            value,
                          );
                          _update(
                            settings.copyWith(
                              visualProfile: profile,
                              reduceMotion:
                                  profile == ReaderVisualProfile.electronicInk
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
                  _SettingsSection(
                    title: 'Lectura',
                    children: [
                      _DropdownSetting(
                        label: 'Vista predeterminada',
                        value: settings.defaultViewMode,
                        values: const {
                          'document': 'Documento',
                          'book': 'Libro',
                        },
                        onChanged: (value) =>
                            _update(settings.copyWith(defaultViewMode: value)),
                      ),
                      _DropdownSetting(
                        label: 'Disposición de páginas',
                        value: settings.defaultPageLayout,
                        values: const {
                          'single': 'Una página',
                          'facing': 'Dos páginas',
                          'continuous': 'Continua',
                        },
                        onChanged: (value) => _update(
                          settings.copyWith(defaultPageLayout: value),
                        ),
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
                      ListTile(
                        title: const Text('Márgenes'),
                        subtitle: Slider(
                          value: settings.defaultMargin,
                          min: 8,
                          max: 72,
                          divisions: 8,
                          label: '${settings.defaultMargin.round()} px',
                          onChanged: (value) =>
                              _update(settings.copyWith(defaultMargin: value)),
                        ),
                        trailing: Text('${settings.defaultMargin.round()} px'),
                      ),
                      SwitchListTile(
                        title: const Text('Restaurar la última posición'),
                        value: settings.restoreLastPosition,
                        onChanged: (value) => _update(
                          settings.copyWith(restoreLastPosition: value),
                        ),
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
                            readingDirection: ReadingDirection.values.byName(
                              value,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                  _SettingsSection(
                    title: 'Biblioteca',
                    children: [
                      SwitchListTile(
                        title: const Text('Mostrar documentos recientes'),
                        value: settings.showRecents,
                        onChanged: (value) =>
                            _update(settings.copyWith(showRecents: value)),
                      ),
                      SwitchListTile(
                        title: const Text('Generar portadas'),
                        value: settings.generateCovers,
                        onChanged: (value) =>
                            _update(settings.copyWith(generateCovers: value)),
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
                        onChanged: (value) => _update(
                          settings.copyWith(
                            maxCacheMegabytes: int.parse(value),
                          ),
                        ),
                      ),
                      ListTile(
                        title: const Text('Limpiar caché'),
                        subtitle: const Text(
                          'Elimina portadas y renderizados temporales.',
                        ),
                        trailing: OutlinedButton(
                          onPressed: () => _runWithNotice(
                            context,
                            services.clearCache,
                            'Caché eliminada.',
                          ),
                          child: const Text('Limpiar'),
                        ),
                      ),
                      ListTile(
                        title: const Text('Eliminar historial'),
                        subtitle: const Text(
                          'Quita recientes, progreso y marcadores.',
                        ),
                        trailing: OutlinedButton(
                          onPressed: () => _confirmAndRun(
                            context,
                            title: '¿Eliminar historial?',
                            body:
                                'Se eliminarán recientes, progreso, '
                                'preferencias por documento y marcadores.',
                            action: services.deleteHistory,
                          ),
                          child: const Text('Eliminar'),
                        ),
                      ),
                    ],
                  ),
                  _SettingsSection(
                    title: 'Privacidad',
                    children: [
                      SwitchListTile(
                        title: const Text('Registrar sesiones de lectura'),
                        subtitle: const Text(
                          'Solo tiempo activo aproximado, guardado localmente.',
                        ),
                        value: settings.recordReadingSessions,
                        onChanged: (value) => _update(
                          settings.copyWith(recordReadingSessions: value),
                        ),
                      ),
                      ListTile(
                        title: const Text('Abrir ubicación de datos'),
                        subtitle: Text(services.paths.dataDirectory.path),
                        trailing: OutlinedButton(
                          onPressed: services.openDataLocation,
                          child: const Text('Abrir'),
                        ),
                      ),
                      ListTile(
                        title: const Text('Eliminar todos los datos locales'),
                        subtitle: const Text(
                          'No elimina los archivos PDF originales.',
                        ),
                        trailing: FilledButton.tonal(
                          onPressed: () => _confirmAndRun(
                            context,
                            title: '¿Eliminar todos los datos?',
                            body:
                                'Se borrarán biblioteca, progreso, '
                                'marcadores, sesiones, ajustes y caché. '
                                'Los PDF originales no se modificarán.',
                            action: services.deleteAllLocalData,
                          ),
                          child: const Text('Eliminar'),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Future<void> _update(AppSettings settings) =>
      services.settings.update(settings);

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
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Eliminar'),
          ),
        ],
      ),
    );
    if (confirmed == true) await action();
  }
}

final class _SettingsSection extends StatelessWidget {
  const _SettingsSection({required this.title, required this.children});

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 0, 12, 8),
            child: Text(title, style: Theme.of(context).textTheme.titleMedium),
          ),
          DecoratedBox(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              border: Border.all(color: Theme.of(context).dividerColor),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Column(
              children: [
                for (var index = 0; index < children.length; index++) ...[
                  children[index],
                  if (index + 1 < children.length) const Divider(height: 1),
                ],
              ],
            ),
          ),
        ],
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
  });

  final String label;
  final String value;
  final Map<String, String> values;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(label),
      trailing: SizedBox(
        width: 220,
        child: DropdownButtonFormField<String>(
          initialValue: value,
          isExpanded: true,
          decoration: const InputDecoration(border: InputBorder.none),
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
