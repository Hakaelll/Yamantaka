import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../storage/repositories/annotation_repository.dart';
import '../domain/annotation.dart';

enum AnnotationGrouping { document, page, state }

enum AnnotationDateFilter { any, last7Days, last30Days }

final class GlobalAnnotationsDialog extends StatefulWidget {
  const GlobalAnnotationsDialog({
    super.key,
    required this.repository,
    required this.documentTitle,
    required this.onOpen,
    this.onExport,
  });

  final AnnotationRepository repository;
  final String Function(String documentId) documentTitle;
  final ValueChanged<PdfAnnotation> onOpen;
  final Future<void> Function(List<PdfAnnotation> annotations)? onExport;

  @override
  State<GlobalAnnotationsDialog> createState() =>
      _GlobalAnnotationsDialogState();
}

final class _GlobalAnnotationsDialogState
    extends State<GlobalAnnotationsDialog> {
  final _query = TextEditingController();
  final _tag = TextEditingController();
  final Set<String> _selected = {};
  AnnotationType? _type;
  AnnotationColor? _color;
  AnnotationReviewState? _state;
  AnnotationGrouping _grouping = AnnotationGrouping.document;
  AnnotationDateFilter _date = AnnotationDateFilter.any;
  List<PdfAnnotation> _annotations = const [];
  int _cursor = 0;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  @override
  void dispose() {
    _query.dispose();
    _tag.dispose();
    super.dispose();
  }

  void _reload() {
    final annotations = widget.repository.all(
      type: _type,
      color: _color,
      reviewState: _state,
      query: _query.text,
      tag: _tag.text,
      updatedAfter: switch (_date) {
        AnnotationDateFilter.any => null,
        AnnotationDateFilter.last7Days => DateTime.now().toUtc().subtract(
          const Duration(days: 7),
        ),
        AnnotationDateFilter.last30Days => DateTime.now().toUtc().subtract(
          const Duration(days: 30),
        ),
      },
    );
    annotations.sort((left, right) {
      final primary = switch (_grouping) {
        AnnotationGrouping.document =>
          widget
              .documentTitle(left.documentId)
              .compareTo(widget.documentTitle(right.documentId)),
        AnnotationGrouping.page => left.pageIndex.compareTo(right.pageIndex),
        AnnotationGrouping.state => left.reviewState.index.compareTo(
          right.reviewState.index,
        ),
      };
      return primary != 0 ? primary : left.updatedAt.compareTo(right.updatedAt);
    });
    setState(() {
      _annotations = annotations;
      _selected.removeWhere(
        (id) => !_annotations.any((annotation) => annotation.id == id),
      );
      _cursor = _cursor.clamp(0, (_annotations.length - 1).clamp(0, 79));
    });
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 920, maxHeight: 720),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space16),
              child: Row(
                children: [
                  const Icon(Icons.rate_review_outlined),
                  const SizedBox(width: AppSpacing.space8),
                  Expanded(
                    child: Text(
                      strings.get('annotations.globalTitle'),
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                  ),
                  IconButton(
                    tooltip: strings.get('annotations.previous'),
                    onPressed: _cursor <= 0 || _annotations.isEmpty
                        ? null
                        : () => _move(-1),
                    icon: const Icon(Icons.arrow_back_rounded),
                  ),
                  IconButton(
                    tooltip: strings.get('annotations.next'),
                    onPressed:
                        _cursor >= _annotations.length - 1 ||
                            _annotations.isEmpty
                        ? null
                        : () => _move(1),
                    icon: const Icon(Icons.arrow_forward_rounded),
                  ),
                  if (widget.onExport != null)
                    IconButton(
                      tooltip: strings.get('reader.exportAnnotations'),
                      onPressed: _annotations.isEmpty
                          ? null
                          : () => widget.onExport!(_annotations),
                      icon: const Icon(Icons.download_outlined),
                    ),
                  IconButton(
                    tooltip: strings.get('action.close'),
                    onPressed: () => Navigator.pop(context),
                    icon: const Icon(Icons.close_rounded),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space12),
              child: Wrap(
                spacing: AppSpacing.space8,
                runSpacing: AppSpacing.space8,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  SizedBox(
                    width: 260,
                    child: TextField(
                      controller: _query,
                      decoration: InputDecoration(
                        prefixIcon: const Icon(Icons.search_rounded),
                        labelText: strings.get('annotations.search'),
                      ),
                      onChanged: (_) => _reload(),
                    ),
                  ),
                  SizedBox(
                    width: 150,
                    child: TextField(
                      controller: _tag,
                      decoration: InputDecoration(
                        prefixIcon: const Icon(Icons.label_outline_rounded),
                        labelText: strings.get('annotations.filterTag'),
                      ),
                      onChanged: (_) => _reload(),
                    ),
                  ),
                  _Filter<AnnotationReviewState>(
                    value: _state,
                    values: AnnotationReviewState.values,
                    allLabel: strings.get('annotations.allStates'),
                    label: (value) =>
                        strings.get('annotations.state.${value.name}'),
                    onChanged: (value) {
                      _state = value;
                      _reload();
                    },
                  ),
                  _Filter<AnnotationType>(
                    value: _type,
                    values: AnnotationType.values,
                    allLabel: strings.get('annotations.allTypes'),
                    label: (value) =>
                        strings.get('annotations.type.${value.name}'),
                    onChanged: (value) {
                      _type = value;
                      _reload();
                    },
                  ),
                  _Filter<AnnotationColor>(
                    value: _color,
                    values: AnnotationColor.values,
                    allLabel: strings.get('annotations.allColors'),
                    label: (value) =>
                        strings.get('annotations.color.${value.name}'),
                    onChanged: (value) {
                      _color = value;
                      _reload();
                    },
                  ),
                  Tooltip(
                    message: strings.get('annotations.groupBy'),
                    child: DropdownButton<AnnotationGrouping>(
                      value: _grouping,
                      onChanged: (value) {
                        if (value == null) return;
                        _grouping = value;
                        _reload();
                      },
                      items: [
                        for (final grouping in AnnotationGrouping.values)
                          DropdownMenuItem(
                            value: grouping,
                            child: Text(
                              strings.get('annotations.group.${grouping.name}'),
                            ),
                          ),
                      ],
                    ),
                  ),
                  Tooltip(
                    message: strings.get('annotations.filterDate'),
                    child: DropdownButton<AnnotationDateFilter>(
                      value: _date,
                      onChanged: (value) {
                        if (value == null) return;
                        _date = value;
                        _reload();
                      },
                      items: [
                        for (final date in AnnotationDateFilter.values)
                          DropdownMenuItem(
                            value: date,
                            child: Text(
                              strings.get('annotations.date.${date.name}'),
                            ),
                          ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            if (_selected.isNotEmpty)
              Material(
                color: Theme.of(context).colorScheme.secondaryContainer,
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppSpacing.space12,
                    vertical: AppSpacing.space8,
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          strings.get(
                            'annotations.selected',
                            parameters: {'count': _selected.length},
                          ),
                        ),
                      ),
                      PopupMenuButton<AnnotationReviewState>(
                        tooltip: strings.get('annotations.changeState'),
                        onSelected: _setSelectedState,
                        itemBuilder: (context) => [
                          for (final state in AnnotationReviewState.values)
                            PopupMenuItem(
                              value: state,
                              child: Text(
                                strings.get('annotations.state.${state.name}'),
                              ),
                            ),
                        ],
                        icon: const Icon(Icons.task_alt_rounded),
                      ),
                      IconButton(
                        tooltip: strings.get('annotations.addTag'),
                        onPressed: _addTag,
                        icon: const Icon(Icons.label_outline_rounded),
                      ),
                      IconButton(
                        tooltip: strings.get('action.clearSelection'),
                        onPressed: () => setState(_selected.clear),
                        icon: const Icon(Icons.clear_all_rounded),
                      ),
                    ],
                  ),
                ),
              ),
            const Divider(height: 1),
            Expanded(
              child: _annotations.isEmpty
                  ? Center(child: Text(strings.get('annotations.empty')))
                  : ListView.builder(
                      itemCount: _annotations.length,
                      itemBuilder: (context, index) {
                        final annotation = _annotations[index];
                        return _AnnotationRow(
                          key: ValueKey(annotation.id),
                          annotation: annotation,
                          title: widget.documentTitle(annotation.documentId),
                          selected: _selected.contains(annotation.id),
                          active: index == _cursor,
                          onSelected: (value) => setState(() {
                            if (value) {
                              _selected.add(annotation.id);
                            } else {
                              _selected.remove(annotation.id);
                            }
                          }),
                          onOpen: () {
                            _cursor = index;
                            widget.onOpen(annotation);
                          },
                          onSaved: (value) {
                            widget.repository.save(value);
                          },
                          onCopyReference: () => Clipboard.setData(
                            ClipboardData(
                              text:
                                  '${widget.documentTitle(annotation.documentId)} — p. ${annotation.pageIndex + 1}',
                            ),
                          ),
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }

  void _move(int delta) {
    setState(
      () => _cursor = (_cursor + delta).clamp(0, _annotations.length - 1),
    );
    widget.onOpen(_annotations[_cursor]);
  }

  void _setSelectedState(AnnotationReviewState state) {
    widget.repository.updateReviewState(_selected, state);
    _reload();
  }

  Future<void> _addTag() async {
    final controller = TextEditingController();
    final tag = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(
          YamantakaLocalizations.of(context).get('annotations.addTag'),
        ),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: 80,
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(
              YamantakaLocalizations.of(context).get('action.cancel'),
            ),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, controller.text),
            child: Text(YamantakaLocalizations.of(context).get('action.apply')),
          ),
        ],
      ),
    );
    controller.dispose();
    if (tag == null || tag.trim().isEmpty) return;
    widget.repository.addTag(_selected, tag);
    _reload();
  }
}

