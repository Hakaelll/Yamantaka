import 'dart:convert';

import 'package:crypto/crypto.dart';

import '../../features/workspaces/domain/workspace_models.dart';
import '../database/app_database.dart';
import 'document_repository.dart';

final class WorkspaceRepository {
  const WorkspaceRepository(this._database, this._documents);

  final AppDatabase _database;
  final DocumentRepository _documents;

  List<WorkspaceSnapshot> all() {
    final rows = _database.database.select(
      'SELECT * FROM workspaces ORDER BY is_default DESC, updated_at DESC',
    );
    return rows
        .map((row) {
          final id = row['id'] as String;
          final tabRows = _database.database.select(
            'SELECT * FROM workspace_tabs WHERE workspace_id = ? ORDER BY sort_order',
            [id],
          );
          final tabs = <DocumentTab>[];
          for (final tabRow in tabRows) {
            final document = _documents.byId(tabRow['document_id'] as String);
            if (document == null) continue;
            tabs.add(
              DocumentTab(
                id: tabRow['id'] as String,
                documentId: document.id,
                path: document.path,
                title: document.effectiveTitle,
                pageIndex: tabRow['page_index'] as int,
                zoomLevel: tabRow['zoom_level'] as double,
                pageLayout: tabRow['page_layout'] as String,
                readingMode: tabRow['reading_mode'] as String,
                scrollOffset: tabRow['scroll_offset'] as double,
                readingProgress: document.readingProgress,
                isPinned: (tabRow['is_pinned'] as int) == 1,
                resourceState: TabResourceState.suspended,
                readingTheme: document.preferredTheme.name,
                isMissing: document.isMissing,
                isExternallyModified: document.hasFileChanged,
              ),
            );
          }
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
            createdAt: DateTime.parse(row['created_at'] as String),
            updatedAt: DateTime.parse(row['updated_at'] as String),
          );
        })
        .toList(growable: false);
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
          split_json, collection_id, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          name = excluded.name,
          is_default = excluded.is_default,
          active_tab_id = excluded.active_tab_id,
          panel_section = excluded.panel_section,
          panel_width = excluded.panel_width,
          split_json = excluded.split_json,
          collection_id = excluded.collection_id,
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
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    );
  }

  void delete(String id) {
    _database.database.execute('DELETE FROM workspaces WHERE id = ?', [id]);
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
        axis:
            SplitAxis.values
                .where((value) => value.name == map['axis'])
                .firstOrNull ??
            SplitAxis.vertical,
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
}
