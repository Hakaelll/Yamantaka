import 'dart:convert';
import 'dart:typed_data';

const testPdfPassword = 'yamantaka-test';

Uint8List createTestPdf({bool includeThirdPage = false}) {
  final objects = <String>[
    '<< /Type /Catalog /Pages 2 0 R >>',
    if (includeThirdPage)
      '<< /Type /Pages /Kids [3 0 R 5 0 R 8 0 R] /Count 3 >>'
    else
      '<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] '
        '/Resources << /Font << /F1 4 0 R >> >> /Contents 6 0 R >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] '
        '/Resources << /Font << /F1 4 0 R >> >> /Contents 7 0 R >>',
    _stream('BT /F1 18 Tf 36 340 Td (Yamantaka page one) Tj ET'),
    _stream('BT /F1 18 Tf 36 340 Td (Yamantaka page two) Tj ET'),
    if (includeThirdPage) ...[
      '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] '
          '/Resources << /Font << /F1 4 0 R >> >> /Contents 9 0 R >>',
      _stream('BT /F1 18 Tf 36 340 Td (Yamantaka page three) Tj ET'),
    ],
  ];
  final buffer = StringBuffer('%PDF-1.4\n');
  final offsets = <int>[0];
  for (var index = 0; index < objects.length; index++) {
    offsets.add(utf8.encode(buffer.toString()).length);
    buffer
      ..write('${index + 1} 0 obj\n')
      ..write(objects[index])
      ..write('\nendobj\n');
  }
  final xrefOffset = utf8.encode(buffer.toString()).length;
  buffer
    ..write('xref\n0 ${objects.length + 1}\n')
    ..write('0000000000 65535 f \n');
  for (final offset in offsets.skip(1)) {
    buffer.write('${offset.toString().padLeft(10, '0')} 00000 n \n');
  }
  buffer
    ..write('trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\n')
    ..write('startxref\n$xrefOffset\n%%EOF\n');
  return Uint8List.fromList(utf8.encode(buffer.toString()));
}

Uint8List createPdfWithPageCount(int pageCount) {
  if (pageCount < 1 || pageCount > 1000) {
    throw RangeError.range(pageCount, 1, 1000, 'pageCount');
  }
  final pageObjects = [
    for (var index = 0; index < pageCount; index++) 4 + index * 2,
  ];
  final objects = <String>[
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [${pageObjects.map((id) => '$id 0 R').join(' ')}] '
        '/Count $pageCount >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
  ];
  for (var index = 0; index < pageCount; index++) {
    final contentObject = 5 + index * 2;
    objects
      ..add(
        '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
        '/Resources << /Font << /F1 3 0 R >> >> '
        '/Contents $contentObject 0 R >>',
      )
      ..add(
        _stream(
          'BT /F1 18 Tf 54 720 Td '
          '(Yamantaka performance page ${index + 1}) Tj ET',
        ),
      );
  }
  final buffer = StringBuffer('%PDF-1.4\n');
  final offsets = <int>[0];
  for (var index = 0; index < objects.length; index++) {
    offsets.add(utf8.encode(buffer.toString()).length);
    buffer
      ..write('${index + 1} 0 obj\n')
      ..write(objects[index])
      ..write('\nendobj\n');
  }
  final xrefOffset = utf8.encode(buffer.toString()).length;
  buffer
    ..write('xref\n0 ${objects.length + 1}\n')
    ..write('0000000000 65535 f \n');
  for (final offset in offsets.skip(1)) {
    buffer.write('${offset.toString().padLeft(10, '0')} 00000 n \n');
  }
  buffer
    ..write('trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\n')
    ..write('startxref\n$xrefOffset\n%%EOF\n');
  return Uint8List.fromList(utf8.encode(buffer.toString()));
}

