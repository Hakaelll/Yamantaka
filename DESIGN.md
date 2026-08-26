# Sistema de diseño de Yamantaka

## 1. Visión

Yamantaka aplica minimalismo editorial contemporáneo a una herramienta de
escritorio. No debe llamar la atención sobre sí misma: debe facilitar que la
persona entre rápidamente en el documento y permanezca en él.

La interfaz se construye mediante proporción, alineación, contraste y ritmo.
Los elementos aparecen cuando son necesarios y se retiran cuando dejan de
aportar valor. El minimalismo no consiste en eliminar información esencial,
sino en presentarla de forma progresiva, contextual y silenciosa.

El resultado combina:

- la calma de un lector de libros;
- la precisión de una herramienta profesional;
- la eficiencia de una utilidad nativa;
- la claridad de una publicación editorial;
- una interfaz que desaparece durante la lectura.

## 2. Principios de diseño

### El documento primero

El PDF recibe la mayor superficie y el mayor contraste. Ningún encabezado,
panel o estado decorativo debe competir con la página.

### Divulgación progresiva

Las funciones avanzadas viven en paneles, menús o la paleta de comandos. Solo
las acciones frecuentes permanecen visibles.

### Densidad calmada

La aplicación puede mostrar metadatos útiles sin convertir cada elemento en
una tarjeta. Separadores, ritmo vertical y alineación sustituyen a cajas
innecesarias.

### Precisión táctil y visual

Un icono puede medir 18 px y conservar un área interactiva de 40 px en
escritorio o 48 px en táctil. Discreción visual no significa objetivos
pequeños.

### Continuidad

Abrir controles, cambiar de página o mostrar un panel no desplaza
innecesariamente el documento. Las dimensiones reservadas y los placeholders
evitan saltos.

### Familiaridad sin imitación

Los patrones deben resultar comprensibles para quien ya conoce un lector PDF,
sin copiar Acrobat, un navegador ni una plantilla Material.

### Rendimiento perceptible

Cada acción recibe feedback inmediato. Si una operación continúa, el estado
explica qué ocurre y permite cancelarla cuando corresponde.

## 3. Personalidad visual

Yamantaka es sobria, cálida, precisa y silenciosa. La biblioteca recuerda a un
índice editorial; el modo documento, a una herramienta nativa; el modo libro,
a una mesa despejada.

Debe sentirse moderna sin:

- modas de cristal o desenfoque;
- gradientes decorativos;
- sombras profundas;
- logotipos dominantes;
- radios exagerados;
- animaciones elásticas;
- saturación alta.

## 4. Jerarquía de información

Orden de atención:

1. Página o documento que se va a abrir.
2. Acción primaria contextual: continuar, abrir, buscar o recuperar.
3. Estado de lectura: página, progreso, selección.
4. Navegación secundaria y herramientas.
5. Metadatos y explicaciones.

La jerarquía se expresa primero con tamaño, peso, alineación y espacio. El
acento se reserva para foco, selección, progreso y acciones primarias.

## 5. Sistema de tokens

Los tokens viven en `lib/app/theme/app_tokens.dart` y los colores semánticos en
una `ThemeExtension`. Los widgets consumen nombres semánticos, no valores
hexadecimales ni números arbitrarios.

### Colores semánticos

```text
appBackground
surfacePrimary
surfaceSecondary
surfaceElevated
surfaceSunken
readerCanvas
textPrimary
textSecondary
textMuted
borderSubtle
borderStrong
accentPrimary
accentHover
accentPressed
accentSoft
selectionBackground
focusRing
danger
warning
success
readerPageShadow
```

### Espaciado

```text
space2  = 2
space4  = 4
space6  = 6
space8  = 8
space12 = 12
space16 = 16
space20 = 20
space24 = 24
space28 = 28
space32 = 32
space40 = 40
space48 = 48
space64 = 64
```

### Tamaños

