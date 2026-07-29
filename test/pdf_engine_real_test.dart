import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/pdf_engine/adapters/pdfrx_document_engine.dart';
import 'package:yamantaka/pdf_engine/models/pdf_models.dart';

import 'support/pdf_fixture.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'PDFium opens, renders, extracts and searches a generated PDF',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-engine-',
      );
      await pdfrxInitialize(tmpPath: directory.path);
      final file = File(
        '${directory.path}${Platform.pathSeparator}fixture.pdf',
      );
      await file.writeAsBytes(createTestPdf());
      final engine = PdfrxDocumentEngine(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(() async {
        await engine.dispose();
        await directory.delete(recursive: true);
      });

      final document = await engine.openDocument(file.path);
      expect(document.pageCount, 2);
      expect(
        await engine.extractPageText(
          documentId: document.documentId,
          pageIndex: 0,
        ),
        contains('Yamantaka page one'),
      );
      expect(
        await engine.searchText(
          documentId: document.documentId,
          query: 'page two',
        ),
        hasLength(1),
      );
      // Validate PDFium's native bitmap in this headless test. Converting it to
      // dart:ui Image requires a live Flutter rasterizer and is covered by the
      // Windows integration test instead.
      final nativePage = await engine
          .documentFor(document.documentId)
          .pages
          .first
          .ensureLoaded();
      final rendered = await nativePage.render(
        width: 306,
        height: 396,
        fullWidth: 306,
        fullHeight: 396,
        backgroundColor: 0xFFFFFFFF,
      );
      expect(rendered, isNotNull);
      expect(rendered!.width, 306);
      expect(rendered.height, 396);
      expect(rendered.pixels, hasLength(306 * 396 * 4));
      var nonWhitePixels = 0;
      for (var offset = 0; offset < rendered.pixels.length; offset += 4) {
        if (rendered.pixels[offset] < 245 ||
            rendered.pixels[offset + 1] < 245 ||
            rendered.pixels[offset + 2] < 245) {
          nonWhitePixels++;
        }
      }
      expect(nonWhitePixels, greaterThan(100));
      rendered.dispose();

      await engine.closeDocument(document.documentId);
    },
  );

  test('PDFium maps a corrupt local PDF to a typed failure', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-corrupt-pdf-',
    );
    await pdfrxInitialize(tmpPath: directory.path);
    final file = File('${directory.path}${Platform.pathSeparator}corrupt.pdf');
    await file.writeAsString('This is not a PDF.');
    final engine = PdfrxDocumentEngine(AppLogger(minimumLevel: LogLevel.error));
    addTearDown(() async {
      await engine.dispose();
      await directory.delete(recursive: true);
    });

    await expectLater(
      engine.openDocument(file.path),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.kind,
          'kind',
          FailureKind.corruptPdf,
        ),
      ),
    );
  });

  test(
    'render cancellation is counted without retaining pending work',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-cancelled-render-',
      );
      await pdfrxInitialize(tmpPath: directory.path);
      final file = File(
        '${directory.path}${Platform.pathSeparator}fixture.pdf',
      );
      await file.writeAsBytes(createTestPdf());
      final engine = PdfrxDocumentEngine(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(() async {
        await engine.dispose();
        await directory.delete(recursive: true);
      });
      final document = await engine.openDocument(file.path);
      final token = PdfOperationToken()..cancel();

      await expectLater(
        engine.renderPage(
          documentId: document.documentId,
          pageIndex: 0,
          scale: 1,
          cancellationToken: token,
        ),
        throwsA(_failure(FailureKind.cancelled)),
      );
      expect(engine.diagnostics.renderRequests, 1);
      expect(engine.diagnostics.cancelledRenders, 1);
      expect(engine.diagnostics.pendingRenders, 0);
      await engine.closeDocument(document.documentId);
    },
  );

  test('PDFium requires, rejects and accepts a real PDF password', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-encrypted-pdf-',
    );
    await pdfrxInitialize(tmpPath: directory.path);
    final file = File(
      '${directory.path}${Platform.pathSeparator}encrypted.pdf',
    );
    await file.writeAsBytes(createEncryptedTestPdf());
    final engine = PdfrxDocumentEngine(AppLogger(minimumLevel: LogLevel.error));
    addTearDown(() async {
      await engine.dispose();
      await directory.delete(recursive: true);
    });

    await expectLater(
      engine.openDocument(file.path),
      throwsA(_failure(FailureKind.passwordRequired)),
    );
    await expectLater(
      engine.openDocument(file.path, password: 'incorrect'),
      throwsA(_failure(FailureKind.wrongPassword)),
    );

    final document = await engine.openDocument(
      file.path,
      password: testPdfPassword,
    );
    expect(document.pageCount, 2);
    expect(document.isEncrypted, isTrue);
    expect(
      await engine.extractPageText(
        documentId: document.documentId,
        pageIndex: 0,
      ),
      contains('Yamantaka page one'),
    );
    await engine.closeDocument(document.documentId);
  });
}

Matcher _failure(FailureKind kind) =>
    isA<AppFailure>().having((failure) => failure.kind, 'kind', kind);
