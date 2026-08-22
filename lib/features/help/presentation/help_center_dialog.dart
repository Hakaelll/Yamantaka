import 'package:flutter/material.dart';

import '../../../app/theme/app_tokens.dart';
import '../../../core/i18n/app_localizations.dart';
import '../domain/help_content.dart';

final class HelpCenterDialog extends StatefulWidget {
  const HelpCenterDialog({super.key, required this.language});

  final AppLanguage language;

  @override
  State<HelpCenterDialog> createState() => _HelpCenterDialogState();
}

final class _HelpCenterDialogState extends State<HelpCenterDialog> {
  final TextEditingController _query = TextEditingController();
  HelpTopic? _expanded;

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final articles = HelpContent.search(widget.language, _query.text);
    final strings = YamantakaLocalizations.of(context);
    return AlertDialog(
      icon: const Icon(Icons.help_outline_rounded),
      title: Text(strings.get('help.title')),
      content: SizedBox(
        width: 620,
        height: 520,
        child: Column(
          children: [
            TextField(
              controller: _query,
              autofocus: true,
              decoration: InputDecoration(
                labelText: strings.get('help.search'),
                prefixIcon: const Icon(Icons.search_rounded),
                suffixIcon: _query.text.isEmpty
                    ? null
                    : IconButton(
                        tooltip: strings.get('action.clear'),
                        onPressed: () {
                          _query.clear();
                          setState(() {});
                        },
                        icon: const Icon(Icons.clear_rounded),
                      ),
              ),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: AppSpacing.space12),
            Expanded(
              child: articles.isEmpty
                  ? Center(child: Text(strings.get('help.noResults')))
                  : ListView.builder(
                      itemCount: articles.length,
                      itemBuilder: (context, index) {
                        final article = articles[index];
                        final expanded = _expanded == article.topic;
                        return Column(
                          children: [
                            Semantics(
                              expanded: expanded,
                              child: ExpansionTile(
                                initiallyExpanded: expanded,
                                onExpansionChanged: (value) => setState(
                                  () =>
                                      _expanded = value ? article.topic : null,
                                ),
                                title: Text(article.title),
                                subtitle: Text(article.summary),
                                childrenPadding: const EdgeInsets.fromLTRB(
                                  AppSpacing.space16,
                                  0,
                                  AppSpacing.space16,
                                  AppSpacing.space16,
                                ),
                                children: [
                                  Align(
                                    alignment: Alignment.centerLeft,
                                    child: Text(article.body),
                                  ),
                                ],
                              ),
                            ),
                            const Divider(height: 1),
                          ],
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
      actions: [
        FilledButton(
          onPressed: () => Navigator.pop(context),
          child: Text(strings.get('action.close')),
        ),
      ],
    );
  }
}
