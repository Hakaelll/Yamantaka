# Arquitectura

## Presentación editorial 0.15

`library_navigation.dart` es una parte de Biblioteca que delega en sus filtros
y recarga paginada existentes. No filtra en memoria una página parcial ni
altera el máximo de 80 documentos materializados. El cálculo de navegación
por teclado usa el ancho real del catálogo, descontando el rail.

Biblioteca y el chrome del lector consumen el tema global. La configuración
del lienzo permanece separada y no cambia ownership, cachés ni geometría de
PDFium. Los filtros de anotaciones se redistribuyen en la cabecera, no mediante
la reconstrucción del visor.

Las elecciones visuales de consolidación se guardan por pareja de candidatos
y campo; cambiar de candidato no aplica silenciosamente otra decisión.
SQLite v11 y el servicio transaccional de consolidación no cambian.

## Límites

Yamantaka usa módulos por función con cuatro responsabilidades pragmáticas:
dominio tipado, casos de uso/controladores, infraestructura y presentación.
Los widgets no ejecutan SQL ni llaman a PDFium directamente.

`PdfDocumentEngine` es el límite entre el producto y `pdfrx`. Los modelos del
dominio no importan paquetes de UI, SQLite o PDF. `AppServices` compone
dependencias durante el arranque; cada estado mutable vive en un controlador
pequeño y observable.

La copia mínima de pdfrx 2.4.7 en `third_party/pdfrx` corrige temporalmente el
ciclo de vida del codec de imagen en Windows y contiene el render progresivo y
la virtualización vertical instrumentada. Está aislada, conserva la licencia
MIT y no cambia los contratos de Yamantaka; `PATCHES.md` inventaría cada cambio
y ADR-009 e YAM-014 definen su retirada.

## Apertura

1. Una fuente de entrada (selector, drop o argumento) entrega una ruta.
2. `DocumentService` valida extensión, existencia, tamaño y ruta normalizada.
3. El motor abre el PDF fuera del flujo de pintura y devuelve metadatos.
4. El repositorio inserta o actualiza el documento sin duplicar la ruta.
5. El lector restaura preferencias y posición.
6. La vista solicita únicamente páginas visibles y próximas.

Los errores técnicos se convierten en `AppFailure` antes de llegar a la UI.

## Renderizado

El adaptador conserva un mapa de documentos abiertos. Cada solicitud se
identifica por huella de contenido, página, rotación, escala física, DPI,
región y generación. El coordinador deduplica trabajo nativo, pero entrega un
handle por consumidor: cancelar una miniatura no cancela la página visible si
ambas comparten render. Visible, próxima, prefetch y miniatura tienen prioridad
decreciente. Subir de generación cancela trabajo anterior y un resultado
obsoleto se descarta antes de publicarlo.

Páginas completas (72 MiB), regiones/teselas (24 MiB), texto estructurado/plano
(8 MiB) y miniaturas en disco tienen ownership y presupuestos separados. Un
bitmap se limita a 16.384 px por eje y 192 MiB antes de llegar a PDFium. El
`MemoryGovernor` reduce las LRU bajo presión; desalojar, cerrar o limpiar libera
la `ui.Image` una sola vez. El cierre cancela y espera jobs/tokens nativos antes
de disponer el documento. Los invariantes completos están en
`docs/architecture/pdf-rendering-0.4.md`.

El widget concreto queda detrás de `PdfViewerAdapter`. Informa página, zoom y
fracción visible de la página a `PdfViewController`; `ReaderController` aplica
debounce y persiste esos datos. Al reabrir, el adaptador restaura primero el
modo de zoom y luego el desplazamiento aproximado.

`ReaderScreen` conserva el widget del adaptador con una clave formada solo por
documento y configuración de render. Panel, resize, chrome, progreso y
marcadores no reconstruyen el visor. El panel monta una única sección; en
compacto/medio es overlay con scrim y en expandido su ancho se resta de las
restricciones del visor para recentrar la página. Su ancho se guarda después de
la duración semántica de persistencia sin cambios. `PdfrxDocumentEngine.diagnostics`
expone únicamente contadores locales de solicitudes, caché, deduplicación,
cancelación y recursos abiertos para pruebas/Profile; no persiste ni transmite.

