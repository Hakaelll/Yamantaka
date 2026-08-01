import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/library_view.dart';

void main() {
  test('saved views do not retain text queries by default', () {
    final view = SavedLibraryView(
      id: 'view',
      name: 'Favoritos',
      rules: const [
        LibraryViewRule(kind: LibraryViewRuleKind.favorite, boolean: true),
        LibraryViewRule(kind: LibraryViewRuleKind.text, text: 'private'),
      ],
    ).normalized();
    expect(view.rules, hasLength(1));
    expect(view.toJson()['persisted_text_query'], isFalse);
  });

  test('view evaluator combines typed rules locally', () {
    final document = _document(isFavorite: true, progress: 0.5);
    expect(
      LibraryViewEvaluator.matches(document, const [
        LibraryViewRule(kind: LibraryViewRuleKind.favorite, boolean: true),
        LibraryViewRule(
          kind: LibraryViewRuleKind.readingState,
          readingState: LibraryViewReadingState.inProgress,
        ),
        LibraryViewRule(
          kind: LibraryViewRuleKind.minimumProgress,
          progress: 0.4,
        ),
      ]),
      isTrue,
    );
  });
}

LibraryDocument _document({bool isFavorite = false, double progress = 0}) {
  final now = DateTime.utc(2026, 1, 1);
  return LibraryDocument(
    id: 'doc',
    path: 'doc.pdf',
    normalizedPath: 'doc.pdf',
    fileName: 'doc.pdf',
    title: 'Local book',
    pageCount: 10,
    fileSize: 10,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    readingProgress: progress,
    fingerprint: 'fingerprint',
    isFavorite: isFavorite,
  );
}
