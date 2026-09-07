import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../annotations/domain/annotation.dart';
import '../../navigation/application/navigation_history_controller.dart';
import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

enum _NavigatorKind {
  all,
  outline,
  thumbnails,
  search,
  bookmarks,
  annotations,
  history,
  positions,
}

final class _NavigatorEntry {
  const _NavigatorEntry({
    required this.kind,
    required this.pageIndex,
    required this.title,
    required this.detail,
  });

  final _NavigatorKind kind;
  final int pageIndex;
  final String title;
  final String detail;

  String get searchable => '$title $detail ${pageIndex + 1}'.toLowerCase();
}

Future<void> showDocumentNavigatorDialog(
  BuildContext context,
  ReaderController controller,
) => showDialog<void>(
  context: context,
  barrierDismissible: true,
  builder: (context) => DocumentNavigatorDialog(controller: controller),
);

final class DocumentNavigatorDialog extends StatefulWidget {
  const DocumentNavigatorDialog({super.key, required this.controller});

  final ReaderController controller;

  @override
  State<DocumentNavigatorDialog> createState() =>
      _DocumentNavigatorDialogState();
}

final class _DocumentNavigatorDialogState
    extends State<DocumentNavigatorDialog> {
  final TextEditingController _filter = TextEditingController();
  final FocusNode _filterFocus = FocusNode(debugLabel: 'Document navigator');
  Timer? _searchTimer;
  _NavigatorKind _kind = _NavigatorKind.all;
  int _selectedIndex = 0;
  bool _compactDetail = false;

  @override
  void initState() {
    super.initState();
    widget.controller.search.addListener(_refresh);
  }

  @override
  void dispose() {
    _searchTimer?.cancel();
    widget.controller.search.removeListener(_refresh);
    _filter.dispose();
    _filterFocus.dispose();
    super.dispose();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  List<_NavigatorEntry> _entries(YamantakaLocalizations strings) {
    final entries = <_NavigatorEntry>[];
    void appendOutline(List<dynamic> items, int depth) {
      for (final item in items) {
        final pageIndex = item.pageIndex as int?;
        if (pageIndex != null) {
          entries.add(
            _NavigatorEntry(
              kind: _NavigatorKind.outline,
              pageIndex: pageIndex,
              title: '${'  ' * depth}${item.title}',
              detail: strings.get('reader.outline'),
            ),
          );
        }
        appendOutline(item.children as List<dynamic>, depth + 1);
      }
    }

    appendOutline(widget.controller.outline, 0);
    for (
      var pageIndex = 0;
      pageIndex < widget.controller.pageCount;
      pageIndex++
    ) {
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.thumbnails,
          pageIndex: pageIndex,
          title: strings.get(
            'reader.page',
            parameters: {'page': pageIndex + 1},
          ),
          detail: strings.get('reader.thumbnails'),
        ),
      );
    }
    for (final match in widget.controller.search.matches) {
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.search,
          pageIndex: match.pageIndex,
          title: match.snippet,
          detail: strings.get(
            'reader.page',
            parameters: {'page': match.pageIndex + 1},
          ),
        ),
      );
    }
    for (final bookmark in widget.controller.bookmarks.bookmarks) {
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.bookmarks,
          pageIndex: bookmark.pageIndex,
          title: bookmark.label?.trim().isNotEmpty == true
              ? bookmark.label!
              : strings.get('reader.bookmarks'),
          detail: strings.get(
            'reader.page',
            parameters: {'page': bookmark.pageIndex + 1},
          ),
        ),
      );
    }
    for (final PdfAnnotation annotation
        in widget.controller.annotations?.annotations ??
            const <PdfAnnotation>[]) {
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.annotations,
          pageIndex: annotation.pageIndex,
          title:
              annotation.note ??
              annotation.selectedText ??
              strings.get('annotation.type.${annotation.type.name}'),
          detail:
              '${strings.get('annotation.type.${annotation.type.name}')} · ${strings.get('reader.page', parameters: {'page': annotation.pageIndex + 1})}',
        ),
      );
    }
    for (final location
        in widget.controller.navigationHistory?.recent ?? const []) {
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.history,
          pageIndex: location.pageIndex,
          title: location.label,
          detail: strings.get(
            'reader.page',
            parameters: {'page': location.pageIndex + 1},
          ),
        ),
      );
    }
    void addPosition(String slot, ReaderTemporaryPosition? position) {
      if (position == null) return;
      entries.add(
        _NavigatorEntry(
          kind: _NavigatorKind.positions,
          pageIndex: position.pageIndex,
          title: '${strings.get('reader.temporaryPosition')} $slot',
          detail: strings.get(
            'reader.page',
            parameters: {'page': position.pageIndex + 1},
          ),
        ),
      );
    }

    addPosition('A', widget.controller.temporaryPositionA);
    addPosition('B', widget.controller.temporaryPositionB);
    final query = _filter.text.trim().toLowerCase();
    return entries
        .where((entry) => _kind == _NavigatorKind.all || entry.kind == _kind)
        .where((entry) => query.isEmpty || entry.searchable.contains(query))
        .toList(growable: false);
  }

  void _onFilterChanged(String value) {
    setState(() {
      _selectedIndex = 0;
      _compactDetail = false;
    });
    _searchTimer?.cancel();
    _searchTimer = Timer(const Duration(milliseconds: 160), () {
      if (mounted) unawaited(widget.controller.search.search(value));
    });
  }

  KeyEventResult _onKey(FocusNode _, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final entries = _entries(YamantakaLocalizations.of(context));
    if (event.logicalKey == LogicalKeyboardKey.escape) {
      Navigator.pop(context);
      return KeyEventResult.handled;
    }
    if (entries.isEmpty) return KeyEventResult.ignored;
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      setState(() => _selectedIndex = (_selectedIndex + 1) % entries.length);
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      setState(
        () => _selectedIndex =
            (_selectedIndex - 1 + entries.length) % entries.length,
      );
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter) {
      unawaited(
        _activate(entries[_selectedIndex.clamp(0, entries.length - 1)]),
      );
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Future<void> _activate(_NavigatorEntry entry) async {
    await widget.controller.navigateTo(
      entry.pageIndex,
      reason: switch (entry.kind) {
        _NavigatorKind.outline => NavigationReason.outline,
        _NavigatorKind.bookmarks => NavigationReason.bookmark,
        _NavigatorKind.annotations => NavigationReason.annotation,
        _NavigatorKind.search => NavigationReason.searchResult,
        _NavigatorKind.thumbnails => NavigationReason.thumbnail,
        _NavigatorKind.history ||
        _NavigatorKind.positions => NavigationReason.history,
        _NavigatorKind.all => NavigationReason.distantPage,
      },
      label: entry.title,
      focusDestination: true,
    );
    if (mounted) Navigator.pop(context);
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final size = MediaQuery.sizeOf(context);
    final compact =
        AppBreakpoints.classify(size.width) == AppWindowClass.compact ||
        size.height < 620;
    final entries = _entries(strings);
    if (_selectedIndex >= entries.length) _selectedIndex = 0;
    final selected = entries.isEmpty ? null : entries[_selectedIndex];
    final document = widget.controller.document;
    final stateLabel = document?.isMissing == true
        ? strings.get('library.fileMissing')
        : document?.hasFileChanged == true
        ? strings.get('library.fileModified')
        : null;
    return Focus(
      autofocus: true,
      onKeyEvent: _onKey,
      child: Dialog(
        insetPadding: const EdgeInsets.all(AppSpacing.space16),
        child: ConstrainedBox(
          constraints: BoxConstraints(
            maxWidth: compact ? 620 : 980,
            maxHeight: compact ? size.height - 32 : 720,
          ),
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 16, 8, 8),
                child: Row(
                  children: [
                    if (compact && _compactDetail)
                      IconButton(
                        key: const Key('document-navigator-back'),
                        tooltip: strings.get('action.back'),
                        onPressed: () => setState(() => _compactDetail = false),
                        icon: const Icon(Icons.arrow_back_rounded),
                      ),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            strings.get('reader.documentNavigator'),
                            style: Theme.of(context).textTheme.titleLarge,
                          ),
                          if (stateLabel != null)
                            Text(
                              stateLabel,
                              style: Theme.of(context).textTheme.labelMedium
                                  ?.copyWith(color: context.appColors.warning),
                            ),
                        ],
                      ),
                    ),
                    IconButton(
                      tooltip: strings.get('action.close'),
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(Icons.close_rounded),
                    ),
                  ],
                ),
              ),
              if (!_compactDetail) ...[
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 20),
                  child: TextField(
                    key: const Key('document-navigator-filter'),
                    controller: _filter,
                    focusNode: _filterFocus,
                    autofocus: true,
                    onChanged: _onFilterChanged,
                    decoration: InputDecoration(
                      hintText: strings.get('reader.navigatorFilter'),
                      prefixIcon: const Icon(Icons.search_rounded),
                    ),
                  ),
                ),
                SizedBox(
                  height: 30 + MediaQuery.textScalerOf(context).scale(24),
                  child: ListView(
                    scrollDirection: Axis.horizontal,
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    children: [
                      for (final kind in _NavigatorKind.values)
                        Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 3),
                          child: ChoiceChip(
                            label: Text(_kindLabel(strings, kind)),
                            selected: _kind == kind,
                            onSelected: (_) => setState(() {
                              _kind = kind;
                              _selectedIndex = 0;
                            }),
                          ),
                        ),
                    ],
                  ),
                ),
              ],
              const Divider(height: 1),
              Expanded(
                child: compact
                    ? _compactDetail && selected != null
                          ? _preview(strings, selected)
                          : _list(strings, entries, compact: true)
                    : Row(
                        children: [
                          Expanded(flex: 5, child: _list(strings, entries)),
                          const VerticalDivider(width: 1),
                          Expanded(
                            flex: 4,
                            child: selected == null
                                ? _empty(strings)
                                : _preview(strings, selected),
                          ),
                        ],
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _list(
    YamantakaLocalizations strings,
    List<_NavigatorEntry> entries, {
    bool compact = false,
  }) {
    if (entries.isEmpty) return _empty(strings);
    return ListView.separated(
      key: const Key('document-navigator-list'),
      itemCount: entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1, indent: 56),
      itemBuilder: (context, index) {
        final entry = entries[index];
        return ListTile(
          selected: index == _selectedIndex,
          leading: Icon(_kindIcon(entry.kind)),
          title: Text(
            entry.title,
            style: Theme.of(context).textTheme.bodyMedium,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          subtitle: Text(
            entry.detail,
            style: Theme.of(context).textTheme.bodySmall,
          ),
          trailing: Text(
            '${entry.pageIndex + 1}',
            style: Theme.of(context).textTheme.labelMedium,
          ),
          onTap: () => setState(() {
            _selectedIndex = index;
            if (compact) _compactDetail = true;
          }),
          onLongPress: () => unawaited(_activate(entry)),
        );
      },
    );
  }

  Widget _preview(YamantakaLocalizations strings, _NavigatorEntry entry) =>
      Center(
        child: SingleChildScrollView(
          child: Padding(
            padding: const EdgeInsets.all(AppSpacing.space24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(_kindIcon(entry.kind), size: 44),
                const SizedBox(height: AppSpacing.space16),
                Text(
                  entry.title,
                  textAlign: TextAlign.center,
                  maxLines: 5,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: AppSpacing.space8),
                Text(
                  entry.detail,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: AppSpacing.space24),
                FilledButton.icon(
                  key: const Key('document-navigator-open'),
                  onPressed: () => unawaited(_activate(entry)),
                  icon: const Icon(Icons.arrow_forward_rounded),
                  label: Text(
                    strings.get(
                      'reader.goToPageNumber',
                      parameters: {'page': entry.pageIndex + 1},
                    ),
                  ),
                ),
                const SizedBox(height: AppSpacing.space8),
                Text(
                  strings.get('reader.navigatorHistoryHint'),
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.labelSmall,
                ),
              ],
            ),
          ),
        ),
      );

  Widget _empty(YamantakaLocalizations strings) => Center(
    child: Padding(
      padding: const EdgeInsets.all(AppSpacing.space24),
      child: Text(
        strings.get('reader.navigatorEmpty'),
        textAlign: TextAlign.center,
      ),
    ),
  );

  String _kindLabel(YamantakaLocalizations strings, _NavigatorKind kind) =>
      strings.get('reader.navigator.${kind.name}');

  IconData _kindIcon(_NavigatorKind kind) => switch (kind) {
    _NavigatorKind.all => Icons.explore_outlined,
    _NavigatorKind.outline => Icons.account_tree_outlined,
    _NavigatorKind.thumbnails => Icons.grid_view_outlined,
    _NavigatorKind.search => Icons.search_rounded,
    _NavigatorKind.bookmarks => Icons.bookmarks_outlined,
    _NavigatorKind.annotations => Icons.rate_review_outlined,
    _NavigatorKind.history => Icons.history_rounded,
    _NavigatorKind.positions => Icons.compare_arrows_rounded,
  };
}