`ReadingUnitNavigation` define la unidad que se lee: una página, una secuencia
continua o un pliego. Barra inferior, zonas laterales, flechas, `PageUp`,
`PageDown`, `Home`, `End` y espacio llaman a las mismas operaciones de
`ReaderController`; ningún widget calcula índices por su cuenta. Cambiar de modo
normaliza la página activa. `PageLayoutMode` expone directamente una página,
dos páginas y continua; `coverPolicy` e `initialSide` corrigen la paridad del
pliego por documento sin reordenar ni modificar el PDF.

`ReaderViewportMetrics` es la única geometría de los modos paginados. Calcula
slots, márgenes discretos, lomo de 10–16 px, rectángulo de página/pliego, escala
de ajuste y centro permitido. El normalizador de `pdfrx` lo aplica a cada
transformación: si el contenido cabe elimina offsets residuales y, si desborda,
limita el paneo sin perder zonas. El mismo cálculo se reutiliza en zoom manual,
ajuste de página/ancho y resize. Cada instancia nativa adjunta el
`PdfViewController` con un propietario para que la destrucción tardía de una
vista anterior no desconecte la nueva durante un cambio de modo.

El layout reserva únicamente geometría liviana para todas las páginas. En modo
Continuo una búsqueda binaria encuentra el rango que cruza viewport y anillo de
precarga, por lo que el trabajo de pintura no crece linealmente con 5.000
páginas. `pdfrx` agenda visibles antes que próximas, cancela el exterior y
mantiene el preview anterior hasta refinar la región tras 75 ms de reposo. La
previsualización se limita a 110 DPI y el fundido final dura 90 ms o cero con
movimiento reducido.

## Persistencia

SQLite v11 guarda documentos, preferencias, marcadores, sesiones, anotaciones,
etiquetas, colecciones, workspaces, perfiles, enriquecimiento, relaciones de
temas e índice FTS reconstruible. Las
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

`DocumentEnrichmentController` recorre secuencialmente los documentos nuevos o
con huella/versión obsoleta y se pausa durante interacción/lectura activa. El
usuario puede reanalizar una selección o toda la biblioteca sin tocar campos
manuales. El motor expone metadatos PDFium y bloques estructurados de hasta
cuatro páginas iniciales; el caso de uso limita la muestra a 12 páginas/64 KiB
y no la persiste. `document_enrichment` conserva estado,
sugerencias, confianza, huella y versión; `document_topics` separa procedencia
automática/manual. Las consultas SQL calculan facetas y recuentos sobre toda la
biblioteca aunque la presentación esté paginada.

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
un contenedor JSON `yamantaka-local-backup` versión 9 con ajustes y tablas
privadas permitidas, pero nunca bytes PDF ni FTS reconstruible. También importa
los formatos 1–8 con las tablas posteriores vacías y defaults seguros. La vista
previa valida formato, versión,
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

## Iteracion de escritorio y ciclo de vida de render — 2026-08-01

El contrato `RenderedPdfPage` devuelve una imagen prestada por el motor. La
cache LRU conserva `ui.Image` directamente y es la unica propietaria de su
liberacion al desalojar, limpiar o cerrar un documento. Las miniaturas, el
panel lateral y otros consumidores solo leen o codifican la imagen; nunca la
disponen. Las imagenes que superan el presupuesto se rastrean por documento y
se liberan al cerrar o limpiar para no convertir una pagina grande en una
fuga.

`PdfOperationToken` tiene listeners de cancelacion. El motor conecta cada
operacion con el token nativo de pdfrx y comprueba la cancelacion antes de
publicar una imagen en cache. Las portadas en segundo plano y las miniaturas
del panel cancelan trabajos obsoletos, deduplican por pagina/escala y limpian
archivos parciales si la operacion no llega a persistirse.

El viewport sigue siendo la fuente unica para unidad de lectura, ajuste,
centrado, top-align de `Ajustar ancho` y restauracion de zoom/offset del
historial. El lector conserva el mismo contrato para Documento y Libro,
incluye portada sola y pliegues 2-3, 4-5, y oculta tambien las zonas laterales
de avance en pantalla completa.

