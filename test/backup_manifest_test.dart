import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/backup/backup_manifest.dart';

void main() {
  test('backup manifest round trips without PDF payloads', () {
    const manifest = BackupManifest(
      format: 'yamantaka-local-backup',
      version: 3,
      schemaVersion: 3,
      exportedAt: DateTime.utc(2026, 1, 1),
      categories: [
        BackupCategory(
          name: 'documents',
          rows: 2,
          sha256:
              'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        ),
      ],
      containsPdfFiles: false,
    );
    final json = <String, dynamic>{
      ...manifest.toJson(),
      'exportedAt': '2026-01-01T00:00:00Z',
    };
    final decoded = BackupManifest.fromJson(json);
    expect(decoded.containsPdfFiles, isFalse);
    expect(decoded.categories.single.rows, 2);
  });

  test('rejects malformed category hashes', () {
    expect(
      () => BackupManifest.fromJson({
        'format': 'yamantaka-local-backup',
        'version': 3,
        'schemaVersion': 4,
        'exportedAt': '2026-01-01T00:00:00Z',
        'containsPdfFiles': false,
        'categories': [
          {'name': 'documents', 'rows': 1, 'sha256': 'abc'},
        ],
      }),
      throwsFormatException,
    );
  });
}
