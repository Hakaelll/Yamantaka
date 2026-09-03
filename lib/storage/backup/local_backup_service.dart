// ignore_for_file: prefer_initializing_formals

import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';

import '../../app/app_config.dart';
import '../../core/backup/backup_manifest.dart';
import '../../app/theme/app_density.dart';
import '../../app/theme/app_tokens.dart';
import '../../features/reader/domain/reader_models.dart';
import '../../features/settings/application/settings_controller.dart';
import '../database/app_database.dart';
import '../paths/app_paths.dart';
import '../repositories/app_settings_repository.dart';

final class BackupPreview {
  const BackupPreview({
    required this.version,
    required this.exportedAt,
    required this.documentCount,
    required this.bookmarkCount,
    required this.sessionCount,
    required this.annotationCount,
    required this.schemaVersion,
    required this.fileBytes,
    required this.entityCount,
    required this.documentAdds,
    required this.documentReplacements,
    required this.documentRemovals,
    required this.identityConflicts,
    required this.tableCounts,
  });

  final int version;
  final DateTime exportedAt;
  final int documentCount;
  final int bookmarkCount;
  final int sessionCount;
  final int annotationCount;
  final int schemaVersion;
  final int fileBytes;
  final int entityCount;
  final int documentAdds;
  final int documentReplacements;
  final int documentRemovals;
  final int identityConflicts;
  final Map<String, int> tableCounts;

  bool get replacesExistingData =>
      documentReplacements > 0 || documentRemovals > 0;
}

abstract final class BackupSecurityPolicy {
  static const bool includesPdfFiles = false;
  static const bool usesApplicationEncryption = false;
  static const String encryptionDecision =
      'No custom encryption: use an OS-encrypted volume or encrypted archive.';
}

final class LocalBackupService {
  const LocalBackupService({
    required AppDatabase database,
    required AppPaths paths,
    required SettingsController settings,
  }) : _database = database,
       _paths = paths,
       _settings = settings;

  static const format = 'yamantaka-local-backup';
  static const version = 9;
  static const maxBackupBytes = 128 * 1024 * 1024;

  final AppDatabase _database;
  final AppPaths _paths;
  final SettingsController _settings;

  Future<void> exportTo(String path) async {
    final payload = _snapshot();
    final destination = File(path);
    final partial = File('$path.part');
    final previous = File('$path.previous');
    var movedPrevious = false;
    var installed = false;
    try {
      await partial.writeAsString(
        const JsonEncoder.withIndent('  ').convert(payload),
        flush: true,
      );
      if (await previous.exists()) await previous.delete();
      if (await destination.exists()) {
        await destination.rename(previous.path);
        movedPrevious = true;
      }
      await partial.rename(destination.path);
      installed = true;
      if (movedPrevious && await previous.exists()) {
        await previous.delete();
      }
    } on Object {
      if (await partial.exists()) await partial.delete();
      if (!installed &&
          movedPrevious &&
          !await destination.exists() &&
          await previous.exists()) {
        await previous.rename(destination.path);
      }
      rethrow;
    }
  }

  Future<BackupPreview> preview(String path) async {
    final payload = await _readAndValidate(path);
    final tables = payload['tables']! as Map<String, dynamic>;
    final incomingDocuments = (tables['documents']! as List)
        .cast<Map<String, dynamic>>();
    final currentDocuments = _database.database.select(
      'SELECT id, fingerprint FROM documents',
    );
    final incomingById = {
      for (final row in incomingDocuments) row['id'] as String: row,
    };
    final currentById = {
      for (final row in currentDocuments)
        row['id'] as String: row['fingerprint'] as String,
    };
    final incomingIds = incomingById.keys.toSet();
    final currentIds = currentById.keys.toSet();
    final sharedIds = incomingIds.intersection(currentIds);
    final tableCounts = <String, int>{
      for (final entry in tables.entries)
        entry.key: (entry.value as List).length,
    };
    final manifest = payload['manifest'] as Map<String, dynamic>?;
    return BackupPreview(
      version: payload['version']! as int,
      exportedAt: DateTime.parse(payload['exportedAt']! as String),
      documentCount: (tables['documents']! as List).length,
      bookmarkCount: (tables['bookmarks']! as List).length,
      sessionCount: (tables['reading_sessions']! as List).length,
      annotationCount: (tables['annotations']! as List).length,
      schemaVersion: manifest?['schemaVersion'] as int? ?? 1,
      fileBytes: await File(path).length(),
      entityCount: tableCounts.values.fold(0, (total, count) => total + count),
      documentAdds: incomingIds.difference(currentIds).length,
      documentReplacements: sharedIds.length,
      documentRemovals: currentIds.difference(incomingIds).length,
      identityConflicts: sharedIds
          .where(
            (id) =>
                incomingById[id]!['fingerprint'] as String != currentById[id],
          )
          .length,
      tableCounts: Map.unmodifiable(tableCounts),
    );
  }

