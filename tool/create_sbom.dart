import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ffi/ffi.dart';

Future<void> main(List<String> arguments) async {
  if (arguments.length != 3) {
    stderr.writeln(
      'Usage: dart run tool/create_sbom.dart <bundle> <output> <version>',
    );
    exitCode = 64;
    return;
  }
  final bundle = Directory(arguments[0]);
  final output = File(arguments[1]);
  final productVersion = arguments[2];
  if (!bundle.existsSync()) {
    throw ArgumentError.value(bundle.path, 'bundle', 'Directory not found.');
  }
  final packages = _parsePubLock(File('pubspec.lock').readAsLinesSync());
  final flutterVersion = _flutterVersion();
  final packageRoots = _packageRoots();
  final packageEvidence = <String, _PackageEvidence>{};
  for (final package in packages) {
    final resolvedVersion = package.source == 'sdk'
        ? flutterVersion
        : package.version;
    final digest =
        package.sha256 ??
        await _directoryDigest(packageRoots[package.name], package.name);
    packageEvidence[package.name] = _PackageEvidence(
      version: resolvedVersion,
      sha256: digest,
    );
  }
  final packageVersions = {
    for (final package in packages)
      package.name: packageEvidence[package.name]!.version,
  };
  final licenses = _packageLicenses();
  final nativeFiles =
      bundle
          .listSync(recursive: true, followLinks: false)
          .whereType<File>()
          .where((file) {
            final extension = file.path.split('.').last.toLowerCase();
            return extension == 'exe' || extension == 'dll';
          })
          .toList(growable: false)
        ..sort((left, right) => left.path.compareTo(right.path));
  final nativeComponents = <Map<String, Object?>>[];
  for (final file in nativeFiles) {
    final relative = file.path
        .substring(bundle.path.length + 1)
        .replaceAll('\\', '/');
    final evidence = _nativeEvidence(
      file: file,
      relativePath: relative,
      productVersion: productVersion,
      flutterVersion: flutterVersion,
      packageVersions: packageVersions,
    );
    nativeComponents.add({
      'type': 'file',
      'name': relative,
      'version': evidence.version,
      'bom-ref': 'file:$relative',
      'hashes': [
        {
          'alg': 'SHA-256',
          'content': await sha256.bind(file.openRead()).first.then((v) => '$v'),
        },
      ],
      'licenses': [
        {
          'license': evidence.spdxId == null
              ? {'name': evidence.licenseName}
              : {'id': evidence.spdxId},
        },
      ],
      'properties': [
        {'name': 'yamantaka:source', 'value': evidence.source},
        {'name': 'yamantaka:provenance', 'value': evidence.provenance},
        {'name': 'yamantaka:signatureExpected', 'value': 'false'},
        {'name': 'yamantaka:distribution', 'value': 'local-only'},
      ],
    });
  }
  final payload = <String, Object?>{
    'bomFormat': 'CycloneDX',
    'specVersion': '1.5',
    'serialNumber': 'urn:uuid:${_stableUuid(productVersion, packages)}',
    'version': 1,
    'metadata': {
      'timestamp': DateTime.now().toUtc().toIso8601String(),
      'tools': [
        {
          'vendor': 'Yamantaka',
          'name': 'tool/create_sbom.dart',
          'version': '1',
        },
      ],
      'component': {
        'type': 'application',
        'name': 'yamantaka',
        'version': productVersion,
        'bom-ref': 'pkg:generic/yamantaka@$productVersion',
      },
      'properties': [
        {'name': 'yamantaka:containsPdfFiles', 'value': 'false'},
        {'name': 'yamantaka:networkRequiredAtRuntime', 'value': 'false'},
      ],
    },
    'components': [
      for (final package in packages)
        {
          'type': 'library',
          'name': package.name,
          'version': packageEvidence[package.name]!.version,
          'bom-ref':
              'pkg:pub/${package.name}@${packageEvidence[package.name]!.version}',
          'purl':
              'pkg:pub/${package.name}@${packageEvidence[package.name]!.version}',
          'hashes': [
            {
              'alg': 'SHA-256',
              'content': packageEvidence[package.name]!.sha256,
            },
          ],
          'scope': package.direct ? 'required' : 'optional',
          'licenses': [
            {
              'license': licenses[package.name]?.spdxId != null
                  ? {'id': licenses[package.name]!.spdxId}
                  : {'name': licenses[package.name]?.name ?? 'NOASSERTION'},
            },
          ],
          'properties': [
            {'name': 'yamantaka:source', 'value': package.source},
            {
              'name': 'yamantaka:licenseEvidence',
              'value': licenses[package.name] == null
                  ? 'not found'
                  : 'resolved package cache',
            },
            if (licenses[package.name]?.sha256 case final licenseHash?)
              {'name': 'yamantaka:licenseSha256', 'value': licenseHash},
          ],
        },
      ...nativeComponents,
    ],
  };
  output.parent.createSync(recursive: true);
  output.writeAsStringSync(
    const JsonEncoder.withIndent('  ').convert(payload),
    flush: true,
  );
}

