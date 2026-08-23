enum LibraryTopic {
  history('Historia'),
  politics('Política'),
  philosophy('Filosofía'),
  literature('Literatura'),
  science('Ciencia'),
  technology('Tecnología'),
  economics('Economía'),
  society('Sociedad'),
  psychology('Psicología'),
  art('Arte'),
  religionMythology('Religión y mitología'),
  law('Derecho'),
  education('Educación'),
  health('Salud'),
  nature('Naturaleza'),
  biography('Biografía');

  const LibraryTopic(this.label);

  final String label;
}

enum EnrichmentStatus { pending, analyzing, completed, review, noText, failed }

enum EnrichmentOrigin { automatic, manual }

final class DocumentTopicAssignment {
  const DocumentTopicAssignment({
    required this.topic,
    required this.confidence,
    required this.origin,
  });

  final LibraryTopic topic;
  final double confidence;
  final EnrichmentOrigin origin;

  bool get isManual => origin == EnrichmentOrigin.manual;
}

final class DocumentEnrichment {
  const DocumentEnrichment({
    required this.documentId,
    required this.fingerprint,
    required this.analyzerVersion,
    required this.status,
    this.suggestedTitle,
    this.suggestedAuthor,
    this.titleConfidence,
    this.authorConfidence,
    this.topicsManual = false,
    this.analyzedAt,
    this.errorCode,
  });

  final String documentId;
  final String fingerprint;
  final int analyzerVersion;
  final EnrichmentStatus status;
  final String? suggestedTitle;
  final String? suggestedAuthor;
  final double? titleConfidence;
  final double? authorConfidence;
  final bool topicsManual;
  final DateTime? analyzedAt;
  final String? errorCode;

  bool get needsReview =>
      status == EnrichmentStatus.review &&
      (suggestedTitle != null || suggestedAuthor != null);
}

final class LibraryFacet {
  const LibraryFacet({required this.value, required this.count});

  final String value;
  final int count;
}