La politica de escritorio valida explicitamente 16:9, 16:10, 1920x1080 y
2560x1440. La biblioteca ajusta slots horizontales independientes de la
altura de fila: evita overflow en compactos y usa hasta 8 columnas en Full HD
y 10 en 2K cuando hay muchos artefactos. Portadas y miniaturas solicitan
ancho/alto a DPR y filtrado de alta calidad.

Los atajos Page Up/Page Down permanecen reservados para navegacion estructural;
el registro rechaza asignaciones y overrides persistidos que intenten
usarlos para otra accion.

## Composición contextual 0.6

La iteración 0.6 mantiene el chrome progresivo fuera de la clave del visor: la
barra superior puede ocultarse, mientras `ReaderBottomControls` cambia entre
una dock de reposo y los controles activos mediante estado de presentación. El
panel acoplado cambia su selector horizontal por un rail, pero continúa
montando solo el contenido de la sección activa. El feedback del divisor es
estado efímero del widget y el ancho persistente conserva el debounce existente.

`ArtifactLibraryView` sigue siendo dueño del contexto de selección. El inspector
recibe un `LibraryDocument` ya materializado y callbacks; no consulta SQLite ni
abre PDFium. `LibraryController` conserva el ID aunque un filtro lo excluya y la
presentación ofrece restablecer el query. La previsualización de Ajustes consume
`AppSettings`, tokens y componentes de chrome, sin crear un segundo lector.

## Retícula visual 0.7 y estabilidad del visor

La iteración 0.7 amplía únicamente presentación y contratos visuales. Los
aliases de estado y `AppTypeScale` viven en el tema; Biblioteca materializa el
conteo de filtros y el documento seleccionado ya cargado, sin consultas ni
apertura de PDFium desde el inspector. Los estados de enriquecimiento son
marcas derivadas de `LibraryDocument`.

`ReaderTopBar`, `ReaderBottomControls` y el rail lateral cambian de composición
sin entrar en `_viewerCacheKey`. La navegación de la dock se centra mediante un
`Stack` solo cuando existe ancho suficiente; compacto y medio usan la misma
familia en flujo. Mostrar chrome, cambiar la sección, hacer hover o arrastrar el
divisor no reemplaza el adaptador ni el `RepaintBoundary` del PDF.

La preview de Ajustes es un diagrama de widgets sin `PdfDocumentRef`: su selector
vive en estado local y nunca persiste. La paleta calcula su preview desde la
entrada ya filtrada y documentos recientes materializados. No se añadieron red,
servicios, dependencias, tablas ni cambios de esquema.

## Archivo privado 0.8 y límites de estado

`ArtifactLibraryView` incorpora un modo de vista efímero `grid/list`. Las dos
composiciones consumen la misma lista materializada, `ValueListenable` de
selección y callbacks existentes; no se añade tabla, migración ni preferencia.
La banda de filtros deriva exclusivamente del estado actual del controlador y
cada chip revierte su grupo mediante la API ya disponible. La lista cambia a
fila compacta según ancho y escalado de texto sin perder estado semántico.

`GlobalSearchDialog` puede dividir resultados y preview, pero la selección sigue
perteneciendo a `GlobalSearchController` y la preview no indexa ni abre un
segundo documento. El editor de metadatos mantiene por separado fuente
detectada, override y borrador de UI; cancelar descarta el borrador y guardar
solo escribe metadatos locales.

Los componentes compartidos `AppMenuItemContent`, `AppDialogHeader` y
`AppDangerZone` son presentación pura. `ReaderTopBar`,
`ReaderBottomControls` y el panel cambian forma y etiquetas sin aportar panel,
chrome, hover, foco, breakpoint o tamaño a `_viewerCacheKey`. PDFium, SQLite,
backup, red y modelo de persistencia permanecen sin cambios.

La marca se deriva de los assets rastreados en `HEAD` mediante transformación
de croma 1:1. El script conserva dimensiones, alfa, coordenadas y luminancia
local y aplica la misma operación a cada frame ICO; no usa el resultado
generativo como fuente del producto.

## Densidad 0.9 y margen responsive sin identidad nueva