String _flutterVersion() {
  final file = File('.flutter-sdk/bin/cache/flutter.version.json');
  if (!file.existsSync()) return 'unknown-local-sdk';
  final decoded = jsonDecode(file.readAsStringSync());
  if (decoded is! Map) return 'unknown-local-sdk';
  return decoded['frameworkVersion'] as String? ?? 'unknown-local-sdk';
}

Map<String, Directory> _packageRoots() {
  final configFile = File('.dart_tool/package_config.json');
  if (!configFile.existsSync()) return const {};
  final decoded = jsonDecode(configFile.readAsStringSync());
  if (decoded is! Map<String, Object?> || decoded['packages'] is! List) {
    return const {};
  }
  final base = configFile.parent.uri;
  return {
    for (final raw in decoded['packages']! as List)
      if (raw is Map && raw['name'] is String && raw['rootUri'] is String)
        raw['name']! as String: Directory.fromUri(
          base.resolve(raw['rootUri']! as String),
        ),
  };
}

Future<String> _directoryDigest(Directory? root, String packageName) async {
  if (root == null || !root.existsSync()) {
    throw StateError('Package source is unavailable for $packageName.');
  }
  final files =
      root
          .listSync(recursive: true, followLinks: false)
          .whereType<File>()
          .toList(growable: false)
        ..sort((left, right) => left.path.compareTo(right.path));
  final manifest = StringBuffer();
  for (final file in files) {
    final relative = file.path
        .substring(root.path.length + 1)
        .replaceAll('\\', '/');
    final digest = await sha256.bind(file.openRead()).first;
    manifest.writeln('$relative:$digest');
  }
  return sha256.convert(utf8.encode(manifest.toString())).toString();
}

