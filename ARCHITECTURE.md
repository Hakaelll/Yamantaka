# Arquitectura

## Límites

Yamantaka usa módulos por función con cuatro responsabilidades pragmáticas:
dominio tipado, casos de uso/controladores, infraestructura y presentación.
Los widgets no ejecutan SQL ni llaman a PDFium directamente.

`PdfDocumentEngine` es el límite entre el producto y `pdfrx`. Los modelos del
dominio no importan paquetes de UI, SQLite o PDF. `AppServices` compone
dependencias durante el arranque; cada estado mutable vive en un controlador
pequeño y observable.

La copia mínima de pdfrx 2.4.7 en `third_party/pdfrx` corrige temporalmente el
ciclo de vida del codec de imagen en Windows. Está aislada, conserva la licencia
MIT y no cambia los contratos de Yamantaka; ADR-009 e YAM-014 definen su retirada.

## Apertura

1. Una fuente de entrada (selector, drop o argumento) entrega una ruta.
2. `DocumentService` valida extensión, existencia, tamaño y ruta normalizada.
3. El motor abre el PDF fuera del flujo de pintura y devuelve metadatos.
4. El repositorio inserta o actualiza el documento sin duplicar la ruta.
5. El lector restaura preferencias y posición.
6. La vista solicita únicamente páginas visibles y próximas.

Los errores técnicos se convierten en `AppFailure` antes de llegar a la UI.

## Renderizado

El adaptador conserva un mapa de documentos abiertos. Las solicitudes llevan
documento, página, escala, rotación y región. Una caché LRU limita imágenes por
peso estimado, deduplica solicitudes y libera recursos al cerrar. Los renders
activos reciben un token de cancelación; el cierre los cancela y espera su
finalización antes de disponer PDFium. El primer hito usa el visor virtualizado
de `pdfrx`; el contrato de regiones queda listo para teselas futuras.

El widget concreto queda detrás de `PdfViewerAdapter`. Informa página, zoom y
fracción visible de la página a `PdfViewController`; `ReaderController` aplica
debounce y persiste esos datos. Al reabrir, el adaptador restaura primero el
modo de zoom y luego el desplazamiento aproximado.

`ReaderScreen` conserva el widget del adaptador con una clave formada solo por
documento y configuración de render. Panel, resize, chrome, progreso y
marcadores no reconstruyen el visor. El panel monta una única sección; su ancho
se guarda después de 240 ms sin cambios. `PdfrxDocumentEngine.diagnostics`
expone únicamente contadores locales de solicitudes, caché, deduplicación,
cancelación y recursos abiertos para pruebas/Profile; no persiste ni transmite.

## Persistencia

SQLite v3 guarda documentos, preferencias, marcadores, sesiones, anotaciones,
etiquetas, colecciones, workspaces, perfiles e índice FTS reconstruible. Las
migraciones son incrementales, crean una copia pre-migración y se ejecutan en
transacción; `PRAGMA integrity_check` cierra el gate. Los PDF permanecen en su
ubicación original.

Preferencias globales pequeñas, idioma y geometría de ventana usan el almacén
nativo asíncrono. Caché y miniaturas viven en el directorio de caché de la
aplicación. `document_covers` conserva página fuente, miniaturas pequeña/grande,
fecha, modificación del origen, confianza, modo manual/automático, versión de
algoritmo, recorte y proporción. El límite elimina primero las más antiguas y
limpia su referencia en SQLite.

La posición se guarda al cambiar de página, al cerrar y con debounce; nunca por
cada píxel. `LibraryController` retiene selección, scroll, consulta y filtros al
entrar en el lector sin escribir la consulta en disco. El estado de lectura
seleccionado, atajos, panel, sección, ancho, chrome y política de recuperación
viven como preferencias globales validadas.

## Portadas y biblioteca de artefactos

`ThumbnailService` es el único caso de uso que renderiza y persiste portadas.
Puntúa como máximo seis páginas a escala baja combinando prioridad de posición,
densidad de bordes, contraste, espacio uniforme, longitud/líneas del texto y
marcadores de índice. Si la mejora no supera el umbral vuelve a la página uno.
Después renderiza una miniatura pequeña y otra de selección, actualiza SQLite y
notifica a la biblioteca. La cola de importación es secuencial, cancelable por
documento y nunca bloquea su registro inicial.

Una elección manual conserva página y modo. Si cambia el archivo, la imagen se
considera inválida y se vuelve a renderizar desde esa página; el algoritmo no
la sustituye. El selector abre el PDF localmente, muestra hasta seis previews,
acepta cualquier número de página válido y cierra siempre el handle.

`LibraryArtifactLayout` es puro y probado. Un hash FNV estable del ID produce
desplazamientos de ±8/±12 px, giro de ±1,25° y una variación mínima de escala.
La geometría mantiene celdas invisibles sin solapamiento. De siete documentos
en adelante la UI usa `SliverGrid.builder`; selección, scroll, búsqueda y orden
no alteran el jitter. `ArtifactDocument` solo pinta/expone semántica y emite
acciones; persistencia, PDFium y SQL quedan fuera del widget.

