import '../../reader/domain/reader_models.dart';
import 'document_cover.dart';

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
    this.sourceTitle,
    this.sourceAuthor,
    this.titleOverride,
    this.authorOverride,
    this.libraryNote,
    this.lastPageIndex = 0,
    this.readingProgress = 0,
    this.preferredViewMode = ReaderMode.document,
    this.preferredTheme = ReaderTheme.ivory,
    this.isMissing = false,
    this.hasFileChanged = false,
    this.thumbnailPath,
    this.coverMetadata,
    this.isFavorite = false,
  });

  final String id;
  final String path;
  final String normalizedPath;
  final String fileName;
  final String? title;
  final String? author;

  /// Metadata detected from the PDF or imported with the document.
  final String? sourceTitle;
  final String? sourceAuthor;

  /// Explicit local overrides. They never modify the PDF file.
  final String? titleOverride;
  final String? authorOverride;
  final String? libraryNote;
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
  final bool hasFileChanged;
  final String fingerprint;
  final String? thumbnailPath;
  final DocumentCoverMetadata? coverMetadata;
  final bool isFavorite;

  String get effectiveTitle => titleOverride ?? title ?? fileName;
  String? get effectiveAuthor => authorOverride ?? author;
  bool get hasMetadataOverrides =>
      titleOverride != null || authorOverride != null;
  bool get hasLocalMetadata => hasMetadataOverrides || libraryNote != null;

  LibraryDocument copyWith({
    String? id,
    String? path,
    String? normalizedPath,
    String? fileName,
    String? title,
    String? author,
    String? sourceTitle,
    String? sourceAuthor,
    String? titleOverride,
    String? authorOverride,
    Object? libraryNote = _keepMetadataValue,
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
    bool? hasFileChanged,
    String? fingerprint,
    String? thumbnailPath,
    DocumentCoverMetadata? coverMetadata,
    bool? isFavorite,
  }) => LibraryDocument(
    id: id ?? this.id,
    path: path ?? this.path,
    normalizedPath: normalizedPath ?? this.normalizedPath,
    fileName: fileName ?? this.fileName,
    title: title ?? this.title,
    author: author ?? this.author,
    sourceTitle: sourceTitle ?? this.sourceTitle,
    sourceAuthor: sourceAuthor ?? this.sourceAuthor,
    titleOverride: titleOverride ?? this.titleOverride,
    authorOverride: authorOverride ?? this.authorOverride,
    libraryNote: identical(libraryNote, _keepMetadataValue)
        ? this.libraryNote
        : _normalizedLibraryNote(libraryNote as String?),
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
    hasFileChanged: hasFileChanged ?? this.hasFileChanged,
    fingerprint: fingerprint ?? this.fingerprint,
    thumbnailPath: thumbnailPath ?? this.thumbnailPath,
    coverMetadata: coverMetadata ?? this.coverMetadata,
    isFavorite: isFavorite ?? this.isFavorite,
  );

  LibraryDocument withMetadataOverrides({
    Object? title = _keepMetadataValue,
    Object? author = _keepMetadataValue,
    Object? note = _keepMetadataValue,
  }) {
    final nextTitle = identical(title, _keepMetadataValue)
        ? titleOverride
        : _normalizedMetadataValue(title as String?);
    final nextAuthor = identical(author, _keepMetadataValue)
        ? authorOverride
        : _normalizedMetadataValue(author as String?);
    final nextNote = identical(note, _keepMetadataValue)
        ? libraryNote
        : _normalizedLibraryNote(note as String?);
    final baseTitle = sourceTitle ?? this.title;
    final baseAuthor = sourceAuthor ?? this.author;
    return LibraryDocument(
      id: id,
      path: path,
      normalizedPath: normalizedPath,
      fileName: fileName,
      title: nextTitle ?? baseTitle,
      author: nextAuthor ?? baseAuthor,
      sourceTitle: baseTitle,
      sourceAuthor: baseAuthor,
      titleOverride: nextTitle,
      authorOverride: nextAuthor,
      libraryNote: nextNote,
      pageCount: pageCount,
      fileSize: fileSize,
      fileModifiedAt: fileModifiedAt,
      addedAt: addedAt,
      lastOpenedAt: lastOpenedAt,
      lastPageIndex: lastPageIndex,
      readingProgress: readingProgress,
      preferredViewMode: preferredViewMode,
      preferredTheme: preferredTheme,
      isMissing: isMissing,
      hasFileChanged: hasFileChanged,
      fingerprint: fingerprint,
      thumbnailPath: thumbnailPath,
      coverMetadata: coverMetadata,
      isFavorite: isFavorite,
    );
  }

  static String? _normalizedMetadataValue(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    if (clean.length > 512) {
      throw ArgumentError.value(value, 'metadata', 'Maximum length is 512.');
    }
    return clean;
  }

  static String? _normalizedLibraryNote(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    if (clean.length > 4000) {
      throw ArgumentError.value(
        value,
        'libraryNote',
        'Maximum length is 4000.',
      );
    }
    return clean;
  }
}

const Object _keepMetadataValue = Object();
