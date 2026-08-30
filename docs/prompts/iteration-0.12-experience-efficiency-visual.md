# Prompt maestro — Yamantaka 0.12: flujo, velocidad y claridad visual

Trabaja directamente sobre el repositorio actual de Yamantaka y completa una
iteración integral `0.12.0+13` partiendo de `0.11.0+12`. No te limites a proponer
ideas: inspecciona el código, implementa, prueba, mide, documenta y genera un
release local verificable para Windows. Mantén intactos los datos del usuario y
no publiques, firmes, subas ni envíes nada a servicios externos.

## Objetivo de producto

Haz que Yamantaka se sienta más rápido, más fácil de entender y más refinado en
tres momentos: volver a una lectura, trabajar dentro de un PDF y gestionar una
biblioteca grande. Reduce decisiones repetitivas y superficies redundantes sin
convertir la aplicación en un dashboard recargado. La mejora debe seguir siendo
local-first, sobria, accesible y centrada en el documento.

El resultado debe mejorar de forma demostrable:

1. orientación y recuperación de contexto;
2. velocidad percibida y trabajo de fondo;
3. flujo de anotación y revisión;
4. organización de bibliotecas grandes;
5. coherencia visual y adaptación a Windows;
6. teclado, Narrator, contraste y texto ampliado;
7. resiliencia, diagnósticos y facilidad para probar una build.

## Límites no negociables

- Sin cuentas, nube, sincronización, telemetría remota, anuncios ni red durante
  el uso normal.
- Sin OCR, IA remota ni análisis que envíe texto, títulos, rutas o PDFs.
- No modificar los bytes ni los píxeles del PDF original.
- Mantener PDFium/pdfrx detrás de sus adaptadores y SQLite/FTS5 como fuente local.
- No añadir una dependencia sin justificar tamaño, licencia, superficie de
  ataque, mantenimiento y alternativa con la plataforma o el SDK actual.
- Mantener como máximo dos documentos nativos calientes salvo que una medición
  reproducible demuestre que otra política es más segura y barata.
- Ningún hover, panel, breakpoint, diálogo, chrome o animación puede entrar en
  la identidad cacheada del visor.
- No alterar, regenerar ni reinterpretar el símbolo de Yamantaka. Todos sus
  assets deben conservar geometría, alfa, color y hash.
- No desactivar Smart App Control, Defender, alto contraste ni otra protección.
- No presentar una build sin firma como distribución confiable. Puede ser un
  release local de prueba si manifiesto, hashes y estado de firma son explícitos.
- No afirmar que una prueba física pasó si el monitor, DPI, idioma, Narrator o
  estado del sistema no estuvo realmente disponible.

## Fase 0 — auditoría y línea base obligatorias

Antes de editar:

1. Lee `README.md`, `DESIGN.md`, `ARCHITECTURE.md`, `TESTING.md`,
   `CHANGELOG.md`, `NEXT_TODOS.md`, los ADR y la documentación de 0.11.
2. Revisa el diff y conserva cambios ajenos. No limpies ni restaures archivos
   que no pertenezcan a esta iteración.
3. Ejecuta el gate actual completo y registra commit, estado dirty, Flutter,
   Dart, Windows, CPU, RAM, GPU, monitor, resolución, DPI real y locale.
4. Conserva una línea base 0.11 de cinco pasadas para startup, primera
   Biblioteca, apertura del primer PDF, navegación, búsqueda, portadas, memoria
   y retorno a Biblioteca.
5. Genera o reutiliza fixtures sintéticos sin datos privados: 1/100/1.000/10.000
   páginas, vector pesado, raster extremo, tamaños mixtos, texto ausente,
   biblioteca 100/5.000/25.000/100.000 y 1.000 anotaciones.
6. Captura la referencia visual 0.11 antes de cambiar UI. No sobrescribas una
   base para hacer desaparecer una diferencia.

Documenta primero los problemas observados, su severidad, evidencia y criterio
de salida. No rediseñes una superficie estable sin un problema concreto.

## Fase 1 — inicio útil y continuidad inmediata

Rediseña la parte superior de Biblioteca como una zona de continuidad compacta,
no como un dashboard:

- muestra una sola acción primaria de retorno a lectura;
- permite alternar entre el último documento, hasta tres fijados y cerrados
  recientes sin duplicar tarjetas completas;
- presenta página, progreso, última actividad, meta opcional y estado
  ausente/modificado con texto, no solo color;
