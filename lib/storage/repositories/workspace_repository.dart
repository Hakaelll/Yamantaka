import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/library/domain/library_document.dart';
import '../../features/workspaces/domain/workspace_models.dart';
import '../database/app_database.dart';
import 'document_repository.dart';

final class WorkspaceRepository {
  WorkspaceRepository(this._database, DocumentRepository documents);

  final AppDatabase _database;
  List<WorkspaceSnapshot>? _cachedWorkspaces;
  int? _cachedTotalChanges;

  List<WorkspaceSnapshot> all() {
    final db = _database.database;
    final totalChanges =
        db.select('SELECT total_changes() AS value').single['value'] as int;
    final cached = _cachedWorkspaces;
    if (cached != null && _cachedTotalChanges == totalChanges) return cached;
    final rows = db.select(
      'SELECT * FROM workspaces ORDER BY is_default DESC, updated_at DESC',
    );
    final tabRows = db.select('''
      SELECT wt.*, d.path, d.file_name,
        COALESCE(d.title_override, d.title, d.file_name) AS document_title,
        d.reading_progress, d.preferred_theme, d.is_missing, d.is_modified
      FROM workspace_tabs wt
      JOIN documents d ON d.id = wt.document_id
      ORDER BY wt.workspace_id, wt.sort_order
    ''');
    final tabsByWorkspace = <String, List<DocumentTab>>{};
    for (final row in tabRows) {
      tabsByWorkspace
          .putIfAbsent(row['workspace_id'] as String, () => <DocumentTab>[])
          .add(
            DocumentTab(
              id: row['id'] as String,
              documentId: row['document_id'] as String,
              path: row['path'] as String,
              title: row['document_title'] as String,
              pageIndex: row['page_index'] as int,
              zoomLevel: (row['zoom_level'] as num).toDouble(),
              pageLayout: row['page_layout'] as String,
              readingMode: row['reading_mode'] as String,
              scrollOffset: (row['scroll_offset'] as num).toDouble(),
              readingProgress: (row['reading_progress'] as num).toDouble(),
              isPinned: (row['is_pinned'] as int) == 1,
              resourceState: TabResourceState.suspended,
              readingTheme: row['preferred_theme'] as String,
              isMissing: (row['is_missing'] as int) == 1,
              isExternallyModified: (row['is_modified'] as int) == 1,
              queueStatus: _enumByName(
                WorkspaceQueueStatus.values,
                row['queue_status'] as String? ?? 'pending',
                WorkspaceQueueStatus.pending,
              ),
              blockingReason: row['blocking_reason'] as String?,
              nextAction: row['next_action'] as String?,
            ),
          );
    }
    final workspaces = rows
        .map((row) {
          final id = row['id'] as String;
          final tabs = tabsByWorkspace[id] ?? const <DocumentTab>[];
          return WorkspaceSnapshot(
            id: id,
            name: row['name'] as String,
            tabs: tabs,
            isDefault: (row['is_default'] as int) == 1,
            activeTabId: row['active_tab_id'] as String?,
            panelSection: row['panel_section'] as String?,
            panelWidth: row['panel_width'] as double,
            split: _decodeSplit(row['split_json'] as String?),
            collectionId: row['collection_id'] as String?,
            queryJson: row['query_json'] as String? ?? '{}',
            grouping: _enumByName(
              WorkspaceGrouping.values,
              row['grouping'] as String? ?? 'none',
              WorkspaceGrouping.none,
            ),
            sortMode: row['sort_mode'] as String? ?? 'lastOpened',
            density: _enumByName(
              WorkspaceDensity.values,
              row['density'] as String? ?? 'comfortable',
              WorkspaceDensity.comfortable,
            ),
            visibleColumns: _stringList(row['visible_columns_json'] as String?),
            focusMode: (row['focus_mode'] as int? ?? 0) == 1,
            createdAt: DateTime.parse(row['created_at'] as String),
            updatedAt: DateTime.parse(row['updated_at'] as String),
          );
        })
        .toList(growable: false);
    _cachedWorkspaces = workspaces;
    _cachedTotalChanges = totalChanges;
    return workspaces;
  }

