const int _mask = 0xffffffff;

/// Small dependency-free SHA-256 implementation for deterministic verification
/// tools. It is intentionally kept separate from application code so the
/// offline corpus gate does not depend on Pub resolution.
String sha256Hex(List<int> input) {
  final bytes = <int>[...input, 0x80];
  while (bytes.length % 64 != 56) {
    bytes.add(0);
  }
  final bitLength = input.length * 8;
  for (var shift = 56; shift >= 0; shift -= 8) {
    bytes.add((bitLength >> shift) & 0xff);
  }

  var h0 = 0x6a09e667;
  var h1 = 0xbb67ae85;
  var h2 = 0x3c6ef372;
  var h3 = 0xa54ff53a;
  var h4 = 0x510e527f;
  var h5 = 0x9b05688c;
  var h6 = 0x1f83d9ab;
  var h7 = 0x5be0cd19;

  for (var offset = 0; offset < bytes.length; offset += 64) {
    final words = List<int>.filled(64, 0);
    for (var index = 0; index < 16; index++) {
      final position = offset + index * 4;
      words[index] =
          ((bytes[position] << 24) |
              (bytes[position + 1] << 16) |
              (bytes[position + 2] << 8) |
              bytes[position + 3]) &
          _mask;
    }
    for (var index = 16; index < 64; index++) {
      final s0 =
          _rotr(words[index - 15], 7) ^
          _rotr(words[index - 15], 18) ^
          (words[index - 15] >> 3);
      final s1 =
          _rotr(words[index - 2], 17) ^
          _rotr(words[index - 2], 19) ^
          (words[index - 2] >> 10);
      words[index] = (words[index - 16] + s0 + words[index - 7] + s1) & _mask;
    }

    var a = h0;
    var b = h1;
    var c = h2;
    var d = h3;
    var e = h4;
    var f = h5;
    var g = h6;
    var h = h7;
    for (var index = 0; index < 64; index++) {
      final s1 = _rotr(e, 6) ^ _rotr(e, 11) ^ _rotr(e, 25);
      final choose = (e & f) ^ ((~e) & g);
      final temporary1 =
          (h + s1 + choose + _roundConstants[index] + words[index]) & _mask;
      final s0 = _rotr(a, 2) ^ _rotr(a, 13) ^ _rotr(a, 22);
      final majority = (a & b) ^ (a & c) ^ (b & c);
      final temporary2 = (s0 + majority) & _mask;
      h = g;
      g = f;
      f = e;
      e = (d + temporary1) & _mask;
      d = c;
      c = b;
      b = a;
      a = (temporary1 + temporary2) & _mask;
    }

    h0 = (h0 + a) & _mask;
    h1 = (h1 + b) & _mask;
    h2 = (h2 + c) & _mask;
    h3 = (h3 + d) & _mask;
    h4 = (h4 + e) & _mask;
    h5 = (h5 + f) & _mask;
    h6 = (h6 + g) & _mask;
    h7 = (h7 + h) & _mask;
  }

  return [
    h0,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    h7,
  ].map((word) => word.toRadixString(16).padLeft(8, '0')).join();
}

int _rotr(int value, int shift) =>
    ((value >> shift) | (value << (32 - shift))) & _mask;

const List<int> _roundConstants = <int>[
  0x428a2f98,
  0x71374491,
  0xb5c0fbcf,
  0xe9b5dba5,
  0x3956c25b,
  0x59f111f1,
  0x923f82a4,
  0xab1c5ed5,
  0xd807aa98,
  0x12835b01,
  0x243185be,
  0x550c7dc3,
  0x72be5d74,
  0x80deb1fe,
  0x9bdc06a7,
  0xc19bf174,
  0xe49b69c1,
  0xefbe4786,
  0x0fc19dc6,
  0x240ca1cc,
  0x2de92c6f,
  0x4a7484aa,
  0x5cb0a9dc,
  0x76f988da,
  0x983e5152,
  0xa831c66d,
  0xb00327c8,
  0xbf597fc7,
  0xc6e00bf3,
  0xd5a79147,
  0x06ca6351,
  0x14292967,
  0x27b70a85,
  0x2e1b2138,
  0x4d2c6dfc,
  0x53380d13,
  0x650a7354,
  0x766a0abb,
  0x81c2c92e,
  0x92722c85,
  0xa2bfe8a1,
  0xa81a664b,
  0xc24b8b70,
  0xc76c51a3,
  0xd192e819,
  0xd6990624,
  0xf40e3585,
  0x106aa070,
  0x19a4c116,
  0x1e376c08,
  0x2748774c,
  0x34b0bcb5,
  0x391c0cb3,
  0x4ed8aa4a,
  0x5b9cca4f,
  0x682e6ff3,
  0x748f82ee,
  0x78a5636f,
  0x84c87814,
  0x8cc70208,
  0x90befffa,
  0xa4506ceb,
  0xbef9a3f7,
  0xc67178f2,
];