  Future<String> importFrom(String path) async {
    final payload = await _readAndValidate(path);
    final backups = Directory('${_paths.dataDirectory.path}/backups');
    await backups.create(recursive: true);
    final stamp = DateTime.now().toUtc().toIso8601String().replaceAll(':', '-');
    final safetyPath = '${backups.path}/pre-import-$stamp.json';
    await exportTo(safetyPath);

    final tables = payload['tables']! as Map<String, dynamic>;
    final previousSettings = _settings.settings;
    final database = _database.database;
    var committed = false;
    try {
      database.execute('BEGIN IMMEDIATE');
      try {
        _replaceTables(tables);
        database.execute('COMMIT');
        committed = true;
      } catch (_) {
        database.execute('ROLLBACK');
        rethrow;
      }

      final rawSettings = payload['settings'];
      if (rawSettings is Map<String, dynamic>) {
        await _settings.updateStrict(_settingsFromMap(rawSettings));
      }
    } catch (_) {
      if (committed) await _restoreSafetyDatabase(safetyPath);
      try {
        await _settings.updateStrict(previousSettings);
      } on Object {
        // Preserve the original import error; the next startup can use the
        // pre-import file if the preference store is unavailable.
      }
      rethrow;
    }
    return safetyPath;
  }

  void _replaceTables(Map<String, dynamic> tables) {
    final database = _database.database;
    database
      ..execute('DELETE FROM duplicate_merge_history')
      ..execute('DELETE FROM operation_journal')
      ..execute('DELETE FROM automation_rules')
      ..execute('DELETE FROM document_revision_members')
      ..execute('DELETE FROM document_revision_groups')
      ..execute('DELETE FROM document_relations')
      ..execute('DELETE FROM workspace_tabs')
      ..execute('DELETE FROM workspaces')
      ..execute('DELETE FROM collection_documents')
      ..execute('DELETE FROM collections')
      ..execute('DELETE FROM document_topics')
      ..execute('DELETE FROM document_enrichment')
      ..execute('DELETE FROM document_tags')
      ..execute('DELETE FROM annotation_tags')
      ..execute('DELETE FROM tags')
      ..execute('DELETE FROM annotations')
      ..execute('DELETE FROM reading_profiles')
      ..execute('DELETE FROM library_search')
      ..execute('DELETE FROM search_index_state')
      ..execute('DELETE FROM reading_sessions')
      ..execute('DELETE FROM bookmarks')
      ..execute('DELETE FROM document_preferences')
      ..execute('DELETE FROM document_notes')
      ..execute('DELETE FROM documents')
      ..execute('DELETE FROM app_metadata');
    _restoreTable('documents', _documentColumns, tables['documents']! as List);
    _restoreTable(
      'document_enrichment',
      _enrichmentColumns,
      tables['document_enrichment']! as List,
    );
    _restoreTable(
      'document_topics',
      _topicColumns,
      tables['document_topics']! as List,
    );
    _restoreTable(
      'document_preferences',
      _preferenceColumns,
      tables['document_preferences']! as List,
    );
    _restoreTable(
      'document_notes',
      _noteColumns,
      tables['document_notes']! as List,
    );
    _restoreTable('bookmarks', _bookmarkColumns, tables['bookmarks']! as List);
    _restoreTable(
      'reading_sessions',
      _sessionColumns,
      tables['reading_sessions']! as List,
    );
    _restoreTable(
      'annotations',
      _annotationColumns,
      tables['annotations']! as List,
    );
    _restoreTable('tags', _tagColumns, tables['tags']! as List);
    _restoreTable(
      'annotation_tags',
      _annotationTagColumns,
      tables['annotation_tags']! as List,
    );
    _restoreTable(
      'document_tags',
      _documentTagColumns,
      tables['document_tags']! as List,
    );
    _restoreTable(
      'collections',
      _collectionColumns,
      tables['collections']! as List,
    );
    _restoreTable(
      'collection_documents',
      _collectionDocumentColumns,
      tables['collection_documents']! as List,
    );
    _restoreTable(
      'reading_profiles',
      _readingProfileColumns,
      tables['reading_profiles']! as List,
    );
    _restoreTable(
      'workspaces',
      _workspaceColumns,
      tables['workspaces']! as List,
    );
    _restoreTable(
      'workspace_tabs',
      _workspaceTabColumns,
      tables['workspace_tabs']! as List,
    );
    _restoreTable(
      'document_relations',
      _relationColumns,
      tables['document_relations']! as List,
    );
    _restoreTable(
      'document_revision_groups',
      _revisionGroupColumns,
      tables['document_revision_groups']! as List,
    );
    _restoreTable(
      'document_revision_members',
      _revisionMemberColumns,
      tables['document_revision_members']! as List,
    );
    _restoreTable(
      'automation_rules',
      _automationRuleColumns,
      tables['automation_rules']! as List,
    );
    _restoreTable(
      'operation_journal',
      _journalColumns,
      tables['operation_journal']! as List,
    );
    _restoreTable(
      'duplicate_merge_history',
      _duplicateMergeHistoryColumns,
      tables['duplicate_merge_history']! as List,
    );
    _restoreTable(
      'app_metadata',
      _metadataColumns,
      tables['app_metadata']! as List,
    );
  }