final class _Filter<T> extends StatelessWidget {
  const _Filter({
    required this.value,
    required this.values,
    required this.allLabel,
    required this.label,
    required this.onChanged,
  });

  final T? value;
  final List<T> values;
  final String allLabel;
  final String Function(T value) label;
  final ValueChanged<T?> onChanged;

  @override
  Widget build(BuildContext context) => DropdownButton<T?>(
    value: value,
    onChanged: onChanged,
    items: [
      DropdownMenuItem<T?>(value: null, child: Text(allLabel)),
      for (final item in values)
        DropdownMenuItem<T?>(value: item, child: Text(label(item))),
    ],
  );
}

final class _AnnotationRow extends StatefulWidget {
  const _AnnotationRow({
    super.key,
    required this.annotation,
    required this.title,
    required this.selected,
    required this.active,
    required this.onSelected,
    required this.onOpen,
    required this.onSaved,
    required this.onCopyReference,
  });

  final PdfAnnotation annotation;
  final String title;
  final bool selected;
  final bool active;
  final ValueChanged<bool> onSelected;
  final VoidCallback onOpen;
  final ValueChanged<PdfAnnotation> onSaved;
  final VoidCallback onCopyReference;

  @override
  State<_AnnotationRow> createState() => _AnnotationRowState();
}

