# Iteración visual 0.8: archivo privado y lectura de precisión

Estado: implementada y verificada localmente. El diagnóstico y la dirección se
escribieron antes de modificar widgets de la iteración 0.8; la sección final
registra el resultado observado.

## Intención

Yamantaka 0.8 debe percibirse como un archivo privado de documentos y una mesa
de lectura de precisión para Windows. La aplicación sigue siendo local-first,
offline, no destructiva y basada en PDFium. La revisión es profunda en jerarquía,
densidad, estados, relaciones entre superficies y comportamiento de escritorio;
no cambia el contrato de datos ni el ciclo de vida del visor.

La identidad visual se apoya en dos ambientes relacionados:

- la biblioteca es un índice oscuro, espacial y silencioso, con información
  operativa legible y comparación rápida entre documentos;
- el lector es una mesa editorial neutra en la que la página es el objeto
  principal y el cromo solo aparece para orientar o actuar.

No se persigue novedad decorativa. Cada borde, superficie elevada, radio y color
debe explicar agrupación, interacción, selección, riesgo o estado del documento.

## Diagnóstico de la baseline 0.7

### Sistema global y shell

Funciona la separación semántica de temas, el contraste, el mínimo tipográfico de
12 px, los breakpoints no solapados y el tratamiento consistente de foco, hover y
pressed. La baseline también contiene buenas bases para teclado y reducción de
movimiento.

Todavía resulta genérico el uso extensivo de cápsulas, cuadros flotantes y
`AlertDialog`. El sistema distingue interacción, pero no expresa con suficiente
precisión estados de documento, procedencia local, acciones destructivas ni
operaciones cuantificables. Varias superficies elevadas compiten a la vez y crean
ruido. En compacto, parte del contenido se apila correctamente, aunque algunas
barras conservan demasiados iconos sin contexto. En ultrawide, aparecen vacíos
grandes sin aumentar la capacidad de comparación. El escalado de texto puede
volver frágiles las filas horizontales con controles de ancho fijo.

Dirección 0.8:

- reducir radios de contenedores de trabajo y reservar la cápsula para estados o
  controles segmentados;
- introducir componentes compartidos para filas de menú, encabezados de diálogo,
  estados y acciones peligrosas;
- mantener `focus > pressed > selected > hover > rest` como orden visual;
- separar semánticamente ausencia, modificación externa, revisión, éxito,
  advertencia y error;
- usar elevación solo para capas temporales u objetos que realmente se superponen.

### Biblioteca

Funciona su identidad oscura, la selección violeta, el lienzo de portadas, el
inspector contextual y la recuperación explícita de documentos ausentes. La
información de progreso y revisión existe y la navegación con teclado conserva
selección.

La pantalla, sin embargo, ofrece casi exclusivamente una constelación de portadas.
Eso es expresivo con pocos documentos pero débil para comparar título, autor,
estado, progreso y fecha en colecciones grandes. La barra de comandos depende
demasiado de iconos y no deja visible el contexto exacto de filtros. El vacío
inicial es genérico; el overlay inferior puede parecer un HUD web; y las portadas
normales pierden presencia por opacidad y desaturación. En compacto, la barra se
comprime pero mantiene carga cognitiva. En ultrawide, crece el espacio entre
objetos más que la utilidad.

Dirección 0.8:

- encabezado = identidad de colección + recuento/resultados + acción primaria;
- barra de comandos = búsqueda, modo de vista y filtros, en ese orden;
- vista de cuadrícula para reconocimiento visual y vista de lista densa para
  comparación; ambas comparten selección, teclado e inspector;
- banda explícita de filtros activos con borrado reversible;
- documentos normales a opacidad y saturación completas; ausente, modificado y
  pendiente de revisión usan señales semánticas, nunca solo opacidad;
- inspector visualmente unido al área de resultados, con una sola acción primaria;
- vacío inicial orientado al archivo local y cero resultados orientado a corregir
  consulta/filtros.

### Lector y cromo

Funciona la unificación de la barra superior y del dock inferior, el centro
geométrico de navegación, el panel redimensionable y el hecho crítico de que
mostrar u ocultar cromo no altera `_viewerCacheKey`. Los modos página única,
doble, continua, fullscreen e inmersivo están bien separados.

