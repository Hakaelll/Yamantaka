// ignore_for_file: prefer_initializing_formals

import 'dart:convert';
import 'dart:io';

import '../../app/theme/app_density.dart';
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
  });

  final int version;
  final DateTime exportedAt;
  final int documentCount;
  final int bookmarkCount;
  final int sessionCount;
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
  static const version = 1;

  final AppDatabase _database;
  final AppPaths _paths;
  final SettingsController _settings;

  Future<void> exportTo(String path) async {
    final payload = _snapshot();
    await File(path).writeAsString(
      const JsonEncoder.withIndent('  ').convert(payload),
      flush: true,
    );
  }

  Future<BackupPreview> preview(String path) async {
    final payload = await _readAndValidate(path);
    final tables = payload['tables']! as Map<String, dynamic>;
    return BackupPreview(
      version: payload['version']! as int,
      exportedAt: DateTime.parse(payload['exportedAt']! as String),
      documentCount: (tables['documents']! as List).length,
      bookmarkCount: (tables['bookmarks']! as List).length,
      sessionCount: (tables['reading_sessions']! as List).length,
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
    final database = _database.database;
    database.execute('BEGIN IMMEDIATE');
    try {
      database
        ..execute('DELETE FROM reading_sessions')
        ..execute('DELETE FROM bookmarks')
        ..execute('DELETE FROM document_preferences')
        ..execute('DELETE FROM documents')
        ..execute('DELETE FROM app_metadata');
      _restoreTable(
        'documents',
        _documentColumns,
        tables['documents']! as List,
      );
      _restoreTable(
        'document_preferences',
        _preferenceColumns,
        tables['document_preferences']! as List,
      );
      _restoreTable(
        'bookmarks',
        _bookmarkColumns,
        tables['bookmarks']! as List,
      );
      _restoreTable(
        'reading_sessions',
        _sessionColumns,
        tables['reading_sessions']! as List,
      );
      _restoreTable(
        'app_metadata',
        _metadataColumns,
        tables['app_metadata']! as List,
      );
      database.execute('COMMIT');
    } catch (_) {
      database.execute('ROLLBACK');
      rethrow;
    }

    final rawSettings = payload['settings'];
    if (rawSettings is Map<String, dynamic>) {
      await _settings.update(_settingsFromMap(rawSettings));
    }
    return safetyPath;
  }

  Map<String, dynamic> _snapshot() => {
    'format': format,
    'version': version,
    'exportedAt': DateTime.now().toUtc().toIso8601String(),
    'containsPdfFiles': false,
    'settings': _settingsToMap(_settings.settings),
    'tables': {
      'documents': _tableRows('documents'),
      'document_preferences': _tableRows('document_preferences'),
      'bookmarks': _tableRows('bookmarks'),
      'reading_sessions': _tableRows('reading_sessions'),
      'app_metadata': _tableRows('app_metadata'),
    },
  };

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
    final decoded = jsonDecode(await file.readAsString());
    if (decoded is! Map<String, dynamic> ||
        decoded['format'] != format ||
        decoded['version'] != version ||
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
    _validateTable(tables, 'documents', _documentColumns);
    _validateTable(tables, 'document_preferences', _preferenceColumns);
    _validateTable(tables, 'bookmarks', _bookmarkColumns);
    _validateTable(tables, 'reading_sessions', _sessionColumns);
    _validateTable(tables, 'app_metadata', _metadataColumns);
    if (decoded['settings'] is! Map<String, dynamic>) {
      throw const FormatException('La copia no contiene ajustes válidos.');
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
        for (final column in columns) row[column],
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
      interfaceDensity: enumValue(
        AppInterfaceDensity.values,
        map['interfaceDensity'],
        AppInterfaceDensity.comfortable,
      ),
      defaultViewMode: string('defaultViewMode', 'document'),
      defaultPageLayout: string('defaultPageLayout', 'continuous'),
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
      readerPanelWidth: number('readerPanelWidth', 320).clamp(280, 480),
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

const _preferenceColumns = <String>[
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
