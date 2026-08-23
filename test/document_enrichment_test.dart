import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/document_enrichment_controller.dart';
import 'package:yamantaka/features/library/domain/document_enrichment.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/library_query.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  group('local enrichment rules', () {
    test('PDF metadata wins with automatic confidence', () {
      final result = LocalDocumentEnrichment.analyze(
        metadata: const PdfDocumentMetadata(
          title: 'The Dispossessed',
          author: 'Ursula K. Le Guin',
        ),
        coverBlocks: const [],
        sampledText: '',
        fileName: 'scan_0042.pdf',
      );

      expect(result.title?.value, 'The Dispossessed');
      expect(result.title?.confidence, greaterThanOrEqualTo(0.8));
      expect(result.author?.value, 'Ursula K. Le Guin');
      expect(result.author?.confidence, greaterThanOrEqualTo(0.8));
    });

    test('structured cover finds a title and explicit author', () {
      final result = LocalDocumentEnrichment.analyze(
        metadata: const PdfDocumentMetadata(),
        coverBlocks: const [
          PdfStructuredTextBlock(
            text: 'La sociedad abierta',
            left: 40,
            top: 80,
            width: 420,
            height: 64,
            pageWidth: 500,
            pageHeight: 800,
          ),
          PdfStructuredTextBlock(
            text: 'Por Karl Popper',
            left: 80,
            top: 240,
            width: 240,
            height: 22,
            pageWidth: 500,
            pageHeight: 800,
          ),
        ],
        sampledText: 'sociedad democracia gobierno cultura social',
        fileName: 'documento.pdf',
      );

      expect(result.title?.value, 'La sociedad abierta');
      expect(result.title?.confidence, greaterThanOrEqualTo(0.8));
      expect(result.author?.value, 'Karl Popper');
      expect(result.author?.confidence, greaterThanOrEqualTo(0.8));
      expect(
        result.topics.map((topic) => topic.topic),
        containsAll([LibraryTopic.society, LibraryTopic.politics]),
      );
    });

    test('unmarked cover author stays reviewable and no text stays empty', () {
      final uncertain = LocalDocumentEnrichment.analyze(
        metadata: const PdfDocumentMetadata(),
        coverBlocks: const [
          PdfStructuredTextBlock(
            text: 'Ensayos sobre el tiempo',
            left: 20,
            top: 40,
            width: 440,
            height: 60,
            pageWidth: 500,
            pageHeight: 800,
          ),
          PdfStructuredTextBlock(
            text: 'María Zambrano',
            left: 120,
            top: 260,
            width: 260,
            height: 20,
            pageWidth: 500,
            pageHeight: 800,
          ),
        ],
        sampledText: '',
        fileName: 'ensayos.pdf',
      );
      final noText = LocalDocumentEnrichment.analyze(
        metadata: const PdfDocumentMetadata(),
        coverBlocks: const [],
        sampledText: '',
        fileName: 'scan_0099.pdf',
      );

      expect(uncertain.author?.value, 'María Zambrano');
      expect(uncertain.author?.confidence, inInclusiveRange(0.55, 0.79));
      expect(noText.hasExtractableText, isFalse);
      expect(noText.title, isNull);
      expect(noText.author, isNull);
      expect(noText.topics, isEmpty);
    });

    test('classifies reproducible Spanish and English topic vocabularies', () {
      final result = LocalDocumentEnrichment.analyze(
        metadata: const PdfDocumentMetadata(),
        coverBlocks: const [],
        sampledText:
            'historia guerra siglo historical war century '
            'software programming computer algorithm technology '
            'economía mercado capital finanzas inflación',
        fileName: 'temas.pdf',
      );

      expect(result.topics, hasLength(3));
      expect(
        result.topics.map((topic) => topic.topic),
        containsAll([
          LibraryTopic.history,
          LibraryTopic.technology,
          LibraryTopic.economics,
        ]),
      );
    });
  });

  test('controller bounds sampling and preserves manual fields', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-enrichment-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final file = File('${directory.path}${Platform.pathSeparator}book.pdf');
    await file.writeAsBytes(const [0x25, 0x50, 0x44, 0x46]);
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = DocumentRepository(database);
    final now = file.lastModifiedSync().toUtc();
    final document = LibraryDocument(
      id: 'book',
      path: file.path,
      normalizedPath: file.path.toLowerCase(),
      fileName: 'book.pdf',
      title: 'book.pdf',
      titleOverride: 'Mi título manual',
      authorOverride: 'Mi autor manual',
      pageCount: 40,
      fileSize: file.lengthSync(),
      fileModifiedAt: now,
      addedAt: now,
      lastOpenedAt: now,
      fingerprint: 'fingerprint-a',
    );
    repository.save(document);
    final engine = _EnrichmentEngine(pageCount: 40);
    final controller = DocumentEnrichmentController(repository, engine);
    addTearDown(controller.dispose);

    await controller.analyzeDocument(repository.byId('book')!);
    final enriched = repository.byId('book')!;
    expect(engine.extractedPages.length, lessThanOrEqualTo(12));
    expect(engine.closedDocuments, 1);
    expect(enriched.effectiveTitle, 'Mi título manual');
    expect(enriched.effectiveAuthor, 'Mi autor manual');
    expect(enriched.sourceTitle, 'Historia política contemporánea');
    expect(enriched.sourceAuthor, 'Clara Campoamor');
    expect(
      enriched.topics.map((topic) => topic.topic),
      contains(LibraryTopic.history),
    );

    controller.setManualTopics('book', {LibraryTopic.art});
    await controller.analyzeDocument(repository.byId('book')!);
    final manual = repository.byId('book')!;
    expect(manual.enrichment?.topicsManual, isTrue);
    expect(manual.topics.map((topic) => topic.topic), [LibraryTopic.art]);
    expect(
      repository
          .page(
            const LibraryQuery(
              authors: {'mi autor manual'},
              topics: {LibraryTopic.art},
            ),
          )
          .documents,
      hasLength(1),
    );
  });
}

final class _EnrichmentEngine extends StubPdfDocumentEngine {
  _EnrichmentEngine({required this.pageCount});

  final int pageCount;
  final List<int> extractedPages = [];
  int closedDocuments = 0;

  @override
  Future<OpenedPdfDocument> openDocument(
    String path, {
    String? password,
  }) async {
    return OpenedPdfDocument(
      documentId: 'opened',
      path: path,
      pageCount: pageCount,
      pageSizes: List.filled(pageCount, const PdfPageSize(500, 800)),
      isEncrypted: false,
    );
  }

  @override
  Future<PdfDocumentMetadata> readDocumentMetadata(String documentId) async =>
      const PdfDocumentMetadata(
        title: 'Historia política contemporánea',
        author: 'Clara Campoamor',
      );

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
    cancellationToken,
  }) async {
    extractedPages.add(pageIndex);
    return List.filled(9000, 'historia política gobierno guerra').join(' ');
  }

  @override
  Future<void> closeDocument(String documentId) async {
    closedDocuments++;
  }
}
