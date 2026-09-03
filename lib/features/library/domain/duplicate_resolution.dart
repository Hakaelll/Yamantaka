import 'library_document.dart';

enum DuplicateMatchKind { none, exactFingerprint, probableMetadata, samePath }

final class DuplicateEvidence {
  const DuplicateEvidence({
    required this.kind,
    required this.score,
    this.sameFingerprint = false,
    this.sameNormalizedPath = false,
    this.sameFileName = false,
    this.sameSize = false,
    this.modifiedWithinSeconds,
  });

  final DuplicateMatchKind kind;
  final int score;
  final bool sameFingerprint;
  final bool sameNormalizedPath;
  final bool sameFileName;
  final bool sameSize;
  final int? modifiedWithinSeconds;

  bool get requiresConfirmation => kind != DuplicateMatchKind.exactFingerprint;

  Map<String, Object?> toJson() => {
    'kind': kind.name,
    'score': score,
    'same_fingerprint': sameFingerprint,
    'same_normalized_path': sameNormalizedPath,
    'same_file_name': sameFileName,
    'same_size': sameSize,
    if (modifiedWithinSeconds != null)
      'modified_within_seconds': modifiedWithinSeconds,
  };
}

final class DuplicateCandidate {
  const DuplicateCandidate({
    required this.existing,
    required this.incoming,
    required this.evidence,
  });

  final LibraryDocument existing;
  final LibraryDocument incoming;
  final DuplicateEvidence evidence;

  /// Exact fingerprints are safe to propose for a merge, but the caller still
  /// owns the explicit confirmation step and the pre-backup.
  bool get canSuggestMerge =>
      evidence.kind == DuplicateMatchKind.exactFingerprint;
}

abstract final class DuplicateClassifier {
  static DuplicateEvidence classify(
    LibraryDocument existing,
    LibraryDocument incoming,
  ) {
    final sameFingerprint =
        existing.fingerprint.isNotEmpty &&
        existing.fingerprint == incoming.fingerprint;
    final samePath = existing.normalizedPath == incoming.normalizedPath;
    final sameName =
        existing.fileName.toLowerCase() == incoming.fileName.toLowerCase();
    final sameSize = existing.fileSize == incoming.fileSize;
    final modifiedDelta = existing.fileModifiedAt
        .difference(incoming.fileModifiedAt)
        .inSeconds
        .abs();
    final score =
        (sameFingerprint ? 100 : 0) +
        (samePath ? 40 : 0) +
        (sameName ? 20 : 0) +
        (sameSize ? 15 : 0) +
        (modifiedDelta <= 2 ? 5 : 0);
    final kind = sameFingerprint
        ? DuplicateMatchKind.exactFingerprint
        : samePath
        ? DuplicateMatchKind.samePath
        : score >= 35
        ? DuplicateMatchKind.probableMetadata
        : DuplicateMatchKind.none;
    return DuplicateEvidence(
      kind: kind,
      score: score,
      sameFingerprint: sameFingerprint,
      sameNormalizedPath: samePath,
      sameFileName: sameName,
      sameSize: sameSize,
      modifiedWithinSeconds: modifiedDelta,
    );
  }
}

enum MetadataConflictChoice {
  existing,
  incoming,
  nonEmptyExisting,
  nonEmptyIncoming,
}

final class MetadataMergeResult {
  const MetadataMergeResult({
    required this.title,
    required this.author,
    required this.libraryNote,
    required this.conflicts,
  });

  final String? title;
  final String? author;
  final String? libraryNote;
  final List<String> conflicts;

  bool get requiresReview => conflicts.isNotEmpty;
}

abstract final class MetadataMergePolicy {
  /// Existing local overrides win; otherwise a non-empty incoming value fills
  /// a missing field. Conflicts are reported instead of silently discarded.
  static MetadataMergeResult merge(
    LibraryDocument existing,
    LibraryDocument incoming, {
    MetadataConflictChoice conflict = MetadataConflictChoice.nonEmptyExisting,
  }) {
    final title = _mergeField(
      'title',
      existing.effectiveTitle,
      incoming.effectiveTitle,
      conflict,
    );
    final author = _mergeField(
      'author',
      existing.effectiveAuthor,
      incoming.effectiveAuthor,
      conflict,
    );
    final libraryNote = _mergeField(
      'libraryNote',
      existing.libraryNote,
      incoming.libraryNote,
      conflict,
    );
    return MetadataMergeResult(
      title: title.value,
      author: author.value,
      libraryNote: libraryNote.value,
      conflicts: [
        if (title.conflict) 'title',
        if (author.conflict) 'author',
        if (libraryNote.conflict) 'library_note',
      ],
    );
  }

  static MetadataMergeResult mergePerField(
    LibraryDocument existing,
    LibraryDocument incoming,
    Map<String, MetadataConflictChoice> choices,
  ) {
    final title = _mergeField(
      'title',
      existing.effectiveTitle,
      incoming.effectiveTitle,
      choices['title'] ?? MetadataConflictChoice.nonEmptyExisting,
    );
    final author = _mergeField(
      'author',
      existing.effectiveAuthor,
      incoming.effectiveAuthor,
      choices['author'] ?? MetadataConflictChoice.nonEmptyExisting,
    );
    final libraryNote = _mergeField(
      'libraryNote',
      existing.libraryNote,
      incoming.libraryNote,
      choices['library_note'] ?? MetadataConflictChoice.nonEmptyExisting,
    );
    return MetadataMergeResult(
      title: title.value,
      author: author.value,
      libraryNote: libraryNote.value,
      conflicts: [
        if (title.conflict) 'title',
        if (author.conflict) 'author',
        if (libraryNote.conflict) 'library_note',
      ],
    );
  }

  static ({String? value, bool conflict}) _mergeField(
    String _,
    String? existing,
    String? incoming,
    MetadataConflictChoice choice,
  ) {
    final left = existing?.trim();
    final right = incoming?.trim();
    if (left == null || left.isEmpty) {
      return (value: right, conflict: false);
    }
    if (right == null || right.isEmpty || left == right) {
      return (value: left, conflict: false);
    }
    final value = switch (choice) {
      MetadataConflictChoice.existing ||
      MetadataConflictChoice.nonEmptyExisting => left,
      MetadataConflictChoice.incoming ||
      MetadataConflictChoice.nonEmptyIncoming => right,
    };
    return (value: value, conflict: true);
  }
}