Uint8List createHeavyVectorPdf({int pageCount = 8, int shapesPerPage = 1200}) {
  if (pageCount < 1 || pageCount > 100) {
    throw RangeError.range(pageCount, 1, 100, 'pageCount');
  }
  if (shapesPerPage < 1 || shapesPerPage > 5000) {
    throw RangeError.range(shapesPerPage, 1, 5000, 'shapesPerPage');
  }
  final pageObjects = [
    for (var index = 0; index < pageCount; index++) 4 + index * 2,
  ];
  final objects = <String>[
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [${pageObjects.map((id) => '$id 0 R').join(' ')}] '
        '/Count $pageCount >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
  ];
  for (var page = 0; page < pageCount; page++) {
    final contentObject = 5 + page * 2;
    final drawing = StringBuffer('q\n');
    for (var shape = 0; shape < shapesPerPage; shape++) {
      final column = shape % 30;
      final row = (shape ~/ 30) % 38;
      final shade = ((shape + page) % 9 + 1) / 10;
      drawing
        ..writeln('${shade.toStringAsFixed(1)} g')
        ..writeln('${20 + column * 19} ${30 + row * 19} 12 12 re f');
    }
    drawing
      ..writeln('0 g')
      ..writeln(
        'BT /F1 18 Tf 54 752 Td '
        '(Yamantaka heavy fixture ${page + 1}) Tj ET',
      )
      ..writeln('Q');
    objects
      ..add(
        '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
        '/Resources << /Font << /F1 3 0 R >> >> '
        '/Contents $contentObject 0 R >>',
      )
      ..add(_stream(drawing.toString()));
  }
  final buffer = StringBuffer('%PDF-1.4\n');
  final offsets = <int>[0];
  for (var index = 0; index < objects.length; index++) {
    offsets.add(utf8.encode(buffer.toString()).length);
    buffer
      ..write('${index + 1} 0 obj\n')
      ..write(objects[index])
      ..write('\nendobj\n');
  }
  final xrefOffset = utf8.encode(buffer.toString()).length;
  buffer
    ..write('xref\n0 ${objects.length + 1}\n')
    ..write('0000000000 65535 f \n');
  for (final offset in offsets.skip(1)) {
    buffer.write('${offset.toString().padLeft(10, '0')} 00000 n \n');
  }
  buffer
    ..write('trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\n')
    ..write('startxref\n$xrefOffset\n%%EOF\n');
  return Uint8List.fromList(utf8.encode(buffer.toString()));
}

