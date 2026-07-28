import '../../features/reader/domain/reader_models.dart';
import '../database/app_database.dart';

final class DocumentPreferencesRepository {
  const DocumentPreferencesRepository(this._database);

  final AppDatabase _database;

  bool exists(String documentId) => _database.database.select(
    'SELECT 1 FROM document_preferences WHERE document_id = ? LIMIT 1',
    [documentId],
  ).isNotEmpty;

  DocumentPreferences load(String documentId) {
    final rows = _database.database.select(
      'SELECT * FROM document_preferences WHERE document_id = ? LIMIT 1',
      [documentId],
    );
    if (rows.isEmpty) return DocumentPreferences(documentId: documentId);
    final row = rows.first;
    return DocumentPreferences(
      documentId: documentId,
      viewMode: _enumByName(
        ReaderMode.values,
        row['view_mode'] as String,
        ReaderMode.document,
      ),
      readingTheme: _enumByName(
        ReaderTheme.values,
        row['reading_theme'] as String,
        ReaderTheme.ivory,
      ),
      zoomMode: _enumByName(
        ZoomMode.values,
        row['zoom_mode'] as String,
        ZoomMode.fitPage,
      ),
      zoomLevel: row['zoom_level'] as double,
      pageLayout: _enumByName(
        PageLayoutMode.values,
        row['page_layout'] as String,
        PageLayoutMode.continuous,
      ),
      pageRotation: row['page_rotation'] as int,
      horizontalMargin: row['horizontal_margin'] as double,
      verticalMargin: row['vertical_margin'] as double,
      controlsAutoHide: (row['controls_auto_hide'] as int) == 1,
      scrollOffset: row['scroll_offset'] as double,
      updatedAt: DateTime.parse(row['updated_at'] as String),
    );
  }

  void save(DocumentPreferences preferences) {
    final now = DateTime.now().toUtc().toIso8601String();
    _database.database.execute(
      '''
      INSERT INTO document_preferences (
        document_id, view_mode, reading_theme, zoom_mode, zoom_level,
        page_layout, page_rotation, horizontal_margin, vertical_margin,
        controls_auto_hide, scroll_offset, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(document_id) DO UPDATE SET
        view_mode = excluded.view_mode,
        reading_theme = excluded.reading_theme,
        zoom_mode = excluded.zoom_mode,
        zoom_level = excluded.zoom_level,
        page_layout = excluded.page_layout,
        page_rotation = excluded.page_rotation,
        horizontal_margin = excluded.horizontal_margin,
        vertical_margin = excluded.vertical_margin,
        controls_auto_hide = excluded.controls_auto_hide,
        scroll_offset = excluded.scroll_offset,
        updated_at = excluded.updated_at
      ''',
      [
        preferences.documentId,
        preferences.viewMode.name,
        preferences.readingTheme.name,
        preferences.zoomMode.name,
        preferences.zoomLevel,
        preferences.pageLayout.name,
        preferences.pageRotation,
        preferences.horizontalMargin,
        preferences.verticalMargin,
        preferences.controlsAutoHide ? 1 : 0,
        preferences.scrollOffset,
        now,
      ],
    );
  }

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) => values.where((value) => value.name == name).firstOrNull ?? fallback;
}
