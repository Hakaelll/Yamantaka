import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

enum AppLanguage { spanish, english }

extension AppLanguageCode on AppLanguage {
  String get code => switch (this) {
    AppLanguage.spanish => 'es',
    AppLanguage.english => 'en',
  };

  static AppLanguage fromCode(String? code) =>
      code?.toLowerCase() == 'en' ? AppLanguage.english : AppLanguage.spanish;
}

/// Small typed catalog. Feature-specific copy can be added without allowing
/// a missing key to leak into a visible surface.
final class YamantakaLocalizations {
  const YamantakaLocalizations(this.language);

  final AppLanguage language;

  static const LocalizationsDelegate<YamantakaLocalizations> delegate =
      _YamantakaLocalizationsDelegate();

  static YamantakaLocalizations of(BuildContext context) =>
      Localizations.of<YamantakaLocalizations>(
        context,
        YamantakaLocalizations,
      ) ??
      const YamantakaLocalizations(AppLanguage.spanish);

  String get libraryTitle => get('library.title');
  String get libraryOpenPdf => get('library.openPdf');
  String get libraryEmptyTitle => get('library.emptyTitle');
  String get libraryEmptyBody => get('library.emptyBody');
  String get librarySearch => get('library.search');
  String get libraryNoResults => get('library.noResults');
  String get libraryCheckFiles => get('library.checkFiles');
  String get libraryCheckTitle => get('library.checkTitle');
  String get readerLoadingTitle => get('reader.loadingTitle');
  String get readerLoadingBody => get('reader.loadingBody');
  String get readerTitle => get('reader.title');
  String get settingsTitle => get('settings.title');
  String get settingsBack => get('settings.back');
  String get searchTitle => get('search.title');
  String get searchField => get('search.field');
  String get searchEmptyTitle => get('search.emptyTitle');
  String get searchEmptyBody => get('search.emptyBody');
  String get searchNoResultsTitle => get('search.noResultsTitle');
  String get searchNoResultsBody => get('search.noResultsBody');
  String get commandSearch => get('command.searchHint');
  String get commandSection => get('command.section');
  String get commandMove => get('command.move');
  String get commandRun => get('command.run');
  String get commandEmpty => get('command.empty');

  String fileCheckSummary({
    required int available,
    required int missing,
    required int modified,
    required int failed,
    required int cancelled,
  }) => get(
    'library.checkSummary',
    parameters: {
      'available': available,
      'missing': missing,
      'modified': modified,
      'failed': failed,
      'cancelled': cancelled,
    },
  );

  @visibleForTesting
  static Set<String> catalogKeys(AppLanguage language) =>
      Set.unmodifiable(_catalog[language]!.keys);

  String get(String key, {Map<String, Object?> parameters = const {}}) {
    final value =
        _catalog[language]?[key] ?? _catalog[AppLanguage.spanish]![key] ?? key;
    return value.replaceAllMapped(RegExp(r'\{(\w+)\}'), (match) {
      final parameter = parameters[match.group(1)];
      return parameter?.toString() ?? match.group(0)!;
    });
  }

  String count(
    String key,
    int value, {
    Map<String, Object?> parameters = const {},
  }) => get(
    value == 1 ? '$key.one' : '$key.other',
    parameters: {'count': value, ...parameters},
  );