## Copias, historial y recuperación

`LocalBackupService` es el único límite de exportación/importación. Serializa
un contenedor JSON `yamantaka-local-backup` versión 2 con ajustes y tablas
privadas permitidas, pero nunca bytes PDF ni FTS reconstruible. También importa
v1 con las tablas nuevas vacías. La vista previa valida formato, versión,
columnas, tipos y recuentos antes de mutar. La importación crea primero otra
copia local y reemplaza las filas en una transacción, respetando el orden de
claves foráneas.

`ReadingSessionRepository` compone historial mediante un join acotado; la
presentación agrupa fechas, pero no calcula perfiles personales. El límite se
aplica al cerrar sesiones y al cambiar la preferencia.

La recuperación de arranque es una política del shell: ruta explícita gana;
“iniciar en biblioteca” o desactivación impiden restaurar; solo se intenta una
vez. Una ruta ausente o apertura fallida se elimina de la preferencia para
evitar bucles. El contexto de página/zoom/layout sigue perteneciendo al
repositorio por documento; panel/chrome pertenecen al shell.

## Estado

- `LibraryController`: recientes, ausentes y continuar.
- `ReaderController`: documento activo, página, zoom, diseño y rotación.
- `SearchController`: consulta, resultados, selección y cancelación.
- `GlobalSearchController`/`LibraryIndexController`: FTS opt-in, consulta
  paginada e indexación secuencial cancelable.
- `BookmarkController`: marcadores del documento.
- `AnnotationController`: CRUD, filtros, estado de revisión y undo.
- `DocumentTabsController`/`WorkspaceController`: contextos documentales
  acotados y snapshots transaccionales sin mantener todos los PDF abiertos ni
  mostrar una barra de pestañas.
- `OrganizationController`: colecciones, etiquetas y vistas inteligentes.
- `SettingsController`: preferencias globales y perfil visual.
- `AppCommandRegistry`: definiciones, opciones seguras, conflictos, bindings y
  etiquetas resueltas sin ejecutar lógica de dominio.
- Estado local de vista: consulta de biblioteca, panel activo, drag temporal y
  visibilidad inmediata de controles.

Los modelos son inmutables. `ListenableBuilder` limita reconstrucciones al
controlador relevante.

## Multiplataforma

Flutter contiene dominio, casos de uso y UI. Adaptadores encapsulan rutas,
ventana, selector y PDFium. Los breakpoints no suponen ratón ni hover. Android e
iOS comparten el mismo núcleo; futuras variantes pueden reemplazar motor,
almacén de rutas o entrada sin reescribir el lector.

## Seguridad

Los PDF son entradas no confiables. Se valida el archivo, se limita la escala,
se capturan fallos del motor y no se ejecuta JavaScript incrustado. Los destinos
internos navegan dentro del visor. Un enlace externo muestra URL y confirmación,
solo admite `http`, `https` o `mailto` y se abre en la aplicación del sistema.
Los logs omiten texto, rutas, nombres, consultas, contraseñas y contenido. El
fork temporal de pdfrx elimina también mensajes debug que interpolaban la
clave/ruta de `PdfDocumentRef` o detalles de fuentes embebidas. Copias e
historial no abren conexiones y las rutas solo aparecen en UI/archivo exportado
por acción explícita.

## Tercera iteración: datos y ciclo de vida

Las anotaciones pertenecen al documento por huella y página. Su rectángulo se
normaliza en rango 0–1 para sobrevivir a DPI, zoom y rotación. Si la identidad
de contenido deja de coincidir, el registro se conserva como `needsReview`; no
se recoloca por heurística. El panel opera sobre una lista acotada por documento
y nunca modifica bytes PDF.

Los contextos documentales no tienen una barra visible. Solo el documento
activo y como máximo un contexto vecino pueden estar calientes; los demás
conservan estado serializable y quedan suspendidos. Un workspace guarda orden,
pin, documento activo y navegación en una transacción. El contrato
`SplitViewState` ya define
orientación/ratio y la frontera exacta de 1100 px, pero todavía no instancia un
segundo visor: split y comparación siguen bloqueados hasta medir ownership,
cierre y memoria de dos motores reales.

La búsqueda global usa FTS5 únicamente tras opt-in de la sesión. Indexa un PDF
cada vez, cierra su handle incluso en error y no ejecuta OCR. Consultas y
snippets son efímeros; el índice puede pausarse, cancelarse, reconstruirse o
borrarse y se excluye del backup. Colecciones y vistas inteligentes son
relaciones/consultas locales; no mueven archivos ni vigilan carpetas.

Cada arranque marca una sesión no cerrada y cada cierre normal la limpia. Tres
arranques consecutivos no limpios activan safe mode: preferencias visuales por
defecto, sin restaurar workspace/documento y sin borrar datos. Diagnóstico
consume una allowlist de versión, plataforma, esquema, conteos, integridad,
caché y métricas del motor; enmascara rutas y excluye títulos, consultas y
contenido de anotaciones.