  WorkspaceSnapshot save({
    String? id,
    required String name,
    required List<DocumentTab> tabs,
    String? activeTabId,
    String? panelSection,
    double panelWidth = 320,
    SplitViewState split = const SplitViewState(),
    String? collectionId,
    bool isDefault = false,
    String queryJson = '{}',
    WorkspaceGrouping grouping = WorkspaceGrouping.none,
    String sortMode = 'lastOpened',
    WorkspaceDensity density = WorkspaceDensity.comfortable,
    List<String> visibleColumns = const ['document', 'status', 'progress'],
    bool focusMode = false,
  }) {
    final cleanName = name.trim();
    if (cleanName.isEmpty || cleanName.length > 80) {
      throw ArgumentError.value(name, 'name');
    }
    final now = DateTime.now().toUtc();
    final workspaceId =
        id ??
        sha256
            .convert(
              utf8.encode('workspace:$cleanName:${now.microsecondsSinceEpoch}'),
            )
            .toString();
    final db = _database.database;
    db.execute('BEGIN IMMEDIATE');
    try {
      if (isDefault) db.execute('UPDATE workspaces SET is_default = 0');
      db.execute(
        '''
        INSERT INTO workspaces(
          id, name, is_default, active_tab_id, panel_section, panel_width,
          split_json, collection_id, created_at, updated_at, query_json,
          grouping, sort_mode, density, visible_columns_json, focus_mode
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          name = excluded.name,
          is_default = excluded.is_default,
          active_tab_id = excluded.active_tab_id,
          panel_section = excluded.panel_section,
          panel_width = excluded.panel_width,
          split_json = excluded.split_json,
          collection_id = excluded.collection_id,
          query_json = excluded.query_json,
          grouping = excluded.grouping,
          sort_mode = excluded.sort_mode,
          density = excluded.density,
          visible_columns_json = excluded.visible_columns_json,
          focus_mode = excluded.focus_mode,
          updated_at = excluded.updated_at
        ''',
        [
          workspaceId,
          cleanName,
          isDefault ? 1 : 0,
          activeTabId,
          panelSection,
          panelWidth.clamp(
            WorkspaceLayoutConstraints.sidePanelMinWidth,
            WorkspaceLayoutConstraints.sidePanelMaxWidth,
          ),
          jsonEncode(_encodeSplit(split)),
          collectionId,
          now.toIso8601String(),
          now.toIso8601String(),
          _validatedJsonObject(queryJson),
          grouping.name,
          sortMode.trim().isEmpty ? 'lastOpened' : sortMode.trim(),
          density.name,
          jsonEncode(visibleColumns.toSet().take(12).toList()),
          focusMode ? 1 : 0,
        ],
      );
      db.execute('DELETE FROM workspace_tabs WHERE workspace_id = ?', [
        workspaceId,
      ]);
      for (var index = 0; index < tabs.length; index++) {
        final tab = tabs[index];
        db.execute(
          '''
          INSERT INTO workspace_tabs(
            id, workspace_id, document_id, sort_order, is_pinned, page_index,
            zoom_level, page_layout, reading_mode, scroll_offset
            , queue_status, blocking_reason, next_action
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
          ''',
          [
            tab.id,
            workspaceId,
            tab.documentId,
            index,
            tab.isPinned ? 1 : 0,
            tab.pageIndex,
            tab.zoomLevel,
            tab.pageLayout,
            tab.readingMode,
            tab.scrollOffset,
            tab.queueStatus.name,
            _cleanQueueText(tab.blockingReason),
            _cleanQueueText(tab.nextAction),
          ],
        );
      }
      db.execute('COMMIT');
    } catch (_) {
      db.execute('ROLLBACK');
      rethrow;
    }
    return all().firstWhere((workspace) => workspace.id == workspaceId);
  }

