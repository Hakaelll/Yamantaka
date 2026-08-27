enum PageDestinationError { empty, invalidFormat, outOfRange }

final class PageDestination {
  const PageDestination._({this.page, this.error});

  final int? page;
  final PageDestinationError? error;

  bool get isValid => page != null && error == null;

  static PageDestination parse(String input, {required int pageCount}) {
    final value = input.trim().toLowerCase();
    if (value.isEmpty) {
      return const PageDestination._(error: PageDestinationError.empty);
    }
    if (pageCount < 1) {
      return const PageDestination._(error: PageDestinationError.outOfRange);
    }
    if (const {
      'first',
      'start',
      'inicio',
      'primera',
      'primero',
    }.contains(value)) {
      return const PageDestination._(page: 1);
    }
    if (const {
      'last',
      'end',
      'fin',
      'ultima',
      'última',
      'ultimo',
      'último',
    }.contains(value)) {
      return PageDestination._(page: pageCount);
    }
    if (value.endsWith('%')) {
      final percentage = double.tryParse(
        value.substring(0, value.length - 1).trim().replaceAll(',', '.'),
      );
      if (percentage == null) {
        return const PageDestination._(
          error: PageDestinationError.invalidFormat,
        );
      }
      if (percentage < 0 || percentage > 100) {
        return const PageDestination._(error: PageDestinationError.outOfRange);
      }
      final page = percentage == 0
          ? 1
          : (pageCount * percentage / 100).ceil().clamp(1, pageCount);
      return PageDestination._(page: page);
    }
    final page = int.tryParse(value);
    if (page == null) {
      return const PageDestination._(error: PageDestinationError.invalidFormat);
    }
    if (page < 1 || page > pageCount) {
      return const PageDestination._(error: PageDestinationError.outOfRange);
    }
    return PageDestination._(page: page);
  }
}