La barra y el dock siguen pareciendo cápsulas flotantes de una interfaz web. El
dock marca siempre el tema como seleccionado aunque el control abre un menú. La
etiqueta fija “LECTURA” aporta menos que el modo de página actual. La cabecera
deja aire sin jerarquía útil y el panel lateral se percibe plano y algo vacío. En
compacto, el número de acciones exige más priorización; en ultrawide, el lector no
debe ensanchar la página ni desplazar la navegación.

Dirección 0.8:

- barra superior como franja de instrumento sobria, con grupos reconocibles y
  radios moderados;
- dock inferior compacto, centrado por geometría y con el modo de página como
  contexto; el tema deja de fingir un estado seleccionado;
- panel lateral como espacio de trabajo conectado: rail estable, encabezado,
  área de contenido y resize handle inequívoco;
- chrome resting con menor contraste y chrome active con contraste completo;
- la página, el margen, la sombra y el canvas deben leerse como una sola mesa;
- no introducir ninguna dependencia de barra, panel, tamaño o breakpoint en la
  identidad del visor PDF.

### Búsqueda global y paleta de comandos

Funcionan el teclado, el historial reciente, las categorías y el resumen de los
resultados. La paleta ya ofrece previsualización en ancho suficiente.

La selección gris se acerca visualmente a un estado deshabilitado y la búsqueda
global es una lista única incluso cuando sobra ancho. Los estados de indexación y
carga dependen de indicadores genéricos y la vista previa apenas establece
relación con el resultado. Faltan distinciones más fuertes entre comando,
documento, página, anotación, colección y etiqueta.

Dirección 0.8:

- búsqueda global en una columna en compacto y split-view en expanded;
- selección con borde/acento, nunca con apariencia disabled;
- lista como fuente de verdad y preview como detalle del elemento seleccionado;
- carga cuantificada cuando el índice conoce progreso, carga indeterminada con
  texto cuando no lo conoce, y error con reintento cuando la operación es
  recuperable;
- paleta con filas de menú compartidas, icono, etiqueta, atajo y categoría.

### Ajustes

Funciona la arquitectura de tres columnas en expanded, la navegación por
categorías y la existencia de una previsualización viva. Las filas mantienen
targets adecuados y se apilan en ancho reducido.

El resultado es demasiado plano y claro: navegación, contenido y preview se
parecen en jerarquía, el preview es pequeño y pasivo, y las descripciones no
siempre explican dónde se aplica un ajuste, si es local o si puede revertirse. Los
anchos fijos son vulnerables al escalado de texto. Las acciones peligrosas solo se
separan con una línea. Un error de persistencia llega a interpolar texto técnico.

Dirección 0.8:

- navegación, contenido y preview con tres niveles de superficie, no tres cards;
- secciones internas y filas más densas, pero con separación semántica clara;
- preview mostrando reposo, hover/foco/selección y panel, no una miniatura pasiva;
- descripciones que indiquen alcance y reversibilidad cuando sea relevante;
- acciones destructivas en una zona de peligro explícita y separada;
- errores tipados para el usuario, sin excepciones o rutas internas.

### Metadatos

Funciona el modelo no destructivo: fuente detectada, override local y nota local
son datos separados, y guardar nunca reescribe el PDF.

El editor actual muestra campos sin hacer visible esa procedencia. No existe una
lectura clara de “detectado frente a local”, de cambios sin guardar o de cómo
restaurar un campo a la fuente. Temas y nota compiten al mismo nivel.

Dirección 0.8:

- identidad (título/autor), organización (temas) y nota en grupos sucesivos;
- valor detectado visible debajo de cada override;
- indicador local y estado de cambios sin guardar;
- restauración por campo y cancelación total sin tocar el PDF;
- una sola acción primaria: guardar metadatos locales.

### Anotaciones, paneles y diálogos secundarios

Funcionan los filtros por tipo/color, undo/redo, búsqueda, exportación local y los
estados vacíos específicos de outline, marcadores y anotaciones.

Los menús de anotación son inconsistentes con los del lector y la biblioteca; la
acción de borrar no se diferencia suficientemente. Varios diálogos secundarios
son `AlertDialog` genéricos con icono, título y botones poco jerarquizados. Algunos
loadings usan spinner aun cuando existe progreso o una fase nombrable.

Dirección 0.8:

- una fila de menú común con icono, etiqueta, atajo, checked y destructivo;
- diálogo de escritorio con header, cuerpo scrolleable y footer estable;
- una única acción primaria por diálogo; peligro aislado y explicado;
- estados específicos con recuperación: localizar, reintentar, cancelar o
  deshacer según la operación;
