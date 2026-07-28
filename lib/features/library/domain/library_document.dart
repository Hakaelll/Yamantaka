import '../../reader/domain/reader_models.dart';

final class LibraryDocument {
  const LibraryDocument({
    required this.id,
    required this.path,
    required this.normalizedPath,
    required this.fileName,
    required this.pageCount,
    required this.fileSize,
    required this.fileModifiedAt,
    required this.addedAt,
    required this.lastOpenedAt,
    required this.fingerprint,
    this.title,
    this.author,
    this.lastPageIndex = 0,
    this.readingProgress = 0,
    this.preferredViewMode = ReaderMode.document,
    this.preferredTheme = ReaderTheme.ivory,
    this.isMissing = false,
    this.thumbnailPath,
  });

  final String id;
  final String path;
  final String normalizedPath;
  final String fileName;
  final String? title;
  final String? author;
  final int pageCount;
  final int fileSize;
  final DateTime fileModifiedAt;
  final DateTime addedAt;
  final DateTime lastOpenedAt;
  final int lastPageIndex;
  final double readingProgress;
  final ReaderMode preferredViewMode;
  final ReaderTheme preferredTheme;
  final bool isMissing;
  final String fingerprint;
  final String? thumbnailPath;

  LibraryDocument copyWith({
    String? path,
    String? normalizedPath,
    String? fileName,
    String? title,
    String? author,
    int? pageCount,
    int? fileSize,
    DateTime? fileModifiedAt,
    DateTime? addedAt,
    DateTime? lastOpenedAt,
    int? lastPageIndex,
    double? readingProgress,
    ReaderMode? preferredViewMode,
    ReaderTheme? preferredTheme,
    bool? isMissing,
    String? fingerprint,
    String? thumbnailPath,
  }) => LibraryDocument(
    id: id,
    path: path ?? this.path,
    normalizedPath: normalizedPath ?? this.normalizedPath,
    fileName: fileName ?? this.fileName,
    title: title ?? this.title,
    author: author ?? this.author,
    pageCount: pageCount ?? this.pageCount,
    fileSize: fileSize ?? this.fileSize,
    fileModifiedAt: fileModifiedAt ?? this.fileModifiedAt,
    addedAt: addedAt ?? this.addedAt,
    lastOpenedAt: lastOpenedAt ?? this.lastOpenedAt,
    lastPageIndex: lastPageIndex ?? this.lastPageIndex,
    readingProgress: readingProgress ?? this.readingProgress,
    preferredViewMode: preferredViewMode ?? this.preferredViewMode,
    preferredTheme: preferredTheme ?? this.preferredTheme,
    isMissing: isMissing ?? this.isMissing,
    fingerprint: fingerprint ?? this.fingerprint,
    thumbnailPath: thumbnailPath ?? this.thumbnailPath,
  );
}
