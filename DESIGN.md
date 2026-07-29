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
topBarHeight            = 46
bottomControlHeight      = 44
sidePanelWidth           = 320
compactSidePanelWidth    = 304
desktopControlExtent     = 40
touchControlExtent       = 48
dialogMaxWidth           = 600
commandPaletteMaxHeight  = 520
contentMaxWidth          = 1000
readerMinimumPageWidth   = 340
readerMaximumPageWidth   = 920
readerEdgeNavigation     = 64
readerSpreadGutter       = 16
```

### Movimiento

```text
motionInstant          = 0 ms
motionFeedback         = 80 ms
motionFast             = 120 ms
motionStandard         = 160 ms
motionPanel            = 200 ms
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
| Metadato secundario | 11 | 400 | 1,35 |
| Etiqueta auxiliar | 12 | 500 | 1,30 |
| Estado compacto | 13 | 400 | 1,35 |
| Control/interfaz | 14 | 500 | 1,35 |
| Texto destacado | 16 | 500 | 1,35 |
| Título de sección | 18 | 600 | 1,25 |
| Título de pantalla | 22 | 600 | 1,20 |
| Mensaje principal | 28 | 600 | 1,15 |

No se usa texto inferior a 11 px. Páginas, porcentajes y atajos emplean
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
- 4 px: controles compactos;
- 6 px: campos, menús y superficies pequeñas;
- 8 px: diálogo y paleta;
- 10 px: superficie táctil grande.

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

No todas las superficies aparecen a la vez. La barra mide 46 px y puede
ocultarse/revelarse en cualquier modo mediante el mismo comando persistido.

## 13. Biblioteca

Jerarquía:

```text
Encabezado: Yamantaka | Abrir PDF | Ajustes
Continuar leyendo
Recientes | Filtrar | Estado | Ordenar
Acciones secundarias
```

“Continuar leyendo” es dominante cuando existe un documento válido. Combina
portada, título, autor disponible, página, porcentaje, última lectura y una
acción inequívoca. Se presenta como un bloque editorial delimitado, no como un
banner ni una tarjeta elevada.

Los recientes forman una lista sin tarjetas individuales. En expandido se
alinean metadatos en columnas; en compacto se apilan bajo el título. El hover
puede revelar acciones, pero abrir/localizar siempre funciona con teclado.

El filtro opera localmente sobre título, autor y archivo, se enfoca con
`Ctrl+F` y no persiste su texto. Ordena por última apertura, título o progreso.
El estado (`Todos`, `En curso`, `Sin empezar`, `Terminados`) se combina en la
misma evaluación y sí puede persistirse. El selector segmentado tiene recuentos
y desplazamiento horizontal contenido en compacto, sin elevar el app bar. El
contador se actualiza como región semántica y cero coincidencias ofrece
“Limpiar filtro”.

Cada portada solicita una decodificación ajustada a su caja física y DPR, usa
calidad baja para miniatura y vive en un límite de repintado propio. La lista
continúa virtualizada mediante slivers.

El estado vacío contiene mensaje corto, explicación local-first y una sola
acción principal. Las filas expresan “Sin empezar”, “En curso” o “Terminado”.
Un archivo ausente reduce contraste y muestra texto/icono/“Localizar”; un
archivo cambiado usa aviso propio y un error de imagen usa icono roto. El color
nunca es el único indicador. La acción de información es independiente de
abrir y conserva un objetivo accesible.

## 14. Lector

El canvas y las páginas permanecen estables cuando aparecen controles, paneles
o resultados. La página se centra con márgenes proporcionales y un fondo plano,
sin texturas.

La barra superior prioriza por clase de ventana:

```text
Volver | Título flexible | Modo opcional | Buscar | Panel opcional | Paleta | Más
```

La navegación y zoom viven en un control inferior contextual, atajos y paleta.
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

Una página se centra y adapta por anchura o altura. Dos páginas respetan:

- portada individual;
- pares e impares;
- LTR/RTL;
- separación central de 12–24 px;
- cambio automático a una página si cada hoja tendría menos de 340 px.

Las zonas laterales miden 64 px, son visualmente transparentes, ofrecen
feedback discreto, no invaden el centro y respetan RTL.

## 17. Panel lateral

Ancho inicial: 320 px; límites: 280–480 px. En compacto se acota además al 86 %
del viewport.

Muestra una sola sección: miniaturas, índice, marcadores o búsqueda. Tiene
encabezado de 46 px, cierre visible, contenido desplazable y selección clara.
En compacto/medio aparece como superposición con scrim; en expandido se alinea
al borde sin mover bruscamente la página. Un divisor de 10 px admite ratón,
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

Cada ajuste es una fila sencilla con etiqueta, descripción corta y control.
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
