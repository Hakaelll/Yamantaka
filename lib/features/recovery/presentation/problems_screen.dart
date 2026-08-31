import 'package:flutter/material.dart';

import '../../../app/theme/app_components.dart';
import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../domain/recovery_models.dart';

final class ProblemsScreen extends StatelessWidget {
  const ProblemsScreen({
    super.key,
    required this.problems,
    required this.onBack,
    required this.onCheckIntegrity,
    required this.onDiscard,
    required this.onResume,
  });

  final List<RecoveryProblem> problems;
  final VoidCallback onBack;
  final VoidCallback onCheckIntegrity;
  final ValueChanged<RecoveryProblem> onDiscard;
  final ValueChanged<RecoveryProblem> onResume;

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          tooltip: strings.get('action.back'),
          onPressed: onBack,
          icon: const Icon(Icons.arrow_back_rounded),
        ),
        title: Text(strings.get('problems.title')),
        actions: [
          TextButton.icon(
            onPressed: onCheckIntegrity,
            icon: const Icon(Icons.fact_check_outlined, size: 18),
            label: Text(strings.get('problems.checkIntegrity')),
          ),
          const SizedBox(width: AppSpacing.space8),
        ],
      ),
      body: problems.isEmpty
          ? AppStatusView(
              icon: Icons.health_and_safety_outlined,
              title: strings.get('problems.title'),
              message: strings.get('problems.empty'),
            )
          : ListView.separated(
              padding: const EdgeInsets.all(AppSpacing.space16),
              itemCount: problems.length,
              separatorBuilder: (_, _) =>
                  const SizedBox(height: AppSpacing.space8),
              itemBuilder: (context, index) {
                final problem = problems[index];
                return Card(
                  child: ListTile(
                    minVerticalPadding: AppSpacing.space12,
                    leading: const Icon(Icons.warning_amber_rounded),
                    title: Text(strings.get('problems.${problem.code}')),
                    subtitle: Text(
                      strings.get(
                        problem.canResume
                            ? 'problems.canResume'
                            : 'problems.localOnly',
                      ),
                    ),
                    trailing: Wrap(
                      spacing: AppSpacing.space4,
                      children: [
                        if (problem.canResume)
                          IconButton(
                            tooltip: strings.get('problems.resume'),
                            onPressed: () => onResume(problem),
                            icon: const Icon(Icons.play_arrow_rounded),
                          ),
                        if (problem.canDiscard)
                          IconButton(
                            tooltip: strings.get('problems.discard'),
                            onPressed: () => onDiscard(problem),
                            icon: const Icon(Icons.clear_rounded),
                          ),
                      ],
                    ),
                  ),
                );
              },
            ),
    );
  }
}