- añade una vista temporal `Actividad local` con aperturas, cierres, marcadores
  y anotaciones recientes, limitada y generada desde datos ya locales;
- permite borrar esa actividad o desactivar su registro desde privacidad;
- al restaurar, conserva página, zoom, layout, modo, tema, scroll, panel,
  selección del navegador y foco lógico;
- ofrece `Deshacer cierre` durante unos segundos sin mantener un handle nativo;
- explica de forma inequívoca cuándo un elemento está oculto por filtros,
  agrupación o una vista guardada.

La actividad no debe almacenar fragmentos de texto PDF, consultas ni rutas en
diagnósticos. Define retención, límite y backup de cada nuevo dato.

## Fase 2 — espacio de lectura más eficiente

Mejora el flujo dentro del lector sin añadir chrome permanente:

- amplía `Ctrl+J` con destinos recientes, filtros por tipo y agrupación por
  capítulo/página, manteniendo apertura rápida y retorno al origen;
- añade una `pila de retorno` visible solo bajo demanda para saltos desde
  búsqueda, enlaces, marcadores y anotaciones;
- permite fijar dos posiciones A/B con nombres opcionales y alternar con un
  único atajo, conservando zoom y ancla por posición;
- implementa una vista de comparación real solo si puede mantener dos visores
  medidos, aislados y con recursos acotados; de lo contrario conserva A/B y
  documenta por qué no se anuncia split;
- añade ayuda contextual de atajos desde la paleta, filtrable y adaptada a los
  overrides actuales;
- unifica mensajes de primera/última página, destino obsoleto, archivo cambiado
  y resultado no disponible;
- mantiene Documento, Libro y Continua centrados sin reservar espacio para
  controles ocultos;
- conserva la instancia del visor al abrir/cerrar navegador, panel, ayuda,
  paleta, menú o inspector y al cruzar 680/1100 px.

Prueba 100 saltos consecutivos, ida/vuelta A/B, enlaces internos, historial y
`Alt+←/Alt+→` sin deriva de zoom, página, scroll ni foco.

## Fase 3 — revisión y anotaciones de alto rendimiento

Convierte Anotaciones en un flujo de revisión completo pero contenido:

- conserva highlight, subrayado, tachado y nota de página existentes;
- permite añadir nota opcional, etiquetas locales y estado `pendiente/revisada`
  a una anotación sin tocar el PDF;
- añade filtro combinado por tipo, color, etiqueta, estado y rango de páginas;
- permite selección múltiple, cambiar estado/color/etiqueta y exportar solo la
  selección con resumen previo;
- añade `Siguiente pendiente` y `Anterior pendiente` operables por teclado;
- muestra texto seleccionado y contexto de forma accesible, truncada y
  expandible; nunca carga páginas completas para construir una lista;
- conserva undo/redo transaccional para edición y acciones masivas;
- detecta anotaciones obsoletas tras cambiar la huella y ofrece revisar,
  relocalizar por texto exacto localmente o mantener como nota de página;
- si la selección es multipágina o no tiene geometría fiable, explica el
  fallback antes de guardar y no inventa coordenadas.

Mide y prueba 1.000 anotaciones, cambios masivos, exportación Markdown/JSON y
navegación rápida sin materializar ni renderizar todas las páginas.

## Fase 4 — Biblioteca grande, consultas guardadas e importación clara

Haz que la Biblioteca escale sin perder legibilidad:

- unifica filtros, orden, agrupación y búsqueda en un modelo de consulta local,
  serializable y versionado;
- añade hasta diez `Vistas guardadas` locales con nombre, consulta y orden, sin
  duplicar documentos ni crear carpetas;
- incorpora filtros por colección, etiqueta, disponibilidad, fecha de apertura,
  fecha de adición y rango de progreso;
- muestra una banda eliminable con cada filtro activo y un restablecimiento
  global; teclado y Narrator deben anunciar resultado y recuento;
- conserva selección, foco y scroll al alternar vista, cuadrícula/lista,
  agrupación, entrar al lector y volver;
- mantiene una única barra de acciones masivas y cabeceras plegables;
- no materializa más de 80 filas ni realiza consultas N+1 o recorridos del
  sistema de archivos durante el primer frame;
- prioriza portadas visibles, cancela las que salen del viewport y registra
  renders iniciados, compartidos, cancelados y evitados sin IDs privados;
