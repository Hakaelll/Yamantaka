import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../storage/repositories/automation_repository.dart';
import '../../library/domain/library_document.dart';
import '../application/local_automation_engine.dart';
import '../domain/local_automation.dart';

final class AutomationManagerDialog extends StatefulWidget {
  const AutomationManagerDialog({
    super.key,
    required this.repository,
    required this.engine,
    required this.documents,
    required this.applyAction,
    required this.applyUndo,
    this.matchContext = const AutomationMatchContext(),
  });

  final AutomationRepository repository;
  final LocalAutomationEngine engine;
  final Iterable<LibraryDocument> documents;
  final AutomationActionApplier applyAction;
  final AutomationUndoApplier applyUndo;
  final AutomationMatchContext matchContext;

  @override
  State<AutomationManagerDialog> createState() =>
      _AutomationManagerDialogState();
}

final class _AutomationManagerDialogState
    extends State<AutomationManagerDialog> {
  List<LocalAutomationRule> _rules = const [];
  AutomationSimulation? _simulation;
  AutomationCancellationToken? _cancellation;
  AutomationApplyResult? _lastResult;
  bool _applying = false;
  bool _simulating = false;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  void _reload() => setState(() => _rules = widget.repository.all());

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 780, maxHeight: 680),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space16),
              child: Row(
                children: [
                  const Icon(Icons.auto_awesome_motion_outlined),
                  const SizedBox(width: AppSpacing.space8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          strings.get('automation.title'),
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                        Text(
                          strings.get('automation.localOnly'),
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                  FilledButton.tonalIcon(
                    onPressed: _applying || _simulating ? null : _createRule,
                    icon: const Icon(Icons.add_rounded, size: 18),
                    label: Text(strings.get('automation.newRule')),
                  ),
                  const SizedBox(width: AppSpacing.space8),
                  IconButton(
                    tooltip: strings.get('action.close'),
                    onPressed: _applying || _simulating
                        ? null
                        : () => Navigator.pop(context),
                    icon: const Icon(Icons.close_rounded),
                  ),
                ],
              ),
            ),
            if (_applying || _simulating) ...[
              const LinearProgressIndicator(),
              ListTile(
                leading: const Icon(Icons.sync_rounded),
                title: Text(
                  strings.get(
                    _simulating
                        ? 'automation.simulating'
                        : 'automation.applying',
                  ),
                ),
                trailing: TextButton.icon(
                  onPressed: _cancellation?.cancel,
                  icon: const Icon(Icons.stop_circle_outlined),
                  label: Text(strings.get('action.cancel')),
                ),
              ),
            ] else if (_simulation != null)
              ListTile(
                leading: const Icon(Icons.science_outlined),
                title: Text(
                  strings.get(
                    'automation.simulationResult',
                    parameters: {
                      'affected': _simulation!.affected,
                      'scanned': _simulation!.scanned,
                    },
                  ),
                ),
                subtitle: _simulation!.truncated
                    ? Text(strings.get('automation.truncated'))
                    : null,
              )
            else if (_lastResult != null)
              ListTile(
                leading: const Icon(Icons.task_alt_rounded),
                title: Text(
                  strings.get(
                    'automation.applyResult',
                    parameters: {
                      'completed': _lastResult!.completed,
                      'total': _lastResult!.total,
                    },
                  ),
                ),
                trailing: TextButton.icon(
                  onPressed: _undoLast,
                  icon: const Icon(Icons.undo_rounded),
                  label: Text(strings.get('action.undo')),
                ),
              ),
            const Divider(height: 1),
            Expanded(
              child: _rules.isEmpty
                  ? Center(child: Text(strings.get('automation.empty')))
                  : ListView.separated(
                      itemCount: _rules.length,
                      separatorBuilder: (_, _) => const Divider(height: 1),
                      itemBuilder: (context, index) {
                        final rule = _rules[index];
                        return ListTile(
                          enabled: !_applying && !_simulating,
                          leading: Switch(
                            value: rule.enabled,
                            onChanged: _applying || _simulating
                                ? null
                                : (enabled) {
                                    widget.repository.save(
                                      LocalAutomationRule(
                                        id: rule.id,
                                        name: rule.name,
                                        trigger: rule.trigger,
                                        actions: rule.actions,
                                        enabled: enabled,
                                        createdAt: rule.createdAt,
                                        updatedAt: rule.updatedAt,
                                      ),
                                    );
                                    _reload();
                                  },
                          ),
                          title: Text(rule.name),
                          subtitle: Text(
                            rule.summary(
                              english:
                                  Localizations.localeOf(
                                    context,
                                  ).languageCode ==
                                  'en',
                            ),
                          ),
                          trailing: Wrap(
                            spacing: AppSpacing.space4,
                            children: [
                              IconButton(
                                tooltip: strings.get('automation.simulate'),
                                onPressed: () => _simulate(rule),
                                icon: const Icon(Icons.science_outlined),
                              ),
                              IconButton(
                                tooltip: strings.get('automation.apply'),
                                onPressed: rule.enabled
                                    ? () => _apply(rule)
                                    : null,
                                icon: const Icon(Icons.play_arrow_rounded),
                              ),
                              IconButton(
                                tooltip: strings.get('action.delete'),
                                onPressed: () {
                                  widget.repository.delete(rule.id);
                                  _reload();
                                },
                                icon: const Icon(Icons.delete_outline_rounded),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _simulate(LocalAutomationRule rule) async {
    final cancellation = AutomationCancellationToken();
    setState(() {
      _simulating = true;
      _cancellation = cancellation;
    });
    try {
      final result = await widget.engine.simulateAsync(
        rule,
        widget.documents,
        context: widget.matchContext,
        cancellationToken: cancellation,
      );
      if (!mounted) return;
      setState(() {
        _simulation = result;
        _lastResult = null;
      });
    } finally {
      if (mounted) {
        setState(() {
          _simulating = false;
          _cancellation = null;
        });
      }
    }
  }

  Future<void> _apply(LocalAutomationRule rule) async {
    final simulationCancellation = AutomationCancellationToken();
    setState(() {
      _simulating = true;
      _cancellation = simulationCancellation;
    });
    late final AutomationSimulation simulation;
    try {
      simulation = await widget.engine.simulateAsync(
        rule,
        widget.documents,
        context: widget.matchContext,
        cancellationToken: simulationCancellation,
      );
    } finally {
      if (mounted) {
        setState(() {
          _simulating = false;
          _cancellation = null;
        });
      }
    }
    if (!mounted) return;
    setState(() => _simulation = simulation);
    if (simulationCancellation.isCancelled) return;
    final strings = YamantakaLocalizations.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('automation.confirmTitle')),
        content: Text(
          strings.get(
            'automation.confirmBody',
            parameters: {'count': simulation.affected},
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: simulation.affected == 0
                ? null
                : () => Navigator.pop(context, true),
            child: Text(strings.get('automation.apply')),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    final cancellation = AutomationCancellationToken();
    setState(() {
      _applying = true;
      _cancellation = cancellation;
      _lastResult = null;
    });
    try {
      final result = await widget.engine.apply(
        rule: rule,
        documents: widget.documents,
        context: widget.matchContext,
        cancellationToken: cancellation,
        applyAction: widget.applyAction,
        rollbackAction: widget.applyUndo,
      );
      if (!mounted) return;
      setState(() {
        _lastResult = result;
        _simulation = null;
      });
    } finally {
      if (mounted) {
        setState(() {
          _applying = false;
          _cancellation = null;
        });
      }
    }
  }

  Future<void> _undoLast() async {
    final result = _lastResult;
    if (result == null) return;
    final undone = await widget.engine.undo(result.journalId, widget.applyUndo);
    if (mounted && undone) setState(() => _lastResult = null);
  }

  Future<void> _createRule() async {
    final rule = await showDialog<LocalAutomationRule>(
      context: context,
      builder: (context) => const _AutomationRuleEditor(),
    );
    if (rule == null) return;
    widget.repository.save(rule);
    _reload();
  }
}

final class _AutomationRuleEditor extends StatefulWidget {
  const _AutomationRuleEditor();

  @override
  State<_AutomationRuleEditor> createState() => _AutomationRuleEditorState();
}

final class _AutomationRuleEditorState extends State<_AutomationRuleEditor> {
  final _name = TextEditingController();
  final _triggerValue = TextEditingController();
  final _actionValue = TextEditingController();
  AutomationTriggerKind _trigger = AutomationTriggerKind.extension;
  AutomationActionKind _action = AutomationActionKind.addTag;

  bool get _triggerNeedsValue => !{
    AutomationTriggerKind.documentImported,
    AutomationTriggerKind.exactDuplicate,
    AutomationTriggerKind.inaccessible,
  }.contains(_trigger);

  @override
  void dispose() {
    _name.dispose();
    _triggerValue.dispose();
    _actionValue.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      title: Text(strings.get('automation.editorTitle')),
      content: SizedBox(
        width: 520,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _name,
              autofocus: true,
              maxLength: 120,
              decoration: InputDecoration(
                labelText: strings.get('automation.name'),
              ),
            ),
            DropdownButtonFormField<AutomationTriggerKind>(
              initialValue: _trigger,
              decoration: InputDecoration(
                labelText: strings.get('automation.trigger'),
              ),
              items: [
                for (final item in AutomationTriggerKind.values)
                  DropdownMenuItem(
                    value: item,
                    child: Text(strings.get('automation.trigger.${item.name}')),
                  ),
              ],
              onChanged: (value) =>
                  setState(() => _trigger = value ?? _trigger),
            ),
            if (_triggerNeedsValue)
              TextField(
                controller: _triggerValue,
                decoration: InputDecoration(
                  labelText: strings.get('automation.matchValue'),
                ),
              ),
            const SizedBox(height: AppSpacing.space12),
            DropdownButtonFormField<AutomationActionKind>(
              initialValue: _action,
              decoration: InputDecoration(
                labelText: strings.get('automation.action'),
              ),
              items: [
                for (final item in AutomationActionKind.values)
                  DropdownMenuItem(
                    value: item,
                    child: Text(strings.get('automation.action.${item.name}')),
                  ),
              ],
              onChanged: (value) => setState(() => _action = value ?? _action),
            ),
            TextField(
              controller: _actionValue,
              decoration: InputDecoration(
                labelText: strings.get('automation.actionValue'),
                helperText: strings.get('automation.actionValueHelp'),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(strings.get('action.cancel')),
        ),
        FilledButton(
          onPressed: () {
            final now = DateTime.now().toUtc();
            final rule = LocalAutomationRule(
              id: '',
              name: _name.text.trim(),
              trigger: AutomationTrigger(
                _trigger,
                value: _triggerNeedsValue ? _triggerValue.text.trim() : null,
              ),
              actions: [
                AutomationAction(_action, value: _actionValue.text.trim()),
              ],
              createdAt: now,
              updatedAt: now,
            );
            if (!rule.isValid ||
                (_triggerNeedsValue && (_triggerValue.text.trim().isEmpty))) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text(strings.get('automation.invalid'))),
              );
              return;
            }
            Navigator.pop(context, rule);
          },
          child: Text(strings.get('action.save')),
        ),
      ],
    );
  }
}
