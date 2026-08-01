import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import '../test/support/pdf_fixture.dart';
import 'sha256.dart';

Future<void> main(List<String> arguments) async {
  if (arguments.length != 1) {
    stderr.writeln(
      'Usage: dart run tool/create_pathological_corpus.dart <output-dir>',
    );
    exitCode = 64;
    return;
  }
  final root = Directory(arguments.single)..createSync(recursive: true);
  final cases = <String, _CorpusCase>{};

  cases['empty'] = _write(root, 'empty.pdf', _emptyPdf(), {
    'open': 'reject-with-typed-error-or-zero-page',
    'render': 'not-requested',
    'text': 'not-requested',
    'close': 'no-live-document',
  });
  final normal = Uint8List.fromList(createTestPdf(includeThirdPage: true));
  cases['normal'] = _write(root, 'normal.pdf', normal, {
    'open': 'success',
    'render': 'first-page',
    'text': 'extractable',
    'search': 'match-Yamantaka',
    'close': 'zero-live-document',
  });
  cases['corrupt'] = _write(
    root,
    'corrupt.pdf',
    Uint8List.fromList([
      0x25,
      0x50,
      0x44,
      0x46,
      0x2d,
      0x39,
      0x39,
      0x2e,
      0x39,
      0x0a,
      ...List<int>.filled(4096, 0x7f),
    ]),
    {
      'open': 'typed-parser-error-no-process-crash',
      'render': 'not-requested',
      'text': 'not-requested',
      'close': 'zero-live-document',
    },
  );
  cases['truncated'] =
      _write(root, 'truncated.pdf', normal.sublist(0, normal.length ~/ 2), {
        'open': 'typed-parser-error-or-recovery',
        'render': 'not-requested-on-error',
        'text': 'not-requested-on-error',
        'close': 'zero-live-document',
      });
  cases['encrypted'] = _write(root, 'encrypted.pdf', createEncryptedTestPdf(), {
    'open': 'password-required',
    'render': 'success-after-authorized-password',
    'text': 'success-after-authorized-password',
    'close': 'zero-live-document-and-cleared-password',
  });
  cases['long-1000'] =
      _write(root, 'long-1000-pages.pdf', createPdfWithPageCount(1000), {
        'open': 'progressive-index',
        'render': 'first-page-before-full-index',
        'text': 'cancelable',
        'search': 'incremental-or-cancelable',
        'close': 'zero-live-document',
      });
  cases['raster-extreme'] =
      _write(root, 'raster-extreme.pdf', _extremeRasterPdf(), {
        'open': 'success-or-safe-size-error',
        'render': 'tile-or-bounded-size-error-no-full-bitmap',
        'text': 'not-required',
        'close': 'zero-live-document',
      });
  cases['vector-heavy'] = _write(
    root,
    'vector-heavy.pdf',
    createHeavyVectorPdf(pageCount: 12, shapesPerPage: 2000),
    {
      'open': 'success',
      'render': 'bounded-and-cancelable',
      'text': 'extractable',
      'close': 'zero-live-document',
    },
  );
  cases['mixed-rotation'] =
      _write(root, 'mixed-rotation.pdf', _mixedRotationPdf(), {
        'open': 'success',
        'render': 'all-rotations',
        'text': 'extractable',
        'close': 'zero-live-document',
      });

  final manifest = <String, Object?>{
    'format': 'yamantaka-pathological-pdf-corpus',
    'schemaVersion': 1,
    'generatorVersion': 1,
    'generatedAt': DateTime.now().toUtc().toIso8601String(),
    'containsPersonalData': false,
    'containsCopyrightedContent': false,
    'passwordFixture': 'yamantaka-test',
    'cases': [
      for (final entry in cases.entries)
        {
          'id': entry.key,
          'file': entry.value.file,
          'bytes': entry.value.bytes,
          'sha256': entry.value.sha256,
          'expectations': entry.value.expectations,
          'limits': {
            'maxOpenSeconds': entry.key == 'long-1000' ? 20 : 10,
            'maxRenderSeconds': entry.key == 'raster-extreme' ? 15 : 10,
            'maxImageBytes': 128 * 1024 * 1024,
            'maxLiveDocumentsAfterClose': 0,
            'maxPrivateLogFields': 0,
          },
        },
    ],
  };
  final manifestPath = File('${root.path}/manifest.json');
  await manifestPath.writeAsString(
    const JsonEncoder.withIndent('  ').convert(manifest),
    flush: true,
  );
  stdout.writeln(root.absolute.path);
}

