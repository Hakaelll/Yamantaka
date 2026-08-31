import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../../../storage/repositories/document_relation_repository.dart';
import '../../library/domain/library_document.dart';
import '../domain/document_relation.dart';

final class DocumentRelationsDialog extends StatefulWidget {
  const DocumentRelationsDialog({
    super.key,
    required this.repository,
    required this.document,
    required this.candidates,
    required this.onOpenDocument,
  });

  final DocumentRelationRepository repository;
  final LibraryDocument document;
  final List<LibraryDocument> candidates;
  final ValueChanged<String> onOpenDocument;

  @override
  State<DocumentRelationsDialog> createState() =>
      _DocumentRelationsDialogState();
}

final class _DocumentRelationsDialogState
    extends State<DocumentRelationsDialog> {
  List<DocumentRelationView> _relations = const [];
  List<DocumentRevisionGroup> _revisions = const [];

  @override
  void initState() {
    super.initState();
    _reload();
  }

  void _reload() => setState(() {
    _relations = widget.repository.forDocument(widget.document.id);
    _revisions = widget.repository.revisionGroupsForDocument(
      widget.document.id,
    );
  });

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 720, maxHeight: 680),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(AppSpacing.space16),
              child: Row(
                children: [
                  const Icon(Icons.hub_outlined),
                  const SizedBox(width: AppSpacing.space8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          strings.get('relations.title'),
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                        Text(
                          widget.document.effectiveTitle,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  FilledButton.tonalIcon(
                    onPressed: widget.candidates.isEmpty ? null : _addRelation,
                    icon: const Icon(Icons.add_link_rounded, size: 18),
                    label: Text(strings.get('relations.add')),
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
            Expanded(
              child: ListView(
                children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                    child: Text(
                      strings.get('relations.links'),
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                  ),
                  if (_relations.isEmpty)
                    ListTile(title: Text(strings.get('relations.empty')))
                  else
                    for (final relation in _relations)
                      ListTile(
                        leading: Icon(_relationIcon(relation)),
                        title: Text(
                          relation.otherDocumentTitle,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        subtitle: Text(_relationLabel(strings, relation)),
                        onTap: () =>
                            widget.onOpenDocument(relation.otherDocumentId),
                        trailing: IconButton(
                          tooltip: strings.get('relations.remove'),
                          onPressed: () {
                            widget.repository.remove(relation.relation.id);
                            _reload();
                          },
                          icon: const Icon(Icons.link_off_rounded),
                        ),
                      ),
                  const Divider(),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 4, 16, 4),
                    child: Row(
                      children: [
                        Expanded(
                          child: Text(
                            strings.get('relations.revisions'),
                            style: Theme.of(context).textTheme.titleSmall,
                          ),
                        ),
                        TextButton.icon(
                          onPressed: widget.candidates.isEmpty
                              ? null
                              : _createRevisionGroup,
                          icon: const Icon(Icons.library_add_outlined),
                          label: Text(strings.get('relations.groupRevisions')),
                        ),
                      ],
                    ),
                  ),
                  if (_revisions.isEmpty)
                    ListTile(title: Text(strings.get('relations.noRevisions')))
                  else
                    for (final group in _revisions)
                      ExpansionTile(
                        leading: const Icon(Icons.difference_outlined),
                        title: Text(group.name),
                        subtitle: Text(
                          strings.get(
                            'relations.revisionCount',
                            parameters: {'count': group.documentIds.length},
                          ),
                        ),
                        children: [
                          RadioGroup<String>(
                            groupValue: group.currentDocumentId,
                            onChanged: (value) {
                              if (value == null) return;
                              widget.repository.setCurrentRevision(
                                group.id,
                                value,
                              );
                              _reload();
                            },
                            child: Column(
                              children: [
                                for (final documentId in group.documentIds)
                                  RadioListTile<String>(
                                    value: documentId,
                                    title: Text(_titleFor(documentId)),
                                    subtitle: Text(
                                      strings.get('relations.currentRevision'),
                                    ),
                                    secondary: IconButton(
                                      tooltip: strings.get('action.open'),
                                      onPressed: () =>
                                          widget.onOpenDocument(documentId),
                                      icon: const Icon(
                                        Icons.open_in_new_rounded,
                                      ),
                                    ),
                                  ),
                              ],
                            ),
                          ),
                        ],
                      ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _addRelation() async {
    final strings = YamantakaLocalizations.of(context);
    LibraryDocument target = widget.candidates.first;
    var type = DocumentRelationType.related;
    final accepted = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(strings.get('relations.add')),
          content: SizedBox(
            width: 480,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButtonFormField<LibraryDocument>(
                  initialValue: target,
                  decoration: InputDecoration(
                    labelText: strings.get('relations.document'),
                  ),
                  items: [
                    for (final document in widget.candidates.take(80))
                      DropdownMenuItem(
                        value: document,
                        child: SizedBox(
                          width: 360,
                          child: Text(
                            document.effectiveTitle,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ),
                  ],
                  onChanged: (value) => target = value ?? target,
                ),
                const SizedBox(height: AppSpacing.space12),
                DropdownButtonFormField<DocumentRelationType>(
                  initialValue: type,
                  decoration: InputDecoration(
                    labelText: strings.get('relations.type'),
                  ),
                  items: [
                    for (final value in DocumentRelationType.values)
                      DropdownMenuItem(
                        value: value,
                        child: Text(
                          strings.get('relations.type.${value.name}'),
                        ),
                      ),
                  ],
                  onChanged: (value) =>
                      setDialogState(() => type = value ?? type),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(strings.get('action.cancel')),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(strings.get('relations.add')),
            ),
          ],
        ),
      ),
    );
    if (accepted != true) return;
    widget.repository.create(
      sourceDocumentId: widget.document.id,
      targetDocumentId: target.id,
      type: type,
    );
    _reload();
  }

  Future<void> _createRevisionGroup() async {
    final strings = YamantakaLocalizations.of(context);
    final name = TextEditingController(
      text: strings.get(
        'relations.defaultRevisionName',
        parameters: {'title': widget.document.effectiveTitle},
      ),
    );
    LibraryDocument target = widget.candidates.first;
    final accepted = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.get('relations.groupRevisions')),
        content: SizedBox(
          width: 480,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: name,
                maxLength: 120,
                decoration: InputDecoration(
                  labelText: strings.get('relations.groupName'),
                ),
              ),
              DropdownButtonFormField<LibraryDocument>(
                initialValue: target,
                items: [
                  for (final document in widget.candidates.take(80))
                    DropdownMenuItem(
                      value: document,
                      child: SizedBox(
                        width: 360,
                        child: Text(
                          document.effectiveTitle,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ),
                ],
                onChanged: (value) => target = value ?? target,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.get('relations.groupRevisions')),
          ),
        ],
      ),
    );
    final cleanName = name.text.trim();
    name.dispose();
    if (accepted != true || cleanName.isEmpty) return;
    widget.repository.createRevisionGroup(
      name: cleanName,
      documentIds: [widget.document.id, target.id],
      currentDocumentId: widget.document.id,
    );
    _reload();
  }

  String _titleFor(String id) {
    if (id == widget.document.id) return widget.document.effectiveTitle;
    return widget.candidates
            .where((document) => document.id == id)
            .firstOrNull
            ?.effectiveTitle ??
        id;
  }

  static IconData _relationIcon(DocumentRelationView view) => view.isReplacedBy
      ? Icons.history_rounded
      : switch (view.relation.type) {
          DocumentRelationType.related => Icons.link_rounded,
          DocumentRelationType.replaces => Icons.upgrade_rounded,
          DocumentRelationType.sourceOf => Icons.source_outlined,
          DocumentRelationType.attachmentOf => Icons.attach_file_rounded,
        };

  static String _relationLabel(
    YamantakaLocalizations strings,
    DocumentRelationView view,
  ) => strings.get(
    view.isReplacedBy
        ? 'relations.type.replacedBy'
        : 'relations.type.${view.relation.type.name}',
  );
}
