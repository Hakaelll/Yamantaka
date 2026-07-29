final class DocumentCollection {
  const DocumentCollection({
    required this.id,
    required this.name,
    required this.icon,
    required this.sortOrder,
    required this.isPinned,
    required this.documentCount,
    required this.createdAt,
    required this.updatedAt,
  });

  final String id;
  final String name;
  final String icon;
  final int sortOrder;
  final bool isPinned;
  final int documentCount;
  final DateTime createdAt;
  final DateTime updatedAt;
}

final class DocumentTag {
  const DocumentTag({
    required this.id,
    required this.name,
    required this.color,
    required this.usageCount,
  });

  final String id;
  final String name;
  final String color;
  final int usageCount;
}

enum SmartCollection {
  inProgress('En curso'),
  notStarted('Sin empezar'),
  finished('Terminados'),
  annotated('Con anotaciones'),
  bookmarked('Con marcadores'),
  missing('Ausentes'),
  recentlyOpened('Abiertos recientemente'),
  recentlyAdded('Añadidos recientemente');

  const SmartCollection(this.label);
  final String label;
}