```text
topBarHeight            = 64
libraryHeaderHeight      = 84
bottomControlHeight      = 48
sidePanelWidth           = 344
compactSidePanelWidth    = 360
desktopControlExtent     = 40
touchControlExtent       = 48
dialogMaxWidth           = 600
commandPaletteMaxHeight  = 520
commandPaletteExpandedWidth = 780
settingsNavigationWidth  = 236
settingsPreviewWidth     = 320
contentMaxWidth          = 1000
readerMinimumPageWidth   = 340
readerMaximumPageWidth   = 920
readerEdgeNavigation     = 64
readerSpreadGutter       = 2
```

### Movimiento

```text
motionInstant          = 0 ms
motionFeedback         = 80 ms
motionFast             = 120 ms
motionStandard         = 160 ms
motionPanel            = 200 ms
motionDialog           = 200 ms
motionInspector        = 160 ms
motionControlsAutoHide = 3 s
```

Con movimiento reducido, las transiciones espaciales usan `motionInstant`.

## 6. Color

### Tema claro

| Token | Valor |
| --- | --- |
| `appBackground` | `#F1F0EC` |
| `surfacePrimary` | `#F8F7F4` |
| `surfaceSecondary` | `#F4F2ED` |
| `surfaceElevated` | `#FDFCFA` |
| `surfaceSunken` | `#EAE8E2` |
| `textPrimary` | `#20211F` |
| `textSecondary` | `#62635E` |
| `textMuted` | `#85857E` |
| `borderSubtle` | `#DAD8D1` |
| `borderStrong` | `#C3C0B7` |
| `accentPrimary` | `#655D4C` |
| `accentHover` | `#574F40` |
| `accentPressed` | `#484236` |
| `accentSoft` | `#E7E2D7` |
| `selectionBackground` | `#E7E2D7` |
| `danger` | `#A23F38` |
| `warning` | `#8A6428` |
| `success` | `#4F7058` |
| `focusRing` | `#756B55` |
| `readerPageShadow` | `#24000000` (ARGB) |

### Tema oscuro

| Token | Valor |
| --- | --- |
| `appBackground` | `#171816` |
| `surfacePrimary` | `#1E1F1D` |
| `surfaceSecondary` | `#222320` |
| `surfaceElevated` | `#272825` |
| `surfaceSunken` | `#121311` |
| `textPrimary` | `#ECECE7` |
| `textSecondary` | `#B5B5AE` |
| `textMuted` | `#85867F` |
| `borderSubtle` | `#343631` |
| `borderStrong` | `#4A4C46` |
| `accentPrimary` | `#B5AA8E` |
| `accentHover` | `#C4B99C` |
| `accentPressed` | `#D2C6A8` |
| `accentSoft` | `#343127` |
| `selectionBackground` | `#343127` |
| `danger` | `#DF8077` |
| `warning` | `#D4AA69` |
| `success` | `#8BB293` |
| `focusRing` | `#C6B998` |
| `readerPageShadow` | `#52000000` (ARGB) |

El acento nunca ocupa grandes superficies. El negro puro se reserva para el
entorno de lectura Negro.

### Biblioteca espacial

Inicio y Biblioteca usan una paleta oscura estable e independiente del tema
editorial: fondo `#080808`, superficies negras y grises, texto `#F5F5F5` y
controles monocromáticos. El violeta `#B58AFF` se reserva para borde, progreso
y halo del archivo seleccionado; el foco permanece blanco e independiente del
color. El concepto evoca profundidad sin texturas ni animación continua.

### Entornos de lectura

| Tema | Canvas |
| --- | --- |
| Claro | `#F3F3F1` |
| Marfil | `#EAE5D8` |
| Sepia | `#D8CAB2` |
| Gris | `#B9BAB6` |
| Oscuro | `#222320` |
| Negro | `#000000` |

El PDF no se filtra ni se modifica. La sombra de página es muy sutil en fondos
claros, mínima en gris, inexistente en oscuro/negro y desactivada en tinta
electrónica.

## 7. Tipografía