  Future<void> _restoreSafetyDatabase(String safetyPath) async {
    try {
      final safety = await _readAndValidate(safetyPath);
      final tables = safety['tables']! as Map<String, dynamic>;
      final database = _database.database;
      database.execute('BEGIN IMMEDIATE');
      try {
        _replaceTables(tables);
        database.execute('COMMIT');
      } catch (_) {
        database.execute('ROLLBACK');
        rethrow;
      }
    } on Object {
      // The original import error remains the actionable one. The pre-import
      // file is deliberately retained for manual recovery.
    }
  }

  Map<String, dynamic> _snapshot() {
    final tables = <String, List<Map<String, Object?>>>{
      'documents': _tableRows('documents'),
      'document_enrichment': _tableRows('document_enrichment'),
      'document_topics': _tableRows('document_topics'),
      'document_preferences': _tableRows('document_preferences'),
      'document_notes': _tableRows('document_notes'),
      'bookmarks': _tableRows('bookmarks'),
      'reading_sessions': _tableRows('reading_sessions'),
      'annotations': _tableRows('annotations'),
      'tags': _tableRows('tags'),
      'annotation_tags': _tableRows('annotation_tags'),
      'document_tags': _tableRows('document_tags'),
      'collections': _tableRows('collections'),
      'collection_documents': _tableRows('collection_documents'),
      'reading_profiles': _tableRows('reading_profiles'),
      'workspaces': _tableRows('workspaces'),
      'workspace_tabs': _tableRows('workspace_tabs'),
      'document_relations': _tableRows('document_relations'),
      'document_revision_groups': _tableRows('document_revision_groups'),
      'document_revision_members': _tableRows('document_revision_members'),
      'automation_rules': _tableRows('automation_rules'),
      'operation_journal': _tableRows('operation_journal'),
      'duplicate_merge_history': _tableRows('duplicate_merge_history'),
      'app_metadata': _tableRows('app_metadata'),
    };
    final exportedAt = DateTime.now().toUtc();
    final settings = _settingsToMap(_settings.settings);
    final manifest = BackupManifest(
      format: format,
      version: version,
      schemaVersion: AppConfig.databaseVersion,
      exportedAt: exportedAt,
      containsPdfFiles: false,
      categories: [
        for (final entry in tables.entries)
          BackupCategory(
            name: entry.key,
            rows: entry.value.length,
            sha256: _hashRows(entry.value),
          ),
        BackupCategory(
          name: 'settings',
          rows: 1,
          sha256: _hashRows([settings]),
        ),
      ],
    );
    return {
      'format': format,
      'version': version,
      'exportedAt': exportedAt.toIso8601String(),
      'containsPdfFiles': false,
      'manifest': manifest.toJson(),
      'settings': settings,
      'tables': tables,
    };
  }