/// Legal AES-128 fixture generated from [createTestPdf] with pypdf 6.10.0.
Uint8List createEncryptedTestPdf() => base64Decode(
  'JVBERi0xLjQKJeLjz9MKMSAwIG9iago8PAovUHJvZHVjZXIgPDkyZDMyMzEwMzYwZjY1YjYyYzc3NWRhNTMwOWMx'
  'OGI2ZTIxYzE0NmExMThjZjExMzk4ZWIwZGQ2N2UxOTIzNGU+Cj4+CmVuZG9iagoyIDAgb2JqCjw8Ci9UeXBlIC9Q'
  'YWdlcwovQ291bnQgMgovS2lkcyBbIDQgMCBSIDcgMCBSIF0KPj4KZW5kb2JqCjMgMCBvYmoKPDwKL1R5cGUgL0Nh'
  'dGFsb2cKL1BhZ2VzIDIgMCBSCj4+CmVuZG9iago0IDAgb2JqCjw8Ci9UeXBlIC9QYWdlCi9NZWRpYUJveCBbIDAg'
  'MCAzMDAgNDAwIF0KL1Jlc291cmNlcyA8PAovRm9udCA8PAovRjEgNSAwIFIKPj4KPj4KL0NvbnRlbnRzIDYgMCBS'
  'Ci9QYXJlbnQgMiAwIFIKPj4KZW5kb2JqCjUgMCBvYmoKPDwKL1R5cGUgL0ZvbnQKL1N1YnR5cGUgL1R5cGUxCi9C'
  'YXNlRm9udCAvSGVsdmV0aWNhCj4+CmVuZG9iago2IDAgb2JqCjw8Ci9MZW5ndGggODAKPj4Kc3RyZWFtCqUgMcCT'
  'uNjAtqeSSmFoCPuTg7JwybBLKYHiMAG7mStD9uJFKrN1bJEeFvHIiZ5RS+ADEVhTktK5WJ7lAqbasUZ41GqFHrUD'
  '6BrddIJZ38M7CmVuZHN0cmVhbQplbmRvYmoKNyAwIG9iago8PAovVHlwZSAvUGFnZQovTWVkaWFCb3ggWyAwIDAg'
  'MzAwIDQwMCBdCi9SZXNvdXJjZXMgPDwKL0ZvbnQgPDwKL0YxIDUgMCBSCj4+Cj4+Ci9Db250ZW50cyA4IDAgUgov'
  'UGFyZW50IDIgMCBSCj4+CmVuZG9iago4IDAgb2JqCjw8Ci9MZW5ndGggODAKPj4Kc3RyZWFtCgKdbza9a6oKWv6S'
  'B5mDxI8F+Zpw4CUCUz5jobet+rOr/8RG3sNaYXj4WqkRH0xZY6OYTBkCb+AiiVpBoifuKCUF+hsCE+v0FyBxPQfj'
  'NTHACmVuZHN0cmVhbQplbmRvYmoKOSAwIG9iago8PAovViA0Ci9SIDQKL0xlbmd0aCAxMjgKL1AgNDI5NDk2NzI5'
  'MgovRmlsdGVyIC9TdGFuZGFyZAovTyA8ZmIyYjU5YmJkNDcxNDg3OTNhZGNlOTUzYzJhOGMxYzljNjk3YjM0YjY0'
  'ZTJiMjM5Mzg2NThmZTUyZWVmNzc4ZD4KL1UgPDc0MTA2ZDA0MGMyMGZhZDU5Nzk0MmI5YjhmMWM3MDM0MjhiZjRl'
  'NWU0ZTc1OGE0MTY0MDA0ZTU2ZmZmYTAxMDg+Ci9DRiA8PAovU3RkQ0YgPDwKL0F1dGhFdmVudCAvRG9jT3Blbgov'
  'Q0ZNIC9BRVNWMgovTGVuZ3RoIDE2Cj4+Cj4+Ci9TdG1GIC9TdGRDRgovU3RyRiAvU3RkQ0YKPj4KZW5kb2JqCnhy'
  'ZWYKMCAxMAowMDAwMDAwMDAwIDY1NTM1IGYgCjAwMDAwMDAwMTUgMDAwMDAgbiAKMDAwMDAwMDExMyAwMDAwMCBu'
  'IAowMDAwMDAwMTc4IDAwMDAwIG4gCjAwMDAwMDAyMjcgMDAwMDAgbiAKMDAwMDAwMDM1NSAwMDAwMCBuIAowMDAw'
  'MDAwNDI1IDAwMDAwIG4gCjAwMDAwMDA1NTUgMDAwMDAgbiAKMDAwMDAwMDY4MyAwMDAwMCBuIAowMDAwMDAwODEz'
  'IDAwMDAwIG4gCnRyYWlsZXIKPDwKL1NpemUgMTAKL1Jvb3QgMyAwIFIKL0luZm8gMSAwIFIKL0lEIFsgPDM1Mzcz'
  'NzMwNjUzNDY1MzU2NjYzNjQzODYzMzQ2MjMxNjIzNjYyNjQzNDMyNjIzOTYzMzEzNjY0NjEzNDM3MzU+IDwzNTM3'
  'MzczMDY1MzQ2NTM1NjY2MzY0Mzg2MzM0NjIzMTYyMzY2MjY0MzQzMjYyMzk2MzMxMzY2NDYxMzQzNzM1PiBdCi9F'
  'bmNyeXB0IDkgMCBSCj4+CnN0YXJ0eHJlZgoxMTIwCiUlRU9GCg==',
);

String _stream(String content) =>
    '<< /Length ${utf8.encode(content).length} >>\n'
    'stream\n$content\nendstream';
