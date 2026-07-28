import 'package:flutter/material.dart';

import '../application/reader_controller.dart';
import '../domain/reader_models.dart';

final class ReaderTopBar extends StatelessWidget
    implements PreferredSizeWidget {
  const ReaderTopBar({
    super.key,
    required this.controller,
    required this.onBack,
    required this.onSearch,
    required this.onCommandPalette,
    required this.onShowThumbnails,
    required this.onShowOutline,
    required this.onShowBookmarks,
    required this.onAdjustMargins,
    required this.onOpenSettings,
  });

  final ReaderController controller;
  final VoidCallback onBack;
  final VoidCallback onSearch;
  final VoidCallback onCommandPalette;
  final VoidCallback onShowThumbnails;
  final VoidCallback onShowOutline;
  final VoidCallback onShowBookmarks;
  final VoidCallback onAdjustMargins;
  final VoidCallback onOpenSettings;

  @override
  Size get preferredSize => const Size.fromHeight(48);

  @override
  Widget build(BuildContext context) {
    final preferences = controller.preferences;
    final document = controller.document;
    if (preferences == null || document == null) {
      return const SizedBox(height: 48);
    }
    return AppBar(
      leading: IconButton(
        tooltip: 'Volver a la biblioteca',
        onPressed: onBack,
        icon: const Icon(Icons.arrow_back_rounded),
      ),
      titleSpacing: 0,
      title: Text(
        document.fileName,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      actions: [
        IconButton(
          tooltip: 'Buscar (Ctrl+F)',
          onPressed: onSearch,
          icon: const Icon(Icons.search_rounded),
        ),
        SegmentedButton<ReaderMode>(
          segments: const [
            ButtonSegment(
              value: ReaderMode.document,
              icon: Icon(Icons.description_outlined, size: 18),
              tooltip: 'Modo documento',
            ),
            ButtonSegment(
              value: ReaderMode.book,
              icon: Icon(Icons.menu_book_outlined, size: 18),
              tooltip: 'Modo libro',
            ),
          ],
          selected: {preferences.viewMode},
          showSelectedIcon: false,
          onSelectionChanged: (selection) =>
              controller.setReaderMode(selection.first),
          style: const ButtonStyle(
            visualDensity: VisualDensity.compact,
            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          ),
        ),
        const SizedBox(width: 4),
        IconButton(
          tooltip: 'Paleta de comandos (Ctrl+K)',
          onPressed: onCommandPalette,
          icon: const Icon(Icons.keyboard_command_key_rounded),
        ),
        PopupMenuButton<_ReaderMenuAction>(
          tooltip: 'Más opciones',
          onSelected: (action) {
            switch (action) {
              case _ReaderMenuAction.thumbnails:
                onShowThumbnails();
              case _ReaderMenuAction.outline:
                onShowOutline();
              case _ReaderMenuAction.bookmarks:
                onShowBookmarks();
              case _ReaderMenuAction.rotate:
                controller.rotateClockwise();
              case _ReaderMenuAction.margins:
                onAdjustMargins();
              case _ReaderMenuAction.fullscreen:
                controller.toggleFullScreen();
              case _ReaderMenuAction.settings:
                onOpenSettings();
            }
          },
          itemBuilder: (context) => const [
            PopupMenuItem(
              value: _ReaderMenuAction.thumbnails,
              child: Text('Miniaturas'),
            ),
            PopupMenuItem(
              value: _ReaderMenuAction.outline,
              child: Text('Índice'),
            ),
            PopupMenuItem(
              value: _ReaderMenuAction.bookmarks,
              child: Text('Marcadores'),
            ),
            PopupMenuDivider(),
            PopupMenuItem(
              value: _ReaderMenuAction.rotate,
              child: Text('Girar página'),
            ),
            PopupMenuItem(
              value: _ReaderMenuAction.margins,
              child: Text('Márgenes de lectura'),
            ),
            PopupMenuItem(
              value: _ReaderMenuAction.fullscreen,
              child: Text('Pantalla completa'),
            ),
            PopupMenuItem(
              value: _ReaderMenuAction.settings,
              child: Text('Configuración'),
            ),
          ],
        ),
        const SizedBox(width: 4),
      ],
    );
  }
}

enum _ReaderMenuAction {
  thumbnails,
  outline,
  bookmarks,
  rotate,
  margins,
  fullscreen,
  settings,
}