  static String _hashRows(List<Map<String, Object?>> rows) =>
      sha256.convert(utf8.encode(const JsonEncoder().convert(rows))).toString();

  List<Map<String, Object?>> _tableRows(String table) {
    final rows = _database.database.select('SELECT * FROM $table');
    return [
      for (final row in rows)
        {for (final column in rows.columnNames) column: row[column]},
    ];
  }

  Future<Map<String, dynamic>> _readAndValidate(String path) async {
    final file = File(path);
    if (!await file.exists()) {
      throw const FormatException('La copia seleccionada no existe.');
    }
    if (await file.length() > maxBackupBytes) {
      throw const FormatException('La copia supera el tamaño permitido.');
    }
    final decoded = jsonDecode(await file.readAsString());
    if (decoded is! Map<String, dynamic> ||
        decoded['format'] != format ||
        decoded['version'] is! int ||
        (decoded['version'] as int) < 1 ||
        (decoded['version'] as int) > version ||
        decoded['containsPdfFiles'] != false ||
        DateTime.tryParse(decoded['exportedAt']?.toString() ?? '') == null) {
      throw const FormatException(
        'La copia no pertenece a una versión compatible de Yamantaka.',
      );
    }
    final tables = decoded['tables'];
    if (tables is! Map<String, dynamic>) {
      throw const FormatException('La copia no contiene tablas válidas.');
    }
    if (decoded['version'] == 1) {
      for (final name in const [
        'annotations',
        'tags',
        'annotation_tags',
        'document_tags',
        'collections',
        'collection_documents',
        'reading_profiles',
        'workspaces',
        'workspace_tabs',
      ]) {
        tables.putIfAbsent(name, () => <Object?>[]);
      }
    }
    // Notes were introduced after the v1-v4 backup formats. Treat an absent
    // table as an empty table so older backups remain importable.
    tables.putIfAbsent('document_notes', () => <Object?>[]);
    tables.putIfAbsent('document_enrichment', () => <Object?>[]);
    tables.putIfAbsent('document_topics', () => <Object?>[]);
    for (final name in const [
      'document_relations',
      'document_revision_groups',
      'document_revision_members',
      'automation_rules',
      'operation_journal',
      'duplicate_merge_history',
    ]) {
      tables.putIfAbsent(name, () => <Object?>[]);
    }
    final backupVersion = decoded['version'] as int;
    _validateTable(
      tables,
      'documents',
      backupVersion >= 9
          ? _documentColumns
          : backupVersion >= 4
          ? _v8DocumentColumns
          : backupVersion >= 3
          ? _v3DocumentColumns
          : _legacyDocumentColumns,
    );
    _validateTable(
      tables,
      'document_preferences',
      backupVersion >= 6 ? _preferenceColumns : _legacyPreferenceColumns,
    );
    _validateTable(tables, 'document_enrichment', _enrichmentColumns);
    _validateTable(tables, 'document_topics', _topicColumns);
    _validateTable(tables, 'document_notes', _noteColumns);
    _validateTable(tables, 'bookmarks', _bookmarkColumns);
    _validateTable(tables, 'reading_sessions', _sessionColumns);
    _validateTable(
      tables,
      'annotations',
      backupVersion >= 9
          ? _annotationColumns
          : backupVersion >= 8
          ? _v8AnnotationColumns
          : _v7AnnotationColumns,
    );
    _validateTable(tables, 'tags', _tagColumns);
    _validateTable(tables, 'annotation_tags', _annotationTagColumns);
    _validateTable(tables, 'document_tags', _documentTagColumns);
    _validateTable(tables, 'collections', _collectionColumns);
    _validateTable(tables, 'collection_documents', _collectionDocumentColumns);
    _validateTable(tables, 'reading_profiles', _readingProfileColumns);
    _validateTable(
      tables,
      'workspaces',
      backupVersion >= 8 ? _workspaceColumns : _v7WorkspaceColumns,
    );
    _validateTable(
      tables,
      'workspace_tabs',
      backupVersion >= 8 ? _workspaceTabColumns : _v7WorkspaceTabColumns,
    );
    _validateTable(tables, 'document_relations', _relationColumns);
    _validateTable(tables, 'document_revision_groups', _revisionGroupColumns);
    _validateTable(tables, 'document_revision_members', _revisionMemberColumns);
    _validateTable(tables, 'automation_rules', _automationRuleColumns);
    _validateTable(
      tables,
      'operation_journal',
      backupVersion >= 9 ? _journalColumns : _v8JournalColumns,
    );
    _validateTable(
      tables,
      'duplicate_merge_history',
      _duplicateMergeHistoryColumns,
    );
    _validateTable(tables, 'app_metadata', _metadataColumns);
    if (decoded['settings'] is! Map<String, dynamic>) {
      throw const FormatException('La copia no contiene ajustes válidos.');
    }
    final rawManifest = decoded['manifest'];
    if (rawManifest != null) {
      if (rawManifest is! Map<String, dynamic>) {
        throw const FormatException('El manifest de la copia no es valido.');
      }
      final manifest = BackupManifest.fromJson(rawManifest);
      if (manifest.format != format ||
          manifest.version != decoded['version'] ||
          manifest.schemaVersion > AppConfig.databaseVersion ||
          manifest.containsPdfFiles) {
        throw const FormatException(
          'La copia requiere una version incompatible.',
        );
      }
      final expected = {
        for (final category in manifest.categories) category.name: category,
      };
      for (final entry in tables.entries) {
        final category = expected[entry.key];
        if (category == null) continue;
        final rows = entry.value as List;
        final normalized = [
          for (final raw in rows)
            if (raw is Map<String, dynamic>)
              <String, Object?>{
                for (final item in raw.entries) item.key: item.value,
              },
        ];
        if (category.rows != normalized.length ||
            _hashRows(normalized) != category.sha256) {
          throw FormatException(
            'La categoria ${entry.key} no coincide con su hash.',
          );
        }
      }
      final settings = decoded['settings'] as Map<String, dynamic>;
      final settingsCategory = expected['settings'];
      if (settingsCategory != null && settingsCategory.rows != 1) {
        throw const FormatException('La categoría de ajustes no es válida.');
      }
      if (settingsCategory != null &&
          _hashRows([
                <String, Object?>{
                  for (final entry in settings.entries) entry.key: entry.value,
                },
              ]) !=
              settingsCategory.sha256) {
        throw const FormatException('Los ajustes no coinciden con su hash.');
      }
    }
    return decoded;
  }

