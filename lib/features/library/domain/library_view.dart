import 'library_document.dart';

enum LibraryViewRuleKind {
  text,
  favorite,
  missing,
  readingState,
  minimumProgress,
  maximumProgress,
}

enum LibraryViewReadingState { all, inProgress, notStarted, completed }

final class LibraryViewRule {
  const LibraryViewRule({
    required this.kind,
    this.text = '',
    this.boolean,
    this.readingState,
    this.progress,
  });

  final LibraryViewRuleKind kind;
  final String text;
  final bool? boolean;
  final LibraryViewReadingState? readingState;
  final double? progress;

  LibraryViewRule normalized() => LibraryViewRule(
    kind: kind,
    text: text.trim(),
    boolean: boolean,
    readingState: readingState,
    progress: progress?.clamp(0, 1).toDouble(),
  );

  Map<String, Object?> toJson() => {
    'kind': kind.name,
    if (text.trim().isNotEmpty) 'text': text.trim(),
    if (boolean != null) 'boolean': boolean,
    if (readingState != null) 'reading_state': readingState!.name,
    if (progress != null) 'progress': progress,
  };

  static LibraryViewRule? fromJson(Map<String, Object?> json) {
    final kind = LibraryViewRuleKind.values
        .where((value) => value.name == json['kind'])
        .firstOrNull;
    if (kind == null) return null;
    final state = LibraryViewReadingState.values
        .where((value) => value.name == json['reading_state'])
        .firstOrNull;
    return LibraryViewRule(
      kind: kind,
      text: json['text'] as String? ?? '',
      boolean: json['boolean'] as bool?,
      readingState: state,
      progress: (json['progress'] as num?)?.toDouble(),
    ).normalized();
  }
}

final class SavedLibraryView {
  const SavedLibraryView({
    required this.id,
    required this.name,
    required this.rules,
    this.sort = 'lastOpened',
    this.persistedTextQuery = false,
  });

  final String id;
  final String name;
  final List<LibraryViewRule> rules;
  final String sort;

  /// Text rules are opt-in because a saved query may contain personal data.
  final bool persistedTextQuery;

  SavedLibraryView normalized() => SavedLibraryView(
    id: id.trim(),
    name: name.trim(),
    rules: [
      for (final rule in rules)
        if (persistedTextQuery || rule.kind != LibraryViewRuleKind.text)
          rule.normalized(),
    ],
    sort: sort,
    persistedTextQuery: persistedTextQuery,
  );

  Map<String, Object?> toJson() {
    final value = normalized();
    return {
      'id': value.id,
      'name': value.name,
      'sort': value.sort,
      'persisted_text_query': value.persistedTextQuery,
      'rules': [for (final rule in value.rules) rule.toJson()],
    };
  }
}

abstract final class LibraryViewEvaluator {
  static bool matches(
    LibraryDocument document,
    Iterable<LibraryViewRule> rules,
  ) {
    for (final raw in rules) {
      final rule = raw.normalized();
      final matches = switch (rule.kind) {
        LibraryViewRuleKind.text => _text(document, rule.text),
        LibraryViewRuleKind.favorite =>
          rule.boolean == null || document.isFavorite == rule.boolean,
        LibraryViewRuleKind.missing =>
          rule.boolean == null || document.isMissing == rule.boolean,
        LibraryViewRuleKind.readingState =>
          rule.readingState == null || _state(document) == rule.readingState,
        LibraryViewRuleKind.minimumProgress =>
          rule.progress == null || document.readingProgress >= rule.progress!,
        LibraryViewRuleKind.maximumProgress =>
          rule.progress == null || document.readingProgress <= rule.progress!,
      };
      if (!matches) return false;
    }
    return true;
  }

  static bool _text(LibraryDocument document, String query) {
    final value = query.trim().toLowerCase();
    if (value.isEmpty) return true;
    return [
      document.effectiveTitle,
      document.effectiveAuthor ?? '',
      document.fileName,
      document.libraryNote ?? '',
    ].any((candidate) => candidate.toLowerCase().contains(value));
  }

  static LibraryViewReadingState _state(LibraryDocument document) {
    if (document.readingProgress >= 1) {
      return LibraryViewReadingState.completed;
    }
    if (document.readingProgress > 0) {
      return LibraryViewReadingState.inProgress;
    }
    return LibraryViewReadingState.notStarted;
  }
}