`AppSizes` concentra las medidas verificables del chrome 0.9: barra 56/52 px,
dock 48/38 px, cabecera y rail lateral de 52 px. Ninguna entra en
`_viewerCacheKey`. El visor cacheado sigue dependiendo del documento y de
configuración PDF estable; abrir panel, cambiar sección, ocultar chrome, pasar
un breakpoint o redimensionar la ventana conserva la misma instancia.

El margen preferido del documento se entrega estable a `PdfrxViewerAdapter`.
`_PdfrxViewerState` deriva el margen efectivo de sus constraints (12/16/24/32
px) y actualiza `PdfViewerParams` desde el mismo estado. Así la página gana área
útil y se recentra en la columna restante sin reconstruir el documento ni
alterar PDFium, caché, zoom, paneo o renderizado progresivo.

El modo grid/list continúa siendo estado efímero. La transición 0.9 envuelve la
composición siguiente en un único `TweenAnimationBuilder`, evitando montar dos
scrollables con el mismo controlador; no reinicia selección, filtros ni offset.
No hay migraciones, dependencias, telemetría ni nuevos accesos de red.

## Continuidad y métricas locales 0.10

`DocumentTabsController` continúa almacenando solo estado ligero y aplica tres
estados de recurso: activo, caliente y suspendido. `ReaderController` mantiene
un mapa acotado a `AppConfig.maxWarmTabs` de handles `OpenedPdfDocument`; al
cambiar de pestaña guarda posición/preferencias, desconecta el viewer y conserva
como máximo dos documentos nativos. La política de pestañas decide qué IDs
pueden seguir calientes y `retainWarmDocuments` cierra de forma determinista el
resto. Volver a un documento con fingerprint idéntico reutiliza su handle; un
cambio externo invalida y cierra la entrada antes de abrir de nuevo.

El selector avanzado traduce número, porcentaje e inicio/fin a una página
canónica antes de tocar el controlador. Una entrada inválida permanece en el
diálogo y el historial usa `NavigationHistoryController`; no se persiste texto
de entrada.

La búsqueda global filtra tipos dentro de SQL FTS5 y conserva placeholders para
todos los valores. Anotaciones, marcadores y organización se regeneran una vez
por sesión de diálogo, no por pulsación; cada consulta usa debounce, generación
cancelable y paginación. Los resultados unen únicamente flags locales de
ausencia/modificación y nunca exponen rutas ni texto en diagnósticos.

`StartupTimeline` distingue bootstrap, base de datos, primera Biblioteca,
primer frame y primer PDF. `LocalPerformanceMonitor` registra duraciones con
nombres constantes (`library_first_page`, `library_page_more`,
`global_search`, `pdf_open` y renderizado); ningún sample contiene documento,
ruta, título, consulta o página.

## Navegación, sesiones e índice reanudable 0.11

`DocumentNavigatorDialog` agrega destinos ya existentes del lector sin abrir
otro documento ni entrar en `_viewerCacheKey`. El filtro consulta índice,
miniaturas, resultados internos, marcadores, anotaciones, historial y A/B; la
activación termina siempre en `ReaderController.navigateTo`, por lo que la
historia conserva el origen y el visor mantiene zoom y handle. El adaptador
solo se reconstruye por identidad o configuración PDF estable.

`PdfViewConfiguration.onTextSelectionChanged` es el único puente nuevo hacia
`pdfrx`. El adaptador lee rangos permitidos por copia, convierte los bounds de
fragmento a `NormalizedRect` usando el tamaño real de página y entrega un modelo
sin dependencia del motor. `ReaderController` acepta geometría precisa solo
para una página y rectángulos válidos; en selección multipágina, sin texto o
sin bounds crea una nota de página sin rectángulos. El repositorio de
anotaciones conserva undo/redo, edición, huella y exportación versionada.

El contexto adicional de sesión, cerrados recientes y metas se serializa bajo
claves versionadas de `app_metadata`. Esta tabla ya forma parte del backup y
permite evolucionar datos auxiliares sin una migración del esquema ni del
formato de copia. Las entradas guardan estado de navegación y rutas locales,
pero nunca consultas ni texto PDF. Los handles siguen sujetos a dos calientes
y las pestañas fijadas a tres.

