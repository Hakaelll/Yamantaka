# Iteración integral 0.11: orientación, sesiones y escala local

Estado: implementación, comparación, gate y empaquetado local completados el
28 de agosto de 2026; firma y matriz física ampliada siguen separadas.

## Intención

Yamantaka 0.11 debe reducir el coste de volver a encontrar contexto dentro de
un PDF y dentro de una biblioteca grande. La mejora se apoya en tres ideas: un
navegador documental único, sesiones de lectura que puedan cerrarse sin perder
su estado y trabajo local incremental que se pueda pausar, reanudar y explicar.

Se conservan como límites no negociables el uso sin cuenta, nube, telemetría ni
red; PDFium/pdfrx detrás de su adaptador; SQLite/FTS5 local; un máximo de dos
documentos nativos calientes; la identidad estable del visor; y la ausencia de
modificaciones sobre los píxeles o bytes del PDF.

## Auditoría inicial obligatoria

La revisión partió de `0.10.0+11` con el árbol limpio. Se leyeron README,
DESIGN, ARCHITECTURE, TESTING, CHANGELOG, NEXT_TODOS, los ADR vigentes y esta
serie de iteraciones visuales. La base aprobó:

- formato de 217 archivos, sin cambios;
- `git diff --check`;
- análisis estático sin incidencias;
- 266 pruebas unitarias/widget;
- contratos nativos, corpus patológico y benchmark de Biblioteca;
- build Release de Windows y smoke de ocho segundos con cero conexiones;
- golden editorial 0.10.

El gate reproducible está en `build/verification-011-baseline/`. La matriz DPI
física se omitió explícitamente; no se presenta como aprobada.

## Diagnóstico previo a implementar

### Orientación documental

1. Miniaturas, índice, marcadores, anotaciones y búsqueda están repartidos en
   cinco secciones del panel. El historial y las posiciones A/B viven además en
   comandos separados. El usuario debe recordar dónde está cada destino antes
   de poder filtrarlo.
2. `Ctrl+J` solo abre o cierra el panel y no ofrece una entrada directa al
   conjunto de destinos. El panel conserva correctamente la instancia del
   visor, por lo que el navegador nuevo debe respetar ese mismo límite.
3. El historial ya conserva página, zoom y desplazamiento. Los saltos desde
   índice, marcador y anotación registran origen, pero no existe una superficie
   única que haga visible el regreso mediante `Alt+Left`.
4. En ancho expandido hay espacio suficiente para lista y detalle. En compacto,
   una segunda columna comprime contenido y debe convertirse en lista, detalle
   y retorno explícito.

### Sesiones y continuidad

1. Las sesiones actuales solo guardan páginas inicial/final y segundos activos.
   El estado rico —zoom, layout, tema, panel, sección y posición— vive en
   preferencias y pestañas, no en el cierre histórico.
2. El controlador conserva hasta diez pestañas cerradas únicamente en memoria y
   permite fijar sin límite específico. No hay una superficie local de
   “cerrados recientemente” que sobreviva a una sesión de la aplicación.
3. Continuar y reiniciar existen como decisiones derivadas del progreso, pero
   falta una acción explícita Terminado y una meta local opcional, discreta y
   sin notificaciones.

### Anotaciones ligadas a selección

1. El dominio y SQLite ya soportan texto seleccionado, contexto, color,
   comentario, rotación, huella y rectángulos normalizados; edición, undo/redo y
   exportación versionada también existen.
2. El adaptador pdfrx expone selección real y rangos con geometría por
   fragmento, pero Yamantaka todavía no conecta ese evento con el dominio. Por
   ello la interfaz solo ofrece notas ligadas a página.
3. La integración debe normalizar rectángulos con el tamaño de página real,
   conservar una alternativa honesta de nota de página si falta geometría y no
   añadir ninguna capa que modifique el bitmap del PDF.

### Biblioteca e índice

1. Biblioteca ya pagina a 80, usa slivers, conserva selección/scroll y no
   recorre el sistema de archivos en la primera carga. La agrupación por autor
   existe como filtro de consulta, pero no hay un modelo general para autor,
   colección, etiqueta y estado con expansión estable.