_CorpusCase _write(
  Directory root,
  String name,
  List<int> bytes,
  Map<String, String> expectations,
) {
  File('${root.path}/$name').writeAsBytesSync(bytes, flush: true);
  final digest = sha256Hex(bytes);
  return _CorpusCase(
    file: name,
    bytes: bytes.length,
    sha256: digest,
    expectations: expectations,
  );
}

Uint8List _emptyPdf() => Uint8List.fromList(utf8.encode('%PDF-1.4\n%%EOF\n'));

Uint8List _extremeRasterPdf() {
  const width = 2048;
  const height = 8192;
  final pixels = Uint8List(width * height);
  for (var y = 0; y < height; y++) {
    for (var x = 0; x < width; x++) {
      pixels[y * width + x] = ((x + y) % 256);
    }
  }
  return _singleImagePdf(
    width: width,
    height: height,
    mediaWidth: 20000,
    mediaHeight: 80000,
    pixels: pixels,
  );
}

Uint8List _singleImagePdf({
  required int width,
  required int height,
  required int mediaWidth,
  required int mediaHeight,
  required Uint8List pixels,
}) {
  final content = Uint8List.fromList(
    utf8.encode('q\n$mediaWidth 0 0 $mediaHeight 0 0 cm\n/Im1 Do\nQ'),
  );
  final objects = <_PdfObject>[
    _PdfObject('<< /Type /Catalog /Pages 2 0 R >>'),
    _PdfObject('<< /Type /Pages /Kids [3 0 R] /Count 1 >>'),
    _PdfObject(
      '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 $mediaWidth $mediaHeight] '
      '/Resources << /XObject << /Im1 4 0 R >> >> /Contents 5 0 R >>',
    ),
    _PdfObject(
      '<< /Type /XObject /Subtype /Image /Width $width /Height $height '
      '/ColorSpace /DeviceGray /BitsPerComponent 8 /Length ${pixels.length} >>',
      stream: pixels,
    ),
    _PdfObject('<< /Length ${content.length} >>', stream: content),
  ];
  return _serializePdf(objects);
}

Uint8List _mixedRotationPdf() {
  final objects = <_PdfObject>[
    _PdfObject('<< /Type /Catalog /Pages 2 0 R >>'),
    _PdfObject('<< /Type /Pages /Kids [4 0 R 5 0 R 6 0 R 7 0 R] /Count 4 >>'),
    _PdfObject('<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>'),
    for (var rotation = 0; rotation < 360; rotation += 90)
      _PdfObject(
        '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
        '/Rotate $rotation /Resources << /Font << /F1 3 0 R >> >> '
        '/Contents ${8 + rotation ~/ 90} 0 R >>',
      ),
    for (var page = 1; page <= 4; page++)
      () {
        final content = Uint8List.fromList(
          utf8.encode('BT /F1 14 Tf 40 740 Td (rotation $page) Tj ET'),
        );
        return _PdfObject('<< /Length ${content.length} >>', stream: content);
      }(),
  ];
  return _serializePdf(objects);
}

Uint8List _serializePdf(List<_PdfObject> objects) {
  final bytes = BytesBuilder(copy: false)..add(utf8.encode('%PDF-1.4\n'));
  final offsets = <int>[0];
  for (var index = 0; index < objects.length; index++) {
    offsets.add(bytes.length);
    final object = objects[index];
    bytes.add(utf8.encode('${index + 1} 0 obj\n${object.dictionary}\n'));
    if (object.stream != null) {
      bytes.add(utf8.encode('stream\n'));
      bytes.add(object.stream!);
      bytes.add(utf8.encode('\nendstream\n'));
    }
    bytes.add(utf8.encode('endobj\n'));
  }
  final xref = bytes.length;
  bytes.add(utf8.encode('xref\n0 ${objects.length + 1}\n'));
  bytes.add(utf8.encode('0000000000 65535 f \n'));
  for (final offset in offsets.skip(1)) {
    bytes.add(utf8.encode('${offset.toString().padLeft(10, '0')} 00000 n \n'));
  }
  bytes.add(
    utf8.encode(
      'trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\n'
      'startxref\n$xref\n%%EOF\n',
    ),
  );
  return bytes.takeBytes();
}

final class _PdfObject {
  const _PdfObject(this.dictionary, {this.stream});

  final String dictionary;
  final Uint8List? stream;
}

final class _CorpusCase {
  const _CorpusCase({
    required this.file,
    required this.bytes,
    required this.sha256,
    required this.expectations,
  });

  final String file;
  final int bytes;
  final String sha256;
  final Map<String, String> expectations;
}