  void _validateTable(
    Map<String, dynamic> tables,
    String name,
    List<String> columns,
  ) {
    final rows = tables[name];
    if (rows is! List || rows.length > 100000) {
      throw FormatException('La tabla $name no es válida.');
    }
    for (final row in rows) {
      if (row is! Map<String, dynamic> ||
          columns.any((column) => !row.containsKey(column))) {
        throw FormatException('La tabla $name está incompleta.');
      }
      for (final value in row.values) {
        if (value != null &&
            value is! String &&
            value is! int &&
            value is! double) {
          throw FormatException('La tabla $name contiene datos no válidos.');
        }
      }
    }
  }

  void _restoreTable(String table, List<String> columns, List rows) {
    final placeholders = List.filled(columns.length, '?').join(', ');
    final sql =
        'INSERT INTO $table (${columns.join(', ')}) VALUES ($placeholders)';
    for (final raw in rows) {
      final row = raw as Map<String, dynamic>;
      _database.database.execute(sql, [
        for (final column in columns)
          row[column] ??
              (table == 'documents' && column == 'is_favorite'
                  ? 0
                  : table == 'documents' && column == 'restore_blocked'
                  ? 0
                  : table == 'annotations' && column == 'review_state'
                  ? 'pending'
                  : table == 'workspaces' && column == 'query_json'
                  ? '{}'
                  : table == 'workspaces' && column == 'grouping'
                  ? 'none'
                  : table == 'workspaces' && column == 'sort_mode'
                  ? 'lastOpened'
                  : table == 'workspaces' && column == 'density'
                  ? 'comfortable'
                  : table == 'workspaces' && column == 'visible_columns_json'
                  ? '["document","status","progress"]'
                  : table == 'workspaces' && column == 'focus_mode'
                  ? 0
                  : table == 'workspace_tabs' && column == 'queue_status'
                  ? 'pending'
                  : table == 'operation_journal' && column == 'phase'
                  ? 'planned'
                  : table == 'operation_journal' &&
                        column == 'resume_capability'
                  ? 'none'
                  : table == 'document_preferences' && column == 'cover_policy'
                  ? ReaderCoverPolicy.noCover.name
                  : table == 'document_preferences' && column == 'initial_side'
                  ? ReaderInitialSide.left.name
                  : null),
      ]);
    }
  }

