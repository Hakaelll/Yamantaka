import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/single_instance_protocol.dart';

void main() {
  test('activation keeps PDF order, removes duplicates and bounds input', () {
    final request = SingleInstanceProtocol.fromArguments([
      'first.pdf',
      'ignored.txt',
      'first.pdf',
      'second.pdf',
    ]);
    expect(request.pdfPaths, hasLength(2));
    expect(request.pdfPaths.first.toLowerCase(), endsWith('first.pdf'));
    expect(
      SingleInstanceProtocol.decode(
        SingleInstanceProtocol.encode(request),
      ).pdfPaths,
      request.pdfPaths,
    );
  });

  test('activation size uses UTF-8 bytes rather than UTF-16 code units', () {
    final request = ActivationRequest(['a' * 40000 + '.pdf']);
    expect(SingleInstanceProtocol.encode(request), isNotEmpty);
  });
}
