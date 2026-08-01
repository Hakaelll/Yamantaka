import 'package:flutter_test/flutter_test.dart';

import '../tool/sha256.dart';

void main() {
  test('dependency-free SHA-256 matches the standard empty vector', () {
    expect(
      sha256Hex(const <int>[]),
      'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    );
  });

  test('dependency-free SHA-256 matches a short vector', () {
    expect(
      sha256Hex('abc'.codeUnits),
      'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
    );
  });
}