Windows usa `Segoe UI Variable`, con `Segoe UI` y la fuente del sistema como
alternativas. No se empaquetan fuentes.

| Uso | Tamaño | Peso | Altura |
| --- | ---: | ---: | ---: |
| Metadato secundario | 12 | 400 | 1,35 |
| Etiqueta auxiliar/atajo | 12 | 500 | 1,35 |
| Cifra y progreso | 12 | 500, tabular | 1,30 |
| Control/interfaz | 14 | 500 | 1,35 |
| Encabezado contextual | 15 | 600 | 1,30 |
| Título de documento | 17 | 600 | 1,30 |
| Título de sección | 19 | 600 | 1,25 |
| Título de pantalla | 24 | 600 | 1,20 |
| Mensaje principal | 28 | 600 | 1,15 |

No se usa información esencial inferior a 12 px. Páginas, porcentajes y atajos emplean
números tabulares. Los títulos largos usan elipsis y ofrecen el nombre completo
en tooltip o semántica.

## 8. Espaciado y densidad

La cuadrícula base es 4 px y admite 2/6 px para precisión. Escritorio usa
objetivos de 40 × 40; táctil, 48 × 48.

- Compacto: padding horizontal 20 px.
- Medio: padding horizontal 28 px.
- Expandido: padding horizontal 32–40 px.
- Filas editoriales: 64–76 px según contenido.
- Separación entre secciones: 32–40 px.
- Separación entre página y canvas: proporcional, nunca menor de 12 px.

La interfaz es compacta, no apretada. No se usan márgenes propios de una
landing page ni grandes huecos sin una función de ritmo.

Los perfiles de escritorio de referencia son 1920x1080, 1920x1200 y 2560x1440;
el panel lateral se resta del viewport antes de calcular la geometria del PDF.
El PDF conserva escala legible y el espacio horizontal sobrante se mantiene
equilibrado, sin estirar la pagina para llenar un monitor ultrawide.

La preferencia de densidad tiene dos variantes tipadas:

- cómoda: filas recientes con 12 px verticales, ajustes con 16 px y secciones
  separadas 40 px;
- compacta: filas recientes con 8 px verticales, ajustes con 10 px y secciones
  separadas 28 px.

Ambas conservan objetivos esenciales de al menos 40 px en escritorio. La
densidad modifica chrome y ritmo, nunca escala tipografía ni zoom del PDF.

## 9. Forma, bordes y elevación

Radios:

- 2 px: selección o indicador;
- 3 px: página PDF o portada;
- 10 px: control discreto;
- 12 px: campo;
- 16 px: superficie flotante;
- 18 px: panel;
- 20 px: diálogo;
- cápsula: selector segmentado, dock y grupo de herramientas.

Los bordes son de 1 px y bajo contraste; selección y foco aumentan contraste.
No hay bordes dobles. Solo menús, paleta y diálogos emplean elevación, en dos
niveles como máximo. Las sombras de página son contenidas y no separan todos
los elementos.

## 10. Iconografía

Se usa exclusivamente la familia Material Symbols incluida por Flutter, con
preferencia por variantes `outlined`/`rounded` de trazo. Tamaño visual de
16–20 px dentro de áreas mayores.

Todo icono sin texto incluye:

- tooltip;
- etiqueta semántica;
- estado de foco;
- nombre accesible.

Un estado activo añade fondo suave y, cuando procede, un indicador de forma;
no depende solo del color.

## 11. Movimiento

- Feedback de hover/presión: 80 ms.
- Controles contextuales: 120 ms.
- Cambio estándar: 160 ms.
- Panel o diálogo: 180–220 ms.
- Ninguna animación decorativa supera 250 ms.

Se animan opacidad, desplazamientos pequeños y paneles. No se anima
decorativamente la página, el zoom, el progreso, sombras grandes ni
desenfoques. Las curvas no rebotan ni hacen overshoot.

## 12. Estructura de ventana

