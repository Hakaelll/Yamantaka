import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../application/duplicate_resolution_service.dart';
import '../domain/duplicate_resolution.dart';

final class DuplicateReviewResult {
  const DuplicateReviewResult({
    required this.candidate,
    required this.action,
    required this.conflictChoices,
  });

  final DuplicateCandidate candidate;
  final DuplicateResolutionAction action;
  final Map<String, MetadataConflictChoice> conflictChoices;
}

final class DuplicateReviewDialog extends StatefulWidget {
  const DuplicateReviewDialog({
    super.key,
    required this.candidates,
    required this.impactFor,
  });

  final List<DuplicateCandidate> candidates;
  final DuplicateMergeImpact Function(DuplicateCandidate candidate) impactFor;

  @override
  State<DuplicateReviewDialog> createState() => _DuplicateReviewDialogState();
}

final class _DuplicateReviewDialogState extends State<DuplicateReviewDialog> {
  var _index = 0;
  final Map<String, MetadataConflictChoice> _choices = {};

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    if (widget.candidates.isEmpty) {
      return AlertDialog(
        title: Text(strings.get('duplicates.title')),
        content: Text(strings.get('duplicates.empty')),
        actions: [
          FilledButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.close')),
          ),
        ],
      );
    }
    final candidate = widget.candidates[_index];
    final impact = widget.impactFor(candidate);
    final evidence = candidate.evidence;
    return AlertDialog(
      icon: const Icon(Icons.content_copy_rounded),
      title: Text(strings.get('duplicates.title')),
      content: SizedBox(
        width: 620,
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                strings.get(
                  'duplicates.position',
                  parameters: {
                    'current': _index + 1,
                    'total': widget.candidates.length,
                  },
                ),
                style: Theme.of(context).textTheme.labelLarge,
              ),
              const SizedBox(height: AppSpacing.space12),
              Semantics(
                label: strings.get('duplicates.evidence'),
                child: Text(
                  strings.get(
                    'duplicates.evidenceSummary',
                    parameters: {
                      'kind': evidence.kind.name,
                      'score': evidence.score,
                      'fingerprint': evidence.sameFingerprint,
                      'path': evidence.sameNormalizedPath,
                      'name': evidence.sameFileName,
                      'size': evidence.sameSize,
                    },
                  ),
                ),
              ),
              const SizedBox(height: AppSpacing.space12),
              _DocumentSide(
                label: strings.get('duplicates.keep'),
                title: candidate.existing.effectiveTitle,
                path: candidate.existing.path,
              ),
              const SizedBox(height: AppSpacing.space8),
              _DocumentSide(
                label: strings.get('duplicates.mergeFrom'),
                title: candidate.incoming.effectiveTitle,
                path: candidate.incoming.path,
              ),
              const SizedBox(height: AppSpacing.space12),
              Text(
                strings.get(
                  'duplicates.impact',
                  parameters: {'count': impact.localRecords},
                ),
              ),
              for (final field in impact.conflictFields)
                Padding(
                  padding: const EdgeInsets.only(top: AppSpacing.space8),
                  child: DropdownButtonFormField<MetadataConflictChoice>(
                    key: ValueKey('duplicate-conflict-$field'),
                    initialValue:
                        _choices[field] ?? MetadataConflictChoice.existing,
                    decoration: InputDecoration(
                      labelText: strings.get(
                        'duplicates.conflict',
                        parameters: {'field': field},
                      ),
                    ),
                    items: [
                      DropdownMenuItem(
                        value: MetadataConflictChoice.existing,
                        child: Text(strings.get('duplicates.useExisting')),
                      ),
                      DropdownMenuItem(
                        value: MetadataConflictChoice.incoming,
                        child: Text(strings.get('duplicates.useIncoming')),
                      ),
                    ],
                    onChanged: (choice) {
                      if (choice != null) _choices[field] = choice;
                    },
                  ),
                ),
            ],
          ),
        ),
      ),
      actions: [
        IconButton(
          tooltip: strings.get('action.previous'),
          onPressed: _index == 0 ? null : () => setState(() => _index--),
          icon: const Icon(Icons.chevron_left_rounded),
        ),
        IconButton(
          tooltip: strings.get('action.next'),
          onPressed: _index + 1 >= widget.candidates.length
              ? null
              : () => setState(() => _index++),
          icon: const Icon(Icons.chevron_right_rounded),
        ),
        TextButton(
          onPressed: () => Navigator.pop(
            context,
            DuplicateReviewResult(
              candidate: candidate,
              action: DuplicateResolutionAction.keepBoth,
              conflictChoices: const {},
            ),
          ),
          child: Text(strings.get('duplicates.keepBoth')),
        ),
        FilledButton.icon(
          onPressed: () => Navigator.pop(
            context,
            DuplicateReviewResult(
              candidate: candidate,
              action: DuplicateResolutionAction.merge,
              conflictChoices: Map.unmodifiable(_choices),
            ),
          ),
          icon: const Icon(Icons.merge_rounded, size: 18),
          label: Text(strings.get('duplicates.merge')),
        ),
      ],
    );
  }
}

final class _DocumentSide extends StatelessWidget {
  const _DocumentSide({
    required this.label,
    required this.title,
    required this.path,
  });

  final String label;
  final String title;
  final String path;

  @override
  Widget build(BuildContext context) => Semantics(
    container: true,
    label: label,
    child: DecoratedBox(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(AppRadii.panel),
      ),
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.space12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: Theme.of(context).textTheme.labelMedium),
            Text(title, maxLines: 2, overflow: TextOverflow.ellipsis),
            Text(path, maxLines: 2, overflow: TextOverflow.ellipsis),
          ],
        ),
      ),
    ),
  );
}
