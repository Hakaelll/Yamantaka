import 'library_document.dart';

final class MetadataEditState {
  const MetadataEditState({
    required this.sourceTitle,
    required this.sourceAuthor,
    required this.titleOverride,
    required this.authorOverride,
    this.libraryNote,
  });

  factory MetadataEditState.fromDocument(LibraryDocument document) =>
      MetadataEditState(
        sourceTitle: document.sourceTitle ?? document.title,
        sourceAuthor: document.sourceAuthor ?? document.author,
        titleOverride: document.titleOverride,
        authorOverride: document.authorOverride,
        libraryNote: document.libraryNote,
      );

  final String? sourceTitle;
  final String? sourceAuthor;
  final String? titleOverride;
  final String? authorOverride;
  final String? libraryNote;

  String get effectiveTitle => titleOverride ?? sourceTitle ?? '';
  String? get effectiveAuthor => authorOverride ?? sourceAuthor;

  MetadataEditState copyWith({
    String? sourceTitle,
    String? sourceAuthor,
    String? titleOverride,
    String? authorOverride,
    String? libraryNote,
  }) => MetadataEditState(
    sourceTitle: sourceTitle ?? this.sourceTitle,
    sourceAuthor: sourceAuthor ?? this.sourceAuthor,
    titleOverride: titleOverride ?? this.titleOverride,
    authorOverride: authorOverride ?? this.authorOverride,
    libraryNote: libraryNote ?? this.libraryNote,
  );

  MetadataEditState normalized() => MetadataEditState(
    sourceTitle: _clean(sourceTitle),
    sourceAuthor: _clean(sourceAuthor),
    titleOverride: _clean(titleOverride),
    authorOverride: _clean(authorOverride),
    libraryNote: _cleanNote(libraryNote),
  );

  LibraryDocument applyTo(LibraryDocument document) {
    final value = normalized();
    return document.withMetadataOverrides(
      title: value.titleOverride,
      author: value.authorOverride,
      note: value.libraryNote,
    );
  }

  Map<String, Object?> toJson() => {
    'source_title': sourceTitle,
    'source_author': sourceAuthor,
    'title_override': titleOverride,
    'author_override': authorOverride,
    'library_note': libraryNote,
  };

  static String? _clean(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    if (clean.length > 512) {
      throw ArgumentError.value(value, 'metadata', 'Maximum length is 512.');
    }
    return clean;
  }

  static String? _cleanNote(String? value) {
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