```text
Chrome nativo
Barra contextual compacta
Área principal
Panel lateral opcional
Progreso discreto
Capas temporales
```

No todas las superficies aparecen a la vez. La barra mide 64 px y puede
ocultarse/revelarse en cualquier modo mediante el mismo comando persistido.

## 13. Biblioteca

Jerarquía:

```text
Barra discreta: Biblioteca + contador | Más | Abrir PDF | Ajustes
Comando local flotante: Buscar | Selección | Filtrar/ordenar
Espacio de artefactos virtualizado
Información tipográfica de selección/hover/foco
```

La portada es el objeto. No existe una tarjeta con título permanente: la página
respeta su proporción, usa radio mínimo, sombra difusa y un jitter estable de
posición/giro. Selección eleva 6 px, escala 3,5 % y proyecta un halo violeta
acotado; hover eleva 2 px y escala 1,5 %. Los no seleccionados reducen
saturación/opacidad sin desaparecer. Progreso es una línea de 4 px con pista
gris tenue; el relleno es
blanco/gris salvo en la selección, donde pasa a violeta. Terminado usa un
anillo y favorito un marcador.

Uno a seis documentos forman una constelación centrada con espacio negativo.
Desde siete se usa una cuadrícula invisible con variaciones controladas y
scroll vertical. `SliverGrid.builder` recicla objetos en bibliotecas grandes.
La semilla usa el ID estable: cargar portada, filtrar o reiniciar no mueve la
composición.

Un clic selecciona, doble clic/Enter abre y el menú contextual reúne abrir,
favorito, portada, información, revelar y retirar. Flechas navegan por la
geometría; Inicio/Fin saltan a extremos, Suprimir confirma retirada y Escape
cierra overlay o selección. El foco tiene contorno propio y no depende de
sombra/color.

`Ctrl+F` abre un campo flotante con foco y contador; Escape lo cierra y restaura
la colección. Estado, favorito y orden viven en un popover compacto; autores y
temas forman facetas combinables con recuentos completos. Agrupar por autor crea
secciones dinámicas y coloca Autor desconocido al final. Colecciones,
workspaces, importación, recuperación y búsqueda global están bajo “Más”.

Cada portada solicita una decodificación ajustada a caja física/DPR, usa la
miniatura pequeña a distancia y la grande al seleccionar. El placeholder es una
placa neutra con inicial o estado de error. Título, autor, progreso y fecha solo
aparecen para selección, hover o foco en un bloque tipográfico ligero.

En escritorio se validan dos familias de proporcion: 16:9 y 16:10. En 1920x1080
la cuadricula de muchos documentos llega a ocho columnas; en 2560x1440 puede
llegar a diez. Los slots horizontales y la altura de fila son independientes,
por lo que una ventana compacta no fuerza una portada de 240 px ni produce
overflow. Todas las imagenes solicitan cacheWidth/cacheHeight ajustados a DPR.

El estado vacío contiene mensaje corto, explicación local-first y una sola
acción principal. Un archivo ausente reduce contraste y anuncia “Localizar”; un
error de portada mantiene geometría. El color nunca es el único indicador.

## 14. Lector

El canvas y las páginas permanecen estables cuando aparecen controles o
resultados. Abrir un panel no reconstruye el visor: en expandido recalcula su
área útil y recentra la página; en compacto/medio el panel se superpone. La
página usa márgenes proporcionales y un fondo plano, sin texturas.

La barra superior superpuesta se reparte en tres grupos flotantes y prioriza
por clase de ventana:

```text
Volver | Título flexible | Buscar | Panel opcional | Paleta | Más
```

La navegación y zoom viven en un control inferior contextual, atajos y paleta.
Una página, Dos páginas y Continua viven en la paleta `Ctrl+K`, no como selector
permanente de la barra.
El progreso es una línea de 2 px. En compacto se ocultan acciones menos
frecuentes dentro de un único “Más”, sin eliminarlas. Los grupos de modo y
panel pueden ocultarse como unidades; su elección es local y persistente. Un
estado activo añade fondo/forma y semántica, no solo otro color. Tooltips,
menús y paleta presentan el atajo resuelto.