`LibraryGrouping` es presentación efímera sobre la página SQL materializada.
Colecciones y etiquetas cargan sus membresías completas mediante una consulta
por relación, no una consulta por fila. Cada documento se asigna a una sola
sección visible y plegar no cambia selección ni offset. `LibraryQuery` limita
todo resultado a 80 elementos.

`search_index_state.indexed_pages` es un cursor contiguo seguro. Las páginas de
texto ya escritas se detectan por SQL para conservar prioridad fuera de orden;
solo se avanza el cursor cuando el prefijo está completo. Una huella distinta,
estado fallido o cursor incompatible reinicia el documento. El índice abre un
handle propio porque el handle del visor pertenece al árbol y ciclo de vida de
`pdfrx`; compartirlo cruzaría propietarios nativos y podría cerrar o usar un
documento mientras el viewer renderiza. Ambos handles se cierran de forma
determinista y la cola sigue limitada a un trabajo de indexación.

Las tarjetas y filas materializadas notifican entrada y salida del viewport.
Entrar eleva la portada a prioridad visible; salir cancela el trabajo
recreable pendiente. `ThumbnailService` cuenta intentos de render,
decodificaciones aún no iniciadas que se evitaron y cancelaciones offscreen.
Esos contadores solo contienen enteros y se incorporan al diagnóstico seguro;
no conservan identificadores, rutas, títulos ni contenido. Volver a materializar
la tarjeta crea una generación nueva y descarta cualquier resultado tardío.

El índice consulta la cola de prioridad antes de cada documento y antes de cada
página, no solo al comenzar la operación. Así un documento o página que se
vuelve visible durante el indexado se procesa en el siguiente turno sin
cancelar el handle nativo actual. Las consultas globales usan generaciones
independientes: una búsqueda solapada puede terminar, pero solo la generación
más reciente publica resultados.

Cerrar el lector sincroniza primero pestaña y snapshot, añade una entrada
deduplicada a cerrados recientes, libera el handle y vuelve a Biblioteca. La
acción visible `Cerrar sesión y conservar contexto` y el botón Atrás recorren
exactamente la misma ruta; no existe un cierre visual que omita persistencia.

## Continuidad, relaciones y journal 0.12

SQLite v10 mantiene `documents` como raíz de identidad y añade tres conjuntos
ortogonales: estado editorial de anotaciones, vistas/cola de espacios y grafos
de relaciones/revisiones. `automation_rules` solo acepta enums allowlist;
`operation_journal` conserva cantidades, estado, códigos seguros y datos de undo,
pero nunca rutas, títulos, consultas o contenido. Las FK eliminan relaciones
rotas al retirar un documento de la Biblioteca sin tocar el PDF.

`WorkspaceRepository.all()` carga espacios y pestañas en dos consultas, y
`AnnotationRepository` y `DocumentRelationRepository` precargan membresías por
lote. Ninguna vista materializa más de 80 documentos. El escaneo exclusivo de
reglas usa keyset pagination, un límite duro de 100.000 y cede el hilo cada 80.
La simulación y la aplicación comparten además un límite total de 30 segundos;
agotarlo termina como cancelación registrada y aplica rollback cuando procede.

La sesión atómica vive en `app_metadata`: se escribe primero como estado sucio,
se actualiza con debounce y solo se marca limpia tras cierre normal. Al arrancar,
una sesión sucia ofrece restaurar o empezar limpio. Los journals `running` se
marcan `interrupted`, aparecen en una pantalla no modal y pueden reanudarse como
una ejecución nueva o descartarse. Cancelar o fallar intenta rollback inverso y
si este no es seguro conserva un estado terminal `failed`, nunca una tarea zombi.

La comparación posee únicamente el segundo handle. Cada lado renderiza su página
visible con token de generación; cerrar cancela y libera el secundario, mientras
el primario sigue perteneciendo al lector. El modo Foco, página, zoom, scroll,
panel y pestaña activa se conservan en el espacio, no en la identidad del visor.

FTS5 añade una LRU de ocho consultas por un segundo. La clave contiene expresión,
filtros, límite y offset; cualquier mutación de metadatos, páginas, anotaciones,
marcadores u organización la invalida. El TTL limita la obsolescencia de flags
externos y evita mantener un segundo índice en memoria.

