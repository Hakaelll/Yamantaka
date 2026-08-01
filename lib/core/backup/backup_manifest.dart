import 'dart:convert';

final class BackupManifest {
  const BackupManifest({
    required this.format,
    required this.version,
    required this.schemaVersion,
    required this.exportedAt,
    required this.categories,
    required this.containsPdfFiles,
  });

  final String format;
  final int version;
  final int schemaVersion;
  final DateTime exportedAt;
  final List<BackupCategory> categories;
  final bool containsPdfFiles;

  Map<String, Object?> toJson() => {
    'format': format,
    'version': version,
    'schemaVersion': schemaVersion,
    'exportedAt': exportedAt.toUtc().toIso8601String(),
    'containsPdfFiles': containsPdfFiles,
    'categories': [for (final category in categories) category.toJson()],
  };

  String encode() => jsonEncode(toJson());

  static BackupManifest fromJson(Map<String, dynamic> json) {
    final categories = json['categories'];
    if (json['format'] is! String ||
        json['version'] is! num ||
        json['schemaVersion'] is! num ||
        (json['schemaVersion'] as num).toInt() < 0 ||
        DateTime.tryParse(json['exportedAt']?.toString() ?? '') == null ||
        json['containsPdfFiles'] != false ||
        categories is! List ||
        categories.length > 64) {
      throw const FormatException('Manifest de backup no válido.');
    }
    return BackupManifest(
      format: json['format'] as String,
      version: (json['version'] as num).toInt(),
      schemaVersion: (json['schemaVersion'] as num).toInt(),
      exportedAt: DateTime.parse(json['exportedAt'] as String),
      containsPdfFiles: false,
      categories: [
        for (final raw in categories)
          if (raw is Map<String, dynamic>)
            BackupCategory.fromJson(raw)
          else
            throw const FormatException('Categoria de backup no valida.'),
      ],
    );
  }
}

final class BackupCategory {
  const BackupCategory({
    required this.name,
    required this.rows,
    required this.sha256,
  });

  final String name;
  final int rows;
  final String sha256;

  Map<String, Object> toJson() => {
    'name': name,
    'rows': rows,
    'sha256': sha256,
  };

  static BackupCategory fromJson(Map<String, dynamic> json) {
    final name = json['name'];
    final rows = json['rows'];
    final hash = json['sha256'];
    if (name is! String ||
        name.isEmpty ||
        name.length > 64 ||
        rows is! num ||
        rows.toInt() < 0 ||
        rows.toInt() > 100000 ||
        hash is! String ||
        !RegExp(r'^[a-f0-9]{64}$').hasMatch(hash)) {
      throw const FormatException('Categoría de backup no válida.');
    }
    return BackupCategory(name: name, rows: rows.toInt(), sha256: hash);
  }
}