## 15. Modo documento

Prioriza precisión, búsqueda, miniaturas, índice, página, zoom y ajuste. Puede
mostrar más densidad que el modo libro, pero el PDF sigue dominando.

La vista continua usa placeholders de altura estable, separaciones
consistentes y render bajo demanda. Cambiar zoom no debe perder la página
actual.

## 16. Modo libro

Prioriza página, márgenes, ritmo, progreso y continuidad. La barra y controles
se ocultan sin cambiar el layout.

Una página se centra y adapta por anchura o altura. Dos páginas permanecen en
Libro incluso con una ventana estrecha, reduciendo margen y escala de forma
determinista. El pliego respeta:

- portada individual;
- pares e impares;
- LTR/RTL;
- canal central de 2 px y sombras interiores simétricas;
- páginas de distinto tamaño, última hoja sin pareja y portada individual.

Las zonas laterales miden 64 px, son visualmente transparentes, ofrecen
feedback discreto, no invaden el centro y respetan RTL.

## 17. Panel lateral

Ancho inicial: 344 px; límites: 288–480 px. En compacto se acota además al 88 %
del viewport.

Muestra una sola sección: miniaturas, índice, marcadores, anotaciones o
búsqueda. Tiene encabezado compacto, selector interno de sección, cierre
visible, contenido desplazable y selección clara.
En compacto/medio aparece como superposición con scrim; en expandido ocupa una
columna propia, resta siempre su ancho al visor y recentra la página. Fijar
mantiene el estado persistente, pero no altera este contrato geométrico. Un
divisor de 10 px admite ratón,
flechas y semántica de valor. El ancho se guarda con debounce; solo se monta la
sección activa y las miniaturas visibles reutilizan su futuro/scroll. Escape lo
cierra y restaura el foco.

## 18. Paleta de comandos

La paleta conserva el minimalismo al concentrar acciones poco frecuentes.
Ancho de 520–600 px y altura máxima de 520 px.

Incluye campo de búsqueda, etiqueta de grupo, comandos, hasta ocho documentos
recientes, selección por teclado, atajos visibles y estado vacío. La misma
consulta busca título, autor y archivo sin persistirse. Arriba/abajo cambia
selección; Enter ejecuta; Escape cierra. Los resultados son filas densas, no
tarjetas, y la selección combina fondo suave e indicador.

## 19. Ajustes

En escritorio, una navegación lateral pequeña cambia entre Apariencia,
Lectura, Barra del lector, Atajos, Biblioteca, Historial, Copias locales y
Privacidad. En compacto, las secciones se apilan.

Cada ajuste es una fila sencilla con etiqueta, descripción corta, separador y
control; la sección no se encierra en una tarjeta. El contenido usa un ancho
máximo legible y, en escritorio, los controles se alinean en una columna de
240 px; en compacto vuelven al flujo apilado.
Los cambios reversibles se aplican inmediatamente. Las acciones destructivas
exigen confirmación proporcional. Apariencia incluye una previsualización
pequeña de tema, densidad, canvas y perfil e-ink. No se encierra cada sección
en una tarjeta.

La ficha del documento es un diálogo de datos en dos columnas, ruta
seleccionable y acciones jerarquizadas. “Retirar” y “Reiniciar” confirman;
“Retirar” declara que el PDF no se borra. Historial y copias son superficies
secundarias: agrupan sin gráficos y presentan recuentos/vista previa antes de
una mutación.

## 20. Estados de interfaz

Todo estado importante combina icono o forma, título breve, explicación y
acción cuando existe recuperación:

- biblioteca inicial: “Cargando biblioteca…”;
- PDF: “Abriendo documento…”;
- render: placeholder estable;
- búsqueda: progreso cancelable;
- sin resultados/sin texto: explicación y siguiente acción;
- ausente: localizar o retirar;
- cambiado: revisar/abrir como versión actual;
- portada fallida: icono específico, sin cambiar geometría;
- error: reintentar/volver;
- contraseña: campo enfocado y error contextual;
- marcador/caché/restauración: snackbar breve.

