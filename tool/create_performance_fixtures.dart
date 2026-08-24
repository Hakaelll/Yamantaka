import 'dart:convert';
import 'dart:io';

import '../test/support/pdf_fixture.dart';

void main(List<String> arguments) {
  if (arguments.length != 1) {
    stderr.writeln(
      'Usage: dart run tool/create_performance_fixtures.dart <output-dir>',
    );
    exitCode = 64;
    return;
  }
  final root = Directory(arguments.single)..createSync(recursive: true);
  final library = Directory('${root.path}/library')
    ..createSync(recursive: true);
  for (var index = 1; index <= 40; index++) {
    final name = index.toString().padLeft(2, '0');
    File(
      '${library.path}/fixture-$name.pdf',
    ).writeAsBytesSync(createPdfWithPageCount(2 + index % 4), flush: true);
  }
  final longDocument = File('${root.path}/long-100-pages.pdf');
  longDocument.writeAsBytesSync(createPdfWithPageCount(100), flush: true);
  const scalePageCounts = [1, 50, 500, 5000];
  for (final pageCount in scalePageCounts) {
    File(
      '${root.path}/scale-$pageCount-pages.pdf',
    ).writeAsBytesSync(createPdfWithPageCount(pageCount), flush: true);
  }
  final heavyDocument = File('${root.path}/heavy-vector-8-pages.pdf');
  heavyDocument.writeAsBytesSync(createHeavyVectorPdf(), flush: true);
  File('${root.path}/manifest.json').writeAsStringSync(
    const JsonEncoder.withIndent('  ').convert({
      'schemaVersion': 2,
      'libraryDocuments': 40,
      'longDocumentPages': 100,
      'scaleDocumentPages': scalePageCounts,
      'heavyVectorPages': 8,
      'heavyVectorShapesPerPage': 1200,
      'containsPersonalData': false,
      'generatedAt': DateTime.now().toUtc().toIso8601String(),
    }),
    flush: true,
  );
  stdout.writeln(root.absolute.path);
}
