import '../../library/domain/library_document.dart';

enum AutomationTriggerKind {
  documentImported,
  sourceFolder,
  extension,
  author,
  tag,
  collection,
  exactDuplicate,
  inaccessible,
}

enum AutomationActionKind {
  addTag,
  addCollection,
  addWorkspace,
  setReadingState,
  setPriority,
  addReadingQueue,
}

final class AutomationTrigger {
  const AutomationTrigger(this.kind, {this.value});

  final AutomationTriggerKind kind;
  final String? value;

  Map<String, Object?> toJson() => {'kind': kind.name, 'value': value};

  factory AutomationTrigger.fromJson(Map<String, Object?> json) =>
      AutomationTrigger(
        AutomationTriggerKind.values.firstWhere(
          (item) => item.name == json['kind'],
          orElse: () => AutomationTriggerKind.documentImported,
        ),
        value: json['value'] as String?,
      );
}

final class AutomationAction {
  const AutomationAction(this.kind, {required this.value});

  final AutomationActionKind kind;
  final String value;

  Map<String, Object?> toJson() => {'kind': kind.name, 'value': value};

  factory AutomationAction.fromJson(Map<String, Object?> json) =>
      AutomationAction(
        AutomationActionKind.values.firstWhere(
          (item) => item.name == json['kind'],
          orElse: () => AutomationActionKind.addTag,
        ),
        value: json['value'] as String? ?? '',
      );
}

final class LocalAutomationRule {
  const LocalAutomationRule({
    required this.id,
    required this.name,
    required this.trigger,
    required this.actions,
    required this.createdAt,
    required this.updatedAt,
    this.enabled = true,
  });

  final String id;
  final String name;
  final AutomationTrigger trigger;
  final List<AutomationAction> actions;
  final bool enabled;
  final DateTime createdAt;
  final DateTime updatedAt;

  bool get isValid =>
      name.trim().isNotEmpty &&
      name.length <= 120 &&
      actions.isNotEmpty &&
      actions.length <= 8 &&
      actions.every((action) => action.value.trim().isNotEmpty);

  String summary({bool english = false}) {
    final triggerText = english
        ? 'When ${trigger.kind.name}'
        : 'Cuando ${trigger.kind.name}';
    final actionText = actions
        .map((action) => '${action.kind.name}: ${action.value}')
        .join(', ');
    return '$triggerText, ${english ? 'apply' : 'aplicar'} $actionText.';
  }
}

final class AutomationMatchContext {
  const AutomationMatchContext({
    this.tagIds = const {},
    this.collectionIds = const {},
    this.duplicateFingerprints = const {},
    this.importedDocumentIds = const {},
  });

  final Map<String, Set<String>> tagIds;
  final Map<String, Set<String>> collectionIds;
  final Set<String> duplicateFingerprints;
  final Set<String> importedDocumentIds;
}

final class AutomationSimulation {
  const AutomationSimulation({
    required this.documentIds,
    required this.scanned,
    required this.truncated,
  });

  final List<String> documentIds;
  final int scanned;
  final bool truncated;

  int get affected => documentIds.length;
}

bool automationMatches(
  LocalAutomationRule rule,
  LibraryDocument document,
  AutomationMatchContext context,
) {
  final value = rule.trigger.value?.trim().toLowerCase();
  return switch (rule.trigger.kind) {
    AutomationTriggerKind.documentImported =>
      context.importedDocumentIds.contains(document.id),
    AutomationTriggerKind.sourceFolder =>
      value != null && document.normalizedPath.toLowerCase().startsWith(value),
    AutomationTriggerKind.extension =>
      value != null && document.fileName.toLowerCase().endsWith(value),
    AutomationTriggerKind.author =>
      value != null && document.effectiveAuthor?.toLowerCase() == value,
    AutomationTriggerKind.tag =>
      value != null && (context.tagIds[value]?.contains(document.id) ?? false),
    AutomationTriggerKind.collection =>
      value != null &&
          (context.collectionIds[value]?.contains(document.id) ?? false),
    AutomationTriggerKind.exactDuplicate =>
      context.duplicateFingerprints.contains(document.fingerprint),
    AutomationTriggerKind.inaccessible => document.isMissing,
  };
}