No se usan spinners indefinidos sin texto ni diálogos para confirmaciones
triviales.

## 21. Diseño adaptable

Breakpoints únicos:

```text
Compacto:  < 680 px
Medio:     680–1099 px
Expandido: >= 1100 px
```

La política no depende solo del ancho total. Resta el panel visible, considera
la relación de aspecto y exige 340 px por página antes de activar vista
enfrentada. La política vive en una función probada, no dispersa en widgets.

## 22. Entrada táctil

Ninguna función esencial depende de hover. El dominio expone avanzar,
retroceder, zoom, controles y marcador sin conocer teclado, ratón o gesto.

En superficies táctiles:

- objetivos de 48 × 48;
- controles inferiores;
- gestos sin conflicto con selección;
- una página en vertical;
- enfrentadas solo en horizontal y con ancho legible.

## 23. Tinta electrónica

El perfil `electronicInk` aplica:

- fondo blanco;
- página única;
- animaciones desactivadas;
- sin sombras, transparencias ni desenfoque;
- controles de alto contraste;
- navegación por página;
- sin continuo por defecto.

Refresco completo configurable, botones físicos, contraste y modos
calidad/velocidad permanecen pendientes hasta disponer de dispositivo o
emulador apropiado. No se simula hardware no validado.

## 24. Accesibilidad

Objetivo mínimo: WCAG AA en controles.

- recorrido completo por teclado y orden lógico;
- foco inequívoco de 2 px con separación visual;
- texto escalable y sin cortes;
- iconos etiquetados;
- estados no dependientes solo del color;
- objetivos táctiles;
- movimiento reducido;
- contraste alto;
- semántica para página, progreso y acciones.

La matriz incluye 100 %, 125 %, 150 % y 200 % de escalado, texto aumentado,
tema oscuro y foco por teclado. Solo se marca validada cuando exista evidencia
nativa.

## 25. Reglas de implementación

- Centralizar tokens y políticas responsive.
- No usar colores hexadecimales en widgets salvo canvas PDF documentado.
- No repetir radios, duraciones o espacios arbitrarios.
- No usar `Card` para cada agrupación.
- Adaptar la apariencia Material cuando no encaje.
- Mantener comportamientos nativos que ayuden a comprender.
- No sacrificar semántica por componentes personalizados.
- Evitar reconstrucciones costosas, blur y transparencias grandes.
- No añadir paquetes visuales.
- Mantener presentación desacoplada de PDFium y SQLite.
- Documentar toda excepción al sistema de tokens.

## 26. Antipatrones

Quedan prohibidos:

- tarjetas para cada documento o tarjetas anidadas;
- botones con gradientes;
- barras superiores altas;
- logotipos permanentes grandes;
- sombras profundas;
- radios excesivos o cápsulas generalizadas;
- cristal, blur o fondos con textura;
- rebotes u overshoot;
- familias de iconos inconsistentes;
- texto auxiliar inferior a 11 px;
- información duplicada;
- botones sin jerarquía;
- acento en grandes superficies;
- acciones esenciales solo en hover;
- layouts de landing page;
- grandes vacíos sin intención;
- spinner sin contexto;
- diálogos para acciones frecuentes;
- decoración que reste espacio al PDF.

## 27. Criterios de revisión visual

Antes de cerrar una tarea visual:

