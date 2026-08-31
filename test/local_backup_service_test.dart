import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/domain/document_enrichment.dart';
import 'package:yamantaka/features/reader/domain/reader_models.dart';
import 'package:yamantaka/features/settings/application/settings_controller.dart';
import 'package:yamantaka/storage/backup/local_backup_service.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/paths/app_paths.dart';
import 'package:yamantaka/storage/repositories/app_settings_repository.dart';
import 'package:yamantaka/storage/repositories/bookmark_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/reading_session_repository.dart';
import 'package:yamantaka/storage/repositories/preferences_repository.dart';

void main() {
  test(
    'versioned local backup validates, previews and round-trips without PDFs',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'yamantaka-backup-test-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final logger = AppLogger(minimumLevel: LogLevel.error);
      final database = AppDatabase.inMemory(logger);
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      final bookmarks = BookmarkRepository(database);
      final sessions = ReadingSessionRepository(database);
      final preferences = DocumentPreferencesRepository(database);
      final settings = SettingsController(
        AppSettingsRepository(_MemoryPreferencesStore()),
        logger,
      );
      await settings.load();
      final paths = AppPaths(
        dataDirectory: directory,
        cacheDirectory: directory,
        thumbnailDirectory: Directory('${directory.path}/covers'),
      );
      final service = LocalBackupService(
        database: database,
        paths: paths,
        settings: settings,
      );
      final now = DateTime.utc(2026, 7, 29, 10);
      documents.save(
        LibraryDocument(
          id: 'doc-1',
          path: r'C:\Books\book.pdf',
          normalizedPath: r'c:\books\book.pdf',
          fileName: 'book.pdf',
          title: 'Book',
          pageCount: 12,
          fileSize: 2048,
          fileModifiedAt: now,
          addedAt: now,
          lastOpenedAt: now,
          readingProgress: 0.5,
          libraryNote: 'Restore this note',
          fingerprint: 'fingerprint',
        ),
      );
      bookmarks.add('doc-1', 4, label: 'Capítulo');
      documents.setManualTopics('doc-1', {
        LibraryTopic.history,
        LibraryTopic.politics,
      }, analyzerVersion: 1);
      preferences.save(
        const DocumentPreferences(
          documentId: 'doc-1',
          viewMode: ReaderMode.book,
          pageLayout: PageLayoutMode.facing,
          coverPolicy: ReaderCoverPolicy.firstPageIsCover,
          initialSide: ReaderInitialSide.right,
        ),
      );
      final session = sessions.start('doc-1', 1);
      sessions.finish(id: session.id, endPage: 4, activeSeconds: 90);
      await settings.update(
        settings.settings.copyWith(
          darkMode: true,
          readerPanelWidth: 416,
          shortcutOverrides: '{"search":"ctrl-shift-f"}',
        ),
      );

      final backup = File('${directory.path}/backup.json');
      await service.exportTo(backup.path);
      final exported = await backup.readAsString();
      expect((jsonDecode(exported) as Map<String, dynamic>)['version'], 8);
      expect(exported, contains('"containsPdfFiles": false'));
      expect(exported, isNot(contains('%PDF-')));
      final preview = await service.preview(backup.path);
      expect(preview.documentCount, 1);
      expect(preview.bookmarkCount, 1);
      expect(preview.sessionCount, 1);

      documents.remove('doc-1');
      await settings.clear();
      final safetyPath = await service.importFrom(backup.path);

      expect(File(safetyPath).existsSync(), isTrue);
      expect(documents.byId('doc-1')?.readingProgress, 0.5);
      expect(documents.byId('doc-1')?.libraryNote, 'Restore this note');
      expect(
        documents.byId('doc-1')?.topics.map((topic) => topic.topic).toSet(),
        {LibraryTopic.history, LibraryTopic.politics},
      );
      expect(documents.byId('doc-1')?.enrichment?.topicsManual, isTrue);
      expect(bookmarks.forDocument('doc-1').single.label, 'Capítulo');
      expect(
        preferences.load('doc-1').coverPolicy,
        ReaderCoverPolicy.firstPageIsCover,
      );
      expect(preferences.load('doc-1').initialSide, ReaderInitialSide.right);
      expect(sessions.history(), hasLength(1));
      expect(settings.settings.darkMode, isTrue);
      expect(settings.settings.readerPanelWidth, 416);
    },
  );

  test('invalid backup is rejected before replacing current data', () async {
    final directory = await Directory.systemTemp.createTemp(
      'yamantaka-invalid-backup-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final logger = AppLogger(minimumLevel: LogLevel.error);
    final database = AppDatabase.inMemory(logger);
    addTearDown(database.close);
    final documents = DocumentRepository(database);
    final now = DateTime.utc(2026, 7, 29);
    documents.save(
      LibraryDocument(
        id: 'safe',
        path: 'safe.pdf',
        normalizedPath: 'safe.pdf',
        fileName: 'safe.pdf',
        pageCount: 1,
        fileSize: 1,
        fileModifiedAt: now,
        addedAt: now,
        lastOpenedAt: now,
        fingerprint: 'safe',
      ),
    );
    final settings = SettingsController(
      AppSettingsRepository(_MemoryPreferencesStore()),
      logger,
    );
    await settings.load();
    final service = LocalBackupService(
      database: database,
      paths: AppPaths(
        dataDirectory: directory,
        cacheDirectory: directory,
        thumbnailDirectory: directory,
      ),
      settings: settings,
    );
    final invalid = File('${directory.path}/invalid.json');
    await invalid.writeAsString('{"format":"other","version":999}');

    await expectLater(service.importFrom(invalid.path), throwsFormatException);
    expect(documents.byId('safe'), isNotNull);
  });
}

final class _MemoryPreferencesStore implements AppPreferencesStore {
  final Map<String, Object> _values = {};

  @override
  Future<bool?> getBool(String key) async => _values[key] as bool?;

  @override
  Future<double?> getDouble(String key) async => _values[key] as double?;

  @override
  Future<int?> getInt(String key) async => _values[key] as int?;

  @override
  Future<String?> getString(String key) async => _values[key] as String?;

  @override
  Future<void> setBool(String key, bool value) async => _values[key] = value;

  @override
  Future<void> setDouble(String key, double value) async =>
      _values[key] = value;

  @override
  Future<void> setInt(String key, int value) async => _values[key] = value;

  @override
  Future<void> setString(String key, String value) async =>
      _values[key] = value;

  @override
  Future<void> clear() async => _values.clear();
}
