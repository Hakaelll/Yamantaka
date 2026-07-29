import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';

void main() {
  late AppDatabase database;
  late DocumentRepository documents;

  setUp(() {
    database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
    documents = DocumentRepository(database);
  });

  tearDown(() => database.close());

  test('migration creates the complete version-one schema', () {
    final version = database.database
        .select('PRAGMA user_version')
        .first
        .columnAt(0);
    final tables = database.database
        .select(
          "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .map((row) => row['name'])
        .toSet();

    expect(version, 1);
    expect(
      tables,
      containsAll({
        'documents',
        'document_preferences',
        'bookmarks',
        'reading_sessions',
        'app_metadata',
      }),
    );
  });

  test('recent documents are updated without duplicates', () {
    final original = _document();
    documents.save(original);
    documents.save(
      original.copyWith(
        lastOpenedAt: original.lastOpenedAt.add(const Duration(hours: 1)),
        lastPageIndex: 4,
        readingProgress: 0.4,
      ),
    );

    final recent = documents.recent();
    expect(recent, hasLength(1));
    expect(recent.single.lastPageIndex, 4);
    expect(recent.single.readingProgress, 0.4);
  });

  test(
    'document identity follows a moved file and invalidates changed covers',
    () {
      final service = DocumentService(documents);
      final original = service.register(
        file: InspectedFile(
          path: r'C:\Books\book.pdf',
          normalizedPath: r'c:\books\book.pdf',
          fileName: 'book.pdf',
          fileSize: 1024,
          modifiedAt: DateTime.utc(2026, 7, 28),
          fingerprint: 'fingerprint-a',
        ),
        pageCount: 10,
      );
      documents.updateThumbnail(original.id, r'C:\cache\cover.png');

      final changed = service.register(
        file: InspectedFile(
          path: r'C:\Books\book.pdf',
          normalizedPath: r'c:\books\book.pdf',
          fileName: 'book.pdf',
          fileSize: 2048,
          modifiedAt: DateTime.utc(2026, 7, 29),
          fingerprint: 'fingerprint-b',
        ),
        pageCount: 12,
      );
      expect(changed.id, original.id);
      expect(changed.thumbnailPath, isNull);

      final moved = service.register(
        file: InspectedFile(
          path: r'D:\Moved\book.pdf',
          normalizedPath: r'd:\moved\book.pdf',
          fileName: 'book.pdf',
          fileSize: 2048,
          modifiedAt: DateTime.utc(2026, 7, 29),
          fingerprint: 'fingerprint-b',
        ),
        pageCount: 12,
      );
      expect(moved.id, original.id);
      expect(moved.path, r'D:\Moved\book.pdf');
      expect(documents.recent(), hasLength(1));
    },
  );

  test('document preferences survive a repository round trip', () {
    documents.save(_document());
    final repository = DocumentPreferencesRepository(database);
    final expected = DocumentPreferences(
      documentId: 'doc-1',
      viewMode: ReaderMode.book,
      readingTheme: ReaderTheme.sepia,
      zoomMode: ZoomMode.fitWidth,
      zoomLevel: 1.3,
      pageLayout: PageLayoutMode.facing,
      pageRotation: 90,
      horizontalMargin: 44,
      verticalMargin: 36,
      controlsAutoHide: false,
      scrollOffset: 0.25,
    );

    repository.save(expected);
    final actual = repository.load('doc-1');

    expect(repository.exists('doc-1'), isTrue);
    expect(actual.viewMode, ReaderMode.book);
    expect(actual.readingTheme, ReaderTheme.sepia);
    expect(actual.pageLayout, PageLayoutMode.facing);
    expect(actual.pageRotation, 90);
    expect(actual.horizontalMargin, 44);
  });

  test('bookmark creation, replacement and deletion are persisted', () {
    documents.save(_document());
    final repository = BookmarkRepository(database);

    final first = repository.add('doc-1', 3, label: 'Capítulo');
    repository.add('doc-1', 3, label: 'Capítulo revisado');
    expect(repository.forDocument('doc-1'), hasLength(1));
    expect(repository.atPage('doc-1', 3)?.label, 'Capítulo revisado');

    repository.remove(first.id);
    expect(repository.forDocument('doc-1'), isEmpty);
  });

  test('reading session stores bounded active time and final page', () {
    documents.save(_document());
    final repository = ReadingSessionRepository(database);
    final session = repository.start('doc-1', 1);
    repository.finish(id: session.id, endPage: 8, activeSeconds: 42);

    final row = database.database.select(
      'SELECT * FROM reading_sessions WHERE id = ?',
      [session.id],
    ).single;
    expect(row['end_page'], 8);
    expect(row['active_seconds'], 42);
    expect(row['ended_at'], isNotNull);
  });

  test('reading history is bounded, ordered and individually clearable', () {
    documents.save(_document());
    final repository = ReadingSessionRepository(database);
    for (var index = 0; index < 30; index++) {
      final session = repository.start('doc-1', index % 5);
      repository.finish(
        id: session.id,
        endPage: (index % 5) + 1,
        activeSeconds: index,
      );
    }
    repository.trimToLimit(25);
    final history = repository.history(limit: 100);
    expect(history, hasLength(25));
    expect(history.first.documentTitle, 'book.pdf');
    expect(
      history.first.session.startedAt.isAfter(history.last.session.startedAt),
      isTrue,
    );

    repository.delete(history.first.session.id);
    expect(repository.history(limit: 100), hasLength(24));
    repository.clear();
    expect(repository.history(), isEmpty);
  });
}

LibraryDocument _document() {
  final now = DateTime.utc(2026, 7, 28, 18);
  return LibraryDocument(
    id: 'doc-1',
    path: r'C:\Books\book.pdf',
    normalizedPath: r'c:\books\book.pdf',
    fileName: 'book.pdf',
    pageCount: 11,
    fileSize: 1024,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: 'fingerprint',
  );
}
