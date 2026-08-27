import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../application/folder_import_controller.dart';
import '../domain/import_queue.dart';

final class FolderImportDialog extends StatefulWidget {
  const FolderImportDialog({
    super.key,
    required this.controller,
    required this.onImport,
    required this.directoryPath,
    this.onOpenDuplicate,
  });

  final FolderImportController controller;
  final Future<void> Function(bool watchRoot) onImport;
  final String directoryPath;
  final Future<void> Function(FolderImportCandidate candidate)? onOpenDuplicate;

  @override
  State<FolderImportDialog> createState() => _FolderImportDialogState();
}

final class _FolderImportDialogState extends State<FolderImportDialog> {
  bool _watchRoot = false;
  final Set<String> _ignoredDuplicates = {};

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680, maxHeight: 700),
        child: ListenableBuilder(
          listenable: widget.controller,
          builder: (context, _) {
            final newCount = widget.controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.newDocument,
                )
                .length;
            final duplicates = widget.controller.candidates
                .where((item) => item.status == FolderCandidateStatus.duplicate)
                .length;
            final inaccessible = widget.controller.candidates
                .where(
                  (item) => item.status == FolderCandidateStatus.inaccessible,
                )
                .length;
            final invalid = widget.controller.candidates
                .where((item) => item.status == FolderCandidateStatus.invalid)
                .length;
            final summary = widget.controller.lastSummary;
            return Column(
              children: [
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          strings.get('folderImport.title'),
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                      IconButton(
                        tooltip: strings.get('action.close'),
                        onPressed: widget.controller.isImporting
                            ? null
                            : () => Navigator.pop(context),
                        icon: const Icon(Icons.close_rounded),
                      ),
                    ],
                  ),
                ),
                if (summary != null)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(
                      AppSpacing.space16,
                      0,
                      AppSpacing.space16,
                      AppSpacing.space8,
                    ),
                    child: AppStatusLabel(
                      icon: summary.cancelled
                          ? Icons.stop_circle_outlined
                          : Icons.task_alt_rounded,
                      label: strings.get(
                        summary.cancelled
                            ? 'folderImport.summaryCancelled'
                            : 'folderImport.summaryComplete',
                        parameters: {
                          'imported': summary.imported,
                          'duplicates': summary.duplicates,
                          'failed': summary.inaccessible + summary.invalid,
                        },
                      ),
                    ),
                  ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    AppSpacing.space16,
                    0,
                    AppSpacing.space16,
                    AppSpacing.space8,
                  ),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      widget.directoryPath,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                ),
                if (widget.controller.isScanning ||
                    widget.controller.isImporting)
                  LinearProgressIndicator(
                    value: widget.controller.total == 0
                        ? null
                        : widget.controller.processed / widget.controller.total,
                  ),
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppSpacing.space16,
                    vertical: AppSpacing.space8,
                  ),
                  child: Wrap(
                    spacing: AppSpacing.space8,
                    runSpacing: AppSpacing.space8,
                    children: [
                      _CountChip(
                        label: strings.get('folderImport.found'),
                        value: widget.controller.candidates.length,
                      ),
                      _CountChip(
                        label: strings.get('folderImport.new'),
                        value: newCount,
                      ),
                      _CountChip(
                        label: strings.get('folderImport.duplicates'),
                        value: duplicates,
                      ),
                      _CountChip(
                        label: strings.get('folderImport.inaccessible'),
                        value: inaccessible,
                      ),
                      _CountChip(
                        label: strings.get('folderImport.invalid'),
                        value: invalid,
                      ),
                    ],
                  ),
                ),
                const Divider(height: 1),
                Expanded(
                  child: widget.controller.candidates.isEmpty
                      ? AppStatusView(
                          icon: Icons.folder_off_outlined,
                          title: widget.controller.isScanning
                              ? strings.get('folderImport.scanning')
                              : strings.get('folderImport.noneFound'),
                          message: strings.get('folderImport.confirmationBody'),
                          loading: widget.controller.isScanning,
                        )
                      : ListView.builder(
                          itemCount: widget.controller.candidates.length,
                          itemBuilder: (context, index) {
                            final candidate =
                                widget.controller.candidates[index];
                            final workState = widget.controller.workStateFor(
                              candidate,
                            );
                            final duplicateIgnored = _ignoredDuplicates
                                .contains(candidate.path);
                            return ListTile(
                              leading: Icon(
                                workState == null
                                    ? _icon(candidate.status)
                                    : _workIcon(workState),
                              ),
                              title: Text(
                                candidate.fileName,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              subtitle: Text(
                                strings.get(
                                  duplicateIgnored
                                      ? 'folderImport.status.ignored'
                                      : workState == null
                                      ? 'folderImport.status.${candidate.status.name}'
                                      : 'folderImport.work.${workState.name}',
                                ),
                              ),
                              trailing:
                                  candidate.status ==
                                      FolderCandidateStatus.duplicate
                                  ? PopupMenuButton<_DuplicateAction>(
                                      tooltip: strings.get(
                                        'folderImport.duplicateActions',
                                      ),
                                      enabled: !widget.controller.isImporting,
                                      onSelected: (action) =>
                                          _runDuplicateAction(
                                            action,
                                            candidate,
                                          ),
                                      itemBuilder: (context) => [
                                        PopupMenuItem(
                                          value: _DuplicateAction.open,
                                          enabled:
                                              widget.onOpenDuplicate != null,
                                          child: AppMenuItemContent(
                                            icon: Icons.open_in_new_rounded,
                                            label: strings.get(
                                              'folderImport.openExisting',
                                            ),
                                          ),
                                        ),
                                        PopupMenuItem(
                                          value: _DuplicateAction.review,
                                          child: AppMenuItemContent(
                                            icon: Icons.difference_outlined,
                                            label: strings.get(
                                              'folderImport.reviewDifferences',
                                            ),
                                          ),
                                        ),
                                        PopupMenuItem(
                                          value: _DuplicateAction.ignore,
                                          child: AppMenuItemContent(
                                            icon: Icons.visibility_off_outlined,
                                            label: strings.get(
                                              'folderImport.ignore',
                                            ),
                                          ),
                                        ),
                                      ],
                                    )
                                  : null,
                            );
                          },
                        ),
                ),
                const Divider(height: 1),
                CheckboxListTile(
                  value: _watchRoot,
                  onChanged: widget.controller.isImporting
                      ? null
                      : (value) => setState(() => _watchRoot = value ?? false),
                  title: Text(strings.get('folderImport.watch')),
                  subtitle: Text(strings.get('folderImport.watchBody')),
                  controlAffinity: ListTileControlAffinity.leading,
                  dense: true,
                ),
                const Divider(height: 1),
                Padding(
                  padding: const EdgeInsets.all(AppSpacing.space16),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      if (widget.controller.isScanning ||
                          widget.controller.isImporting)
                        TextButton(
                          onPressed: widget.controller.cancel,
                          child: Text(
                            strings.get('folderImport.cancelProcess'),
                          ),
                        )
                      else
                        TextButton(
                          onPressed: () => Navigator.pop(context),
                          child: Text(strings.get('action.close')),
                        ),
                      const SizedBox(width: AppSpacing.space8),
                      FilledButton.icon(
                        onPressed:
                            newCount == 0 ||
                                widget.controller.isScanning ||
                                widget.controller.isImporting
                            ? null
                            : () => widget.onImport(_watchRoot),
                        icon: const Icon(Icons.library_add_outlined, size: 18),
                        label: Text(
                          strings.get(
                            'folderImport.add',
                            parameters: {'count': newCount},
                          ),
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

  static IconData _icon(FolderCandidateStatus status) => switch (status) {
    FolderCandidateStatus.newDocument => Icons.add_circle_outline_rounded,
    FolderCandidateStatus.duplicate => Icons.content_copy_rounded,
    FolderCandidateStatus.inaccessible => Icons.lock_outline_rounded,
    FolderCandidateStatus.invalid => Icons.error_outline_rounded,
  };

  static IconData _workIcon(ImportWorkState state) => switch (state) {
    ImportWorkState.pending => Icons.schedule_rounded,
    ImportWorkState.running => Icons.sync_rounded,
    ImportWorkState.completed => Icons.task_alt_rounded,
    ImportWorkState.failed => Icons.error_outline_rounded,
    ImportWorkState.cancelled => Icons.stop_circle_outlined,
  };

  Future<void> _runDuplicateAction(
    _DuplicateAction action,
    FolderImportCandidate candidate,
  ) async {
    switch (action) {
      case _DuplicateAction.open:
        final open = widget.onOpenDuplicate;
        if (open == null || !mounted) return;
        Navigator.pop(context);
        await open(candidate);
      case _DuplicateAction.review:
        await _showDuplicateReview(candidate);
      case _DuplicateAction.ignore:
        setState(() => _ignoredDuplicates.add(candidate.path));
    }
  }

  Future<void> _showDuplicateReview(FolderImportCandidate candidate) async {
    final existing = widget.controller.duplicateDocumentFor(candidate);
    final incoming = candidate.inspected;
    if (existing == null || incoming == null || !mounted) return;
    final strings = YamantakaLocalizations.of(context);
    final exactContent = existing.fingerprint == incoming.fingerprint;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.difference_outlined),
        title: Text(strings.get('folderImport.reviewTitle')),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                strings.get(
                  exactContent
                      ? 'folderImport.sameContent'
                      : 'folderImport.differentContent',
                ),
              ),
              const SizedBox(height: AppSpacing.space16),
              _DuplicateDifferenceRow(
                label: strings.get('folderImport.incomingFile'),
                value:
                    '${incoming.fileName} · ${_formatBytes(incoming.fileSize)}',
              ),
              _DuplicateDifferenceRow(
                label: strings.get('folderImport.existingFile'),
                value:
                    '${existing.fileName} · ${_formatBytes(existing.fileSize)}',
              ),
              _DuplicateDifferenceRow(
                label: strings.get('folderImport.modified'),
                value:
                    '${incoming.modifiedAt.toLocal()} / ${existing.fileModifiedAt.toLocal()}',
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.close')),
          ),
          if (widget.onOpenDuplicate != null)
            FilledButton(
              onPressed: () async {
                Navigator.pop(context);
                Navigator.pop(this.context);
                await widget.onOpenDuplicate!(candidate);
              },
              child: Text(strings.get('folderImport.openExisting')),
            ),
        ],
      ),
    );
  }

  static String _formatBytes(int bytes) {
    if (bytes < 1024) return '$bytes B';
    final kib = bytes / 1024;
    if (kib < 1024) return '${kib.toStringAsFixed(1)} KiB';
    return '${(kib / 1024).toStringAsFixed(1)} MiB';
  }
}

enum _DuplicateAction { open, review, ignore }

final class _DuplicateDifferenceRow extends StatelessWidget {
  const _DuplicateDifferenceRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: AppSpacing.space8),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 132,
          child: Text(label, style: Theme.of(context).textTheme.labelMedium),
        ),
        Expanded(child: Text(value)),
      ],
    ),
  );
}

final class _CountChip extends StatelessWidget {
  const _CountChip({required this.label, required this.value});

  final String label;
  final int value;

  @override
  Widget build(BuildContext context) =>
      Chip(label: Text('$label: $value'), visualDensity: VisualDensity.compact);
}