2. Las operaciones masivas existen, aunque su entrada está dispersa entre
   menús y selección. La iteración debe converger en una sola barra contextual.
3. El índice prioriza dinámicamente el documento/página visibles y cierra cada
   handle, pero la pausa es solo memoria. `search_index_state.indexed_pages`
   puede convertirse en cursor seguro para reanudar sin repetir páginas.
4. Los estados de progreso indican documentos y páginas, pero no distinguen
   con claridad recuperación de índice corrupto/incompatible ni exponen
   métricas separadas de consulta fría, caliente, filtrada y cancelada.

### Revisión visual de las ocho resoluciones

Se revisaron las 128 capturas 0.10 y las diez comparaciones 0.9/0.10 mediante
las hojas de contacto de `build/visual-review-010-final/`:

- el PDF mantiene el dominio visual en 640×480, 800×600, 1280×720, 1600×900,
  1920×1080, 1920×1200, 2560×1440 y 3440×1440;
- los márgenes grandes de monitores ultrapanorámicos son una protección del
  tamaño de lectura, no espacio que deba llenarse;
- el panel de anotaciones a texto 200 % concentra título, alta, filtros y lista
  en una columna estrecha; necesita divulgación progresiva y menos controles
  simultáneos;
- la búsqueda global funciona a texto 200 %, pero la preview compacta pierde
  jerarquía al crecer el título; el patrón lista → detalle es el adecuado para
  reutilizar en el navegador;
- Biblioteca compacta mantiene la acción primaria y el contexto, aunque el
  agrupado debe evitar cabeceras tipo tarjeta y conservar una sola barra masiva;
- Ajustes compacto muestra primero el índice de secciones, lo cual es correcto;
  el retorno y el alcance local/global deben seguir siendo explícitos;
- menús, paleta y diálogos conservan foco, contraste y objetivos, sin gradiente,
  cristal ni “card soup”.

El contrato 0.10 de barra 56/52 px, dock 48/38 px, objetivos de 40 px, foco de
2 px, radios 10–12 px y transiciones útiles de 90–160 ms se mantiene. La marca
violeta tampoco requiere cambios.

## Línea base de rendimiento 0.10 comparable

Las mediciones se ejecutaron sobre el mismo host, SDK y arnés que usará 0.11.
Los artefactos están en `build/performance/0.10.0-baseline/`.

- Biblioteca: cinco pasadas para 5.000, 25.000 y 100.000 registros; todas
  materializaron exactamente 80 filas, hicieron cero recorridos del sistema de
  archivos y terminaron con integridad `ok`. La pasada fría inicial de 5.000
  fue un outlier; la mediana de las cuatro siguientes quedó entre 18.361 y
  23.578 µs. El p95 de 100.000 fue ruidoso en la quinta pasada, por lo que la
  comparación final usará la mediana entre pasadas, no el mejor valor.
- Funciones avanzadas: cinco pasadas con 500 documentos, 100 anotaciones, diez
  colecciones y cinco pestañas; búsqueda mediana entre 1.023 y 1.665 µs,
  integridad `ok` y crecimiento RSS observado igual a cero.
- Profile nativo: cinco pasadas. Primera página de 5.000 páginas entre 182.065 y
  196.925 µs; apertura pesada p95 entre 18.271 y 20.552 µs. Al terminar cada
  pasada quedaron cero documentos, renders pendientes y bytes de caché. El p95
  de frame/render mostró ruido superior al 10 %, motivo por el que se
  conservaron las cinco muestras.

El perfil actual ya cubre primera Biblioteca/PDF, quince aperturas y cierres
pesados, interacción y scroll continuo con 5.000 páginas. 0.11 ampliará el
arnés con cambio de pestaña, apertura del navegador con 100 destinos, búsqueda
fría/caliente/filtrada/cancelada, cambio de prioridad visible durante indexado y
restauración de cinco pestañas.

## Decisiones de implementación 0.11

