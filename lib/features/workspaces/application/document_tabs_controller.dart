import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter/foundation.dart';

import '../../../app/app_config.dart';
import '../../library/domain/library_document.dart';
import '../../reader/domain/reader_models.dart';
import '../domain/workspace_models.dart';

final class DocumentTabsController extends ChangeNotifier {
  DocumentTabsController({
    this.maxTabs = AppConfig.maxOpenTabs,
    this.maxWarmTabs = AppConfig.maxWarmTabs,
  });

  final int maxTabs;
  final int maxWarmTabs;
  List<DocumentTab> tabs = const [];
  String? activeTabId;
  final List<DocumentTab> _closedTabs = [];

  DocumentTab? get activeTab =>
      tabs.where((tab) => tab.id == activeTabId).firstOrNull;
  bool get canReopenClosed => _closedTabs.isNotEmpty;

  DocumentTab open(LibraryDocument document) {
    final existing = tabs
        .where((tab) => tab.documentId == document.id)
        .firstOrNull;
    if (existing != null) {
      activate(existing.id);
      return activeTab!;
    }
    if (tabs.length >= maxTabs) {
      final disposable = tabs
          .where((tab) => !tab.isPinned && tab.id != activeTabId)
          .lastOrNull;
      if (disposable == null) {
        throw StateError(
          'El límite de documentos abiertos está ocupado por documentos '
          'fijados.',
        );
      }
      close(disposable.id);
    }
    final id = sha256
        .convert(
          utf8.encode(
            'tab:${document.id}:${DateTime.now().microsecondsSinceEpoch}',
          ),
        )
        .toString();
    tabs = [
      ...tabs,
      DocumentTab(
        id: id,
        documentId: document.id,
        path: document.path,
        title: document.title ?? document.fileName,
        pageIndex: document.lastPageIndex,
        zoomLevel: 1,
        pageLayout: PageLayoutMode.continuous.name,
        readingMode: document.preferredViewMode.name,
        scrollOffset: 0,
        readingProgress: document.readingProgress,
        isPinned: false,
        resourceState: TabResourceState.active,
        isMissing: document.isMissing,
        isExternallyModified: document.hasFileChanged,
      ),
    ];
    activeTabId = id;
    _applyResourcePolicy();
    notifyListeners();
    return activeTab!;
  }

  void activate(String id) {
    if (!tabs.any((tab) => tab.id == id)) return;
    activeTabId = id;
    _applyResourcePolicy();
    notifyListeners();
  }

  DocumentTab? close(String id) {
    final target = tabs.where((tab) => tab.id == id).firstOrNull;
    if (target == null) return null;
    final index = tabs.indexOf(target);
    tabs = tabs.where((tab) => tab.id != id).toList(growable: false);
    _closedTabs
      ..add(target)
      ..removeRange(0, (_closedTabs.length - 10).clamp(0, 10));
    if (activeTabId == id) {
      activeTabId = tabs.isEmpty
          ? null
          : tabs[index.clamp(0, tabs.length - 1)].id;
    }
    _applyResourcePolicy();
    notifyListeners();
    return target;
  }

  void closeOthers(String id) {
    final keep = tabs.where((tab) => tab.id == id || tab.isPinned).toList();
    for (final tab in tabs) {
      if (!keep.contains(tab)) _closedTabs.add(tab);
    }
    tabs = keep;
    activeTabId = tabs.any((tab) => tab.id == id) ? id : tabs.firstOrNull?.id;
    _applyResourcePolicy();
    notifyListeners();
  }

  void closeToRight(String id) {
    final index = tabs.indexWhere((tab) => tab.id == id);
    if (index < 0) return;
    final keep = <DocumentTab>[];
    for (var position = 0; position < tabs.length; position++) {
      final tab = tabs[position];
      if (position <= index || tab.isPinned) {
        keep.add(tab);
      } else {
        _closedTabs.add(tab);
      }
    }
    tabs = keep;
    if (!tabs.any((tab) => tab.id == activeTabId)) activeTabId = id;
    _applyResourcePolicy();
    notifyListeners();
  }

  DocumentTab? reopenLastClosed() {
    if (_closedTabs.isEmpty || tabs.length >= maxTabs) return null;
    final tab = _closedTabs.removeLast();
    tabs = [...tabs, tab];
    activeTabId = tab.id;
    _applyResourcePolicy();
    notifyListeners();
    return activeTab;
  }

  void togglePinned(String id) {
    tabs = [
      for (final tab in tabs)
        tab.id == id ? tab.copyWith(isPinned: !tab.isPinned) : tab,
    ];
    notifyListeners();
  }

  void reorder(int oldIndex, int newIndex) {
    if (oldIndex < 0 || oldIndex >= tabs.length) return;
    final mutable = [...tabs];
    newIndex = newIndex.clamp(0, mutable.length - 1);
    final tab = mutable.removeAt(oldIndex);
    mutable.insert(newIndex, tab);
    tabs = List.unmodifiable(mutable);
    notifyListeners();
  }

  void updateActiveNavigation({
    required int pageIndex,
    required double zoomLevel,
    required String pageLayout,
    required String readingMode,
    required double scrollOffset,
    double? readingProgress,
  }) {
    final id = activeTabId;
    if (id == null) return;
    tabs = [
      for (final tab in tabs)
        tab.id == id
            ? tab.copyWith(
                pageIndex: pageIndex,
                zoomLevel: zoomLevel,
                pageLayout: pageLayout,
                readingMode: readingMode,
                scrollOffset: scrollOffset,
                readingProgress: readingProgress,
              )
            : tab,
    ];
  }

  void restore(List<DocumentTab> restored, String? activeId) {
    tabs = restored.take(maxTabs).toList(growable: false);
    activeTabId = tabs.any((tab) => tab.id == activeId)
        ? activeId
        : tabs.firstOrNull?.id;
    _applyResourcePolicy();
    notifyListeners();
  }

  void _applyResourcePolicy() {
    var warmRemaining = maxWarmTabs;
    final reversed = tabs.reversed.map((tab) {
      if (tab.id == activeTabId) {
        return tab.copyWith(resourceState: TabResourceState.active);
      }
      if (warmRemaining > 0) {
        warmRemaining--;
        return tab.copyWith(resourceState: TabResourceState.warm);
      }
      return tab.copyWith(resourceState: TabResourceState.suspended);
    }).toList();
    tabs = reversed.reversed.toList(growable: false);
  }
}