_NativeEvidence _nativeEvidence({
  required File file,
  required String relativePath,
  required String productVersion,
  required String flutterVersion,
  required Map<String, String> packageVersions,
}) {
  final name = relativePath.split('/').last.toLowerCase();
  String packageVersion(String package) =>
      packageVersions[package] ?? 'unknown-locked-version';
  switch (name) {
    case 'yamantaka.exe':
      return _NativeEvidence(
        version: productVersion,
        licenseName: 'LicenseRef-Yamantaka-Private',
        source: 'yamantaka@$productVersion',
        provenance: 'repository source and Windows release runner',
      );
    case 'flutter_windows.dll':
      return _NativeEvidence(
        version: flutterVersion,
        spdxId: 'BSD-3-Clause',
        source: 'flutter@$flutterVersion',
        provenance: 'bundled Flutter Windows engine from the pinned local SDK',
      );
    case 'pdfium.dll':
      return _NativeEvidence(
        version: _windowsFileVersion(file) ?? 'chromium-7811',
        spdxId: 'BSD-3-Clause',
        source: 'pdfium_dart@${packageVersion('pdfium_dart')}',
        provenance:
            'pdfium_dart native hook; bblanchon pdfium-binaries chromium/7811',
      );
    case 'sqlite3.dll':
      return _NativeEvidence(
        version: _sqliteVersion(file),
        licenseName: 'Public Domain (SQLite blessing)',
        source: 'sqlite3@${packageVersion('sqlite3')}',
        provenance: 'sqlite3.dart native hook precompiled Windows x64 library',
      );
    case 'desktop_drop_plugin.dll':
      return _pluginEvidence(
        package: 'desktop_drop',
        version: packageVersion('desktop_drop'),
        spdxId: 'Apache-2.0',
      );
    case 'file_selector_windows_plugin.dll':
      return _pluginEvidence(
        package: 'file_selector_windows',
        version: packageVersion('file_selector_windows'),
        spdxId: 'BSD-3-Clause',
      );
    case 'screen_retriever_windows_plugin.dll':
      return _pluginEvidence(
        package: 'screen_retriever_windows',
        version: packageVersion('screen_retriever_windows'),
        spdxId: 'MIT',
      );
    case 'url_launcher_windows_plugin.dll':
      return _pluginEvidence(
        package: 'url_launcher_windows',
        version: packageVersion('url_launcher_windows'),
        spdxId: 'BSD-3-Clause',
      );
    case 'window_manager_plugin.dll':
      return _pluginEvidence(
        package: 'window_manager',
        version: packageVersion('window_manager'),
        spdxId: 'MIT',
      );
    default:
      throw StateError(
        'Native SBOM evidence is not defined for $relativePath.',
      );
  }
}

_NativeEvidence _pluginEvidence({
  required String package,
  required String version,
  required String spdxId,
}) => _NativeEvidence(
  version: version,
  spdxId: spdxId,
  source: '$package@$version',
  provenance: 'Windows plugin compiled from the locked pub package',
);

String _sqliteVersion(File file) {
  final library = DynamicLibrary.open(file.absolute.path);
  final version = library
      .lookupFunction<Pointer<Utf8> Function(), Pointer<Utf8> Function()>(
        'sqlite3_libversion',
      )
      .call();
  return version.toDartString();
}

String? _windowsFileVersion(File file) {
  final name = file.uri.pathSegments.last.toLowerCase();
  if (name != 'pdfium.dll') return null;
  // The shipped PDFium binary exposes Chromium milestone/build in its file
  // version; keep the known pinned build explicit when Dart cannot read Win32
  // version resources without another native dependency.
  return '149.0.7811.0';
}

Map<String, _LicenseEvidence> _packageLicenses() {
  final configFile = File('.dart_tool/package_config.json');
  if (!configFile.existsSync()) return const {};
  final decoded = jsonDecode(configFile.readAsStringSync());
  if (decoded is! Map<String, Object?> || decoded['packages'] is! List) {
    return const {};
  }
  final base = configFile.parent.uri;
  final result = <String, _LicenseEvidence>{};
  for (final raw in decoded['packages']! as List) {
    if (raw is! Map) continue;
    final package = raw.cast<String, Object?>();
    final name = package['name'] as String?;
    final rootUri = package['rootUri'] as String?;
    if (name == null || rootUri == null) continue;
    final root = Directory.fromUri(base.resolve(rootUri));
    if (!root.existsSync()) continue;
    var licenseFiles = _licenseFiles(root);
    if (licenseFiles.isEmpty && root.path.contains('.flutter-sdk')) {
      var cursor = root;
      while (cursor.parent.path != cursor.path &&
          cursor.uri.pathSegments.where((item) => item.isNotEmpty).last !=
              '.flutter-sdk') {
        cursor = cursor.parent;
      }
      licenseFiles = _licenseFiles(cursor);
    }
    if (licenseFiles.isEmpty) continue;
    final bytes = licenseFiles.first.readAsBytesSync();
    final text = utf8.decode(bytes, allowMalformed: true);
    final hash = sha256.convert(bytes).toString();
    final spdx = _inferSpdx(text);
    result[name] = _LicenseEvidence(
      spdxId: spdx,
      name: spdx ?? 'LicenseRef-${hash.substring(0, 12)}',
      sha256: hash,
    );
  }
  return result;
}