  static const Map<AppLanguage, Map<String, String>> _catalog = {
    AppLanguage.spanish: {
      'app.title': 'Yamantaka',
      'app.openLimit':
          'Se abrieron {count} archivos; el resto se omitió para respetar el '
          'límite de recursos.',
      'action.open': 'Abrir',
      'action.import': 'Importar',
      'action.cancel': 'Cancelar',
      'action.retry': 'Reintentar',
      'action.close': 'Cerrar',
      'action.save': 'Guardar',
      'action.reset': 'Restablecer',
      'action.clear': 'Limpiar',
      'action.loadMore': 'Cargar más',
      'action.delete': 'Eliminar',
      'action.deleteAll': 'Eliminar todo',
      'action.export': 'Exportar',
      'action.undo': 'Deshacer',
      'action.apply': 'Aplicar',
      'action.go': 'Ir',
      'action.create': 'Crear',
      'action.pause': 'Pausar',
      'action.resume': 'Reanudar',
      'action.continue': 'Continuar',
      'action.reread': 'Releer',
      'action.refresh': 'Actualizar',
      'library.title': 'Biblioteca',
      'library.localCollection': 'Colección local',
      'library.openPdf': 'Abrir PDF',
      'library.emptyTitle': 'Una biblioteca tranquila',
      'library.emptyBody':
          'Abre un PDF o arrástralo a esta ventana. El archivo permanece en '
          'su carpeta original y tu progreso se guarda solo aquí.',
      'library.search': 'Buscar en la biblioteca',
      'library.noResults': 'No hay documentos con estos filtros.',
      'library.noQueryResults':
          'No hay documentos que coincidan con “{query}”.',
      'library.closeSearch': 'Cerrar búsqueda (Escape)',
      'library.checkFiles': 'Comprobar archivos',
      'library.checkTitle': 'Comprobación de archivos',
      'library.checkSummary':
          'Disponibles: {available}\nAusentes: {missing}\nModificados: '
          '{modified}\nFallidos: {failed}\nCancelados: {cancelled}',
      'library.empty': 'Aún no hay documentos locales.',
      'library.noPdfsDeleted': 'Los PDF no se borran desde esta acción.',
      'library.analyzing': 'Analizando {current} de {total}…',
      'library.cancelAnalysis': 'Cancelar análisis local',
      'library.safeMode':
          'Modo seguro: la sesión y el índice no se restauraron',
      'library.organizeMore': 'Organizar y más acciones',
      'library.collections': 'Colecciones',
      'library.workspaces': 'Espacios de trabajo',
      'library.importFolder': 'Importar carpeta',
      'library.recoverMoved': 'Recuperar documentos movidos',
      'library.globalSearch': 'Búsqueda global',
      'library.analyzeMetadata': 'Analizar metadatos y temas',
      'library.reanalyze': 'Volver a analizar la biblioteca',
      'library.dropPdf': 'Suelta el PDF para abrirlo',
      'library.reanalyze.description':
          'Se volverán a detectar localmente títulos, autores y temas. Los '
          'títulos, autores y temas editados manualmente se conservarán.',
      'library.analyze': 'Analizar',
      'library.allLibrary': 'Toda la biblioteca',
      'library.newCollection': 'Nueva colección',
      'library.name': 'Nombre',
      'library.smartCollection.inProgress': 'En curso',
      'library.smartCollection.notStarted': 'Sin empezar',
      'library.smartCollection.finished': 'Terminados',
      'library.smartCollection.annotated': 'Con anotaciones',
      'library.smartCollection.bookmarked': 'Con marcadores',
      'library.smartCollection.missing': 'Ausentes',
      'library.smartCollection.recentlyOpened': 'Abiertos recientemente',
      'library.smartCollection.recentlyAdded': 'Añadidos recientemente',
      'library.preparing': 'Preparando tu biblioteca',
      'library.preparing.description':
          'Revisando documentos y portadas guardadas en este equipo.',
      'library.missingDocumentSemantics':
          '{file}, archivo no encontrado, localizar',
      'library.percent': '{value} por ciento',
      'library.fileNotFound': 'Archivo no encontrado',
      'library.fileChanged': 'El archivo ha cambiado',
      'library.locate': 'Localizar',
      'library.finished': 'Terminado',
      'library.inProgress': 'En curso · {value} %',
      'library.notStarted': 'Sin empezar',
      'library.unknownAuthor': 'Autor desconocido',
      'library.analysisPending': 'Análisis pendiente',
      'library.analysisInProgress': 'Análisis local en curso',
      'library.analysisFailed': 'Error de análisis',
      'library.progressValue': 'Progreso: {value} por ciento',
      'time.now': 'ahora',
      'time.minutesAgo': 'hace {count} min',
      'time.hoursAgo': 'hace {count} h',
      'time.daysAgo': 'hace {count} d',
      'library.resultOfTotal': '{result} de {total}',
      'library.commandBar': 'Buscar, seleccionar, filtrar y ordenar',
      'library.exitSelectionMode': 'Salir del modo de selección',
      'library.selectMultiple': 'Seleccionar varios documentos',
      'library.filterAuthorsTopics': 'Filtrar por autores y temas',
      'library.authorsTopicsGrouping': 'Autores, temas y agrupación',
      'library.filterSort': 'Filtrar y ordenar',
      'library.state': 'Estado',
      'library.readingState.all': 'Todos',
      'library.readingState.inProgress': 'En curso',
      'library.readingState.notStarted': 'Sin empezar',
      'library.readingState.completed': 'Terminados',
      'library.favoritesOnly': 'Solo favoritos',
      'library.sort': 'Orden',
      'library.sortOrder.lastOpened': 'Última apertura',
      'library.sortOrder.dateAdded': 'Fecha de incorporación',
      'library.sortOrder.title': 'Título',
      'library.sortOrder.progress': 'Progreso',
      'library.resetFilters': 'Restablecer filtros',
      'library.filtersApplied': '{count} filtros aplicados',
      'library.actionsForSelection':
          'Acciones para {count} documentos seleccionados',
      'library.bulkActions': 'Acciones masivas',
      'library.markFavorite': 'Marcar como favorito',
      'library.unmarkFavorite': 'Quitar de favoritos',
      'library.assignTag': 'Asignar etiqueta',
      'library.addToCollection': 'Añadir a colección',
      'library.removeFromCollection': 'Quitar de colección',
      'library.archive': 'Archivar',
      'library.regenerateCovers': 'Regenerar portadas',
      'library.removeFromLibrary': 'Retirar de la biblioteca',
      'library.clearSelection': 'Limpiar selección',
      'library.artifactSelectionHint':
          'Un clic alterna la selección; Ctrl y Shift amplían la selección',
      'library.artifactOpenHint':
          'Un clic selecciona; doble clic o Enter abre; menú contextual',
      'library.favorite': 'Favorito',
      'library.finishedPages': 'Terminado · {count} páginas',
      'library.notStartedPages': 'Sin empezar · {count} páginas',
      'library.pageProgress': 'Página {page} de {count} · {progress} %',
      'library.metadataNeedsReview': 'Metadatos pendientes de revisión',
      'library.documentUnavailable':
          'Documento no disponible · localizar para continuar',
      'library.artifactMissing': '{title}, archivo no encontrado',
      'library.artifactFinished': '{title}, terminado',
      'library.artifactProgress': '{title}, {progress} por ciento',
      'library.artifactNotStarted': '{title}, sin empezar',
      'time.openedNow': 'abierto ahora',
      'time.openedToday': 'abierto hoy',
      'time.openedYesterday': 'abierto ayer',
      'time.openedDaysAgo': 'abierto hace {count} días',
      'time.openedDate': 'abierto {day}/{month}',
      'library.selectionActivated':
          'Modo de selección activado. Seleccionados {count}.',
      'library.bulkFailed':
          'La acción no se pudo completar. No se ha tocado ningún PDF.',
      'library.noCollections': 'No hay colecciones creadas.',
      'library.collectionSemantics': '{name}, {count} documentos',
      'library.noTags': 'No hay etiquetas creadas.',
      'library.tagSemantics': '{name}, {count} usos',
      'library.removeDocuments': 'Retirar documentos',
      'library.removeDocuments.description':
          'Se retirarán {count} documentos de la biblioteca. Los PDF '
          'originales no se borrarán.',
      'library.remove': 'Retirar',
      'library.actionCompleted': 'Acción completada',
      'library.bulkSummary': '{succeeded} correctos · {failed} con error',
      'library.selectedDocument': 'Documento seleccionado',
      'library.closeContext': 'Cerrar contexto del documento',
      'library.selectionFiltered':
          'El documento seleccionado ya no pertenece a este resultado.',
      'library.showSelectionAgain': 'Volver a mostrarlo',
      'library.operationFailed': 'La operación no se pudo completar.',
      'library.locateFile': 'Localizar archivo',
      'library.continueReading': 'Continuar leyendo',
      'library.changeCover': 'Cambiar portada',
      'library.reanalyzeMetadata': 'Volver a analizar metadatos y temas',
      'library.showInformation': 'Mostrar información',
      'library.showInExplorer': 'Mostrar en el explorador',
      'library.authorsAndTopics': 'Autores y temas',
      'library.groupByAuthor': 'Agrupar por autor',
      'library.groupByAuthor.description':
          'Crea secciones dinámicas sin modificar colecciones.',
      'library.authors': 'Autores',
      'library.noAuthors': 'Todavía no hay autores detectados.',
      'library.topics': 'Temas',
      'library.analyzeFolder': 'Analizar carpeta',
      'library.importSummary':
          'Importados {imported}; {duplicates} duplicados; {failed} con error.',
      'library.searchDocuments': 'Buscar documentos',
      'library.searchingLocalMatches': 'Buscando coincidencias locales…',
      'library.recoverySummary':
          'Recuperados {recovered}; {ambiguous} ambiguos; {modified} '
          'modificados; {notFound} no encontrados.',
      'library.relocationMismatch':
          'El PDF seleccionado no coincide con el documento ausente. No se ha '
          'modificado la biblioteca.',
      'library.removedFromFavorites': 'Quitado de favoritos.',
      'library.addedToFavorites': 'Añadido a favoritos.',
      'library.coverUpdated': 'Portada actualizada.',
      'library.removeFromLibrary.description':
          'Se eliminarán el progreso, marcadores y sesiones locales. El '
          'archivo PDF no se borrará.',
      'library.documentRemoved': 'Documento retirado. El PDF no se ha borrado.',
      'workspace.restoredMissing':
          'Espacio restaurado; {count} documentos no están disponibles.',
      'workspace.title': 'Espacios de trabajo',
      'workspace.saveCurrent': 'Guardar actual',
      'workspace.emptyTitle': 'Sin espacios guardados',
      'workspace.emptyBody':
          'Guarda los documentos, posiciones y panel actuales para recuperar '
          'esta sesión de lectura.',
      'workspace.summary': '{documents} · {missing}',
      'workspace.documents.one': '{count} documento',
      'workspace.documents.other': '{count} documentos',
      'workspace.missing.none': 'todos disponibles',
      'workspace.missing.one': '{count} ausente',
      'workspace.missing.other': '{count} ausentes',
      'workspace.actions': 'Acciones del espacio',
      'workspace.duplicate': 'Duplicar',
      'workspace.saveTitle': 'Guardar espacio',
      'workspace.duplicateTitle': 'Duplicar espacio',
      'workspace.copySuffix': '{name} — copia',
      'workspace.deleteTitle': 'Eliminar espacio',
      'workspace.deleteBody':
          'Se elimina la sesión guardada, no los documentos ni sus '
          'anotaciones.',
      'workspace.name': 'Nombre',
      'diagnostics.title': 'Diagnóstico local',
      'diagnostics.failedTitle': 'No se pudo generar el diagnóstico',
      'diagnostics.failedBody': 'Los datos locales no se han modificado.',
      'diagnostics.safeMode':
          'Modo seguro activo: no se restauró la sesión ni se inició el índice.',
      'diagnostics.metrics': '{count} métricas',
      'diagnostics.copySafe': 'Copiar informe seguro',
      'diagnostics.maintenanceLocal': 'Mantenimiento local',
      'diagnostics.verifyDatabase': 'Verificar base de datos',
      'diagnostics.optimizeDatabase': 'Optimizar base de datos',
      'diagnostics.clearCaches': 'Limpiar cachés',
      'diagnostics.rebuildIndex': 'Reconstruir índice',
      'diagnostics.openDataFolder': 'Abrir carpeta de datos',
      'diagnostics.maintenance': 'Mantenimiento',
      'diagnostics.copied': 'Diagnóstico copiado.',
      'diagnostics.exported': 'Diagnóstico exportado.',
      'diagnostics.writeFailed': 'No se pudo escribir el informe.',
      'diagnostics.integrityOk': 'Integridad correcta.',
      'diagnostics.result': 'Resultado: {result}',
      'diagnostics.optimized': 'Base de datos optimizada.',
      'diagnostics.clearCachesBody': 'Los PDF no se modificarán.',
      'diagnostics.cachesCleared': 'Cachés eliminadas.',
      'diagnostics.rebuildIndexBody':
          'Se eliminará y recreará solo el índice local.',
      'diagnostics.indexRebuilt': 'Índice reconstruido.',
      'cover.title': 'Cambiar portada',
      'cover.previewFailed': 'No se pudieron preparar las páginas.',
      'cover.page': 'Página',
      'cover.pageSemantics': 'Página {page}{selected}',
      'cover.selectedSuffix': ', seleccionada',
      'cover.noneSaved': 'Todavía no hay una portada guardada',
      'cover.currentManual': 'Página actual: {page} · manual',
      'cover.currentAutomatic': 'Página actual: {page} · automática',
      'cover.automaticSelection': 'Selección automática',
      'cover.use': 'Usar portada',
      'cover.invalidPage': 'Introduce una página válida.',
      'cover.saveFailed': 'No se pudo guardar esa portada.',
      'cover.restoreFailed': 'No se pudo restaurar la selección automática.',
      'folderImport.title': 'Importar carpeta',
      'folderImport.found': 'Encontrados',
      'folderImport.new': 'Nuevos',
      'folderImport.duplicates': 'Duplicados',
      'folderImport.inaccessible': 'Inaccesibles',
      'folderImport.invalid': 'No válidos',
      'folderImport.scanning': 'Buscando PDF',
      'folderImport.noneFound': 'No se encontraron PDF',
      'folderImport.confirmationBody':
          'Ningún archivo se añadirá sin confirmación.',
      'folderImport.watch': 'Observar esta carpeta después de importar',
      'folderImport.watchBody':
          'Solo revalidará cambios. No importará, moverá ni borrará PDF '
          'automáticamente.',
      'folderImport.cancelProcess': 'Cancelar proceso',
      'folderImport.add': 'Añadir {count}',
      'folderImport.status.newDocument': 'Nuevo',
      'folderImport.status.duplicate': 'Ya está en la biblioteca',
      'folderImport.status.inaccessible': 'Sin permiso de lectura',
      'folderImport.status.invalid': 'PDF no válido',
      'search.resultUnavailable':
          'El documento del resultado no está disponible.',
      'search.globalResult': 'Resultado de búsqueda global',
      'annotation.exportMarkdown': 'Documento legible y ordenado por página',
      'annotation.versionedJson': 'JSON versionado',
      'annotation.exportJson': 'Datos estructurados para futura importación',
      'reader.savedDocumentUnavailable':
          'Este documento guardado ya no está disponible.',
      'reader.password': 'Contraseña',
      'reader.wrongPasswordRetry':
          'La contraseña no es correcta. Inténtalo de nuevo.',
      'reader.passwordNotStored': 'La contraseña no se guarda.',
      'reader.unavailableAtPath': 'No disponible en la ruta guardada',
      'reader.availableChanged': 'Disponible; cambió desde la última apertura',
      'reader.available': 'Disponible',
      'reader.file': 'Archivo',
      'reader.lastOpened': 'Última apertura',
      'reader.progress': 'Progreso',
      'reader.pageLower': 'página {page}',
      'reader.note': 'Nota',
      'reader.suggestedTitle': 'Título sugerido',
      'reader.suggestedAuthor': 'Autor sugerido',
      'reader.presence': 'Presencia',
      'reader.path': 'Ruta',
      'reader.dismissSuggestion': 'Descartar propuesta',
      'reader.acceptSuggestion': 'Aceptar propuesta',
      'reader.editMetadata': 'Editar metadata',
      'reader.resetProgress': 'Reiniciar progreso',
      'reader.show': 'Mostrar',
      'reader.continue': 'Continuar',
      'reader.metadataApplied': 'Metadatos detectados aplicados.',
      'reader.suggestionDismissed': 'Propuesta descartada.',
      'reader.analysisUpdated': 'Análisis local actualizado.',
      'reader.resetProgress.description':
          'La página guardada volverá al inicio. El PDF y sus marcadores '
          'seguirán intactos.',
      'reader.reset': 'Reiniciar',
      'reader.progressReset': 'Progreso reiniciado.',
      'reader.editLocalMetadata': 'Editar metadata local',
      'reader.emptyRestoresTitle': 'Vacío restaura el título detectado.',
      'reader.libraryNote': 'Nota de biblioteca',
      'reader.storedOutsidePdf': 'Se guarda fuera del PDF.',
      'reader.metadataSaved': 'Metadata local guardada.',
      'reader.invalidMetadata': 'La metadata no es válida.',
      'count.documentsSelected.one': '{count} documento seleccionado',
      'count.documentsSelected.other': '{count} documentos seleccionados',
      'count.selected.one': '{count} seleccionado',
      'count.selected.other': '{count} seleccionados',
      'topic.history': 'Historia',
      'topic.politics': 'Política',
      'topic.philosophy': 'Filosofía',
      'topic.literature': 'Literatura',
      'topic.science': 'Ciencia',
      'topic.technology': 'Tecnología',
      'topic.economics': 'Economía',
      'topic.society': 'Sociedad',
      'topic.psychology': 'Psicología',
      'topic.art': 'Arte',
      'topic.religionMythology': 'Religión y mitología',
      'topic.law': 'Derecho',
      'topic.education': 'Educación',
      'topic.health': 'Salud',
      'topic.nature': 'Naturaleza',
      'topic.biography': 'Biografía',
      'reader.title': 'Lector',
      'reader.loadingTitle': 'Abriendo documento',
      'reader.loadingBody':
          'Preparando páginas, posición y preferencias de lectura.',
      'reader.pageIndicator': '{page} de {count}',
      'reader.currentDocument': 'Documento actual',
      'reader.backToLibrary': 'Volver a la biblioteca',
      'reader.tools': 'Herramientas del lector',
      'reader.openThumbnails': 'Abrir miniaturas',
      'reader.thumbnails': 'Miniaturas',
      'reader.outline': 'Índice',
      'reader.bookmarks': 'Marcadores',
      'reader.annotations': 'Anotaciones',
      'reader.moreOptions': 'Más opciones',
      'reader.openAnotherPdf': 'Abrir otro PDF…',
      'reader.coverAlone': 'Portada sola',
      'reader.firstPageRight': 'Primera página a la derecha',
      'reader.resetPairing': 'Restablecer emparejado',
      'reader.rotatePage': 'Girar página',
      'reader.readingMargins': 'Márgenes de lectura',
      'reader.exitFullScreen': 'Salir de pantalla completa',
      'reader.hideInterface': 'Ocultar interfaz',
      'reader.immersiveReading': 'Lectura inmersiva',
      'reader.previousSpread': 'Pliego anterior',
      'reader.nextSpread': 'Pliego siguiente',
      'reader.fitSpreadWidth': 'Ajustar al ancho del pliego',
      'reader.fitFullSpread': 'Ajustar al pliego completo',
      'reader.navigation': 'Navegación de lectura',
      'reader.activeControls': 'LECTURA',
      'reader.viewAndZoom': 'Vista y zoom',
      'reader.customZoom': 'Zoom personalizado',
      'reader.readingTheme': 'Tema de lectura',
      'reader.moreReadingControls': 'Más controles de lectura',
      'reader.expandReadingControls':
          'Mostrar navegación y zoom. Posición actual: {position}',
      'reader.removeBookmark': 'Retirar marcador',
      'reader.addBookmark': 'Añadir marcador',
      'reader.zoom': 'Zoom',
      'reader.scale': 'Escala',
      'reader.panel': 'Panel de {title}',
      'reader.pinPanel': 'Anclar panel',
      'reader.unpinPanel': 'Desanclar panel',
      'reader.closePanel': 'Cerrar panel',
      'reader.search': 'Buscar',
      'reader.panelSections': 'Secciones del panel',
      'reader.panelWidth': 'Ancho del panel lateral',
      'reader.pixels': '{count} píxeles',
      'reader.page': 'Página {page}',
      'reader.pageThumbnail': 'Miniatura de página {page}',
      'reader.unavailable': 'No disponible',
      'reader.generating': 'Generando…',
      'reader.pageWithAnnotations': 'Página con anotaciones',
      'reader.noOutline': 'Sin índice',
      'reader.noOutline.description':
          'Este documento no incluye una tabla de contenidos.',
      'reader.outlineSection': 'Sección del índice',
      'reader.bookmarkPage': 'Marcar página {page}',
      'reader.noBookmarks': 'Sin marcadores',
      'reader.noBookmarks.description':
          'Guarda una página para volver a ella después.',
      'reader.bookmarkOnPage': 'Marcador en página {page}',
      'reader.shortName': 'Nombre corto',
      'reader.optional': 'Opcional',
      'reader.deleteBookmark': 'Eliminar marcador',
      'reader.annotationsUnavailable': 'Anotaciones no disponibles',
      'reader.annotationsUnavailable.description':
          'El almacén local de anotaciones no está conectado.',
      'reader.annotatePage': 'Anotar página {page}',
      'reader.exportAnnotations': 'Exportar anotaciones',
      'reader.searchNotes': 'Buscar en notas',
      'reader.type': 'Tipo',
      'reader.color': 'Color',
      'reader.all': 'Todos',
      'reader.noAnnotations': 'Sin anotaciones',
      'reader.noAnnotations.description':
          'Añade una nota de página o una marca local para volver a esta idea.',
      'reader.annotationSemantics': '{type}, página {page}, {color}',
      'reader.reviewPosition': 'Revisar posición',
      'reader.annotationOnPage': 'Anotación en página {page}',
      'reader.annotationActions': 'Acciones de anotación',
      'reader.annotationDeleted': 'Anotación eliminada',
      'reader.newAnnotation': 'Nueva anotación',
      'reader.editAnnotation': 'Editar anotación',
      'reader.comment': 'Comentario',
      'reader.optionalTags': 'Etiquetas opcionales',
      'reader.tagsHint': 'estudio, revisar',
      'reader.wordOrPhrase': 'Palabra o frase',
      'reader.cancelSearch': 'Cancelar búsqueda',
      'reader.searchingDocument': 'Buscando en el documento',
      'reader.searchingCancelable': 'Buscando… Puedes cancelar la operación.',
      'reader.searchResultIndicator': '{current} de {count}',
      'reader.previousResult': 'Resultado anterior',
      'reader.nextResult': 'Resultado siguiente',
      'reader.noExtractableText': 'Sin texto extraíble',
      'reader.noExtractableText.description':
          'Este PDF puede ser un escaneo. Yamantaka no incluye OCR.',
      'reader.noDocumentResults.description':
          'Prueba otra palabra o una frase más corta.',
      'reader.match': 'Coincidencia',
      'reader.bookMode': 'Modo Libro',
      'reader.documentMode': 'Modo Documento',
      'reader.renderPageFailed':
          'No se pudo renderizar esta página. Prueba a cambiar de página o '
          'reabrir el documento.',
      'reader.pageHasAnnotations': 'La página {page} tiene anotaciones',
      'reader.positionSaved': 'Posición {slot} guardada.',
      'reader.positionRestored': 'Posición {slot} recuperada.',
      'reader.positionNotSaved': 'La posición {slot} no está guardada.',
      'reader.positionsSwapped': 'Posiciones A y B intercambiadas.',
      'reader.noTemporaryPositions': 'No hay posiciones temporales guardadas.',
      'reader.annotationUndone': 'Cambio de anotación deshecho.',
      'reader.noAnnotationUndo': 'No hay cambios de anotación que deshacer.',
      'reader.annotationRedone': 'Cambio de anotación rehecho.',
      'reader.noAnnotationRedo': 'No hay cambios de anotación que rehacer.',
      'reader.enterFullScreenFailed': 'No se pudo activar pantalla completa',
      'reader.fullScreenHint': 'Pantalla completa · F11 para salir',
      'reader.exitFullScreenFailed': 'No se pudo salir de pantalla completa',
      'reader.pageNumber': 'Número de página',
      'reader.pageRange': 'Entre 1 y {count}',
      'reader.pageScrubber': 'Recorrido rápido de páginas',
      'reader.readingEnvironment': 'Entorno de lectura',
      'reader.environmentChoice': 'Entorno {theme}',
      'reader.documentTitle': 'Título',
      'reader.documentAuthor': 'Autor',
      'reader.documentPages': 'Páginas',
      'reader.documentSize': 'Tamaño',
      'reader.documentModified': 'Modificado',
      'reader.documentLocation': 'Ubicación',
      'reader.pathCopied': 'Ruta copiada.',
      'reader.copyPath': 'Copiar ruta',
      'reader.readingMargins.description':
          'Separa la página del borde sin modificar el PDF.',
      'action.edit': 'Editar',
      'action.copyContent': 'Copiar contenido',
      'annotation.type.highlight': 'Resaltado',
      'annotation.type.underline': 'Subrayado',
      'annotation.type.strikeout': 'Tachado',
      'annotation.type.textNote': 'Nota de texto',
      'annotation.type.pageMarker': 'Marcador visual',
      'annotation.type.pageComment': 'Comentario de página',
      'annotation.color.yellow': 'Amarillo suave',
      'annotation.color.green': 'Verde suave',
      'annotation.color.blue': 'Azul suave',
      'annotation.color.rose': 'Rosa suave',
      'annotation.color.violet': 'Violeta suave',
      'count.annotationsReview.one':
          '{count} anotación necesita revisión porque cambió el documento.',
      'count.annotationsReview.other':
          '{count} anotaciones necesitan revisión porque cambió el documento.',
      'settings.title': 'Ajustes',
      'settings.back': 'Volver',
      'settings.language': 'Idioma',
      'settings.language.spanish': 'Castellano',
      'settings.language.english': 'English',
      'settings.appearance.description':
          'Tema, controles y perfil de pantalla.',
      'settings.language.description':
          'Aplica el catálogo castellano o inglés sin reiniciar.',
      'settings.darkMode': 'Tema oscuro de la aplicación',
      'settings.darkMode.description':
          'Afecta al lector y a los ajustes. La biblioteca conserva su entorno '
          'oscuro y espacial.',
      'settings.density': 'Densidad de la interfaz',
      'settings.density.description':
          'Compacta muestra más contenido sin cambiar el tamaño del PDF.',
      'settings.defaultReaderTheme': 'Tema de lectura predeterminado',
      'settings.defaultReaderTheme.description':
          'Color del lienzo al abrir un documento nuevo.',
      'settings.autoHideControls': 'Ocultar controles automáticamente',
      'settings.autoHideControls.description':
          'Mantiene el documento despejado durante la lectura.',
      'settings.showProgress': 'Mostrar barra de progreso',
      'settings.motion': 'Movimiento',
      'settings.motion.description':
          'Completo, reducido o sin animaciones. Se respeta la opción más '
          'restrictiva del sistema.',
      'settings.visualProfile': 'Perfil visual',
      'settings.visualProfile.description':
          'Tinta electrónica fuerza alto contraste, una página y cero '
          'movimiento.',
      'settings.reading.description': 'Comportamiento inicial y navegación.',
      'settings.readingProfile': 'Perfil de lectura',
      'settings.readingProfile.description':
          'Aplica un conjunto coherente sin mezclar densidad de interfaz y zoom '
          'del contenido.',
      'settings.defaultView': 'Vista predeterminada',
      'settings.pageLayout': 'Disposición de páginas',
      'settings.pageLayout.description':
          'La vista enfrentada solo se activa si ambas páginas son legibles.',
      'settings.zoomMode': 'Ajuste de zoom',
      'settings.margins': 'Márgenes',
      'settings.margins.description': 'Espacio alrededor del documento.',
      'settings.restorePosition': 'Restaurar la última posición',
      'settings.readingDirection': 'Dirección de lectura',
      'settings.readerBar.description':
          'Jerarquía y espacio de las herramientas del lector.',
      'settings.showPanelControls': 'Mostrar accesos al panel',
      'settings.showPanelControls.description':
          'Miniaturas, índice y marcadores seguirán en Más opciones.',
      'settings.panelWidth': 'Ancho del panel lateral',
      'settings.panelWidth.description':
          'Se conserva entre sesiones y también puede ajustarse arrastrando su '
          'borde.',
      'settings.restoreSession': 'Recuperar la sesión al iniciar',
      'settings.restoreSession.description':
          'Reabre de forma segura el último documento y su contexto.',
      'settings.startInLibrary': 'Iniciar siempre en la biblioteca',
      'settings.startInLibrary.description':
          'Tiene prioridad sobre la recuperación automática de sesión.',
      'settings.shortcuts.description':
          'Un conjunto limitado, seguro y restaurable.',
      'settings.restoreShortcuts': 'Restaurar atajos',
      'settings.restoreShortcuts.description':
          'Recupera las combinaciones predeterminadas. Escape, Inicio, Fin y '
          'las flechas básicas permanecen reservadas.',
      'settings.shortcut.description':
          'Solo combinaciones prevalidadas; se rechazan conflictos.',
      'settings.library.description':
          'Recientes, portadas y almacenamiento temporal.',
      'settings.showRecents': 'Mostrar documentos recientes',
      'settings.generateCovers': 'Generar portadas',
      'settings.generateCovers.description':
          'Elige localmente la mejor de las seis primeras páginas y guarda '
          'miniaturas nítidas.',
      'settings.maxCache': 'Tamaño máximo de caché',
      'settings.clearCache': 'Limpiar caché',
      'settings.clearCache.description':
          'Elimina portadas y renderizados temporales.',
      'settings.clearCache.done': 'Caché eliminada.',
      'settings.deleteHistory': 'Eliminar historial',
      'settings.deleteHistory.description':
          'Reinicia progreso, preferencias por documento, marcadores y '
          'sesiones. Conserva biblioteca, anotaciones y colecciones.',
      'settings.deleteHistory.confirmTitle': '¿Eliminar historial?',
      'settings.deleteHistory.confirmBody':
          'Se reiniciarán progreso, preferencias por documento, marcadores y '
          'sesiones. Se conservarán documentos, anotaciones, etiquetas y '
          'colecciones.',
      'settings.history.description':
          'Actividad local agrupada por día y bajo tu control.',
      'settings.recordSessions': 'Registrar sesiones de lectura',
      'settings.recordSessions.description':
          'Solo tiempo activo aproximado, guardado localmente.',
      'settings.historyLimit': 'Límite del historial',
      'settings.historyLimit.description':
          'Las sesiones más antiguas se descartan automáticamente.',
      'settings.viewHistory': 'Ver historial de lectura',
      'settings.viewHistory.description':
          'Consulta páginas, duración y fecha; elimina entradas individuales o '
          'todas.',
      'settings.viewHistory.action': 'Ver historial',
      'settings.dataPrivacy.description':
          'Datos locales, copias versionadas y privacidad.',
      'settings.rebuildableCleared': 'Cachés recreables eliminadas.',
      'settings.deleteBackups.confirmTitle': '¿Eliminar copias locales?',
      'settings.deleteBackups.confirmBody':
          'Solo se eliminarán copias de seguridad ya exportadas. Los PDF y la '
          'biblioteca permanecerán intactos.',
      'settings.exportBackup': 'Exportar copia local',
      'settings.exportBackup.description':
          'Incluye biblioteca, progreso, marcadores, historial y ajustes; nunca '
          'incluye el contenido de los PDF.',
      'settings.importBackup': 'Importar copia local',
      'settings.importBackup.description':
          'Muestra un resumen y pide confirmación. Antes crea una copia de '
          'seguridad automática del estado actual.',
      'settings.openDataLocation': 'Abrir ubicación de datos',
      'settings.deleteAllData': 'Eliminar todos los datos locales',
      'settings.deleteAllData.description':
          'No elimina los archivos PDF originales.',
      'settings.deleteAllData.confirmTitle': '¿Eliminar todos los datos?',
      'settings.deleteAllData.confirmBody':
          'Se borrarán biblioteca, progreso, marcadores, sesiones, ajustes y '
          'caché. Los PDF originales no se modificarán.',
      'settings.advanced.description':
          'Diagnóstico, integridad y recuperación local.',
      'settings.diagnostics': 'Centro de diagnóstico',
      'settings.diagnostics.description':
          'Versiones, esquema, tamaños, recursos e integridad sin rutas, '
          'títulos, consultas ni contenido.',
      'settings.contextualHelp': 'Ayuda contextual',
      'settings.contextualHelp.description':
          'Resumen local de búsqueda, anotaciones, espacios y atajos.',
      'settings.localStorage': 'Almacenamiento local',
      'settings.measuringStorage': 'Calculando tamaños…',
      'settings.originalsNotDeleted':
          'Los archivos PDF originales no se borran desde estas acciones.',
      'settings.clearRebuildable': 'Limpiar recreables',
      'settings.deleteBackups': 'Eliminar copias',
      'settings.allCategories': 'Todas las categorías',
      'settings.localPreferences': 'Preferencias locales',
      'settings.localPreferences.description':
          'Los cambios reversibles se guardan al instante.',
      'settings.appearancePreview':
          'Vista previa: interfaz {theme}, densidad {density}, lienzo {canvas}',
      'settings.preview': 'Vista previa',
      'settings.preview.description':
          'Una muestra real del canvas, la página y el chrome del lector.',
      'settings.previewMode': 'Estado de la demostración',
      'settings.previewMode.resting': 'Reposo',
      'settings.previewMode.active': 'Activo',
      'settings.previewMode.panel': 'Panel abierto',
      'settings.previewMode.electronicInk': 'Tinta electrónica',
      'settings.navigationGroup.preferences': 'PREFERENCIAS',
      'settings.navigationGroup.maintenance': 'MANTENIMIENTO',
      'settings.navigationGroup.privacy': 'PRIVACIDAD',
      'settings.historyEntry': 'Páginas {start}–{end} · {duration}',
      'settings.deleteSession': 'Eliminar esta sesión',
      'settings.historyTitle': 'Historial de lectura',
      'settings.noSessions': 'Sin sesiones guardadas',
      'settings.noSessions.description':
          'El historial aparecerá aquí cuando cierres una sesión de lectura.',
      'settings.updated': 'Ajuste actualizado',
      'settings.saveFailed': 'No se pudo guardar el ajuste: {error}',
      'settings.section.appearance': 'Apariencia',
      'settings.section.reading': 'Lectura',
      'settings.section.readerBar': 'Controles del lector',
      'settings.section.shortcuts': 'Atajos',
      'settings.section.library': 'Biblioteca',
      'settings.section.history': 'Historial',
      'settings.section.dataPrivacy': 'Datos y privacidad',
      'settings.section.advanced': 'Avanzado',
      'settings.value.comfortable': 'Cómoda',
      'settings.value.compact': 'Compacta',
      'settings.value.light': 'Claro',
      'settings.value.ivory': 'Marfil',
      'settings.value.sepia': 'Sepia',
      'settings.value.gray': 'Gris',
      'settings.value.dark': 'Oscuro',
      'settings.value.black': 'Negro',
      'settings.value.motionFull': 'Completo',
      'settings.value.motionReduced': 'Reducido',
      'settings.value.motionNone': 'Sin animaciones',
      'settings.value.standard': 'Estándar',
      'settings.value.electronicInk': 'Tinta electrónica',
      'settings.value.reading': 'Lectura',
      'settings.value.study': 'Estudio',
      'settings.value.presentation': 'Presentación',
      'settings.value.night': 'Noche',
      'settings.value.comparison': 'Comparación',
      'settings.value.document': 'Documento',
      'settings.value.book': 'Libro',
      'settings.value.singlePage': 'Una página',
      'settings.value.facingPages': 'Dos páginas',
      'settings.value.continuous': 'Continua',
      'settings.value.fitWidth': 'Ajustar al ancho',
      'settings.value.fitPage': 'Ajustar a la página',
      'settings.value.leftToRight': 'Izquierda a derecha',
      'settings.value.rightToLeft': 'Derecha a izquierda',
      'storage.database.title': 'Base de datos',
      'storage.database.description':
          'Biblioteca, posiciones y organización local.',
      'storage.covers.title': 'Portadas',
      'storage.covers.description': 'Portadas recreables de los documentos.',
      'storage.renderCache.title': 'Caché de lectura',
      'storage.renderCache.description': 'Imágenes de lectura recreables.',
      'storage.searchIndex.title': 'Índice de búsqueda',
      'storage.searchIndex.description':
          'Índice reconstruible de texto y metadatos.',
      'storage.logs.title': 'Registros',
      'storage.logs.description': 'Registros técnicos redactados.',
      'storage.backups.title': 'Copias',
      'storage.backups.description':
          'Copias de seguridad solicitadas por el usuario.',
      'storage.preferences.title': 'Preferencias',
      'storage.preferences.description': 'Preferencias de la aplicación.',
      'backup.fileType': 'Copia local de Yamantaka',
      'backup.exported': 'Copia local exportada.',
      'backup.exportFailed': 'No se pudo exportar: {error}',
      'backup.imported': 'Copia importada. Respaldo previo guardado en {path}',
      'backup.importFailed':
          'No se pudo importar. Los datos actuales se conservaron.',
      'backup.importPreview':
          'Versión {version} · {documents} documentos · {bookmarks} marcadores '
          '· {annotations} anotaciones · {sessions} sesiones.\n\nSe '
          'sustituirán los datos locales actuales. Antes se guardará una copia '
          'de seguridad automática. Los PDF no se modifican.',
      'count.sessions.one': '{count} sesión',
      'count.sessions.other': '{count} sesiones',
      'shortcut.actionReserved': 'Esta acción está reservada.',
      'shortcut.shift': 'Mayús',
      'shortcut.pageDown': 'Av Pág',
      'shortcut.pageUp': 'Re Pág',
      'shortcut.navigationReserved':
          'Esta combinación está reservada para la navegación.',
      'shortcut.notAllowed': 'La combinación no está permitida.',
      'shortcut.alreadyAssigned':
          'La combinación ya está asignada a otra acción.',
      'privacy.title': 'Privacidad y almacenamiento',
      'privacy.localOnly': 'Los datos permanecen en este dispositivo.',
      'backup.title': 'Copia local',
      'help.title': 'Ayuda',
      'help.search': 'Buscar en la ayuda',
      'help.noResults': 'No hay temas que coincidan.',
      'search.title': 'Buscar en la biblioteca',
      'search.field': 'Documentos, contenido, notas y etiquetas',
      'search.emptyTitle': 'Búsqueda local unificada',
      'search.emptyBody':
          'Busca por metadatos, contenido extraíble, anotaciones, colecciones '
          'y etiquetas. El índice permanece en este dispositivo.',
      'search.noResultsTitle': 'Sin resultados',
      'search.noResultsBody':
          'El documento puede no contener texto extraíble o aún no estar '
          'indexado. Yamantaka no realiza OCR.',
      'search.kind.metadata': 'Documento',
      'search.kind.pageText': 'Texto de página',
      'search.kind.annotation': 'Anotación',
      'search.kind.collection': 'Colección',
      'search.kind.tag': 'Etiqueta',
      'search.indexing': 'Indexando {current} de {total}',
      'search.indexEnabled': 'Índice local activado',
      'search.indexDisabled': 'Índice local desactivado',
      'search.updateIndex': 'Actualizar índice',
      'search.indexMaintenance': 'Mantenimiento del índice',
      'search.rebuildIndex': 'Reconstruir índice',
      'search.clearIndex': 'Borrar índice',
      'command.searchHint': 'Buscar una acción',
      'command.section': 'Comandos',
      'command.move': 'Mover',
      'command.run': 'Ejecutar',
      'command.empty': 'No hay acciones que coincidan.',
      'command.recentDocument': 'Documento reciente',
      'command.recentSection': 'Documentos recientes',
      'command.category.document': 'Documento y aplicación',
      'command.category.navigation': 'Navegación',
      'command.category.panel': 'Panel lateral',
      'command.category.annotation': 'Lectura y anotación',
      'command.category.view': 'Vista del lector',
      'command.preview.selection': 'RESULTADO SELECCIONADO',
      'command.preview.safeAction':
          'Acción inmediata; Enter ejecuta y Escape vuelve al lector.',
      'command.preview.enterHint': 'Pulsa Enter para ejecutar',
      'command.openDocument': 'Abrir documento',
      'command.search': 'Buscar en el documento',
      'command.commandPalette': 'Abrir paleta de comandos',
      'command.goToPage': 'Ir a una página',
      'command.nextPage': 'Página siguiente',
      'command.previousPage': 'Página anterior',
      'command.togglePanel': 'Abrir o cerrar panel',
      'command.toggleBookmark': 'Añadir o retirar marcador',
      'command.zoomIn': 'Aumentar zoom',
      'command.zoomOut': 'Reducir zoom',
      'command.resetZoom': 'Restablecer zoom',
      'command.actualSize': 'Tamaño real (100 %)',
      'command.singlePageLayout': 'Disposición: Una página',
      'command.facingPageLayout': 'Disposición: Dos páginas',
      'command.continuousPageLayout': 'Disposición: Continua',
      'command.showThumbnails': 'Mostrar miniaturas',
      'command.showOutline': 'Mostrar índice',
      'command.showAnnotations': 'Mostrar anotaciones',
      'command.navigationBack': 'Volver a la ubicación anterior',
      'command.navigationForward': 'Avanzar en el historial',
      'command.savePositionA': 'Guardar posición temporal A',
      'command.savePositionB': 'Guardar posición temporal B',
      'command.goToPositionA': 'Ir a la posición temporal A',
      'command.goToPositionB': 'Ir a la posición temporal B',
      'command.swapPositions': 'Intercambiar posiciones A y B',
      'command.undoAnnotation': 'Deshacer cambio de anotación',
      'command.redoAnnotation': 'Rehacer cambio de anotación',
      'command.fitWidth': 'Ajustar al ancho',
      'command.fitPage': 'Ajustar a la página',
      'command.changeReaderTheme': 'Cambiar tema de lectura',
      'command.documentInfo': 'Información del documento',
      'command.fullScreen': 'Pantalla completa',
      'command.toggleChrome': 'Mostrar u ocultar interfaz',
      'command.toggleImmersive': 'Alternar lectura inmersiva',
      'command.settings': 'Abrir configuración',
      'failure.none.title': 'No hay un documento abierto',
      'failure.none.message': 'Elige un PDF local para comenzar a leer.',
      'failure.fileNotFound.title': 'El archivo ya no está disponible',
      'failure.fileNotFound.message':
          'Puedes localizar otra copia del mismo PDF o volver atrás.',
      'failure.accessDenied.title': 'No se puede leer el archivo',
      'failure.accessDenied.message':
          'Comprueba los permisos del archivo y vuelve a intentarlo.',
      'failure.corruptPdf.title': 'El PDF no se puede abrir',
      'failure.corruptPdf.message':
          'El archivo puede estar dañado o usar una variante no compatible.',
      'failure.emptyPdf.title': 'El PDF no contiene páginas',
      'failure.emptyPdf.message':
          'Elige un documento que tenga al menos una página.',
      'failure.passwordRequired.title': 'El PDF está protegido',
      'failure.passwordRequired.message':
          'Introduce la contraseña para continuar. No se guardará.',
      'failure.wrongPassword.title': 'La contraseña no es correcta',
      'failure.wrongPassword.message':
          'Vuelve a introducirla o elige otro documento.',
      'failure.unsupportedFormat.title': 'Formato no compatible',
      'failure.unsupportedFormat.message': 'Yamantaka solo abre archivos PDF.',
      'failure.renderFailed.title': 'No se pudo dibujar la página',
      'failure.renderFailed.message':
          'Puedes reintentar sin modificar el PDF original.',
      'failure.outOfMemory.title': 'No hay memoria suficiente',
      'failure.outOfMemory.message':
          'Cierra otros documentos o reduce el trabajo en segundo plano.',
      'failure.databaseFailed.title': 'No se pudo leer la biblioteca',
      'failure.databaseFailed.message':
          'Verifica la base local desde el diagnóstico y vuelve a intentarlo.',
      'failure.thumbnailFailed.title': 'No se pudo generar la portada',
      'failure.thumbnailFailed.message':
          'La lectura continúa y la portada se puede reintentar después.',
      'failure.restoreFailed.title': 'No se pudo restaurar la sesión',
      'failure.restoreFailed.message':
          'La biblioteca sigue disponible; puedes abrir el PDF manualmente.',
      'failure.cancelled.title': 'Operación cancelada',
      'failure.cancelled.message':
          'No se ha aplicado ningún resultado obsoleto.',
      'failure.invalidInput.title': 'Entrada no válida',
      'failure.invalidInput.message': 'Revisa los datos e inténtalo de nuevo.',
      'failure.unauthorizedLink.title': 'Enlace bloqueado',
      'failure.unauthorizedLink.message':
          'El esquema del enlace no está autorizado por Yamantaka.',
      'failure.unknown.title': 'No se pudo completar la operación',
      'failure.unknown.message':
          'Los datos confirmados se conservan. Puedes volver a intentarlo.',
      'welcome.title': 'Empieza con tu biblioteca local',
      'welcome.body':
          'Abre un PDF, conserva tu posición y mantén tus datos en este '
          'dispositivo. No hay cuentas ni copias automáticas en la nube.',
      'welcome.open': 'Abrir un PDF',
      'welcome.help': 'Ver ayuda',
      'count.documents.one': '{count} documento',
      'count.documents.other': '{count} documentos',
      'count.results.one': '{count} resultado',
      'count.results.other': '{count} resultados',
    },
    AppLanguage.english: {
      'app.title': 'Yamantaka',
      'app.openLimit':
          '{count} files were opened; the rest were skipped to respect the '
          'resource limit.',
      'action.open': 'Open',
      'action.import': 'Import',
      'action.cancel': 'Cancel',
      'action.retry': 'Retry',
      'action.close': 'Close',
      'action.save': 'Save',
      'action.reset': 'Reset',
      'action.clear': 'Clear',
      'action.loadMore': 'Load more',
      'action.delete': 'Delete',
      'action.deleteAll': 'Delete all',
      'action.export': 'Export',
      'action.undo': 'Undo',
      'action.apply': 'Apply',
      'action.go': 'Go',
      'action.create': 'Create',
      'action.pause': 'Pause',
      'action.resume': 'Resume',
      'action.continue': 'Continue',
      'action.reread': 'Read again',
      'action.refresh': 'Refresh',
      'library.title': 'Library',
      'library.localCollection': 'Local collection',
      'library.openPdf': 'Open PDF',
      'library.emptyTitle': 'A quiet library',
      'library.emptyBody':
          'Open or drag a PDF into this window. The file stays in its original '
          'folder and your progress is stored only here.',
      'library.search': 'Search library',
      'library.noResults': 'No documents match these filters.',
      'library.noQueryResults': 'No documents match “{query}”.',
      'library.closeSearch': 'Close search (Escape)',
      'library.checkFiles': 'Check files',
      'library.checkTitle': 'File check',
      'library.checkSummary':
          'Available: {available}\nMissing: {missing}\nModified: '
          '{modified}\nFailed: {failed}\nCancelled: {cancelled}',
      'library.empty': 'There are no local documents yet.',
      'library.noPdfsDeleted': 'PDFs are not deleted by this action.',
      'library.analyzing': 'Analyzing {current} of {total}…',
      'library.cancelAnalysis': 'Cancel local analysis',
      'library.safeMode': 'Safe mode: the session and index were not restored',
      'library.organizeMore': 'Organize and more actions',
      'library.collections': 'Collections',
      'library.workspaces': 'Workspaces',
      'library.importFolder': 'Import folder',
      'library.recoverMoved': 'Recover moved documents',
      'library.globalSearch': 'Global search',
      'library.analyzeMetadata': 'Analyze metadata and topics',
      'library.reanalyze': 'Reanalyze library',
      'library.dropPdf': 'Drop the PDF to open it',
      'library.reanalyze.description':
          'Titles, authors, and topics will be detected locally again. '
          'Manually edited titles, authors, and topics will be preserved.',
      'library.analyze': 'Analyze',
      'library.allLibrary': 'Entire library',
      'library.newCollection': 'New collection',
      'library.name': 'Name',
      'library.smartCollection.inProgress': 'In progress',
      'library.smartCollection.notStarted': 'Not started',
      'library.smartCollection.finished': 'Finished',
      'library.smartCollection.annotated': 'With annotations',
      'library.smartCollection.bookmarked': 'With bookmarks',
      'library.smartCollection.missing': 'Missing',
      'library.smartCollection.recentlyOpened': 'Recently opened',
      'library.smartCollection.recentlyAdded': 'Recently added',
      'library.preparing': 'Preparing your library',
      'library.preparing.description':
          'Reviewing documents and covers stored on this device.',
      'library.missingDocumentSemantics': '{file}, file not found, locate',
      'library.percent': '{value} percent',
      'library.fileNotFound': 'File not found',
      'library.fileChanged': 'The file has changed',
      'library.locate': 'Locate',
      'library.finished': 'Finished',
      'library.inProgress': 'In progress · {value}%',
      'library.notStarted': 'Not started',
      'library.unknownAuthor': 'Unknown author',
      'library.analysisPending': 'Analysis pending',
      'library.analysisInProgress': 'Local analysis in progress',
      'library.analysisFailed': 'Analysis failed',
      'library.progressValue': 'Progress: {value} percent',
      'time.now': 'now',
      'time.minutesAgo': '{count} min ago',
      'time.hoursAgo': '{count} h ago',
      'time.daysAgo': '{count} d ago',
      'library.resultOfTotal': '{result} of {total}',
      'library.commandBar': 'Search, select, filter, and sort',
      'library.exitSelectionMode': 'Exit selection mode',
      'library.selectMultiple': 'Select multiple documents',
      'library.filterAuthorsTopics': 'Filter by authors and topics',
      'library.authorsTopicsGrouping': 'Authors, topics, and grouping',
      'library.filterSort': 'Filter and sort',
      'library.state': 'State',
      'library.readingState.all': 'All',
      'library.readingState.inProgress': 'In progress',
      'library.readingState.notStarted': 'Not started',
      'library.readingState.completed': 'Finished',
      'library.favoritesOnly': 'Favorites only',
      'library.sort': 'Sort',
      'library.sortOrder.lastOpened': 'Last opened',
      'library.sortOrder.dateAdded': 'Date added',
      'library.sortOrder.title': 'Title',
      'library.sortOrder.progress': 'Progress',
      'library.resetFilters': 'Reset filters',
      'library.filtersApplied': '{count} filters applied',
      'library.actionsForSelection': 'Actions for {count} selected documents',
      'library.bulkActions': 'Bulk actions',
      'library.markFavorite': 'Mark as favorite',
      'library.unmarkFavorite': 'Remove from favorites',
      'library.assignTag': 'Assign tag',
      'library.addToCollection': 'Add to collection',
      'library.removeFromCollection': 'Remove from collection',
      'library.archive': 'Archive',
      'library.regenerateCovers': 'Regenerate covers',
      'library.removeFromLibrary': 'Remove from library',
      'library.clearSelection': 'Clear selection',
      'library.artifactSelectionHint':
          'Click toggles selection; Ctrl and Shift extend the selection',
      'library.artifactOpenHint':
          'Click selects; double-click or Enter opens; context menu',
      'library.favorite': 'Favorite',
      'library.finishedPages': 'Finished · {count} pages',
      'library.notStartedPages': 'Not started · {count} pages',
      'library.pageProgress': 'Page {page} of {count} · {progress}%',
      'library.metadataNeedsReview': 'Metadata needs review',
      'library.documentUnavailable':
          'Document unavailable · locate it to continue',
      'library.artifactMissing': '{title}, file not found',
      'library.artifactFinished': '{title}, finished',
      'library.artifactProgress': '{title}, {progress} percent',
      'library.artifactNotStarted': '{title}, not started',
      'time.openedNow': 'opened now',
      'time.openedToday': 'opened today',
      'time.openedYesterday': 'opened yesterday',
      'time.openedDaysAgo': 'opened {count} days ago',
      'time.openedDate': 'opened {day}/{month}',
      'library.selectionActivated': 'Selection mode enabled. {count} selected.',
      'library.bulkFailed':
          'The action could not be completed. No PDF was changed.',
      'library.noCollections': 'No collections have been created.',
      'library.collectionSemantics': '{name}, {count} documents',
      'library.noTags': 'No tags have been created.',
      'library.tagSemantics': '{name}, {count} uses',
      'library.removeDocuments': 'Remove documents',
      'library.removeDocuments.description':
          '{count} documents will be removed from the library. Original PDFs '
          'will not be deleted.',
      'library.remove': 'Remove',
      'library.actionCompleted': 'Action completed',
      'library.bulkSummary': '{succeeded} succeeded · {failed} failed',
      'library.selectedDocument': 'Selected document',
      'library.closeContext': 'Close document context',
      'library.selectionFiltered':
          'The selected document is no longer part of these results.',
      'library.showSelectionAgain': 'Show it again',
      'library.operationFailed': 'The operation could not be completed.',
      'library.locateFile': 'Locate file',
      'library.continueReading': 'Continue reading',
      'library.changeCover': 'Change cover',
      'library.reanalyzeMetadata': 'Reanalyze metadata and topics',
      'library.showInformation': 'Show information',
      'library.showInExplorer': 'Show in File Explorer',
      'library.authorsAndTopics': 'Authors and topics',
      'library.groupByAuthor': 'Group by author',
      'library.groupByAuthor.description':
          'Creates dynamic sections without changing collections.',
      'library.authors': 'Authors',
      'library.noAuthors': 'No authors have been detected yet.',
      'library.topics': 'Topics',
      'library.analyzeFolder': 'Analyze folder',
      'library.importSummary':
          '{imported} imported; {duplicates} duplicates; {failed} failed.',
      'library.searchDocuments': 'Search documents',
      'library.searchingLocalMatches': 'Searching for local matches…',
      'library.recoverySummary':
          '{recovered} recovered; {ambiguous} ambiguous; {modified} modified; '
          '{notFound} not found.',
      'library.relocationMismatch':
          'The selected PDF does not match the missing document. The library '
          'was not changed.',
      'library.removedFromFavorites': 'Removed from favorites.',
      'library.addedToFavorites': 'Added to favorites.',
      'library.coverUpdated': 'Cover updated.',
      'library.removeFromLibrary.description':
          'Local progress, bookmarks, and sessions will be deleted. The PDF '
          'file will not be deleted.',
      'library.documentRemoved': 'Document removed. The PDF was not deleted.',
      'workspace.restoredMissing':
          'Workspace restored; {count} documents are unavailable.',
      'workspace.title': 'Workspaces',
      'workspace.saveCurrent': 'Save current',
      'workspace.emptyTitle': 'No saved workspaces',
      'workspace.emptyBody':
          'Save the current documents, positions, and panel to restore this '
          'reading session.',
      'workspace.summary': '{documents} · {missing}',
      'workspace.documents.one': '{count} document',
      'workspace.documents.other': '{count} documents',
      'workspace.missing.none': 'all available',
      'workspace.missing.one': '{count} missing',
      'workspace.missing.other': '{count} missing',
      'workspace.actions': 'Workspace actions',
      'workspace.duplicate': 'Duplicate',
      'workspace.saveTitle': 'Save workspace',
      'workspace.duplicateTitle': 'Duplicate workspace',
      'workspace.copySuffix': '{name} — copy',
      'workspace.deleteTitle': 'Delete workspace',
      'workspace.deleteBody':
          'The saved session is deleted, not its documents or annotations.',
      'workspace.name': 'Name',
      'diagnostics.title': 'Local diagnostics',
      'diagnostics.failedTitle': 'Diagnostics could not be generated',
      'diagnostics.failedBody': 'Local data was not modified.',
      'diagnostics.safeMode':
          'Safe mode is active: the session was not restored and indexing '
          'was not started.',
      'diagnostics.metrics': '{count} metrics',
      'diagnostics.copySafe': 'Copy safe report',
      'diagnostics.maintenanceLocal': 'Local maintenance',
      'diagnostics.verifyDatabase': 'Verify database',
      'diagnostics.optimizeDatabase': 'Optimize database',
      'diagnostics.clearCaches': 'Clear caches',
      'diagnostics.rebuildIndex': 'Rebuild index',
      'diagnostics.openDataFolder': 'Open data folder',
      'diagnostics.maintenance': 'Maintenance',
      'diagnostics.copied': 'Diagnostics copied.',
      'diagnostics.exported': 'Diagnostics exported.',
      'diagnostics.writeFailed': 'The report could not be written.',
      'diagnostics.integrityOk': 'Integrity check passed.',
      'diagnostics.result': 'Result: {result}',
      'diagnostics.optimized': 'Database optimized.',
      'diagnostics.clearCachesBody': 'PDF files will not be modified.',
      'diagnostics.cachesCleared': 'Caches cleared.',
      'diagnostics.rebuildIndexBody':
          'Only the local index will be deleted and recreated.',
      'diagnostics.indexRebuilt': 'Index rebuilt.',
      'cover.title': 'Change cover',
      'cover.previewFailed': 'Page previews could not be prepared.',
      'cover.page': 'Page',
      'cover.pageSemantics': 'Page {page}{selected}',
      'cover.selectedSuffix': ', selected',
      'cover.noneSaved': 'No cover has been saved yet',
      'cover.currentManual': 'Current page: {page} · manual',
      'cover.currentAutomatic': 'Current page: {page} · automatic',
      'cover.automaticSelection': 'Automatic selection',
      'cover.use': 'Use cover',
      'cover.invalidPage': 'Enter a valid page.',
      'cover.saveFailed': 'That cover could not be saved.',
      'cover.restoreFailed': 'Automatic selection could not be restored.',
      'folderImport.title': 'Import folder',
      'folderImport.found': 'Found',
      'folderImport.new': 'New',
      'folderImport.duplicates': 'Duplicates',
      'folderImport.inaccessible': 'Inaccessible',
      'folderImport.invalid': 'Invalid',
      'folderImport.scanning': 'Looking for PDFs',
      'folderImport.noneFound': 'No PDFs found',
      'folderImport.confirmationBody':
          'No file will be added without confirmation.',
      'folderImport.watch': 'Watch this folder after importing',
      'folderImport.watchBody':
          'Only changes will be revalidated. PDFs will never be imported, '
          'moved, or deleted automatically.',
      'folderImport.cancelProcess': 'Cancel process',
      'folderImport.add': 'Add {count}',
      'folderImport.status.newDocument': 'New',
      'folderImport.status.duplicate': 'Already in the library',
      'folderImport.status.inaccessible': 'No read permission',
      'folderImport.status.invalid': 'Invalid PDF',
      'search.resultUnavailable': 'The result document is unavailable.',
      'search.globalResult': 'Global search result',
      'annotation.exportMarkdown': 'Readable document ordered by page',
      'annotation.versionedJson': 'Versioned JSON',
      'annotation.exportJson': 'Structured data for future import',
      'reader.savedDocumentUnavailable':
          'This saved document is no longer available.',
      'reader.password': 'Password',
      'reader.wrongPasswordRetry': 'The password is incorrect. Try again.',
      'reader.passwordNotStored': 'The password is not stored.',
      'reader.unavailableAtPath': 'Unavailable at the saved path',
      'reader.availableChanged': 'Available; changed since last opened',
      'reader.available': 'Available',
      'reader.file': 'File',
      'reader.lastOpened': 'Last opened',
      'reader.progress': 'Progress',
      'reader.pageLower': 'page {page}',
      'reader.note': 'Note',
      'reader.suggestedTitle': 'Suggested title',
      'reader.suggestedAuthor': 'Suggested author',
      'reader.presence': 'Presence',
      'reader.path': 'Path',
      'reader.dismissSuggestion': 'Dismiss suggestion',
      'reader.acceptSuggestion': 'Accept suggestion',
      'reader.editMetadata': 'Edit metadata',
      'reader.resetProgress': 'Reset progress',
      'reader.show': 'Show',
      'reader.continue': 'Continue',
      'reader.metadataApplied': 'Detected metadata applied.',
      'reader.suggestionDismissed': 'Suggestion dismissed.',
      'reader.analysisUpdated': 'Local analysis updated.',
      'reader.resetProgress.description':
          'The saved page will return to the beginning. The PDF and its '
          'bookmarks will remain intact.',
      'reader.reset': 'Reset',
      'reader.progressReset': 'Progress reset.',
      'reader.editLocalMetadata': 'Edit local metadata',
      'reader.emptyRestoresTitle': 'Empty restores the detected title.',
      'reader.libraryNote': 'Library note',
      'reader.storedOutsidePdf': 'Stored outside the PDF.',
      'reader.metadataSaved': 'Local metadata saved.',
      'reader.invalidMetadata': 'The metadata is invalid.',
      'count.documentsSelected.one': '{count} selected document',
      'count.documentsSelected.other': '{count} selected documents',
      'count.selected.one': '{count} selected',
      'count.selected.other': '{count} selected',
      'topic.history': 'History',
      'topic.politics': 'Politics',
      'topic.philosophy': 'Philosophy',
      'topic.literature': 'Literature',
      'topic.science': 'Science',
      'topic.technology': 'Technology',
      'topic.economics': 'Economics',
      'topic.society': 'Society',
      'topic.psychology': 'Psychology',
      'topic.art': 'Art',
      'topic.religionMythology': 'Religion and mythology',
      'topic.law': 'Law',
      'topic.education': 'Education',
      'topic.health': 'Health',
      'topic.nature': 'Nature',
      'topic.biography': 'Biography',
      'reader.title': 'Reader',
      'reader.loadingTitle': 'Opening document',
      'reader.loadingBody':
          'Preparing pages, position, and reading preferences.',
      'reader.pageIndicator': '{page} of {count}',
      'reader.currentDocument': 'Current document',
      'reader.backToLibrary': 'Back to library',
      'reader.tools': 'Reader tools',
      'reader.openThumbnails': 'Open thumbnails',
      'reader.thumbnails': 'Thumbnails',
      'reader.outline': 'Outline',
      'reader.bookmarks': 'Bookmarks',
      'reader.annotations': 'Annotations',
      'reader.moreOptions': 'More options',
      'reader.openAnotherPdf': 'Open another PDF…',
      'reader.coverAlone': 'Cover alone',
      'reader.firstPageRight': 'First page on the right',
      'reader.resetPairing': 'Reset pairing',
      'reader.rotatePage': 'Rotate page',
      'reader.readingMargins': 'Reading margins',
      'reader.exitFullScreen': 'Exit full screen',
      'reader.hideInterface': 'Hide interface',
      'reader.immersiveReading': 'Immersive reading',
      'reader.previousSpread': 'Previous spread',
      'reader.nextSpread': 'Next spread',
      'reader.fitSpreadWidth': 'Fit spread to width',
      'reader.fitFullSpread': 'Fit full spread',
      'reader.navigation': 'Reading navigation',
      'reader.activeControls': 'READING',
      'reader.viewAndZoom': 'View and zoom',
      'reader.customZoom': 'Custom zoom',
      'reader.readingTheme': 'Reading theme',
      'reader.moreReadingControls': 'More reading controls',
      'reader.expandReadingControls':
          'Show navigation and zoom. Current position: {position}',
      'reader.removeBookmark': 'Remove bookmark',
      'reader.addBookmark': 'Add bookmark',
      'reader.zoom': 'Zoom',
      'reader.scale': 'Scale',
      'reader.panel': '{title} panel',
      'reader.pinPanel': 'Pin panel',
      'reader.unpinPanel': 'Unpin panel',
      'reader.closePanel': 'Close panel',
      'reader.search': 'Search',
      'reader.panelSections': 'Panel sections',
      'reader.panelWidth': 'Side panel width',
      'reader.pixels': '{count} pixels',
      'reader.page': 'Page {page}',
      'reader.pageThumbnail': 'Page {page} thumbnail',
      'reader.unavailable': 'Unavailable',
      'reader.generating': 'Generating…',
      'reader.pageWithAnnotations': 'Page with annotations',
      'reader.noOutline': 'No outline',
      'reader.noOutline.description':
          'This document does not include a table of contents.',
      'reader.outlineSection': 'Outline section',
      'reader.bookmarkPage': 'Bookmark page {page}',
      'reader.noBookmarks': 'No bookmarks',
      'reader.noBookmarks.description': 'Save a page to return to it later.',
      'reader.bookmarkOnPage': 'Bookmark on page {page}',
      'reader.shortName': 'Short name',
      'reader.optional': 'Optional',
      'reader.deleteBookmark': 'Delete bookmark',
      'reader.annotationsUnavailable': 'Annotations unavailable',
      'reader.annotationsUnavailable.description':
          'The local annotation store is not connected.',
      'reader.annotatePage': 'Annotate page {page}',
      'reader.exportAnnotations': 'Export annotations',
      'reader.searchNotes': 'Search notes',
      'reader.type': 'Type',
      'reader.color': 'Color',
      'reader.all': 'All',
      'reader.noAnnotations': 'No annotations',
      'reader.noAnnotations.description':
          'Add a page note or local mark to return to this idea.',
      'reader.annotationSemantics': '{type}, page {page}, {color}',
      'reader.reviewPosition': 'Review position',
      'reader.annotationOnPage': 'Annotation on page {page}',
      'reader.annotationActions': 'Annotation actions',
      'reader.annotationDeleted': 'Annotation deleted',
      'reader.newAnnotation': 'New annotation',
      'reader.editAnnotation': 'Edit annotation',
      'reader.comment': 'Comment',
      'reader.optionalTags': 'Optional tags',
      'reader.tagsHint': 'study, review',
      'reader.wordOrPhrase': 'Word or phrase',
      'reader.cancelSearch': 'Cancel search',
      'reader.searchingDocument': 'Searching document',
      'reader.searchingCancelable': 'Searching… You can cancel the operation.',
      'reader.searchResultIndicator': '{current} of {count}',
      'reader.previousResult': 'Previous result',
      'reader.nextResult': 'Next result',
      'reader.noExtractableText': 'No extractable text',
      'reader.noExtractableText.description':
          'This PDF may be a scan. Yamantaka does not include OCR.',
      'reader.noDocumentResults.description':
          'Try another word or a shorter phrase.',
      'reader.match': 'Match',
      'reader.bookMode': 'Book mode',
      'reader.documentMode': 'Document mode',
      'reader.renderPageFailed':
          'This page could not be rendered. Try changing pages or reopening '
          'the document.',
      'reader.pageHasAnnotations': 'Page {page} has annotations',
      'reader.positionSaved': 'Position {slot} saved.',
      'reader.positionRestored': 'Position {slot} restored.',
      'reader.positionNotSaved': 'Position {slot} has not been saved.',
      'reader.positionsSwapped': 'Positions A and B swapped.',
      'reader.noTemporaryPositions': 'No temporary positions are saved.',
      'reader.annotationUndone': 'Annotation change undone.',
      'reader.noAnnotationUndo': 'There are no annotation changes to undo.',
      'reader.annotationRedone': 'Annotation change redone.',
      'reader.noAnnotationRedo': 'There are no annotation changes to redo.',
      'reader.enterFullScreenFailed': 'Could not enter full screen',
      'reader.fullScreenHint': 'Full screen · press F11 to exit',
      'reader.exitFullScreenFailed': 'Could not exit full screen',
      'reader.pageNumber': 'Page number',
      'reader.pageRange': 'Between 1 and {count}',
      'reader.pageScrubber': 'Quick page scrubber',
      'reader.readingEnvironment': 'Reading environment',
      'reader.environmentChoice': '{theme} environment',
      'reader.documentTitle': 'Title',
      'reader.documentAuthor': 'Author',
      'reader.documentPages': 'Pages',
      'reader.documentSize': 'Size',
      'reader.documentModified': 'Modified',
      'reader.documentLocation': 'Location',
      'reader.pathCopied': 'Path copied.',
      'reader.copyPath': 'Copy path',
      'reader.readingMargins.description':
          'Separates the page from the edge without modifying the PDF.',
      'action.edit': 'Edit',
      'action.copyContent': 'Copy content',
      'annotation.type.highlight': 'Highlight',
      'annotation.type.underline': 'Underline',
      'annotation.type.strikeout': 'Strikeout',
      'annotation.type.textNote': 'Text note',
      'annotation.type.pageMarker': 'Visual marker',
      'annotation.type.pageComment': 'Page comment',
      'annotation.color.yellow': 'Soft yellow',
      'annotation.color.green': 'Soft green',
      'annotation.color.blue': 'Soft blue',
      'annotation.color.rose': 'Soft rose',
      'annotation.color.violet': 'Soft violet',
      'count.annotationsReview.one':
          '{count} annotation needs review because the document changed.',
      'count.annotationsReview.other':
          '{count} annotations need review because the document changed.',
      'settings.title': 'Settings',
      'settings.back': 'Back',
      'settings.language': 'Language',
      'settings.language.spanish': 'Castilian Spanish',
      'settings.language.english': 'English',
      'settings.appearance.description':
          'Theme, controls, and display profile.',
      'settings.language.description':
          'Apply the Spanish or English catalog without restarting.',
      'settings.darkMode': 'Dark application theme',
      'settings.darkMode.description':
          'Affects the reader and settings. The library keeps its dark, '
          'spacious environment.',
      'settings.density': 'Interface density',
      'settings.density.description':
          'Compact shows more content without changing the PDF size.',
      'settings.defaultReaderTheme': 'Default reading theme',
      'settings.defaultReaderTheme.description':
          'Canvas color when opening a new document.',
      'settings.autoHideControls': 'Hide controls automatically',
      'settings.autoHideControls.description':
          'Keeps the document clear while reading.',
      'settings.showProgress': 'Show progress bar',
      'settings.motion': 'Motion',
      'settings.motion.description':
          'Full, reduced, or no animations. The most restrictive system '
          'setting is respected.',
      'settings.visualProfile': 'Visual profile',
      'settings.visualProfile.description':
          'Electronic ink enforces high contrast, one page, and no motion.',
      'settings.reading.description': 'Initial behavior and navigation.',
      'settings.readingProfile': 'Reading profile',
      'settings.readingProfile.description':
          'Applies a coherent set without mixing interface density and content '
          'zoom.',
      'settings.defaultView': 'Default view',
      'settings.pageLayout': 'Page layout',
      'settings.pageLayout.description':
          'Facing view is enabled only when both pages are readable.',
      'settings.zoomMode': 'Zoom fit',
      'settings.margins': 'Margins',
      'settings.margins.description': 'Space around the document.',
      'settings.restorePosition': 'Restore last position',
      'settings.readingDirection': 'Reading direction',
      'settings.readerBar.description': 'Hierarchy and space for reader tools.',
      'settings.showPanelControls': 'Show panel shortcuts',
      'settings.showPanelControls.description':
          'Thumbnails, outline, and bookmarks remain under More options.',
      'settings.panelWidth': 'Side panel width',
      'settings.panelWidth.description':
          'Kept between sessions and adjustable by dragging its edge.',
      'settings.restoreSession': 'Restore session at startup',
      'settings.restoreSession.description':
          'Safely reopens the last document and its context.',
      'settings.startInLibrary': 'Always start in the library',
      'settings.startInLibrary.description':
          'Takes priority over automatic session restoration.',
      'settings.shortcuts.description': 'A limited, safe, and restorable set.',
      'settings.restoreShortcuts': 'Restore shortcuts',
      'settings.restoreShortcuts.description':
          'Restores default combinations. Escape, Home, End, and basic arrows '
          'remain reserved.',
      'settings.shortcut.description':
          'Only prevalidated combinations; conflicts are rejected.',
      'settings.library.description':
          'Recent documents, covers, and temporary storage.',
      'settings.showRecents': 'Show recent documents',
      'settings.generateCovers': 'Generate covers',
      'settings.generateCovers.description':
          'Locally selects the best of the first six pages and stores crisp '
          'thumbnails.',
      'settings.maxCache': 'Maximum cache size',
      'settings.clearCache': 'Clear cache',
      'settings.clearCache.description':
          'Removes covers and temporary renders.',
      'settings.clearCache.done': 'Cache cleared.',
      'settings.deleteHistory': 'Delete history',
      'settings.deleteHistory.description':
          'Resets progress, per-document preferences, bookmarks, and sessions. '
          'Keeps the library, annotations, and collections.',
      'settings.deleteHistory.confirmTitle': 'Delete history?',
      'settings.deleteHistory.confirmBody':
          'Progress, per-document preferences, bookmarks, and sessions will be '
          'reset. Documents, annotations, tags, and collections will be kept.',
      'settings.history.description':
          'Local activity grouped by day and under your control.',
      'settings.recordSessions': 'Record reading sessions',
      'settings.recordSessions.description':
          'Approximate active time only, stored locally.',
      'settings.historyLimit': 'History limit',
      'settings.historyLimit.description':
          'The oldest sessions are discarded automatically.',
      'settings.viewHistory': 'View reading history',
      'settings.viewHistory.description':
          'Review pages, duration, and date; delete individual entries or all '
          'of them.',
      'settings.viewHistory.action': 'View history',
      'settings.dataPrivacy.description':
          'Local data, versioned backups, and privacy.',
      'settings.rebuildableCleared': 'Rebuildable caches cleared.',
      'settings.deleteBackups.confirmTitle': 'Delete local backups?',
      'settings.deleteBackups.confirmBody':
          'Only previously exported backups will be deleted. PDFs and the '
          'library will remain intact.',
      'settings.exportBackup': 'Export local backup',
      'settings.exportBackup.description':
          'Includes the library, progress, bookmarks, history, and settings; '
          'never PDF content.',
      'settings.importBackup': 'Import local backup',
      'settings.importBackup.description':
          'Shows a summary and asks for confirmation. First creates an '
          'automatic backup of the current state.',
      'settings.openDataLocation': 'Open data location',
      'settings.deleteAllData': 'Delete all local data',
      'settings.deleteAllData.description':
          'Does not delete the original PDF files.',
      'settings.deleteAllData.confirmTitle': 'Delete all data?',
      'settings.deleteAllData.confirmBody':
          'The library, progress, bookmarks, sessions, settings, and cache will '
          'be deleted. Original PDFs will not be changed.',
      'settings.advanced.description':
          'Diagnostics, integrity, and local recovery.',
      'settings.diagnostics': 'Diagnostics center',
      'settings.diagnostics.description':
          'Versions, schema, sizes, resources, and integrity without paths, '
          'titles, queries, or content.',
      'settings.contextualHelp': 'Contextual help',
      'settings.contextualHelp.description':
          'Local summary of search, annotations, workspaces, and shortcuts.',
      'settings.localStorage': 'Local storage',
      'settings.measuringStorage': 'Calculating sizes…',
      'settings.originalsNotDeleted':
          'Original PDF files are not deleted by these actions.',
      'settings.clearRebuildable': 'Clear rebuildable data',
      'settings.deleteBackups': 'Delete backups',
      'settings.allCategories': 'All categories',
      'settings.localPreferences': 'Local preferences',
      'settings.localPreferences.description':
          'Reversible changes are stored immediately.',
      'settings.appearancePreview':
          'Preview: {theme} interface, {density} density, {canvas} canvas',
      'settings.preview': 'Preview',
      'settings.preview.description':
          'A live sample of the reader canvas, page, and chrome.',
      'settings.previewMode': 'Demonstration state',
      'settings.previewMode.resting': 'Resting',
      'settings.previewMode.active': 'Active',
      'settings.previewMode.panel': 'Panel open',
      'settings.previewMode.electronicInk': 'Electronic ink',
      'settings.navigationGroup.preferences': 'PREFERENCES',
      'settings.navigationGroup.maintenance': 'MAINTENANCE',
      'settings.navigationGroup.privacy': 'PRIVACY',
      'settings.historyEntry': 'Pages {start}–{end} · {duration}',
      'settings.deleteSession': 'Delete this session',
      'settings.historyTitle': 'Reading history',
      'settings.noSessions': 'No saved sessions',
      'settings.noSessions.description':
          'History will appear here when you close a reading session.',
      'settings.updated': 'Setting updated',
      'settings.saveFailed': 'The setting could not be saved: {error}',
      'settings.section.appearance': 'Appearance',
      'settings.section.reading': 'Reading',
      'settings.section.readerBar': 'Reader controls',
      'settings.section.shortcuts': 'Shortcuts',
      'settings.section.library': 'Library',
      'settings.section.history': 'History',
      'settings.section.dataPrivacy': 'Data and privacy',
      'settings.section.advanced': 'Advanced',
      'settings.value.comfortable': 'Comfortable',
      'settings.value.compact': 'Compact',
      'settings.value.light': 'Light',
      'settings.value.ivory': 'Ivory',
      'settings.value.sepia': 'Sepia',
      'settings.value.gray': 'Gray',
      'settings.value.dark': 'Dark',
      'settings.value.black': 'Black',
      'settings.value.motionFull': 'Full',
      'settings.value.motionReduced': 'Reduced',
      'settings.value.motionNone': 'No animations',
      'settings.value.standard': 'Standard',
      'settings.value.electronicInk': 'Electronic ink',
      'settings.value.reading': 'Reading',
      'settings.value.study': 'Study',
      'settings.value.presentation': 'Presentation',
      'settings.value.night': 'Night',
      'settings.value.comparison': 'Comparison',
      'settings.value.document': 'Document',
      'settings.value.book': 'Book',
      'settings.value.singlePage': 'Single page',
      'settings.value.facingPages': 'Facing pages',
      'settings.value.continuous': 'Continuous',
      'settings.value.fitWidth': 'Fit to width',
      'settings.value.fitPage': 'Fit to page',
      'settings.value.leftToRight': 'Left to right',
      'settings.value.rightToLeft': 'Right to left',
      'storage.database.title': 'Database',
      'storage.database.description':
          'Library, positions, and local organization.',
      'storage.covers.title': 'Covers',
      'storage.covers.description': 'Rebuildable document covers.',
      'storage.renderCache.title': 'Reading cache',
      'storage.renderCache.description': 'Rebuildable reading images.',
      'storage.searchIndex.title': 'Search index',
      'storage.searchIndex.description': 'Rebuildable text and metadata index.',
      'storage.logs.title': 'Logs',
      'storage.logs.description': 'Redacted technical logs.',
      'storage.backups.title': 'Backups',
      'storage.backups.description': 'Backups requested by the user.',
      'storage.preferences.title': 'Preferences',
      'storage.preferences.description': 'Application preferences.',
      'backup.fileType': 'Yamantaka local backup',
      'backup.exported': 'Local backup exported.',
      'backup.exportFailed': 'Export failed: {error}',
      'backup.imported': 'Backup imported. Previous backup stored at {path}',
      'backup.importFailed': 'Import failed. Current data was preserved.',
      'backup.importPreview':
          'Version {version} · {documents} documents · {bookmarks} bookmarks · '
          '{annotations} annotations · {sessions} sessions.\n\nCurrent local '
          'data will be replaced. An automatic backup will be stored first. '
          'PDFs are not modified.',
      'count.sessions.one': '{count} session',
      'count.sessions.other': '{count} sessions',
      'shortcut.actionReserved': 'This action is reserved.',
      'shortcut.shift': 'Shift',
      'shortcut.pageDown': 'Page Down',
      'shortcut.pageUp': 'Page Up',
      'shortcut.navigationReserved':
          'This combination is reserved for navigation.',
      'shortcut.notAllowed': 'This combination is not allowed.',
      'shortcut.alreadyAssigned':
          'This combination is already assigned to another action.',
      'privacy.title': 'Privacy and storage',
      'privacy.localOnly': 'Data stays on this device.',
      'backup.title': 'Local backup',
      'help.title': 'Help',
      'help.search': 'Search help',
      'help.noResults': 'No topics match your search.',
      'search.title': 'Search library',
      'search.field': 'Documents, content, notes, and tags',
      'search.emptyTitle': 'Unified local search',
      'search.emptyBody':
          'Search metadata, extractable content, annotations, collections, '
          'and tags. The index stays on this device.',
      'search.noResultsTitle': 'No results',
      'search.noResultsBody':
          'The document may not contain extractable text or may not be indexed '
          'yet. Yamantaka does not perform OCR.',
      'search.kind.metadata': 'Document',
      'search.kind.pageText': 'Page text',
      'search.kind.annotation': 'Annotation',
      'search.kind.collection': 'Collection',
      'search.kind.tag': 'Tag',
      'search.indexing': 'Indexing {current} of {total}',
      'search.indexEnabled': 'Local index enabled',
      'search.indexDisabled': 'Local index disabled',
      'search.updateIndex': 'Update index',
      'search.indexMaintenance': 'Index maintenance',
      'search.rebuildIndex': 'Rebuild index',
      'search.clearIndex': 'Clear index',
      'command.searchHint': 'Search for an action',
      'command.section': 'Commands',
      'command.move': 'Move',
      'command.run': 'Run',
      'command.empty': 'No matching actions.',
      'command.recentDocument': 'Recent document',
      'command.recentSection': 'Recent documents',
      'command.category.document': 'Document and application',
      'command.category.navigation': 'Navigation',
      'command.category.panel': 'Side panel',
      'command.category.annotation': 'Reading and annotation',
      'command.category.view': 'Reader view',
      'command.preview.selection': 'SELECTED RESULT',
      'command.preview.safeAction':
          'Immediate action; Enter runs it and Escape returns to the reader.',
      'command.preview.enterHint': 'Press Enter to run',
      'command.openDocument': 'Open document',
      'command.search': 'Search in document',
      'command.commandPalette': 'Open command palette',
      'command.goToPage': 'Go to page',
      'command.nextPage': 'Next page',
      'command.previousPage': 'Previous page',
      'command.togglePanel': 'Open or close panel',
      'command.toggleBookmark': 'Add or remove bookmark',
      'command.zoomIn': 'Zoom in',
      'command.zoomOut': 'Zoom out',
      'command.resetZoom': 'Reset zoom',
      'command.actualSize': 'Actual size (100%)',
      'command.singlePageLayout': 'Layout: Single page',
      'command.facingPageLayout': 'Layout: Facing pages',
      'command.continuousPageLayout': 'Layout: Continuous',
      'command.showThumbnails': 'Show thumbnails',
      'command.showOutline': 'Show outline',
      'command.showAnnotations': 'Show annotations',
      'command.navigationBack': 'Back to previous location',
      'command.navigationForward': 'Forward in history',
      'command.savePositionA': 'Save temporary position A',
      'command.savePositionB': 'Save temporary position B',
      'command.goToPositionA': 'Go to temporary position A',
      'command.goToPositionB': 'Go to temporary position B',
      'command.swapPositions': 'Swap positions A and B',
      'command.undoAnnotation': 'Undo annotation change',
      'command.redoAnnotation': 'Redo annotation change',
      'command.fitWidth': 'Fit to width',
      'command.fitPage': 'Fit to page',
      'command.changeReaderTheme': 'Change reading theme',
      'command.documentInfo': 'Document information',
      'command.fullScreen': 'Full screen',
      'command.toggleChrome': 'Show or hide interface',
      'command.toggleImmersive': 'Toggle immersive reading',
      'command.settings': 'Open settings',
      'failure.none.title': 'No document is open',
      'failure.none.message': 'Choose a local PDF to start reading.',
      'failure.fileNotFound.title': 'The file is no longer available',
      'failure.fileNotFound.message':
          'You can locate another copy of the same PDF or go back.',
      'failure.accessDenied.title': 'The file cannot be read',
      'failure.accessDenied.message':
          'Check the file permissions and try again.',
      'failure.corruptPdf.title': 'The PDF cannot be opened',
      'failure.corruptPdf.message':
          'The file may be damaged or use an unsupported variant.',
      'failure.emptyPdf.title': 'The PDF contains no pages',
      'failure.emptyPdf.message': 'Choose a document with at least one page.',
      'failure.passwordRequired.title': 'The PDF is protected',
      'failure.passwordRequired.message':
          'Enter the password to continue. It will not be stored.',
      'failure.wrongPassword.title': 'The password is incorrect',
      'failure.wrongPassword.message':
          'Enter it again or choose another document.',
      'failure.unsupportedFormat.title': 'Unsupported format',
      'failure.unsupportedFormat.message': 'Yamantaka opens PDF files only.',
      'failure.renderFailed.title': 'The page could not be rendered',
      'failure.renderFailed.message':
          'You can retry without changing the original PDF.',
      'failure.outOfMemory.title': 'Not enough memory',
      'failure.outOfMemory.message':
          'Close other documents or reduce background work.',
      'failure.databaseFailed.title': 'The library could not be read',
      'failure.databaseFailed.message':
          'Check the local database in Diagnostics and try again.',
      'failure.thumbnailFailed.title': 'The cover could not be generated',
      'failure.thumbnailFailed.message':
          'Reading can continue and the cover can be retried later.',
      'failure.restoreFailed.title': 'The session could not be restored',
      'failure.restoreFailed.message':
          'The library remains available; you can open the PDF manually.',
      'failure.cancelled.title': 'Operation cancelled',
      'failure.cancelled.message': 'No stale result was applied.',
      'failure.invalidInput.title': 'Invalid input',
      'failure.invalidInput.message': 'Review the data and try again.',
      'failure.unauthorizedLink.title': 'Link blocked',
      'failure.unauthorizedLink.message':
          'The link scheme is not authorized by Yamantaka.',
      'failure.unknown.title': 'The operation could not be completed',
      'failure.unknown.message':
          'Confirmed data was preserved. You can try again.',
      'welcome.title': 'Start with your local library',
      'welcome.body':
          'Open a PDF, keep your position, and keep your data on this device. '
          'There are no accounts or automatic cloud copies.',
      'welcome.open': 'Open a PDF',
      'welcome.help': 'View help',
      'count.documents.one': '{count} document',
      'count.documents.other': '{count} documents',
      'count.results.one': '{count} result',
      'count.results.other': '{count} results',
    },
  };
}

final class _YamantakaLocalizationsDelegate
    extends LocalizationsDelegate<YamantakaLocalizations> {
  const _YamantakaLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) =>
      const ['es', 'en'].contains(locale.languageCode);

  @override
  Future<YamantakaLocalizations> load(Locale locale) => SynchronousFuture(
    YamantakaLocalizations(AppLanguageCode.fromCode(locale.languageCode)),
  );

  @override
  bool shouldReload(_YamantakaLocalizationsDelegate old) => false;
}
