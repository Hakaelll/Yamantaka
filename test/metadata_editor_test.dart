import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/metadata_editor.dart';

void main() {
  test('local metadata override does not change source fields', () {
    final now = DateTime.utc(2026, 1, 1);
    final document = LibraryDocument(
      id: 'doc',
      path: 'doc.pdf',
      normalizedPath: 'doc.pdf',
      fileName: 'doc.pdf',
      title: 'Detected',
      author: 'Author',
      pageCount: 1,
      fileSize: 1,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'hash',
    );
    final edited = MetadataEditState.fromDocument(document)
        .copyWith(
          titleOverride: 'Custom',
          libraryNote: '  Keep this outside the PDF  ',
        )
        .applyTo(document);
    expect(edited.effectiveTitle, 'Custom');
    expect(edited.sourceTitle, 'Detected');
    expect(edited.titleOverride, 'Custom');
    expect(edited.libraryNote, 'Keep this outside the PDF');
  });

  test('blank values normalize to restore-original semantics', () {
    final now = DateTime.utc(2026, 1, 1);
    final document = LibraryDocument(
      id: 'doc',
      path: 'doc.pdf',
      normalizedPath: 'doc.pdf',
      fileName: 'doc.pdf',
      title: 'Detected',
      pageCount: 1,
      fileSize: 1,
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'hash',
    ).withMetadataOverrides(title: 'Custom');
    final restored = MetadataEditState.fromDocument(
      document,
    ).copyWith(titleOverride: ' ').applyTo(document);
    expect(restored.titleOverride, isNull);
    expect(restored.effectiveTitle, 'Detected');
  });

  test('metadata note is bounded and serialized locally', () {
    final state = MetadataEditState(
      sourceTitle: 'Detected',
      sourceAuthor: null,
      titleOverride: null,
      authorOverride: null,
      libraryNote: 'A note',
    );
    expect(state.normalized().toJson()['library_note'], 'A note');
    expect(
      () => state.copyWith(libraryNote: 'x' * 4001).normalized(),
      throwsArgumentError,
    );
  });
}