  static Map<String, Object?> _settingsToMap(AppSettings value) => {
    'darkMode': value.darkMode,
    'defaultReaderTheme': value.defaultReaderTheme,
    'autoHideControls': value.autoHideControls,
    'showProgress': value.showProgress,
    'reduceMotion': value.reduceMotion,
    'disableAnimations': value.disableAnimations,
    'motionPreference': value.resolvedMotionPreference.name,
    'interfaceDensity': value.interfaceDensity.name,
    'defaultViewMode': value.defaultViewMode,
    'defaultPageLayout': value.defaultPageLayout,
    'defaultZoomMode': value.defaultZoomMode,
    'readingDirection': value.readingDirection.name,
    'visualProfile': value.visualProfile.name,
    'defaultMargin': value.defaultMargin,
    'restoreLastPosition': value.restoreLastPosition,
    'showRecents': value.showRecents,
    'generateCovers': value.generateCovers,
    'maxCacheMegabytes': value.maxCacheMegabytes,
    'recordReadingSessions': value.recordReadingSessions,
    'readingHistoryLimit': value.readingHistoryLimit,
    'libraryReadingFilter': value.libraryReadingFilter,
    'readerPanelWidth': value.readerPanelWidth,
    'readerPanelSection': value.readerPanelSection,
    'readerPanelOpen': value.readerPanelOpen,
    'readerChromeVisible': value.readerChromeVisible,
    'showReaderModeControls': value.showReaderModeControls,
    'showReaderPanelControls': value.showReaderPanelControls,
    'shortcutOverrides': value.shortcutOverrides,
    'restoreLastSession': value.restoreLastSession,
    'startInLibrary': value.startInLibrary,
    'lastDocumentPath': value.lastDocumentPath,
    'languageCode': value.languageCode,
    'hasSeenWelcome': value.hasSeenWelcome,
    'windowWidth': value.windowWidth,
    'windowHeight': value.windowHeight,
    'windowX': value.windowX,
    'windowY': value.windowY,
  };

  static AppSettings _settingsFromMap(Map<String, dynamic> map) {
    T enumValue<T extends Enum>(List<T> values, Object? raw, T fallback) {
      return values.where((value) => value.name == raw).firstOrNull ?? fallback;
    }

    bool boolean(String key, bool fallback) =>
        map[key] is bool ? map[key] as bool : fallback;
    String string(String key, String fallback) =>
        map[key] is String ? map[key] as String : fallback;
    double number(String key, double fallback) =>
        map[key] is num ? (map[key] as num).toDouble() : fallback;
    int integer(String key, int fallback) =>
        map[key] is num ? (map[key] as num).toInt() : fallback;

    return AppSettings(
      darkMode: boolean('darkMode', false),
      defaultReaderTheme: string('defaultReaderTheme', 'ivory'),
      autoHideControls: boolean('autoHideControls', true),
      showProgress: boolean('showProgress', true),
      reduceMotion: boolean('reduceMotion', false),
      disableAnimations: boolean('disableAnimations', false),
      motionPreference: enumValue(
        AppMotionPreference.values,
        map['motionPreference'],
        boolean('disableAnimations', false)
            ? AppMotionPreference.none
            : boolean('reduceMotion', false)
            ? AppMotionPreference.reduced
            : AppMotionPreference.full,
      ),
      interfaceDensity: enumValue(
        AppInterfaceDensity.values,
        map['interfaceDensity'],
        AppInterfaceDensity.comfortable,
      ),
      defaultViewMode: string('defaultViewMode', 'document'),
      defaultPageLayout: string('defaultPageLayout', 'single'),
      defaultZoomMode: string('defaultZoomMode', 'fitPage'),
      readingDirection: enumValue(
        ReadingDirection.values,
        map['readingDirection'],
        ReadingDirection.leftToRight,
      ),
      visualProfile: enumValue(
        ReaderVisualProfile.values,
        map['visualProfile'],
        ReaderVisualProfile.standard,
      ),
      defaultMargin: number('defaultMargin', 32).clamp(8, 72),
      restoreLastPosition: boolean('restoreLastPosition', true),
      showRecents: boolean('showRecents', true),
      generateCovers: boolean('generateCovers', true),
      maxCacheMegabytes: integer('maxCacheMegabytes', 256).clamp(64, 1024),
      recordReadingSessions: boolean('recordReadingSessions', true),
      readingHistoryLimit: integer('readingHistoryLimit', 250).clamp(25, 1000),
      libraryReadingFilter: string('libraryReadingFilter', 'all'),
      readerPanelWidth: number(
        'readerPanelWidth',
        AppSizes.readerPanelDefault,
      ).clamp(AppSizes.sidePanelMinWidth, AppSizes.sidePanelMaxWidth),
      readerPanelSection: string('readerPanelSection', ''),
      readerPanelOpen: boolean('readerPanelOpen', false),
      readerChromeVisible: boolean('readerChromeVisible', true),
      showReaderModeControls: boolean('showReaderModeControls', true),
      showReaderPanelControls: boolean('showReaderPanelControls', true),
      shortcutOverrides: string('shortcutOverrides', '{}'),
      restoreLastSession: boolean('restoreLastSession', true),
      startInLibrary: boolean('startInLibrary', false),
      lastDocumentPath: string('lastDocumentPath', ''),
      languageCode: string('languageCode', 'es'),
      hasSeenWelcome: boolean('hasSeenWelcome', false),
      windowWidth: number('windowWidth', 1120).clamp(640, 10000),
      windowHeight: number('windowHeight', 760).clamp(480, 10000),
      windowX: map['windowX'] is num
          ? (map['windowX'] as num).toDouble()
          : null,
      windowY: map['windowY'] is num
          ? (map['windowY'] as num).toDouble()
          : null,
    );
  }
}