  WorkspaceSnapshot duplicate(String id, String name) {
    final source = all().firstWhere((workspace) => workspace.id == id);
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    final tabs = [
      for (final tab in source.tabs)
        DocumentTab(
          id: sha256
              .convert(utf8.encode('workspace-tab:${tab.id}:$timestamp'))
              .toString(),
          documentId: tab.documentId,
          path: tab.path,
          title: tab.title,
          pageIndex: tab.pageIndex,
          zoomLevel: tab.zoomLevel,
          pageLayout: tab.pageLayout,
          readingMode: tab.readingMode,
          scrollOffset: tab.scrollOffset,
          readingProgress: tab.readingProgress,
          isPinned: tab.isPinned,
          resourceState: TabResourceState.suspended,
          readingTheme: tab.readingTheme,
          panelSection: tab.panelSection,
          isMissing: tab.isMissing,
          isExternallyModified: tab.isExternallyModified,
          queueStatus: tab.queueStatus,
          blockingReason: tab.blockingReason,
          nextAction: tab.nextAction,
        ),
    ];
    return save(
      name: name,
      tabs: tabs,
      activeTabId: tabs.firstOrNull?.id,
      panelSection: source.panelSection,
      panelWidth: source.panelWidth,
      split: source.split,
      collectionId: source.collectionId,
      queryJson: source.queryJson,
      grouping: source.grouping,
      sortMode: source.sortMode,
      density: source.density,
      visibleColumns: source.visibleColumns,
      focusMode: source.focusMode,
    );
  }

  void delete(String id) {
    _database.database.execute('DELETE FROM workspaces WHERE id = ?', [id]);
  }

  WorkspaceSnapshot restore(WorkspaceSnapshot snapshot) => save(
    id: snapshot.id,
    name: snapshot.name,
    tabs: snapshot.tabs,
    activeTabId: snapshot.activeTabId,
    panelSection: snapshot.panelSection,
    panelWidth: snapshot.panelWidth,
    split: snapshot.split,
    collectionId: snapshot.collectionId,
    isDefault: snapshot.isDefault,
    queryJson: snapshot.queryJson,
    grouping: snapshot.grouping,
    sortMode: snapshot.sortMode,
    density: snapshot.density,
    visibleColumns: snapshot.visibleColumns,
    focusMode: snapshot.focusMode,
  );

  bool addDocumentToQueue(String workspaceId, LibraryDocument document) {
    final workspace = all().where((item) => item.id == workspaceId).firstOrNull;
    if (workspace == null ||
        workspace.tabs.any((tab) => tab.documentId == document.id) ||
        workspace.tabs.length >= 1000) {
      return false;
    }
    final now = DateTime.now().microsecondsSinceEpoch;
    final tab = DocumentTab(
      id: sha256
          .convert(utf8.encode('queue:$workspaceId:${document.id}:$now'))
          .toString(),
      documentId: document.id,
      path: document.path,
      title: document.effectiveTitle,
      pageIndex: document.lastPageIndex,
      zoomLevel: 1,
      pageLayout: 'single',
      readingMode: document.preferredViewMode.name,
      scrollOffset: 0,
      readingProgress: document.readingProgress,
      isPinned: false,
      resourceState: TabResourceState.suspended,
      readingTheme: document.preferredTheme.name,
      isMissing: document.isMissing,
      isExternallyModified: document.hasFileChanged,
    );
    restore(
      WorkspaceSnapshot(
        id: workspace.id,
        name: workspace.name,
        tabs: [...workspace.tabs, tab],
        isDefault: workspace.isDefault,
        activeTabId: workspace.activeTabId ?? tab.id,
        panelSection: workspace.panelSection,
        panelWidth: workspace.panelWidth,
        split: workspace.split,
        collectionId: workspace.collectionId,
        queryJson: workspace.queryJson,
        grouping: workspace.grouping,
        sortMode: workspace.sortMode,
        density: workspace.density,
        visibleColumns: workspace.visibleColumns,
        focusMode: workspace.focusMode,
        createdAt: workspace.createdAt,
        updatedAt: DateTime.now().toUtc(),
      ),
    );
    return true;
  }

