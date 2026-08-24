import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../domain/document_cover.dart';
import '../domain/library_document.dart';

final class DocumentCoverPickerDialog extends StatefulWidget {
  const DocumentCoverPickerDialog({
    super.key,
    required this.document,
    required this.loadPreviews,
    required this.onUsePage,
    required this.onRestoreAutomatic,
  });

  final LibraryDocument document;
  final Future<List<CoverPreview>> Function() loadPreviews;
  final Future<void> Function(int pageIndex) onUsePage;
  final Future<void> Function() onRestoreAutomatic;

  @override
  State<DocumentCoverPickerDialog> createState() =>
      _DocumentCoverPickerDialogState();
}

final class _DocumentCoverPickerDialogState
    extends State<DocumentCoverPickerDialog> {
  late final Future<List<CoverPreview>> _previews = widget.loadPreviews();
  late final TextEditingController _pageController = TextEditingController(
    text: '${(widget.document.coverMetadata?.sourcePageIndex ?? 0) + 1}',
  );
  int _selectedPageIndex = 0;
  bool _saving = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _selectedPageIndex = widget.document.coverMetadata?.sourcePageIndex ?? 0;
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final metadata = widget.document.coverMetadata;
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      title: Text(strings.get('cover.title')),
      content: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680, maxHeight: 520),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              widget.document.effectiveTitle,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: AppSpacing.space16),
            Flexible(
              child: FutureBuilder<List<CoverPreview>>(
                future: _previews,
                builder: (context, snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (snapshot.hasError) {
                    return Center(
                      child: Text(strings.get('cover.previewFailed')),
                    );
                  }
                  final previews = snapshot.data ?? const [];
                  return GridView.builder(
                    shrinkWrap: true,
                    itemCount: previews.length,
                    gridDelegate:
                        const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: 6,
                          childAspectRatio: 0.68,
                          mainAxisSpacing: AppSpacing.space12,
                          crossAxisSpacing: AppSpacing.space12,
                        ),
                    itemBuilder: (context, index) {
                      final preview = previews[index];
                      final selected = preview.pageIndex == _selectedPageIndex;
                      return Semantics(
                        selected: selected,
                        button: true,
                        label: strings.get(
                          'cover.pageSemantics',
                          parameters: {
                            'page': preview.pageIndex + 1,
                            'selected': selected
                                ? strings.get('cover.selectedSuffix')
                                : '',
                          },
                        ),
                        child: InkWell(
                          onTap: () => _select(preview.pageIndex),
                          borderRadius: BorderRadius.circular(AppRadii.control),
                          child: AnimatedContainer(
                            duration: AppMotion.whenAllowed(
                              MediaQuery.disableAnimationsOf(context),
                              AppMotion.standard,
                            ),
                            padding: const EdgeInsets.all(AppSpacing.space4),
                            decoration: BoxDecoration(
                              border: Border.all(
                                color: selected
                                    ? context.appColors.focusRing
                                    : context.appColors.borderSubtle,
                                width: selected ? 2 : 1,
                              ),
                              borderRadius: BorderRadius.circular(
                                AppRadii.control,
                              ),
                            ),
                            child: Stack(
                              fit: StackFit.expand,
                              children: [
                                Image(
                                  image: MemoryImage(
                                    Uint8List.fromList(preview.pngBytes),
                                  ),
                                  fit: BoxFit.contain,
                                  filterQuality: FilterQuality.high,
                                ),
                                Positioned(
                                  left: 2,
                                  bottom: 2,
                                  child: DecoratedBox(
                                    decoration: BoxDecoration(
                                      color: context.appColors.surfaceElevated
                                          .withValues(alpha: 0.9),
                                      borderRadius: BorderRadius.circular(2),
                                    ),
                                    child: Padding(
                                      padding: const EdgeInsets.symmetric(
                                        horizontal: AppSpacing.space4,
                                        vertical: AppSpacing.space2,
                                      ),
                                      child: Text('${preview.pageIndex + 1}'),
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      );
                    },
                  );
                },
              ),
            ),
            const SizedBox(height: AppSpacing.space16),
            Row(
              children: [
                SizedBox(
                  width: 150,
                  child: TextField(
                    controller: _pageController,
                    enabled: !_saving,
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                    decoration: InputDecoration(
                      labelText: strings.get('cover.page'),
                    ),
                    onSubmitted: (_) => _selectTypedPage(),
                  ),
                ),
                const SizedBox(width: AppSpacing.space12),
                Expanded(
                  child: Text(
                    metadata == null
                        ? strings.get('cover.noneSaved')
                        : metadata.selectionMode == CoverSelectionMode.manual
                        ? strings.get(
                            'cover.currentManual',
                            parameters: {'page': metadata.sourcePageIndex + 1},
                          )
                        : strings.get(
                            'cover.currentAutomatic',
                            parameters: {'page': metadata.sourcePageIndex + 1},
                          ),
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              ],
            ),
            if (_error case final error?) ...[
              const SizedBox(height: AppSpacing.space8),
              Text(
                error,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: context.appColors.danger,
                ),
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: _saving ? null : _restoreAutomatic,
          child: Text(strings.get('cover.automaticSelection')),
        ),
        TextButton(
          onPressed: _saving ? null : () => Navigator.pop(context),
          child: Text(strings.get('action.cancel')),
        ),
        FilledButton(
          onPressed: _saving ? null : _save,
          child: _saving
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(strings.get('cover.use')),
        ),
      ],
    );
  }

  void _select(int pageIndex) {
    setState(() {
      _selectedPageIndex = pageIndex;
      _pageController.text = '${pageIndex + 1}';
      _error = null;
    });
  }

  void _selectTypedPage() {
    final page = int.tryParse(_pageController.text);
    if (page == null || page < 1 || page > widget.document.pageCount) {
      setState(
        () => _error = YamantakaLocalizations.of(
          context,
        ).get('cover.invalidPage'),
      );
      return;
    }
    _select(page - 1);
  }

  Future<void> _save() async {
    _selectTypedPage();
    if (_error != null) return;
    setState(() => _saving = true);
    try {
      await widget.onUsePage(_selectedPageIndex);
      if (mounted) Navigator.pop(context, true);
    } on Object {
      if (mounted) {
        setState(() {
          _saving = false;
          _error = YamantakaLocalizations.of(context).get('cover.saveFailed');
        });
      }
    }
  }

  Future<void> _restoreAutomatic() async {
    setState(() => _saving = true);
    try {
      await widget.onRestoreAutomatic();
      if (mounted) Navigator.pop(context, true);
    } on Object {
      if (mounted) {
        setState(() {
          _saving = false;
          _error = YamantakaLocalizations.of(
            context,
          ).get('cover.restoreFailed');
        });
      }
    }
  }
}
