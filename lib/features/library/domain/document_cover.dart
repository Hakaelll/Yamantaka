enum CoverSelectionMode { automatic, manual }

enum CoverCropMode { contain }

final class DocumentCoverMetadata {
  const DocumentCoverMetadata({
    required this.documentId,
    required this.sourcePageIndex,
    required this.smallThumbnailPath,
    required this.largeThumbnailPath,
    required this.generatedAt,
    required this.sourceModifiedAt,
    required this.confidence,
    required this.selectionMode,
    required this.algorithmVersion,
    required this.cropMode,
    required this.pageAspectRatio,
  });

  final String documentId;
  final int sourcePageIndex;
  final String smallThumbnailPath;
  final String largeThumbnailPath;
  final DateTime generatedAt;
  final DateTime sourceModifiedAt;
  final double confidence;
  final CoverSelectionMode selectionMode;
  final int algorithmVersion;
  final CoverCropMode cropMode;
  final double pageAspectRatio;

  bool isValidFor({
    required DateTime modifiedAt,
    required int currentAlgorithmVersion,
  }) {
    return sourceModifiedAt.toUtc() == modifiedAt.toUtc() &&
        algorithmVersion == currentAlgorithmVersion;
  }
}

final class CoverCandidateMetrics {
  const CoverCandidateMetrics({
    required this.pageIndex,
    required this.uniformSpaceRatio,
    required this.edgeDensity,
    required this.luminanceContrast,
    required this.textLength,
    required this.lineCount,
    required this.hasContentsMarker,
  });

  final int pageIndex;
  final double uniformSpaceRatio;
  final double edgeDensity;
  final double luminanceContrast;
  final int textLength;
  final int lineCount;
  final bool hasContentsMarker;
}

final class ScoredCoverCandidate {
  const ScoredCoverCandidate({
    required this.metrics,
    required this.score,
    required this.confidence,
  });

  final CoverCandidateMetrics metrics;
  final double score;
  final double confidence;
}

final class CoverPreview {
  const CoverPreview({required this.pageIndex, required this.pngBytes});

  final int pageIndex;
  final List<int> pngBytes;
}

abstract final class CoverSelectionPolicy {
  static const int algorithmVersion = 3;
  static const int maximumCandidatePages = 6;

  static ScoredCoverCandidate score(CoverCandidateMetrics metrics) {
    final positionScore = switch (metrics.pageIndex) {
      0 => 40.0,
      1 => 22.0,
      2 => 12.0,
      3 => 8.0,
      4 => 5.0,
      _ => 3.0,
    };
    var score = positionScore;
    if (metrics.uniformSpaceRatio >= 0.28 &&
        metrics.uniformSpaceRatio <= 0.82) {
      score += 10;
    } else if (metrics.uniformSpaceRatio > 0.94) {
      score -= 12;
    }
    if (metrics.edgeDensity >= 0.025 && metrics.edgeDensity <= 0.34) {
      score += 12;
    } else if (metrics.edgeDensity > 0.48) {
      score -= 15;
    }
    if (metrics.luminanceContrast >= 0.12) score += 5;
    if (metrics.textLength <= 220 && metrics.lineCount <= 12) {
      score += 10;
    }
    if (metrics.textLength > 900 || metrics.lineCount > 34) score -= 20;
    if (metrics.hasContentsMarker) score -= 18;
    final confidence = ((score + 20) / 100).clamp(0.0, 1.0);
    return ScoredCoverCandidate(
      metrics: metrics,
      score: score,
      confidence: confidence,
    );
  }

  static ScoredCoverCandidate choose(
    Iterable<CoverCandidateMetrics> candidates,
  ) {
    final scored = candidates.map(score).toList(growable: false);
    if (scored.isEmpty) {
      throw ArgumentError.value(candidates, 'candidates', 'Must not be empty');
    }
    final first = scored.firstWhere(
      (candidate) => candidate.metrics.pageIndex == 0,
      orElse: () => scored.first,
    );
    final best = scored.reduce(
      (current, candidate) =>
          candidate.score > current.score ? candidate : current,
    );
    if (best.metrics.pageIndex == first.metrics.pageIndex) return first;
    if (best.confidence >= 0.70) return best;
    if (best.confidence >= 0.45 && best.score >= first.score + 12) return best;
    return first;
  }
}
