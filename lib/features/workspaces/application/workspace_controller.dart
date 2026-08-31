import 'package:flutter/foundation.dart';

import '../../../storage/repositories/workspace_repository.dart';
import '../../library/domain/library_document.dart';
import '../domain/workspace_models.dart';
import 'document_tabs_controller.dart';

final class WorkspaceController extends ChangeNotifier {
  WorkspaceController(this._repository, this.tabs) {
    reload();
  }

  final WorkspaceRepository _repository;
  final DocumentTabsController tabs;
  List<WorkspaceSnapshot> workspaces = const [];
  String? activeWorkspaceId;
  SplitViewState split = const SplitViewState();
  WorkspaceSnapshot? _lastDeleted;

  bool get canUndoDelete => _lastDeleted != null;

  void reload() {
    workspaces = _repository.all();
    notifyListeners();
  }

  WorkspaceSnapshot saveCurrent({
    required String name,
    String? panelSection,
    double panelWidth = 320,
    String? collectionId,
    bool isDefault = false,
    String queryJson = '{}',
    WorkspaceGrouping grouping = WorkspaceGrouping.none,
    String sortMode = 'lastOpened',
    WorkspaceDensity density = WorkspaceDensity.comfortable,
    List<String> visibleColumns = const ['document', 'status', 'progress'],
    bool focusMode = false,
  }) {
    final saved = _repository.save(
      id: activeWorkspaceId,
      name: name,
      tabs: tabs.tabs,
      activeTabId: tabs.activeTabId,
      panelSection: panelSection,
      panelWidth: panelWidth,
      split: split,
      collectionId: collectionId,
      isDefault: isDefault,
      queryJson: queryJson,
      grouping: grouping,
      sortMode: sortMode,
      density: density,
      visibleColumns: visibleColumns,
      focusMode: focusMode,
    );
    activeWorkspaceId = saved.id;
    reload();
    return saved;
  }

  WorkspaceSnapshot duplicate(String id, String name) {
    final result = _repository.duplicate(id, name);
    reload();
    return result;
  }

  List<DocumentTab> restore(String id) {
    final workspace = workspaces.firstWhere((item) => item.id == id);
    tabs.restore(workspace.tabs, workspace.activeTabId);
    split = workspace.split;
    activeWorkspaceId = id;
    notifyListeners();
    return workspace.tabs;
  }

  void delete(String id) {
    _lastDeleted = workspaces.where((item) => item.id == id).firstOrNull;
    _repository.delete(id);
    if (activeWorkspaceId == id) activeWorkspaceId = null;
    reload();
  }

  WorkspaceSnapshot? undoDelete() {
    final snapshot = _lastDeleted;
    if (snapshot == null) return null;
    _lastDeleted = null;
    final restored = _repository.restore(snapshot);
    reload();
    return restored;
  }

  WorkspaceSnapshot rename(String id, String name) {
    final source = workspaces.firstWhere((item) => item.id == id);
    final renamed = _repository.save(
      id: source.id,
      name: name,
      tabs: source.tabs,
      activeTabId: source.activeTabId,
      panelSection: source.panelSection,
      panelWidth: source.panelWidth,
      split: source.split,
      collectionId: source.collectionId,
      isDefault: source.isDefault,
      queryJson: source.queryJson,
      grouping: source.grouping,
      sortMode: source.sortMode,
      density: source.density,
      visibleColumns: source.visibleColumns,
      focusMode: source.focusMode,
    );
    reload();
    return renamed;
  }

  void reorderQueue(int oldIndex, int newIndex) {
    tabs.reorder(oldIndex, newIndex);
    _persistActive();
  }

  void updateQueueEntry(
    String tabId, {
    WorkspaceQueueStatus? status,
    String? blockingReason,
    String? nextAction,
  }) {
    tabs.updateQueueEntry(
      tabId,
      status: status,
      blockingReason: blockingReason,
      nextAction: nextAction,
    );
    _persistActive();
  }

  DocumentTab? openNext() {
    final next = tabs.nextQueueItem;
    if (next != null) tabs.activate(next.id);
    return next;
  }

  DocumentTab? markAndContinue(String tabId) {
    tabs.updateQueueEntry(tabId, status: WorkspaceQueueStatus.reviewed);
    _persistActive();
    return openNext();
  }

  bool addDocumentToQueue(String workspaceId, LibraryDocument document) {
    final added = _repository.addDocumentToQueue(workspaceId, document);
    if (added) reload();
    return added;
  }

  bool removeDocumentFromQueue(String workspaceId, String documentId) {
    final removed = _repository.removeDocumentFromQueue(
      workspaceId,
      documentId,
    );
    if (removed) reload();
    return removed;
  }

  void updateFocusMode(bool enabled) {
    final id = activeWorkspaceId;
    if (id == null) return;
    final source = workspaces.where((item) => item.id == id).firstOrNull;
    if (source == null || source.focusMode == enabled) return;
    _repository.save(
      id: source.id,
      name: source.name,
      tabs: tabs.tabs,
      activeTabId: tabs.activeTabId,
      panelSection: source.panelSection,
      panelWidth: source.panelWidth,
      split: split,
      collectionId: source.collectionId,
      isDefault: source.isDefault,
      queryJson: source.queryJson,
      grouping: source.grouping,
      sortMode: source.sortMode,
      density: source.density,
      visibleColumns: source.visibleColumns,
      focusMode: enabled,
    );
    reload();
  }

  void updateSplit(SplitViewState value) {
    split = value;
    notifyListeners();
  }

  void _persistActive() {
    final id = activeWorkspaceId;
    if (id == null) return;
    final source = workspaces.where((item) => item.id == id).firstOrNull;
    if (source == null) return;
    _repository.save(
      id: id,
      name: source.name,
      tabs: tabs.tabs,
      activeTabId: tabs.activeTabId,
      panelSection: source.panelSection,
      panelWidth: source.panelWidth,
      split: split,
      collectionId: source.collectionId,
      isDefault: source.isDefault,
      queryJson: source.queryJson,
      grouping: source.grouping,
      sortMode: source.sortMode,
      density: source.density,
      visibleColumns: source.visibleColumns,
      focusMode: source.focusMode,
    );
    reload();
  }
}
