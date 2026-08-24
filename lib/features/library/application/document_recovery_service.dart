import 'dart:io';

import '../../../storage/repositories/document_repository.dart';
import '../domain/library_document.dart';
import 'document_service.dart';

enum RecoveryConfidence { exact, high, ambiguous, none }

final class RecoveryMatch {
  const RecoveryMatch({
    required this.document,
    required this.candidate,
    required this.confidence,
    required this.score,
  });

  final LibraryDocument document;
  final InspectedFile? candidate;
  final RecoveryConfidence confidence;
  final int score;
}

final class RecoverySummary {
  const RecoverySummary({
    required this.recovered,
    required this.ambiguous,
    required this.notFound,
    required this.modified,
  });

  final int recovered;
  final int ambiguous;
  final int notFound;
  final int modified;
}

final class DocumentRecoveryService {
  const DocumentRecoveryService(this._documents, this._documentService);

  final DocumentRepository _documents;
  final DocumentService _documentService;

  Future<List<RecoveryMatch>> findMatches(
    String directoryPath, {
    bool recursive = true,
  }) async {
    final missing = _documents.all().where((document) => document.isMissing);
    final inspected = <InspectedFile>[];
    await for (final entity in Directory(
      directoryPath,
    ).list(recursive: recursive, followLinks: false)) {
      if (entity is! File || !entity.path.toLowerCase().endsWith('.pdf')) {
        continue;
      }
      try {
        inspected.add(await _documentService.inspect(entity.path));
      } on Object {
        // An inaccessible candidate is omitted, never treated as a match.
      }
    }
    return [for (final document in missing) _bestMatch(document, inspected)];
  }

  Future<RecoverySummary> recoverExact(List<RecoveryMatch> matches) async {
    var recovered = 0;
    var ambiguous = 0;
    var notFound = 0;
    var modified = 0;
    for (final match in matches) {
      switch (match.confidence) {
        case RecoveryConfidence.exact:
          _documentService.register(
            file: match.candidate!,
            pageCount: match.document.pageCount,
          );
          recovered++;
        case RecoveryConfidence.high:
          modified++;
        case RecoveryConfidence.ambiguous:
          ambiguous++;
        case RecoveryConfidence.none:
          notFound++;
      }
    }
    return RecoverySummary(
      recovered: recovered,
      ambiguous: ambiguous,
      notFound: notFound,
      modified: modified,
    );
  }

  static RecoveryMatch _bestMatch(
    LibraryDocument document,
    List<InspectedFile> candidates,
  ) {
    final ranked = candidates.map((candidate) {
      var score = 0;
      if (candidate.fingerprint == document.fingerprint) score += 100;
      if (candidate.fileName.toLowerCase() == document.fileName.toLowerCase()) {
        score += 20;
      }
      if (candidate.fileSize == document.fileSize) score += 15;
      final modifiedDelta = candidate.modifiedAt
          .difference(document.fileModifiedAt)
          .abs();
      if (modifiedDelta < const Duration(seconds: 2)) score += 5;
      return (candidate: candidate, score: score);
    }).toList()..sort((left, right) => right.score.compareTo(left.score));
    if (ranked.isEmpty || ranked.first.score < 20) {
      return RecoveryMatch(
        document: document,
        candidate: null,
        confidence: RecoveryConfidence.none,
        score: 0,
      );
    }
    final best = ranked.first;
    final tied =
        ranked.length > 1 && ranked[1].score == best.score && best.score < 100;
    final confidence = best.candidate.fingerprint == document.fingerprint
        ? RecoveryConfidence.exact
        : tied
        ? RecoveryConfidence.ambiguous
        : best.score >= 35
        ? RecoveryConfidence.high
        : RecoveryConfidence.ambiguous;
    return RecoveryMatch(
      document: document,
      candidate: best.candidate,
      confidence: confidence,
      score: best.score,
    );
  }
}
