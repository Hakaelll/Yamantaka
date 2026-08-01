import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/print_request.dart';

void main() {
  test('print range is bounded to the opened document', () {
    final request = const PrintRequest(
      firstPage: -5,
      lastPage: 100,
      landscape: true,
    ).normalized(10);
    expect(request.firstPage, 0);
    expect(request.lastPage, 9);
    expect(request.landscape, isTrue);
  });
}