const _documentColumns = <String>[
  ..._v8DocumentColumns,
  'last_failure_code',
  'restore_blocked',
];

const _v8DocumentColumns = <String>[
  'id',
  'path',
  'normalized_path',
  'file_name',
  'title',
  'author',
  'page_count',
  'file_size',
  'file_modified_at',
  'added_at',
  'last_opened_at',
  'last_page_index',
  'reading_progress',
  'preferred_view_mode',
  'preferred_theme',
  'is_missing',
  'fingerprint',
  'thumbnail_path',
  'is_favorite',
  'source_title',
  'source_author',
  'title_override',
  'author_override',
];

const _v3DocumentColumns = <String>[
  'id',
  'path',
  'normalized_path',
  'file_name',
  'title',
  'author',
  'page_count',
  'file_size',
  'file_modified_at',
  'added_at',
  'last_opened_at',
  'last_page_index',
  'reading_progress',
  'preferred_view_mode',
  'preferred_theme',
  'is_missing',
  'fingerprint',
  'thumbnail_path',
  'is_favorite',
];

const _legacyDocumentColumns = <String>[
  'id',
  'path',
  'normalized_path',
  'file_name',
  'title',
  'author',
  'page_count',
  'file_size',
  'file_modified_at',
  'added_at',
  'last_opened_at',
  'last_page_index',
  'reading_progress',
  'preferred_view_mode',
  'preferred_theme',
  'is_missing',
  'fingerprint',
  'thumbnail_path',
];

const _enrichmentColumns = <String>[
  'document_id',
  'fingerprint',
  'analyzer_version',
  'status',
  'suggested_title',
  'suggested_author',
  'title_confidence',
  'author_confidence',
  'topics_manual',
  'analyzed_at',
  'error_code',
];

const _topicColumns = <String>[
  'document_id',
  'topic_key',
  'confidence',
  'origin',
  'analyzer_version',
  'fingerprint',
  'updated_at',
];

const _preferenceColumns = <String>[
  'document_id',
  'view_mode',
  'reading_theme',
  'zoom_mode',
  'zoom_level',
  'page_layout',
  'cover_policy',
  'initial_side',
  'page_rotation',
  'horizontal_margin',
  'vertical_margin',
  'controls_auto_hide',
  'scroll_offset',
  'updated_at',
];

const _legacyPreferenceColumns = <String>[
  'document_id',
  'view_mode',
  'reading_theme',
  'zoom_mode',
  'zoom_level',
  'page_layout',
  'page_rotation',
  'horizontal_margin',
  'vertical_margin',
  'controls_auto_hide',
  'scroll_offset',
  'updated_at',
];

