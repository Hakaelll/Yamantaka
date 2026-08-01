import '../../reader/domain/reader_models.dart';

enum UnifiedNavigationKind {
  page,
  readingUnit,
  outline,
  bookmark,
  search,
  thumbnail,
  history,
  internalLink,
}

final class UnifiedNavigationTarget {
  const UnifiedNavigationTarget({
    required this.pageIndex,
    required this.kind,
    this.offset = 0,
    this.label = '',
  });

  final int pageIndex;
  final UnifiedNavigationKind kind;
  final double offset;
  final String label;

  UnifiedNavigationTarget canonical({
    required int pageCount,
    required ReaderMode mode,
  }) => UnifiedNavigationTarget(
    pageIndex: ReadingUnitNavigation.canonicalPage(
      pageIndex: pageIndex,
      pageCount: pageCount,
      mode: mode,
    ),
    kind: kind,
    offset: offset.clamp(0, double.infinity).toDouble(),
    label: label,
  );
}

final class NavigationCommand {
  const NavigationCommand({
    required this.target,
    required this.generation,
    required this.origin,
  });

  final UnifiedNavigationTarget target;
  final int generation;
  final String origin;

  bool isCurrent(int currentGeneration) => generation == currentGeneration;
}

final class UnifiedNavigationState {
  const UnifiedNavigationState({this.generation = 0, this.current});

  final int generation;
  final UnifiedNavigationTarget? current;

  UnifiedNavigationState next(UnifiedNavigationTarget target) =>
      UnifiedNavigationState(generation: generation + 1, current: target);
}
