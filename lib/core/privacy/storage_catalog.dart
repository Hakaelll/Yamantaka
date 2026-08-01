enum StorageCategory {
  database,
  covers,
  renderCache,
  searchIndex,
  logs,
  backups,
  preferences,
}

final class StorageEntry {
  const StorageEntry({
    required this.category,
    required this.bytes,
    required this.personal,
    required this.rebuildable,
    required this.canClear,
    required this.description,
  });

  final StorageCategory category;
  final int bytes;
  final bool personal;
  final bool rebuildable;
  final bool canClear;
  final String description;

  Map<String, Object> toJson() => {
    'category': category.name,
    'bytes': bytes,
    'personal': personal,
    'rebuildable': rebuildable,
    'can_clear': canClear,
  };
}

abstract final class StorageCatalog {
  static const descriptions = <StorageCategory, String>{
    StorageCategory.database: 'Biblioteca, posiciones y organización local.',
    StorageCategory.covers: 'Portadas recreables de los documentos.',
    StorageCategory.renderCache: 'Imágenes de lectura recreables.',
    StorageCategory.searchIndex: 'Índice reconstruible de texto y metadatos.',
    StorageCategory.logs: 'Registros técnicos redactados.',
    StorageCategory.backups: 'Copias de seguridad solicitadas por el usuario.',
    StorageCategory.preferences: 'Preferencias de la aplicación.',
  };

  static StorageEntry describe(StorageCategory category, int bytes) =>
      StorageEntry(
        category: category,
        bytes: bytes,
        personal:
            category == StorageCategory.database ||
            category == StorageCategory.backups ||
            category == StorageCategory.preferences,
        rebuildable:
            category == StorageCategory.covers ||
            category == StorageCategory.renderCache ||
            category == StorageCategory.searchIndex ||
            category == StorageCategory.logs,
        canClear:
            category != StorageCategory.database &&
            category != StorageCategory.preferences,
        description: descriptions[category]!,
      );
}