List<File> _licenseFiles(Directory root) =>
    root
        .listSync(followLinks: false)
        .whereType<File>()
        .where((file) {
          final name = file.uri.pathSegments.last.toLowerCase();
          return name.startsWith('license') || name.startsWith('copying');
        })
        .toList(growable: false)
      ..sort((left, right) => left.path.compareTo(right.path));

String? _inferSpdx(String text) {
  final normalized = text.toLowerCase();
  if (normalized.contains('apache license') &&
      normalized.contains('version 2.0')) {
    return 'Apache-2.0';
  }
  if (normalized.contains('permission is hereby granted, free of charge')) {
    return 'MIT';
  }
  if (normalized.contains(
    'redistribution and use in source and binary forms',
  )) {
    return normalized.contains('neither the name')
        ? 'BSD-3-Clause'
        : 'BSD-2-Clause';
  }
  if (normalized.contains('mozilla public license') &&
      normalized.contains('version 2.0')) {
    return 'MPL-2.0';
  }
  return null;
}

List<_LockedPackage> _parsePubLock(List<String> lines) {
  final packages = <_LockedPackage>[];
  String? name;
  String? version;
  String? source;
  String? dependency;
  String? digest;
  void finish() {
    if (name != null && version != null && source != null) {
      packages.add(
        _LockedPackage(
          name: name,
          version: version,
          source: source,
          sha256: digest,
          direct: dependency?.startsWith('direct ') == true,
        ),
      );
    }
  }

  for (final line in lines.skipWhile((line) => line.trim() != 'packages:')) {
    final packageMatch = RegExp(r'^  ([A-Za-z0-9_]+):$').firstMatch(line);
    if (packageMatch != null) {
      finish();
      name = packageMatch.group(1);
      version = null;
      source = null;
      dependency = null;
      digest = null;
      continue;
    }
    if (name == null) continue;
    final value = RegExp(
      r'^\s{4,}([a-zA-Z0-9_]+):\s+"?([^"\r\n]+?)"?\s*$',
    ).firstMatch(line);
    if (value == null) continue;
    switch (value.group(1)) {
      case 'version':
        version = value.group(2);
      case 'source':
        source = value.group(2);
      case 'dependency':
        dependency = value.group(2)?.trim();
      case 'sha256':
        digest = value.group(2);
    }
  }
  finish();
  packages.sort((left, right) => left.name.compareTo(right.name));
  if (packages.isEmpty) throw const FormatException('No packages in lockfile.');
  return packages;
}

String _stableUuid(String version, List<_LockedPackage> packages) {
  final digest = sha256
      .convert(
        utf8.encode(
          '$version:${packages.map((item) => '${item.name}@${item.version}').join(',')}',
        ),
      )
      .toString();
  return '${digest.substring(0, 8)}-${digest.substring(8, 12)}-'
      '5${digest.substring(13, 16)}-a${digest.substring(17, 20)}-'
      '${digest.substring(20, 32)}';
}

final class _LockedPackage {
  const _LockedPackage({
    required this.name,
    required this.version,
    required this.source,
    required this.sha256,
    required this.direct,
  });

  final String name;
  final String version;
  final String source;
  final String? sha256;
  final bool direct;
}

final class _PackageEvidence {
  const _PackageEvidence({required this.version, required this.sha256});

  final String version;
  final String sha256;
}

final class _LicenseEvidence {
  const _LicenseEvidence({
    required this.spdxId,
    required this.name,
    required this.sha256,
  });

  final String? spdxId;
  final String name;
  final String sha256;
}

final class _NativeEvidence {
  const _NativeEvidence({
    required this.version,
    required this.source,
    required this.provenance,
    this.spdxId,
    this.licenseName,
  }) : assert(spdxId != null || licenseName != null);

  final String version;
  final String source;
  final String provenance;
  final String? spdxId;
  final String? licenseName;
}
