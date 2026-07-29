import 'package:flutter/foundation.dart';

import '../../../storage/repositories/workspace_repository.dart';
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
    _repository.delete(id);
    if (activeWorkspaceId == id) activeWorkspaceId = null;
    reload();
  }

  void updateSplit(SplitViewState value) {
    split = value;
    notifyListeners();
  }
}