- mejora Importar carpeta con preview resumida, filtros por resultado,
  selección de nuevos documentos, pausa/reanudación y un informe final
  exportable sin rutas completas;
- evita saltos de layout cuando aparece una portada: reserva proporción y
  tamaño desde metadatos o placeholder estable.

Valida planes SQL, número de consultas y memoria con 5.000, 25.000 y 100.000
registros. Las vistas guardadas deben sobrevivir backup/restauración y una
migración desde 0.11.

## Fase 5 — scheduler, índice y velocidad percibida

Audita el trabajo de fondo con prioridad visible real:

- define presupuestos separados para render visible, prefetch, portada,
  indexado, importación, backup y mantenimiento;
- evita starvation: una ráfaga de trabajo visible no puede impedir para siempre
  una tarea local pausada, pero ninguna tarea de fondo debe degradar zoom o
  scroll activo;
- cambia prioridad del documento y página en ejecución sin reiniciar filas ya
  confirmadas;
- comparte trabajo idéntico mediante leases solo cuando la propiedad y cierre
  nativo sean inequívocos; no compartas el handle del viewer por conveniencia;
- pausa trabajo recreable al minimizar, durante interacción intensa o bajo
  presión de memoria y reanúdalo de forma segura;
- añade mantenimiento FTS incremental y explícito, nunca un VACUUM inesperado
  durante lectura;
- separa métricas fría/caliente/filtrada/cancelada y cache hit/miss;
- registra solo nombres constantes, contadores, duraciones y bytes agregados;
- añade un panel de diagnóstico seguro con colas, trabajo evitado, límites de
  memoria e invariantes de handles, sin rutas, títulos, consultas ni snippets.

Fija presupuestos iniciales usando la mediana 0.11 del mismo host. Una regresión
solo bloquea si supera 10 % y al menos cuatro de cinco pasadas cruzan el límite;
conserva también p50/p95/p99 y rangos para evitar conclusiones por ruido.

## Fase 6 — refinamiento visual y experiencia Windows

Refina, no reemplaces, el sistema visual actual:

- conserva Segoe UI Variable, paleta monocromática, foco independiente del
  acento y violeta de marca intacto;
- reduce bordes y divisores redundantes; usa espacio y alineación antes que
  tarjetas anidadas;
- normaliza alturas, radios, iconos y paddings de Biblioteca, navegador,
  anotaciones, importación, paleta y Ajustes;
- reserva cápsulas para estado, filtro o selección; no conviertas cada acción
  en una píldora;
- diferencia claramente reposo, hover, foco, pulsado, seleccionado,
  deshabilitado, ausente, modificado y obsoleto sin depender solo del color;
- usa movimiento de 90–160 ms solo para continuidad espacial; con movimiento
  reducido todo cambio debe ser inmediato y sin pérdida de contexto;
- evita animar tamaños del visor, portadas durante scroll o listas grandes;
- mejora el uso de espacio en 16:10 y ultrawide sin estirar líneas de texto ni
  separar excesivamente las acciones relacionadas;
- conserva targets de al menos 40 px y evita overflows con texto al 200 %;
- usa controles y semántica propios de Windows cuando aporten comportamiento
  nativo, sin imitar visualmente otra aplicación.

No aceptes cambios visuales basados solo en una captura bonita. Cada uno debe
tener problema, intención, comparación y prueba de estados.

## Fase 7 — accesibilidad, teclado e idiomas

Completa una auditoría de interacción:

- orden de foco determinista en Biblioteca, lector, `Ctrl+J`, importación,
  anotaciones, paleta y Ajustes;
- Escape cierra primero la capa más próxima; Enter activa; Espacio selecciona;
  flechas recorren; Shift+F10 abre contexto; Tab no queda atrapado;
- restaura el foco al elemento de origen después de cerrar cada overlay;
- todas las acciones de icono tienen nombre, estado, valor y hint útiles;
- progreso, filtro, selección, fallback, archivo ausente y destino obsoleto se
  anuncian con texto breve y no repetitivo;
- alto contraste del sistema elimina transparencias o sombras que oculten
  límites y conserva foco visible;
- prueba texto 100/130/150/200 %, es-ES y en-US, incluidos plural, fechas,
  atajos y cadenas largas;
- añade tests semánticos y de teclado; realiza Narrator físico solo si está
  realmente disponible y registra exactamente lo comprobado.

## Fase 8 — resiliencia y privacidad

- Versiona toda persistencia nueva y prueba migración 0.11→0.12, rollback ante
  fallo y backup/restauración.