- progreso cuantificado para importación/análisis y progreso indeterminado con
  fase textual para diagnóstico o búsqueda.

## Reglas de densidad, jerarquía y comportamiento

La densidad se obtiene reduciendo espacios repetidos y mejorando alineación, no
reduciendo targets. Los controles de teclado mantienen al menos 40 px y los de
toque 48 px. El texto esencial permanece en 12 px o más. Las filas de lista usan
figuras tabulares para progreso, páginas, tamaño y fechas.

El orden visual por superficie es:

1. identidad y ubicación actual;
2. contenido o resultado principal;
3. contexto y estado;
4. acciones reversibles;
5. acciones destructivas.

La acción primaria aparece una vez. Un documento ausente prioriza “Localizar”; un
documento disponible prioriza “Continuar leyendo”. La eliminación de la biblioteca
explica siempre que el PDF permanece en disco.

## Contrato responsive

- Compact (`< 680`): una columna, toolbars priorizadas, overlays en vez de paneles
  docked y listas sin columnas auxiliares.
- Medium (`680–1099`): dos regiones cuando mejora una tarea; inspector y panel
  siguen superpuestos para no reducir el visor.
- Expanded (`>= 1100`): lista + preview o navegación + contenido + preview; el
  espacio adicional incrementa comparación y contexto, no el ancho de lectura.
- Ultrawide: anchos máximos y columnas útiles; el contenido editorial no se estira.
- Escalado de texto 150 %: controles pueden saltar de fila y las etiquetas
  conservan el significado completo mediante tooltip/semantics.

## Estados obligatorios

Toda superficie principal debe poder demostrar, cuando aplique: empty, loading
cuantificado o indeterminado, éxito, zero results, error tipado, disabled, hover,
pressed, focus, selected, activo/inactivo, documento ausente, modificación externa,
revisión pendiente, cambios sin guardar y pérdida/restauración de foco.

La opacidad no es el único portador de significado. Icono, texto, borde, patrón o
posición acompañan siempre al color. `Semantics`, tooltip y accesos de teclado
explican la misma acción que la presentación visual.

## Límite arquitectónico

Esta iteración puede cambiar tokens, widgets de cromo, layouts, menús, diálogos,
modelos de vista efímeros y pruebas. No puede cambiar el modelo de persistencia,
la naturaleza local de búsqueda/metadatos/anotaciones, los binarios PDFium ni el
contrato no destructivo.

En el lector, `_viewerCacheKey` seguirá dependiendo solo del documento y de la
configuración que realmente altera el renderizado PDF. Panel, hover, foco, chrome,
breakpoint, tamaño de ventana y acciones de menú no pueden formar parte de esa
clave. Las pruebas deben verificar identidad estable al alternar paneles, cambiar
ancho y activar/ocultar controles.

## Evidencia y aceptación

La iteración se acepta con:

- pruebas de tokens y jerarquía de estados;
- pruebas widget en compact, medium, expanded, ultrawide y texto al 150 %;
- capturas de biblioteca, lector, ajustes, paleta, búsqueda, menús y diálogos;
- comparaciones 0.7 frente a 0.8 para reposo, activo, panel, selección y estados;
- inspección visual real de los PNG generados;
- `dart format`, `flutter analyze`, suite completa y build Windows release;
- ZIP local, manifiesto de archivos, hashes SHA-256, smoke de red y estado de
  firma declarados sin publicar, instalar globalmente ni firmar sin certificado.

Las validaciones Narrator, multimonitor y DPI real se documentan como manuales si
el entorno no permite observarlas físicamente; nunca se declaran superadas por
simulación.

## Resultado implementado

- Biblioteca dispone de grid y lista, títulos visibles, filtros activos
  eliminables y filas compactas específicas para texto 150 %.
- El vacío inicial se redujo tras revisión visual a logo, nombre, frase breve y
  una acción. El logo final no es la variante generativa: se recoloreó la fuente
  original píxel a píxel y se verificaron dimensiones, alfa y luminancia.
- Lector, panel, Ajustes, paleta, búsqueda global, metadatos, menús y diálogos
  adoptan la jerarquía descrita sin migraciones ni dependencias nuevas.
- Pasan 252 pruebas, análisis y la golden 0.8. La evidencia generada contiene 54
  capturas y diez comparaciones 0.7/0.8 inspeccionadas visualmente.
- Narrator, varios monitores y escalas DPI físicas continúan pendientes de una
  sesión manual; no se infieren desde capturas de widgets.