final class _AnnotationRowState extends State<_AnnotationRow> {
  late final TextEditingController _note;
  Timer? _saveTimer;

  @override
  void initState() {
    super.initState();
    _note = TextEditingController(text: widget.annotation.note ?? '');
  }

  @override
  void dispose() {
    _saveTimer?.cancel();
    _flush();
    _note.dispose();
    super.dispose();
  }

  void _scheduleSave() {
    _saveTimer?.cancel();
    _saveTimer = Timer(const Duration(milliseconds: 450), _flush);
  }

  void _flush() {
    final value = _note.text.trim();
    if (value == (widget.annotation.note ?? '')) return;
    widget.onSaved(
      widget.annotation.copyWith(
        note: value,
        updatedAt: DateTime.now().toUtc(),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Material(
      color: widget.active
          ? Theme.of(
              context,
            ).colorScheme.primaryContainer.withValues(alpha: .35)
          : Colors.transparent,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.space8,
          vertical: AppSpacing.space4,
        ),
        child: Row(
          children: [
            Checkbox(
              value: widget.selected,
              onChanged: (v) => widget.onSelected(v ?? false),
            ),
            Container(
              width: 4,
              height: 42,
              color: Color(widget.annotation.color.argb),
            ),
            const SizedBox(width: AppSpacing.space8),
            Expanded(
              flex: 2,
              child: InkWell(
                onTap: widget.onOpen,
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    vertical: AppSpacing.space8,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        widget.title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      Text(
                        '${strings.get('annotations.type.${widget.annotation.type.name}')} · ${strings.get('annotations.page', parameters: {'page': widget.annotation.pageIndex + 1})} · ${strings.get('annotations.state.${widget.annotation.reviewState.name}')}',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              ),
            ),
            Expanded(
              flex: 3,
              child: TextField(
                controller: _note,
                maxLines: 2,
                minLines: 1,
                decoration: InputDecoration(
                  hintText: strings.get('annotations.noteHint'),
                  isDense: true,
                ),
                onChanged: (_) => _scheduleSave(),
                onEditingComplete: _flush,
              ),
            ),
            IconButton(
              tooltip: strings.get('annotations.copyReference'),
              onPressed: widget.onCopyReference,
              icon: const Icon(Icons.content_copy_rounded),
            ),
          ],
        ),
      ),
    );
  }
}
