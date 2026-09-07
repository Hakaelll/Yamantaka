part of 'library_screen.dart';

/// Navigation delegates to the paginated catalog; it never filters a page in UI.
final class _LibraryNavigation extends StatelessWidget {
  const _LibraryNavigation({
    required this.controller,
    required this.collections,
    required this.onReadingStateChanged,
    required this.onOpenSettings,
    this.onCollections,
    this.onWorkspaces,
    this.onGlobalSearch,
    this.onAnnotations,
    this.onProblems,
  });

  final LibraryController controller;
  final List<DocumentCollection> collections;
  final ValueChanged<LibraryReadingState> onReadingStateChanged;
  final VoidCallback onOpenSettings;
  final VoidCallback? onCollections;
  final VoidCallback? onWorkspaces;
  final VoidCallback? onGlobalSearch;
  final VoidCallback? onAnnotations;
  final VoidCallback? onProblems;

  void _filter(
    LibraryReadingState state, {
    bool favorites = false,
    String? collection,
  }) {
    controller.readingState = state.name;
    controller.favoritesOnly = favorites;
    controller.selectedCollectionId = collection;
    controller.selectedSmartCollection = null;
    controller.scrollOffset = 0;
    onReadingStateChanged(state);
    unawaited(controller.reload());
  }

  @override
  Widget build(BuildContext context) {
    final strings = YamantakaLocalizations.of(context);
    final colors = context.appColors;
    Widget item(
      String label,
      IconData icon,
      VoidCallback? action, {
      bool selected = false,
      String? id,
    }) => Padding(
      padding: const EdgeInsets.only(bottom: AppSpacing.space4),
      child: Material(
        color: Colors.transparent,
        child: ListTile(
          key: id == null ? null : ValueKey('library-nav-$id'),
          selected: selected,
          selectedColor: colors.accentPrimary,
          selectedTileColor: colors.selectionBackground,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(AppRadii.control),
          ),
          leading: Icon(icon, size: 20),
          title: Text(
            label,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.labelLarge,
          ),
          onTap: action,
          minLeadingWidth: 20,
          horizontalTitleGap: AppSpacing.space12,
          contentPadding: const EdgeInsets.symmetric(
            horizontal: 12,
            vertical: 4,
          ),
        ),
      ),
    );
    Widget heading(String label) => Padding(
      padding: const EdgeInsets.fromLTRB(12, 24, 12, 10),
      child: Text(label, style: Theme.of(context).textTheme.labelSmall),
    );
    final plain =
        !controller.favoritesOnly &&
        controller.selectedCollectionId == null &&
        controller.selectedSmartCollection == null;
    return Container(
      key: const Key('library-navigation'),
      width: (220 * MediaQuery.textScalerOf(context).scale(1).clamp(1, 1.25)),
      decoration: BoxDecoration(
        color: colors.surfaceSecondary,
        border: BorderDirectional(end: BorderSide(color: colors.borderSubtle)),
      ),
      child: ListView(
        padding: const EdgeInsets.fromLTRB(12, 24, 12, 24),
        children: [
          item(
            strings.libraryTitle,
            Icons.library_books_outlined,
            () => _filter(LibraryReadingState.all),
            selected: plain && controller.readingState == 'all',
            id: 'all',
          ),
          item(
            strings.get('library.navigation.reading'),
            Icons.menu_book_outlined,
            () => _filter(LibraryReadingState.inProgress),
            selected: plain && controller.readingState == 'inProgress',
            id: 'reading',
          ),
          item(
            strings.get('library.navigation.favorites'),
            Icons.bookmark_outline_rounded,
            () => _filter(LibraryReadingState.all, favorites: true),
            selected: controller.favoritesOnly,
            id: 'favorites',
          ),
          item(
            strings.get('library.readingState.completed'),
            Icons.task_alt_rounded,
            () => _filter(LibraryReadingState.completed),
            selected: plain && controller.readingState == 'completed',
            id: 'completed',
          ),
          if (onGlobalSearch != null)
            item(
              strings.get('library.globalSearch'),
              Icons.search_rounded,
              onGlobalSearch,
            ),
          if (onCollections != null ||
              onWorkspaces != null ||
              collections.isNotEmpty) ...[
            heading(strings.get('library.collections')),
            for (final collection in collections.take(12))
              item(
                collection.name,
                Icons.folder_outlined,
                () =>
                    _filter(LibraryReadingState.all, collection: collection.id),
                selected: controller.selectedCollectionId == collection.id,
              ),
            if (onCollections != null)
              item(
                strings.get('library.collections'),
                Icons.create_new_folder_outlined,
                onCollections,
              ),
            if (onWorkspaces != null)
              item(
                strings.get('library.workspaces'),
                Icons.space_dashboard_outlined,
                onWorkspaces,
              ),
          ],
          heading(strings.get('library.navigation.tools')),
          if (onAnnotations != null)
            item(
              strings.get('annotations.globalTitle'),
              Icons.edit_note_rounded,
              onAnnotations,
            ),
          if (onProblems != null)
            item(
              strings.get('problems.title'),
              Icons.health_and_safety_outlined,
              onProblems,
            ),
          item(strings.settingsTitle, Icons.settings_outlined, onOpenSettings),
        ],
      ),
    );
  }
}
