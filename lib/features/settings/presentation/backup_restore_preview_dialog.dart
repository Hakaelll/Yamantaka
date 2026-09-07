import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../storage/backup/local_backup_service.dart';

/// Read-only dry-run plan shown before a backup can replace local data.
final class BackupRestorePreviewDialog extends StatelessWidget {
  const BackupRestorePreviewDialog({super.key, required this.preview});

  final BackupPreview preview;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AppDialogFrame(
      icon: Icons.inventory_2_outlined,
      title: strings.get('settings.importBackup'),
      description: strings.get('backup.previewDescription'),
      maxWidth: AppSizes.dialogMaxWidth,
      content: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          SelectionArea(
            child: Text(
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
          ),
          const SizedBox(height: AppSpacing.space16),
          AppStatusLabel(
            icon: Icons.backup_outlined,
            label: strings.get('backup.preBackupRequired'),
            tone: AppStatusTone.warning,
          ),
          const SizedBox(height: AppSpacing.space16),
          Text(
            strings.get('backup.conflictStrategyTitle'),
            style: Theme.of(context).textTheme.titleSmall,
          ),
          const SizedBox(height: AppSpacing.space6),
          Text(strings.get('backup.conflictStrategy')),
        ],
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