- [ ] El PDF o documento principal domina.
- [ ] La acción primaria se identifica sin explorar menús.
- [ ] No hay overflow, texto cortado ni objetivos inaccesibles.
- [ ] Compacto, medio y expandido tienen una composición deliberada.
- [ ] Tema claro y oscuro conservan contraste y jerarquía.
- [ ] Los seis canvas de lectura mantienen legibilidad.
- [ ] El foco se ve y el recorrido de teclado es lógico.
- [ ] Escape cierra la superficie superior.
- [ ] Movimiento reducido elimina desplazamientos.
- [ ] Carga, vacío, error, ausente y sin resultados explican el estado.
- [ ] No aparecen tarjetas, sombras, radios ni acentos sin función.
- [ ] Las capturas corresponden a código ejecutado.
- [ ] Análisis, pruebas e integración no presentan regresiones.

## 28. Superficies de tercera generación

- El lector no presenta una segunda franja de pestañas. Abrir otro PDF vive en
  el menú compacto y en `Ctrl+O`; biblioteca, recientes y espacios recuperan
  documentos sin restar altura ni desplazar el centro del PDF.
- Al cambiar el ancho de ventana o acoplar el panel, el visor conserva el zoom
  elegido y vuelve a centrar el documento dentro del área útil.
- Menús, paneles, ajustes, diálogos, scrollbars, chips y controles comparten
  superficies, bordes, radios y estados semánticos del tema global.
- Anotaciones reutiliza el panel lateral único. La acción primaria es “Anotar
  página”; búsqueda y filtros permanecen encima de una lista virtualizada.
- Colecciones se eligen en una hoja ligera y la biblioteca conserva la misma
  jerarquía de “continuar” y “recientes”; filtrar no crea otra pantalla.
- Diagnóstico y workspaces son diálogos de gestión, no dashboards permanentes.
- Safe mode se expresa con un indicador sobrio y texto accionable, nunca con
  bloqueo o borrado automático.
- Split no se muestra hasta que existan dos lectores reales y medidos; el
  breakpoint técnico por sí solo no constituye una función visual.

La referencia vigente `windows-reader-editorial-v07.png` se genera desde una
prueba de widgets a 1600×900 y fija Libro con tema Marfil, grupos flotantes,
pliego enfrentado y dock inferior. Las
capturas generacionales se retiraron porque ya no representaban el producto.
Narrador, varios monitores y DPI físico siguen requiriendo validación nativa.

## 29. Sala de lectura digital — 0.6

La selección de Biblioteca tiene dos capas: la portada conserva el acento
violeta y un inspector anclado aporta título, autor, estado, última lectura y
siguiente acción. En expandido es una banda horizontal; en medio una franja
compacta; en compacto una superficie inferior descartable. Su aparición no
recalcula el jitter y su salida dura 160 ms o cero con movimiento reducido.

El lector distingue reposo y actividad. El reposo conserva una dock de 40 px
con progreso, posición y affordance de expansión; la actividad revela los
grupos completos. Ambas son overlays sobre la misma geometría del visor. En
expandido, el panel usa un rail vertical de 52 px; en compacto/medio mantiene
los cinco accesos en la cabecera.

Ajustes reserva en expandido 320 px para una muestra persistente. La muestra
reutiliza controles del producto, refleja canvas, papel, chrome y progreso, y
responde a las preferencias sin abrir PDFium. En compacto vive dentro de
Apariencia y no introduce desplazamiento horizontal.

## 30. Retícula editorial de escritorio — 0.7

Biblioteca convierte cabecera, contador, contexto de filtros e inspector en una
secuencia editorial. El inspector separa identidad, estado y progreso y decide
la acción primaria por disponibilidad y avance. Archivo modificado, análisis y
revisión usan forma e icono además de color.

El lector une navegación y herramientas en una sola banda con divisores. En
expandido la navegación permanece geométricamente centrada; barra, dock y panel
siguen fuera de la clave del visor. El rail acoplado usa una línea vertical y el
divisor distingue reposo, hover, foco y arrastre.

Ajustes agrupa preferencias, mantenimiento y privacidad, subordina las acciones
destructivas y reserva 320 px a una preview manipulable. La paleta añade iconos
por acción y una explicación lateral en expandido sin convertir filas en
tarjetas. Los tres niveles de elevación siguen siendo plano, overlay y diálogo.
