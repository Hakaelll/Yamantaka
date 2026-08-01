import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/duplicate_resolution.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';

void main() {
  test('exact fingerprints are distinguishable from probable metadata', () {
    final left = _document(id: 'left', fingerprint: 'same');
    final exact = _document(id: 'exact', fingerprint: 'same');
    final probable = _document(
      id: 'probable',
      fingerprint: 'different',
      fileName: left.fileName,
      fileSize: left.fileSize,
    );

    expect(
      DuplicateClassifier.classify(left, exact).kind,
      DuplicateMatchKind.exactFingerprint,
    );
    expect(
      DuplicateClassifier.classify(left, probable).kind,
      DuplicateMatchKind.probableMetadata,
    );
    expect(
      DuplicateClassifier.classify(left, probable).requiresConfirmation,
      isTrue,
    );
  });

  test('metadata merge reports conflicts instead of hiding them', () {
    final result = MetadataMergePolicy.merge(
      _document(id: 'left', title: 'Local title', libraryNote: 'Local note'),
      _document(
        id: 'right',
        title: 'Incoming title',
        libraryNote: 'Incoming note',
      ),
    );
    expect(result.title, 'Local title');
    expect(result.libraryNote, 'Local note');
    expect(result.conflicts, contains('title'));
    expect(result.conflicts, contains('library_note'));
    expect(result.requiresReview, isTrue);
  });
}

LibraryDocument _document({
  required String id,
  String fingerprint = 'fingerprint',
  String fileName = 'book.pdf',
  int fileSize = 10,
  String? title,
  String? libraryNote,
}) {
  final now = DateTime.utc(2026, 1, 1);
  return LibraryDocument(
    id: id,
    path: '$id.pdf',
    normalizedPath: '$id.pdf',
    fileName: fileName,
    title: title,
    libraryNote: libraryNote,
    pageCount: 2,
    fileSize: fileSize,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: fingerprint,
  );
}
