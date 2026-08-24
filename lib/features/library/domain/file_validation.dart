enum FileValidationStatus { available, missing, modified, failed }

final class FileValidationCandidate {
  const FileValidationCandidate({
    required this.id,
    required this.path,
    required this.fileSize,
    required this.fileModifiedAt,
    required this.wasMissing,
    required this.wasModified,
  });

  final String id;
  final String path;
  final int fileSize;
  final DateTime fileModifiedAt;
  final bool wasMissing;
  final bool wasModified;
}

final class FileValidationResult {
  const FileValidationResult({required this.id, required this.status});

  final String id;
  final FileValidationStatus status;
}

final class FileValidationSummary {
  const FileValidationSummary({
    this.available = 0,
    this.missing = 0,
    this.modified = 0,
    this.failed = 0,
    this.cancelled = 0,
  });

  final int available;
  final int missing;
  final int modified;
  final int failed;
  final int cancelled;

  int get checked => available + missing + modified + failed;
  int get total => checked + cancelled;

  FileValidationSummary copyWith({
    int? available,
    int? missing,
    int? modified,
    int? failed,
    int? cancelled,
  }) => FileValidationSummary(
    available: available ?? this.available,
    missing: missing ?? this.missing,
    modified: modified ?? this.modified,
    failed: failed ?? this.failed,
    cancelled: cancelled ?? this.cancelled,
  );

  FileValidationSummary add(FileValidationStatus status) => switch (status) {
    FileValidationStatus.available => copyWith(available: available + 1),
    FileValidationStatus.missing => copyWith(missing: missing + 1),
    FileValidationStatus.modified => copyWith(modified: modified + 1),
    FileValidationStatus.failed => copyWith(failed: failed + 1),
  };
}

final class FileValidationProgress {
  const FileValidationProgress({
    required this.isRunning,
    required this.isManual,
    required this.summary,
    required this.total,
  });

  const FileValidationProgress.idle()
    : isRunning = false,
      isManual = false,
      summary = const FileValidationSummary(),
      total = 0;

  final bool isRunning;
  final bool isManual;
  final FileValidationSummary summary;
  final int total;
}