## Resiliencia documental y confianza local 0.13

`DocumentLifecycleSnapshot` es el contrato observable de apertura. Cada intento
recibe una generación y solo la vigente puede registrar metadatos o publicar un
handle; reemplazar o cerrar invalida la generación y libera cualquier resultado
nativo tardío. Los fallos conservan clase, acción posible e identidad candidata,
pero la ruta de reintento permanece solo en memoria. Una apertura correcta limpia
la cuarentena; contraseña requerida o incorrecta nunca la crea.

El diálogo de contraseña es dueño de su controlador y lo destruye al cerrarse.
El shell permite como máximo tres intentos, entrega la clave directamente al
motor y la descarta después. Contraseñas, rutas fallidas y texto del documento no
entran en SQLite, sesión atómica, journal, diagnóstico ni evidencia de
rendimiento. El cierre de `AppServices` espera primero al controlador del lector
y después dispone motor, base y servicios.

La sesión recuperable guarda identidad, huella, número de páginas, fase y
documento en cuarentena. Restaurar compara primero esos campos; una
incompatibilidad vuelve a Biblioteca con una acción explícita. Los journals v11
añaden fase, motivo de interrupción, capacidad real y hash de compatibilidad. La
pantalla Problemas deriva sus acciones de ese contrato y nunca presenta una
reanudación o rollback que no pueda ejecutar.

El backup v9 contiene manifiesto, esquema y conteos por tabla. Su ensayo es de
solo lectura y clasifica altas, reemplazos, eliminaciones y conflictos por
identidad estable. La estrategia es sustituir un snapshot coherente completo,
no resolver campos por última escritura: siempre crea un pre-backup, aplica todas
las tablas en una transacción y conserva el estado anterior si cualquier fila
falla. Los formatos anteriores compatibles reciben defaults seguros.

La consolidación de duplicados opera solo sobre registros locales. Crea un
pre-backup, mueve preferencias, sesiones, marcadores, anotaciones con procedencia,
colecciones, espacios, relaciones y revisiones, colapsa aristas equivalentes y
elimina ciclos propios. `duplicate_merge_history` enlaza la operación con la
copia usada por undo; ningún flujo escribe, mueve o elimina los PDF.

Las consultas interactivas siguen limitadas a 80 elementos y los recorridos de
mantenimiento usan keyset y un techo de 100.000. Incluso las lecturas globales
de repositorio poseen un límite duro. La búsqueda caliente evita reordenar su LRU
cuando la clave ya es la más reciente, sin cambiar resultados ni retener texto
adicional.

## Sistema visual y UX de escritorio 0.14

La iteración no introduce nuevas capas de dominio ni cambia SQLite v11. Los
tokens de presentación viven en `app_tokens.dart`; `AppDialogFrame` y
`AppOperationStatus` concentran, respectivamente, la adaptación de diálogos y
el anuncio de trabajo local prolongado. Los features aportan texto, fase,
progreso y callbacks, pero no duplican medidas ni semántica.

`AppDialogFrame` conserva el `DialogRoute` de Flutter: Escape, traversal de foco
y restauración al invocador siguen en la plataforma. Su contenido es
desplazable, las acciones usan `OverflowBar` y el ancho está acotado por tokens,
de modo que 640×480 y texto al 200 % no ocultan la salida. El campo de
contraseña continúa siendo propiedad efímera del diálogo y nunca cruza esta
frontera de presentación.

Las ocho secciones de Ajustes son una taxonomía de UI sobre los mismos campos y
servicios. Recuperación agrupa sesión, historial y backup; Almacenamiento agrupa
caché, tamaño y ubicación; Privacidad contiene el borrado local explícito. No
hay migración de datos ni cambio de ownership. El rail puede crecer hasta 320 px
con escalado de texto, sin cambiar los breakpoints 680/1100.

El lector no consume el nuevo marco para su canvas: identidad del adaptador,
ownership de handles, cachés, gestos y viewport permanecen intactos. La pasada
visual compara las mismas escenas 0.13/0.14 y la de rendimiento usa cinco
procesos avanzados, cinco de Biblioteca y cinco perfiles nativos por versión.
