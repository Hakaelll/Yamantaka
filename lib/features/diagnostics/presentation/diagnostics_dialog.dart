import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/app_services.dart';
import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
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
    final strings = YamantakaLocalizations.of(context);
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
                          strings.get('diagnostics.title'),
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                      IconButton(
                        tooltip: strings.get('action.refresh'),
                        onPressed: _refresh,
                        icon: const Icon(Icons.refresh_rounded),
                      ),
                      IconButton(
                        tooltip: strings.get('action.close'),
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
                      ? AppStatusView(
                          icon: Icons.error_outline_rounded,
                          title: strings.get('diagnostics.failedTitle'),
                          message: strings.get('diagnostics.failedBody'),
                          tone: AppStatusTone.error,
                        )
                      : ListView(
                          padding: const EdgeInsets.all(AppSpacing.space16),
                          children: [
                            if (widget.services.safeMode)
                              MaterialBanner(
                                content: Text(
                                  strings.get('diagnostics.safeMode'),
                                ),
                                leading: const Icon(
                                  Icons.health_and_safety_outlined,
                                ),
                                actions: const [SizedBox.shrink()],
                              ),
                            for (final entry in report.values.entries)
                              _DiagnosticLine(
                                label: entry.key.replaceAll('_', ' '),
                                value: entry.value is Map
                                    ? strings.get(
                                        'diagnostics.metrics',
                                        parameters: {
                                          'count': (entry.value as Map).length,
                                        },
                                      )
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
                        label: Text(strings.get('diagnostics.copySafe')),
                      ),
                      OutlinedButton.icon(
                        onPressed: report == null
                            ? null
                            : () => _export(report),
                        icon: const Icon(Icons.save_alt_rounded, size: 18),
                        label: Text(strings.get('action.export')),
                      ),
                      PopupMenuButton<_MaintenanceAction>(
                        tooltip: strings.get('diagnostics.maintenanceLocal'),
                        onSelected: _runMaintenance,
                        itemBuilder: (context) => [
                          PopupMenuItem(
                            value: _MaintenanceAction.verify,
                            child: Text(
                              strings.get('diagnostics.verifyDatabase'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.optimize,
                            child: Text(
                              strings.get('diagnostics.optimizeDatabase'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.clearCache,
                            child: Text(strings.get('diagnostics.clearCaches')),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.rebuildIndex,
                            child: Text(
                              strings.get('diagnostics.rebuildIndex'),
                            ),
                          ),
                          PopupMenuItem(
                            value: _MaintenanceAction.openData,
                            child: Text(
                              strings.get('diagnostics.openDataFolder'),
                            ),
                          ),
                        ],
                        child: FilledButton.tonalIcon(
                          onPressed: null,
                          icon: Icon(Icons.build_outlined, size: 18),
                          label: Text(strings.get('diagnostics.maintenance')),
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
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            YamantakaLocalizations.of(context).get('diagnostics.copied'),
          ),
        ),
      );
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
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              YamantakaLocalizations.of(context).get('diagnostics.exported'),
            ),
          ),
        );
      }
    } on FileSystemException {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              YamantakaLocalizations.of(context).get('diagnostics.writeFailed'),
            ),
          ),
        );
      }
    }
  }

  Future<void> _runMaintenance(_MaintenanceAction action) async {
    final strings = YamantakaLocalizations.of(context);
    switch (action) {
      case _MaintenanceAction.verify:
        final result = widget.services.diagnostics.verifyDatabase();
        _notice(
          result == 'ok'
              ? strings.get('diagnostics.integrityOk')
              : strings.get(
                  'diagnostics.result',
                  parameters: {'result': result},
                ),
        );
      case _MaintenanceAction.optimize:
        widget.services.diagnostics.optimizeDatabase();
        _notice(strings.get('diagnostics.optimized'));
      case _MaintenanceAction.clearCache:
        if (await _confirm(
          strings.get('diagnostics.clearCaches'),
          strings.get('diagnostics.clearCachesBody'),
        )) {
          await widget.services.clearCache();
          _notice(strings.get('diagnostics.cachesCleared'));
        }
      case _MaintenanceAction.rebuildIndex:
        if (await _confirm(
          strings.get('diagnostics.rebuildIndex'),
          strings.get('diagnostics.rebuildIndexBody'),
        )) {
          widget.services.libraryIndex.setEnabled(true);
          await widget.services.libraryIndex.rebuild();
          _notice(strings.get('diagnostics.indexRebuilt'));
        }
      case _MaintenanceAction.openData:
        await widget.services.openDataLocation();
    }
    _refresh();
  }

  Future<bool> _confirm(String title, String body) async =>
      await showDialog<bool>(
        context: context,
        builder: (context) {
          final strings = YamantakaLocalizations.of(context);
          return AlertDialog(
            title: Text(title),
            content: Text(body),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(strings.get('action.cancel')),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(strings.get('action.continue')),
              ),
            ],
          );
        },
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