- Añade un pequeño journal de recuperación de sesión que se confirme de forma
  atómica y se descarte al cerrar correctamente.
- Nunca reabras automáticamente un PDF que causó el último fallo; ofrece modo
  seguro y elección del usuario.
- Cancela y libera handles, timers, streams y jobs al cerrar pestaña, sesión,
  ventana o aplicación.
- Distingue ausente, acceso denegado, modificado, corrupto, cifrado y fallo
  interno con acciones apropiadas.
- Mantén logs y diagnósticos redactados y compruébalos con datos señuelo que
  fallen si aparece una ruta, título, consulta, texto o identificador.
- Repite smoke sin red sobre el ejecutable exacto empaquetado.

## Fase 9 — matriz visual y rendimiento

Genera evidencia 0.12 separada de 0.11 para estas superficies:

- Biblioteca vacía, con muchos documentos, filtrada, agrupada, selección y
  vistas guardadas;
- continuidad/actividad local;
- lector Documento, Libro y Continua en reposo/activo;
- panel y navegador `Ctrl+J`;
- anotaciones en lista, revisión y selección múltiple;
- importación en preview, progreso, cancelado y resumen;
- búsqueda, paleta, menús, diálogos y Ajustes.

Usa 640×480, 800×600, 1280×720, 1600×900, 1920×1080, 1920×1200,
2560×1440 y 3440×1440; claro/oscuro donde aplique, alto contraste, movimiento
reducido y texto 130/150/200 %. Genera contact sheets y revisa visualmente cada
captura. Conserva diez o más comparaciones 0.11/0.12 con hashes de fuentes.

Ejecuta cinco pasadas comparables de:

- arranque frío y caliente;
- primera Biblioteca con 100.000 registros;
- scroll rápido y cambios de filtros/vistas;
- portadas visibles, canceladas y evitadas;
- primer PDF de 100 y 10.000 páginas;
- panel con 1.000 anotaciones;
- 100 saltos de navegador y 100 alternancias A/B;
- zoom, scroll continuo, cambio de layout y resize;
- diez pestañas con máximo de dos handles calientes;
- retorno a Biblioteca y estabilización de RSS.

No ocultes resultados ruidosos. Diferencia `pass`, `pass-with-watch` y `fail`
con la regla de reproducibilidad definida en la fase 5.

## Fase 10 — gates, documentación y release

Antes de entregar:

1. Actualiza versión a `0.12.0+13` en todos los lugares autoritativos.
2. Ejecuta formato, `git diff --check`, análisis, suite completa, corpus,
   benchmarks, goldens, matrices, build Release y smoke sin red.
3. Ejecuta la matriz nativa en cada DPI y monitor realmente disponibles.
4. Verifica que ningún gate automatizable quede omitido. Si un requisito físico
   no está disponible, déjalo pendiente con host y motivo exactos.
5. Actualiza `README`, `DESIGN`, `ARCHITECTURE`, `TESTING`, `CHANGELOG`,
   `NEXT_TODOS` y la documentación de iteración.
6. Documenta decisiones: propiedad de handles, migración, scheduler, privacidad,
   accesibilidad, presupuesto y cambios visuales.
7. Genera `build/release/0.12.0+13/` con bundle completo, ZIP, manifiesto por
   archivo, inventario de binarios/firma, `SHA256SUMS.txt` y evidencia sin red.
8. Verifica ZIP contra manifiesto por ruta, tamaño y SHA-256; rechaza rutas
   inseguras, PDFs incluidos o archivos ausentes.
9. Abre el ejecutable exacto del paquete, no una build anterior, y comprueba el
   flujo principal con teclado y cierre normal.
10. No hagas commit, tag, push, firma ni publicación salvo petición explícita.

## Entrega requerida

Entrega un informe final breve pero verificable con:

- cambios funcionales y visuales;
- archivos y documentación principales;
- número exacto de pruebas y resultado de cada gate;
- resumen de las cinco pasadas, regresiones y avisos;
- recuento de capturas, comparaciones y revisión visual;
- pruebas físicas realmente ejecutadas y matriz pendiente;
- ruta, tamaño y SHA-256 del ZIP;
- estado Authenticode de cada binario nativo;
- limitaciones conocidas y siguientes pasos;
- un mensaje de commit detallado en formato Conventional Commits.

No declares terminado hasta que código, pruebas, evidencia, documentación y
release describan exactamente el mismo producto.
