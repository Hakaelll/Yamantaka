import 'package:flutter/material.dart';

import '../../../core/i18n/app_localizations.dart';
import '../../../storage/backup/local_backup_service.dart';

/// Read-only dry-run plan shown before a backup can replace local data.
final class BackupRestorePreviewDialog extends StatelessWidget {
  const BackupRestorePreviewDialog({super.key, required this.preview});

  final BackupPreview preview;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      icon: const Icon(Icons.inventory_2_outlined),
      title: Text(strings.get('settings.importBackup')),
      content: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                strings.get(
                  'backup.importPreview',
                  parameters: {
                    'version': preview.version,
                    'documents': preview.documentCount,
                    'bookmarks': preview.bookmarkCount,
                    'annotations': preview.annotationCount,
                    'sessions': preview.sessionCount,
                    'schema': preview.schemaVersion,
                    'entities': preview.entityCount,
                    'bytes': preview.fileBytes,
                    'adds': preview.documentAdds,
                    'replacements': preview.documentReplacements,
                    'removals': preview.documentRemovals,
                    'conflicts': preview.identityConflicts,
                  },
                ),
              ),
              const SizedBox(height: 16),
              Text(
                strings.get('backup.conflictStrategyTitle'),
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: 6),
              Text(strings.get('backup.conflictStrategy')),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: Text(strings.get('action.cancel')),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, true),
          child: Text(strings.get('action.import')),
        ),
      ],
    );
  }
}
