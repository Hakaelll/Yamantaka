import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/document_service.dart';
import 'package:yamantaka/features/library/application/library_filter.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/domain/document_cover.dart';
import 'package:yamantaka/features/library/domain/library_artifact_layout.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/pdf_engine/cache/thumbnail_service.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main() {
  group('automatic cover selection', () {
    test('renders display covers at readable source resolutions', () {
      expect(ThumbnailService.coverPreviewScale, greaterThanOrEqualTo(0.4));
      expect(ThumbnailService.smallCoverScale, greaterThanOrEqualTo(0.45));
      expect(ThumbnailService.largeCoverScale, greaterThanOrEqualTo(1));
      expect(CoverSelectionPolicy.algorithmVersion, 3);
    });

    test('keeps a reasonable preference for the first page', () {
      final selected = CoverSelectionPolicy.choose([
        _metrics(page: 0, uniform: 0.35, edges: 0.08, contrast: 0.2),
        _metrics(
          page: 1,
          uniform: 0.12,
          edges: 0.62,
          contrast: 0.08,
          textLength: 1400,
          lines: 48,
        ),
      ]);

      expect(selected.metrics.pageIndex, 0);
    });

    test('selects an evident visual cover on a later page', () {
      final selected = CoverSelectionPolicy.choose([
        _metrics(
          page: 0,
          uniform: 0.99,
          edges: 0,
          contrast: 0,
          textLength: 0,
          lines: 0,
        ),
        _metrics(
          page: 1,
          uniform: 0.42,
          edges: 0.12,
          contrast: 0.24,
          textLength: 80,
          lines: 4,
        ),
      ]);

      expect(selected.metrics.pageIndex, 1);
      expect(selected.confidence, greaterThanOrEqualTo(0.70));
    });

    test('falls back to page one when no candidate clearly improves it', () {
      final selected = CoverSelectionPolicy.choose([
        _metrics(page: 0, uniform: 0.98, edges: 0, contrast: 0.02),
        _metrics(
          page: 1,
          uniform: 0.98,
          edges: 0,
          contrast: 0.02,
          contents: true,
        ),
      ]);

      expect(selected.metrics.pageIndex, 0);
    });
  });

  group('cover metadata persistence', () {
    late AppDatabase database;
    late DocumentRepository repository;

    setUp(() {
      database = AppDatabase.inMemory(AppLogger(minimumLevel: LogLevel.error));
      repository = DocumentRepository(database);
    });

    tearDown(() => database.close());

    test('manual cover and favorite survive a repository round trip', () {
      final document = _document();
      repository.save(document);
      repository.saveCover(
        DocumentCoverMetadata(
          documentId: document.id,
          sourcePageIndex: 3,
          smallThumbnailPath: r'C:\cache\doc-small.png',
          largeThumbnailPath: r'C:\cache\doc-large.png',
          generatedAt: DateTime.utc(2026, 7, 29, 12),
          sourceModifiedAt: document.fileModifiedAt,
          confidence: 1,
          selectionMode: CoverSelectionMode.manual,
          algorithmVersion: CoverSelectionPolicy.algorithmVersion,
          cropMode: CoverCropMode.contain,
          pageAspectRatio: 0.72,
        ),
      );
      repository.toggleFavorite(document.id);

      final restored = repository.byId(document.id)!;
      expect(restored.isFavorite, isTrue);
      expect(restored.coverMetadata?.selectionMode, CoverSelectionMode.manual);
      expect(restored.coverMetadata?.sourcePageIndex, 3);
      expect(restored.coverMetadata?.largeThumbnailPath, contains('large'));
      expect(restored.coverMetadata?.pageAspectRatio, 0.72);

      DocumentService(repository).register(
        file: InspectedFile(
          path: document.path,
          normalizedPath: document.normalizedPath,
          fileName: document.fileName,
          fileSize: document.fileSize + 1,
          modifiedAt: document.fileModifiedAt.add(const Duration(minutes: 1)),
          fingerprint: 'changed-but-manual',
        ),
        pageCount: document.pageCount,
      );
      expect(
        repository.byId(document.id)!.coverMetadata?.selectionMode,
        CoverSelectionMode.manual,
      );
    });

    test('changed source invalidates automatically selected cached covers', () {
      final original = _document();
      repository.save(original);
      repository.saveCover(
        DocumentCoverMetadata(
          documentId: original.id,
          sourcePageIndex: 2,
          smallThumbnailPath: 'small.png',
          largeThumbnailPath: 'large.png',
          generatedAt: DateTime.utc(2026, 7, 29),
          sourceModifiedAt: original.fileModifiedAt,
          confidence: 1,
          selectionMode: CoverSelectionMode.automatic,
          algorithmVersion: CoverSelectionPolicy.algorithmVersion,
          cropMode: CoverCropMode.contain,
          pageAspectRatio: 0.7,
        ),
      );

      final updated = DocumentService(repository).register(
        file: InspectedFile(
          path: original.path,
          normalizedPath: original.normalizedPath,
          fileName: original.fileName,
          fileSize: original.fileSize + 1,
          modifiedAt: original.fileModifiedAt.add(const Duration(minutes: 1)),
          fingerprint: 'changed-fingerprint',
        ),
        pageCount: original.pageCount,
      );

      expect(updated.id, original.id);
      expect(repository.byId(original.id)!.coverMetadata, isNull);
      expect(repository.byId(original.id)!.thumbnailPath, isNull);
    });
  });

  group('artifact layout and navigation', () {
    test('jitter is stable, bounded and never changes between evaluations', () {
      final first = LibraryArtifactLayout.jitterFor(r'C:\books\one.pdf');
      final second = LibraryArtifactLayout.jitterFor(r'C:\books\one.pdf');

      expect(second.offsetX, first.offsetX);
      expect(second.offsetY, first.offsetY);
      expect(second.rotationDegrees, first.rotationDegrees);
      expect(first.offsetX.abs(), lessThanOrEqualTo(8));
      expect(first.offsetY.abs(), lessThanOrEqualTo(12));
      expect(first.rotationDegrees.abs(), lessThanOrEqualTo(1.25));
    });

    test('grid reserves enough room for jitter without invalid overlap', () {
      final metrics = LibraryArtifactLayout.metricsFor(
        viewportWidth: 1200,
        documentCount: 100,
      );

      expect(metrics.columns, inInclusiveRange(2, 7));
      expect(metrics.cellExtent - metrics.artifactWidth, greaterThan(24));
    });

    test('keyboard navigation follows the predictable visual grid', () {
      expect(
        LibraryArtifactLayout.indexAfterNavigation(
          currentIndex: 6,
          itemCount: 20,
          columns: 4,
          direction: ArtifactNavigationDirection.up,
        ),
        2,
      );
      expect(
        LibraryArtifactLayout.indexAfterNavigation(
          currentIndex: 18,
          itemCount: 20,
          columns: 4,
          direction: ArtifactNavigationDirection.down,
        ),
        19,
      );
      expect(
        LibraryArtifactLayout.indexAfterNavigation(
          currentIndex: 9,
          itemCount: 20,
          columns: 4,
          direction: ArtifactNavigationDirection.first,
        ),
        0,
      );
    });
  });

  test('combined search, favorite and reading filters stay deterministic', () {
    final favorite = _document(isFavorite: true);
    final emptyTitle = _document(
      id: 'untitled',
      title: '',
      fileName: 'notes.pdf',
      progress: 0.5,
    );
    final result = evaluateLibraryDocuments(
      [emptyTitle, favorite],
      query: 'book',
      favoritesOnly: true,
      readingState: LibraryReadingState.notStarted,
    );

    expect(result.documents, hasLength(1));
    expect(result.documents.single.id, favorite.id);
    expect(documentLabel(emptyTitle), 'notes.pdf');
  });

  test('selection and scroll context survive library reloads', () async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = DocumentRepository(database)..save(_document());
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(controller.dispose);

    await controller.load();
    expect(controller.selectedDocumentId, 'book');
    controller
      ..scrollOffset = 248
      ..query = 'book'
      ..favoritesOnly = true;
    await controller.load();

    expect(controller.selectedDocumentId, 'book');
    expect(controller.scrollOffset, 248);
    expect(controller.query, 'book');
    expect(controller.favoritesOnly, isTrue);
  });
}

CoverCandidateMetrics _metrics({
  required int page,
  required double uniform,
  required double edges,
  required double contrast,
  int textLength = 120,
  int lines = 6,
  bool contents = false,
}) => CoverCandidateMetrics(
  pageIndex: page,
  uniformSpaceRatio: uniform,
  edgeDensity: edges,
  luminanceContrast: contrast,
  textLength: textLength,
  lineCount: lines,
  hasContentsMarker: contents,
);

LibraryDocument _document({
  String id = 'book',
  String? title = 'Book',
  String fileName = 'book.pdf',
  double progress = 0,
  bool isFavorite = false,
}) {
  final now = DateTime.utc(2026, 7, 29, 10);
  return LibraryDocument(
    id: id,
    path: r'C:\books\book.pdf',
    normalizedPath: r'c:\books\book.pdf',
    fileName: fileName,
    title: title,
    pageCount: 12,
    fileSize: 200,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    readingProgress: progress,
    fingerprint: 'fingerprint-$id',
    isFavorite: isFavorite,
  );
}
