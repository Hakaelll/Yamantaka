import 'dart:io';

import '../test/support/pdf_fixture.dart';

void main(List<String> arguments) {
  if (arguments.length != 1) {
    stderr.writeln('Usage: dart run tool/create_test_pdf.dart <output.pdf>');
    exitCode = 64;
    return;
  }
  final output = File(arguments.single)..parent.createSync(recursive: true);
  output.writeAsBytesSync(createTestPdf());
  stdout.writeln(output.absolute.path);
}