  bool removeDocumentFromQueue(String workspaceId, String documentId) {
    final workspace = all().where((item) => item.id == workspaceId).firstOrNull;
    if (workspace == null ||
        !workspace.tabs.any((tab) => tab.documentId == documentId)) {
      return false;
    }
    final tabs = workspace.tabs
        .where((tab) => tab.documentId != documentId)
        .toList(growable: false);
    restore(
      WorkspaceSnapshot(
        id: workspace.id,
        name: workspace.name,
        tabs: tabs,
        isDefault: workspace.isDefault,
        activeTabId:
            workspace.activeTabId != null &&
                tabs.any((tab) => tab.id == workspace.activeTabId)
            ? workspace.activeTabId
            : tabs.firstOrNull?.id,
        panelSection: workspace.panelSection,
        panelWidth: workspace.panelWidth,
        split: workspace.split,
        collectionId: workspace.collectionId,
        queryJson: workspace.queryJson,
        grouping: workspace.grouping,
        sortMode: workspace.sortMode,
        density: workspace.density,
        visibleColumns: workspace.visibleColumns,
        focusMode: workspace.focusMode,
        createdAt: workspace.createdAt,
        updatedAt: DateTime.now().toUtc(),
      ),
    );
    return true;
  }

  static Map<String, Object?> _encodeSplit(SplitViewState split) => {
    'enabled': split.enabled,
    'axis': split.axis.name,
    'ratio': split.ratio,
    'secondaryTabId': split.secondaryTabId,
    'syncPage': split.syncPage,
    'syncZoom': split.syncZoom,
    'comparisonMode': split.comparisonMode,
    'overlayOpacity': split.overlayOpacity,
  };

  static SplitViewState _decodeSplit(String? raw) {
    if (raw == null || raw.isEmpty) return const SplitViewState();
    try {
      final map = (jsonDecode(raw) as Map).cast<String, Object?>();
      return SplitViewState(
        enabled: map['enabled'] as bool? ?? false,
        axis: _enumByName(
          SplitAxis.values,
          map['axis'] as String? ?? '',
          SplitAxis.vertical,
        ),
        ratio: (map['ratio'] as num?)?.toDouble() ?? 0.5,
        secondaryTabId: map['secondaryTabId'] as String?,
        syncPage: map['syncPage'] as bool? ?? false,
        syncZoom: map['syncZoom'] as bool? ?? false,
        comparisonMode: map['comparisonMode'] as bool? ?? false,
        overlayOpacity: (map['overlayOpacity'] as num?)?.toDouble(),
      );
    } on Object {
      return const SplitViewState();
    }
  }

  static String _validatedJsonObject(String value) {
    try {
      return jsonDecode(value) is Map ? value : '{}';
    } on FormatException {
      return '{}';
    }
  }

  static List<String> _stringList(String? raw) {
    if (raw == null) return const ['document', 'status', 'progress'];
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) return const ['document', 'status', 'progress'];
      return decoded.whereType<String>().take(12).toList(growable: false);
    } on FormatException {
      return const ['document', 'status', 'progress'];
    }
  }

  static String? _cleanQueueText(String? value) {
    final clean = value?.trim();
    if (clean == null || clean.isEmpty) return null;
    return clean.substring(0, clean.length.clamp(0, 240));
  }

  static T _enumByName<T extends Enum>(
    List<T> values,
    String name,
    T fallback,
  ) {
    for (final value in values) {
      if (value.name == name) return value;
    }
    return fallback;
  }
}
