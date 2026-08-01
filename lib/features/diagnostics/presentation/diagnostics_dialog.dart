import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/app_services.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../application/diagnostics_service.dart';

final class DiagnosticsDialog extends StatefulWidget {
  const DiagnosticsDialog({super.key, required this.services});

  final AppServices services;

  @override
  State<DiagnosticsDialog> createState() => _DiagnosticsDialogState();
}

final class _DiagnosticsDialogState extends State<DiagnosticsDialog> {
  late Future<SafeDiagnosticsReport> _report = widget.services.diagnostics
      .collect();

  void _refresh() {
    setState(() {
      _report = widget.services.diagnostics.collect();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 720, maxHeight: 700),
        child: FutureBuilder<SafeDiagnosticsReport>(
          future: _report,
          builder: (context, snapshot) {
            final report = snapshot.data;
            return Column(
              children: [
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          'Diagnóstico local',
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                      IconButton(
                        tooltip: 'Actualizar',
                        onPressed: _refresh,
                        icon: const Icon(Icons.refresh_rounded),
                      ),
                      IconButton(
                        tooltip: 'Cerrar',
                        onPressed: () => Navigator.pop(context),
                        icon: const Icon(Icons.close_rounded),
                      ),
                    ],
                  ),
                ),
                const Divider(height: 1),
                Expanded(
                  child: snapshot.connectionState != ConnectionState.done
                      ? const Center(child: CircularProgressIndicator())
                      : report == null
                      ? const AppStatusView(
                          icon: Icons.error_outline_rounded,
                          title: 'No se pudo generar el diagnóstico',
                          message: 'Los datos locales no se han modificado.',
                          tone: AppStatusTone.error,
                        )
                      : ListView(
                          padding: const EdgeInsets.all(AppSpacing.space16),
                          children: [
                            if (widget.services.safeMode)
                              const MaterialBanner(
                                content: Text(
                                  'Modo seguro activo: no se restauró la '
                                  'sesión ni se inició el índice.',
                                ),
                                leading: Icon(Icons.health_and_safety_outlined),
                                actions: [SizedBox.shrink()],
                              ),
                            for (final entry in report.values.entries)
                              _DiagnosticLine(
                                label: entry.key.replaceAll('_', ' '),
                                value: entry.value is Map
                                    ? '${(entry.value as Map).length} métricas'
                                    : '${entry.value}',
                              ),
                          ],
                        ),
                ),
                const Divider(height: 1),
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space12),
                  child: Wrap(
                    spacing: AppSpacing.space8,
                    runSpacing: AppSpacing.space8,
                    alignment: WrapAlignment.end,
                    children: [
                      OutlinedButton.icon(
                        onPressed: report == null ? null : () => _copy(report),
                        icon: const Icon(Icons.copy_rounded, size: 18),
                        label: const Text('Copiar informe seguro'),
                      ),
                      OutlinedButton.icon(
                        onPressed: report == null
                            ? null
                            : () => _export(report),
                        icon: const Icon(Icons.save_alt_rounded, size: 18),
                        label: const Text('Exportar'),
                      ),
                      PopupMenuButton<_MaintenanceAction>(
                        tooltip: 'Mantenimiento local',
                        onSelected: _runMaintenance,
                        itemBuilder: (context) => const [
                          PopupMenuItem(
                            value: _MaintenanceAction.verify,
                            child: Text('Verificar base de datos'),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.optimize,
                            child: Text('Optimizar base de datos'),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.clearCache,
                            child: Text('Limpiar cachés'),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.rebuildIndex,
                            child: Text('Reconstruir índice'),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.openData,
                            child: Text('Abrir carpeta de datos'),
                          ),
                        ],
                        child: FilledButton.tonalIcon(
                          onPressed: null,
                          icon: Icon(Icons.build_outlined, size: 18),
                          label: Text('Mantenimiento'),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Future<void> _copy(SafeDiagnosticsReport report) async {
    await Clipboard.setData(ClipboardData(text: report.toJson()));
    if (mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Diagnóstico copiado.')));
    }
  }

  Future<void> _export(SafeDiagnosticsReport report) async {
    final location = await getSaveLocation(
      suggestedName: 'yamantaka-diagnostico.json',
      acceptedTypeGroups: const [
        XTypeGroup(label: 'JSON', extensions: ['json']),
      ],
    );
    if (location == null) return;
    try {
      await widget.services.diagnostics.exportTo(report, location.path);
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('Diagnóstico exportado.')));
      }
    } on FileSystemException {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('No se pudo escribir el informe.')),
        );
      }
    }
  }

  Future<void> _runMaintenance(_MaintenanceAction action) async {
    switch (action) {
      case _MaintenanceAction.verify:
        final result = widget.services.diagnostics.verifyDatabase();
        _notice(result == 'ok' ? 'Integridad correcta.' : 'Resultado: $result');
      case _MaintenanceAction.optimize:
        widget.services.diagnostics.optimizeDatabase();
        _notice('Base de datos optimizada.');
      case _MaintenanceAction.clearCache:
        if (await _confirm('Limpiar cachés', 'Los PDF no se modificarán.')) {
          await widget.services.clearCache();
          _notice('Cachés eliminadas.');
        }
      case _MaintenanceAction.rebuildIndex:
        if (await _confirm(
          'Reconstruir índice',
          'Se eliminará y recreará solo el índice local.',
        )) {
          widget.services.libraryIndex.setEnabled(true);
          await widget.services.libraryIndex.rebuild();
          _notice('Índice reconstruido.');
        }
      case _MaintenanceAction.openData:
        await widget.services.openDataLocation();
    }
    _refresh();
  }

  Future<bool> _confirm(String title, String body) async =>
      await showDialog<bool>(
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
              child: const Text('Continuar'),
            ),
          ],
        ),
      ) ??
      false;

  void _notice(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }
}

final class _DiagnosticLine extends StatelessWidget {
  const _DiagnosticLine({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: AppSpacing.space6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 220,
          child: Text(label, style: Theme.of(context).textTheme.labelMedium),
        ),
        Expanded(child: SelectableText(value)),
      ],
    ),
  );
}

enum _MaintenanceAction { verify, optimize, clearCache, rebuildIndex, openData }
