import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/search/application/search_controller.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';
import 'package:yamantaka/pdf_engine/presentation/pdf_viewer_adapter.dart';

import 'support/stub_pdf_engine.dart';

void main() {
  test(
    'cancel updates UI state immediately and discards obsolete results',
    () async {
      final engine = _SearchEngine()..pending = Completer();
      final controller = SearchController(
        engine,
        PdfViewController(),
        AppLogger(minimumLevel: LogLevel.error),
      )..attach('pdf-1', 2);
      addTearDown(controller.dispose);

      final future = controller.search('needle');
      await Future<void>.delayed(Duration.zero);
      final token = engine.lastToken!;
      expect(controller.isSearching, isTrue);

      controller.cancel();

      expect(controller.isSearching, isFalse);
      expect(token.isCancelled, isTrue);
      engine.pending!.complete([
        const PdfSearchMatch(
          pageIndex: 1,
          start: 0,
          length: 6,
          snippet: 'needle',
        ),
      ]);
      await future;
      expect(controller.matches, isEmpty);
    },
  );

  test(
    'reports scans without extractable text and navigates result pages',
    () async {
      final engine = _SearchEngine();
      final viewer = PdfViewController();
      var navigatedPage = -1;
      viewer.attach(
        pageCount: 3,
        currentPageIndex: 0,
        zoom: 1,
        goToPage: (page) async => navigatedPage = page,
        zoomBy: (_) async {},
        fitWidth: () async {},
        fitPage: () async {},
      );
      final controller = SearchController(
        engine,
        viewer,
        AppLogger(minimumLevel: LogLevel.error),
      )..attach('pdf-1', 3);
      addTearDown(controller.dispose);

      engine.pageTexts = ['', ' ', ''];
      await controller.search('needle');
      expect(controller.noExtractableText, isTrue);

      engine
        ..matches = const [
          PdfSearchMatch(
            pageIndex: 0,
            start: 0,
            length: 6,
            snippet: 'first needle',
          ),
          PdfSearchMatch(
            pageIndex: 2,
            start: 3,
            length: 6,
            snippet: 'last needle',
          ),
        ]
        ..pageTexts = ['first needle', '', 'last needle'];
      await controller.search('needle');
      expect(navigatedPage, 0);
      expect(controller.matches, hasLength(2));

      await controller.next();
      expect(navigatedPage, 2);
      await controller.previous();
      expect(navigatedPage, 0);
    },
  );
}

final class _SearchEngine extends StubPdfDocumentEngine {
  Completer<List<PdfSearchMatch>>? pending;
  List<PdfSearchMatch> matches = const [];
  List<String> pageTexts = const [];
  PdfOperationToken? lastToken;

  @override
  Future<String> extractPageText({
    required String documentId,
    required int pageIndex,
  }) async => pageTexts[pageIndex];

  @override
  Future<List<PdfSearchMatch>> searchText({
    required String documentId,
    required String query,
    PdfOperationToken? cancellationToken,
  }) {
    lastToken = cancellationToken;
    return pending?.future ?? Future.value(matches);
  }
}