const _noteColumns = <String>['document_id', 'note', 'updated_at'];

const _bookmarkColumns = <String>[
  'id',
  'document_id',
  'page_index',
  'label',
  'created_at',
  'updated_at',
];

const _sessionColumns = <String>[
  'id',
  'document_id',
  'started_at',
  'ended_at',
  'start_page',
  'end_page',
  'active_seconds',
];

const _metadataColumns = <String>['key', 'value', 'updated_at'];

const _annotationColumns = <String>[
  ..._v8AnnotationColumns,
  'origin_document_id',
  'origin_fingerprint',
];

const _v8AnnotationColumns = <String>[
  'id',
  'document_id',
  'page_index',
  'type',
  'color',
  'note',
  'selected_text',
  'context_before',
  'context_after',
  'rects_json',
  'page_rotation',
  'document_fingerprint',
  'status',
  'created_at',
  'updated_at',
  'review_state',
];

const _v7AnnotationColumns = <String>[
  'id',
  'document_id',
  'page_index',
  'type',
  'color',
  'note',
  'selected_text',
  'context_before',
  'context_after',
  'rects_json',
  'page_rotation',
  'document_fingerprint',
  'status',
  'created_at',
  'updated_at',
];

const _tagColumns = <String>[
  'id',
  'name',
  'normalized_name',
  'color',
  'created_at',
  'updated_at',
];

const _v7WorkspaceColumns = <String>[
  'id',
  'name',
  'is_default',
  'active_tab_id',
  'panel_section',
  'panel_width',
  'split_json',
  'collection_id',
  'created_at',
  'updated_at',
];

const _annotationTagColumns = <String>['annotation_id', 'tag_id'];
const _documentTagColumns = <String>['document_id', 'tag_id'];

const _collectionColumns = <String>[
  'id',
  'name',
  'normalized_name',
  'icon',
  'sort_order',
  'is_pinned',
  'created_at',
  'updated_at',
];

const _collectionDocumentColumns = <String>[
  'collection_id',
  'document_id',
  'added_at',
];

const _readingProfileColumns = <String>[
  'id',
  'name',
  'is_builtin',
  'settings_json',
  'created_at',
  'updated_at',
];

const _workspaceColumns = <String>[
  'id',
  'name',
  'is_default',
  'active_tab_id',
  'panel_section',
  'panel_width',
  'split_json',
  'collection_id',
  'created_at',
  'updated_at',
  'query_json',
  'grouping',
  'sort_mode',
  'density',
  'visible_columns_json',
  'focus_mode',
];

const _workspaceTabColumns = <String>[
  'id',
  'workspace_id',
  'document_id',
  'sort_order',
  'is_pinned',
  'page_index',
  'zoom_level',
  'page_layout',
  'reading_mode',
  'scroll_offset',
  'queue_status',
  'blocking_reason',
  'next_action',
];

const _v7WorkspaceTabColumns = <String>[
  'id',
  'workspace_id',
  'document_id',
  'sort_order',
  'is_pinned',
  'page_index',
  'zoom_level',
  'page_layout',
  'reading_mode',
  'scroll_offset',
];

const _relationColumns = <String>[
  'id',
  'source_document_id',
  'target_document_id',
  'relation_type',
  'source_anchor_kind',
  'source_anchor_id',
  'target_anchor_kind',
  'target_anchor_id',
  'created_at',
];

const _revisionGroupColumns = <String>[
  'id',
  'name',
  'current_document_id',
  'created_at',
  'updated_at',
];

const _revisionMemberColumns = <String>['group_id', 'document_id', 'added_at'];

const _automationRuleColumns = <String>[
  'id',
  'name',
  'enabled',
  'trigger_json',
  'actions_json',
  'created_at',
  'updated_at',
];

const _journalColumns = <String>[
  ..._v8JournalColumns,
  'phase',
  'interruption_reason',
  'resume_capability',
  'compatibility_hash',
];

const _v8JournalColumns = <String>[
  'id',
  'operation_kind',
  'status',
  'summary_json',
  'total_items',
  'completed_items',
  'undo_json',
  'error_code',
  'started_at',
  'updated_at',
];

const _duplicateMergeHistoryColumns = <String>[
  'id',
  'kept_document_id',
  'incoming_document_id',
  'backup_path',
  'status',
  'created_at',
  'updated_at',
];
