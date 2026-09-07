import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../app/ui/yama_components.dart';
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
  final Map<String, Map<String, MetadataConflictChoice>> _candidateChoices = {};

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    if (widget.candidates.isEmpty) {
      return AppDialogFrame(
        icon: Icons.content_copy_rounded,
        title: strings.get('duplicates.title'),
        description: strings.get('duplicates.empty'),
        actions: [
          FilledButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.get('action.close')),
          ),
        ],
      );
    }
    final candidate = widget.candidates[_index];
    final choices = _candidateChoices.putIfAbsent(
      '${candidate.existing.id}:${candidate.incoming.id}',
      () => {},
    );
    final impact = widget.impactFor(candidate);
    final evidence = candidate.evidence;
    return AppDialogFrame(
      icon: Icons.content_copy_rounded,
      title: strings.get('duplicates.title'),
      description: strings.get(
        'duplicates.position',
        parameters: {'current': _index + 1, 'total': widget.candidates.length},
      ),
      maxWidth: AppSizes.dialogWideWidth,
      content: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Semantics(
            container: true,
            label: strings.get('duplicates.evidence'),
            child: AppStatusLabel(
              icon: candidate.canSuggestMerge
                  ? Icons.verified_outlined
                  : Icons.compare_arrows_rounded,
              label: strings.get('duplicates.kind.${evidence.kind.name}'),
              tone: candidate.canSuggestMerge
                  ? AppStatusTone.neutral
                  : AppStatusTone.warning,
            ),
          ),
          const SizedBox(height: AppSpacing.space8),
          Text(
            [
              if (evidence.sameFingerprint)
                strings.get('duplicates.sameContent'),
              if (evidence.sameNormalizedPath)
                strings.get('duplicates.sameLocation'),
              if (evidence.sameFileName) strings.get('duplicates.sameName'),
              if (evidence.sameSize) strings.get('duplicates.sameSize'),
            ].join(' · '),
            style: Theme.of(context).textTheme.bodyMedium,
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
                key: ValueKey(
                  'duplicate-conflict-${candidate.incoming.id}-$field',
                ),
                isExpanded: true,
                style: Theme.of(context).textTheme.bodyMedium,
                initialValue: choices[field] ?? MetadataConflictChoice.existing,
                decoration: InputDecoration(
                  labelText: strings.get(
                    'duplicates.conflict',
                    parameters: {
                      'field': strings.get('duplicates.field.$field'),
                    },
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
                  if (choice != null) choices[field] = choice;
                },
              ),
            ),
        ],
      ),
      actions: [
        YamaIconButton(
          semanticLabel: strings.get('action.previous'),
          onPressed: _index == 0 ? null : () => setState(() => _index--),
          icon: Icons.chevron_left_rounded,
        ),
        YamaIconButton(
          semanticLabel: strings.get('action.next'),
          onPressed: _index + 1 >= widget.candidates.length
              ? null
              : () => setState(() => _index++),
          icon: Icons.chevron_right_rounded,
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
              conflictChoices: Map.unmodifiable(choices),
            ),
          ),
          icon: const Icon(Icons.merge_rounded, size: AppIconSizes.standard),
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
    child: Container(
      width: double.infinity,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(AppRadii.panel),
      ),
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.space12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: Theme.of(context).textTheme.labelMedium),
            Text(title, maxLines: 2, overflow: TextOverflow.ellipsis),
            SelectableText(path, style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    ),
  );
}