- `Ctrl+J` abrirá un navegador documental filtrable con índice, miniaturas,
  resultados internos, marcadores, anotaciones, historial y A/B. `Ctrl+Mayús+J`
  conservará el acceso directo al panel lateral.
- El navegador será una capa temporal fuera de la clave del visor. Enter
  navegará mediante `ReaderController.navigateTo`; Escape cerrará y devolverá
  foco; `Alt+Left` restaurará página, zoom y posición de origen.
- Las sesiones ricas y los documentos cerrados recientemente se persisten en
  las claves versionadas de `app_metadata`, ya incluidas en el backup. Esto
  evita una migración de esquema; solo se guarda estado local, nunca texto de
  consultas ni contenido adicional del PDF.
- Se limitarán los documentos fijados a tres. Continuar, Releer y Terminado
  serán acciones explícitas; las metas opcionales usarán páginas o minutos y un
  indicador tipográfico sin gamificación ni avisos.
- La selección pdfrx solo habilitará resaltado, subrayado o tachado cuando
  entregue texto y rectángulos normalizables. En cualquier otro caso se
  ofrecerá una nota de página y se explicará la limitación.
- El índice reanudará desde `indexed_pages`, reajustará prioridad durante la
  ejecución y documentará el caso en que no pueda compartir de forma segura el
  handle activo del visor.
- Agrupación, selección masiva y consultas guardadas seguirán siendo modelos
  locales y acotados. Ningún flujo materializará más de 80 documentos ni hará
  recorridos de archivos en el primer frame.

## Validación física reservada

Narrator, DPI real, maximizar/restaurar/fullscreen, dos monitores con escalas
distintas, puntero, rueda, touchpad, menús contextuales, español/inglés, alto
contraste del sistema y protecciones de Windows solo se marcarán cuando se
ejecuten físicamente. Las matrices widget y los perfiles automatizados no
sustituyen esa evidencia.

## Resultado final reproducible

El gate final se conserva en `build/verification-011-release/` y cubre higiene,
toolchain, dependencias offline, self-check, formato, análisis, las 277 pruebas,
corpus patológico, benchmark de Biblioteca, build Release, smoke sin red,
golden 0.10, golden del navegador 0.11 y matriz nativa a la escala real
disponible. Ninguna fase automatizada se omite.

La evidencia visual contiene 164 PNG y 17 contact sheets: Biblioteca 21,
Lector 43, Búsqueda 16, Ajustes 18, paleta/menús 28, diálogos 28 y comparaciones
10. Importación incluye nuevo, duplicado, inaccesible e inválido en los ocho
tamaños base y los estados de accesibilidad. `verify_brand_assets.ps1`
confirma que los ocho assets de marca son byte a byte iguales a HEAD/0.10.

Los cinco pases de rendimiento dan `pass-with-watch`: cero métricas con
regresión fiable y cero invariantes fallidos. Biblioteca (+17,08 %) y scroll
continuo (+18,98 %) quedan como avisos no fiables porque solo tres de cinco
muestras superan el umbral y los rangos se solapan. No se ocultan ni se usan
para bloquear el release local.

El host disponible solo aporta un monitor 1920×1200, DPI 96/100 %, es-ES y alto
contraste desactivado. La matriz nativa cubre cuatro casos reales en esa escala.
Narrator, en-US, alto contraste del sistema, escalas 125/150/175/200 % y dos
monitores no se marcan como aprobados. El paquete permanece sin firma; es un
artefacto local verificable, no una distribución confiable para terceros.

El ejecutable exacto del ZIP también superó un smoke físico de interacción:
inicio en es-ES, selección de Biblioteca con teclado, apertura con Enter,
navegación de página 1/2 a 2/2 y cierre normal. `Ctrl+J` no se pudo confirmar
mediante la inyección de teclado del controlador de Windows y queda respaldado
solo por la prueba de widget. La captura WGC conservó rectángulos de frames
anteriores, de modo que el estado físico se verificó por UIA y no se incorporó
esa captura a la revisión visual.
