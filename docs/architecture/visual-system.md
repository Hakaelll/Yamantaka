# Sistema visual

La interfaz usa un vocabulario semántico único. Las superficies no deben
introducir colores, radios, sombras o duraciones ad hoc cuando existe un token
equivalente.

## Fuentes de verdad

| Categoría | Fuente | Regla |
| --- | --- | --- |
| Color | `lib/app/theme/app_tokens.dart` | Usar `context.appColors`; claro, oscuro y alto contraste conservan la misma semántica. |
| Espaciado, tamaños y radios | `AppSpacing`, `AppSizes`, `AppRadii` | Elegir el token más cercano; una excepción requiere motivo geométrico local. |
| Elevación y sombras | `AppElevation`, `AppShadow` | Superficies planas por defecto; sombras solo para overlays, páginas y portadas. |
| Movimiento | `AppMotion` | Pasar siempre por `whenAllowed`; reduced motion produce duración cero. |
| Densidad | `AppDensityTokens` | La densidad compacta reduce ritmo, no objetivos de interacción. |
| Layout | `AppBreakpoints`, `AppWindowClass` | Clasificar por espacio lógico disponible, no por resolución física aislada. |
| Tipografía | `AppTheme` | Segoe UI Variable/Segoe UI con fallback; los estilos tienen función semántica. |

## Jerarquía de forma

- `page` (3 px) identifica papel PDF y portadas, nunca chrome.
- `control` (10 px) se reserva para controles editoriales discretos.
- `field` (12 px) unifica entradas y menús.
- `floating`/`panel`/`dialog` (16/18/20 px) escalan con la superficie.
- `capsule` solo agrupa acciones, selector Documento/Libro y dock; no se aplica
  a filas de contenido ni páginas.

Los iconos globales usan objetivos circulares de 40 px. Selección, hover,
presión y foco combinan superficie, borde y semántica; el color no actúa solo.

## Identidad de Biblioteca

`AppTheme.library` utiliza `AppColorTokens.library` para mantener el fondo
negro `#080808` y superficies en escala de grises tanto si la aplicación
general está clara como oscura. `librarySelectionAccent` reserva el violeta
`#B58AFF` para el archivo seleccionado; Lector y Ajustes conservan las paletas
editoriales.

No hay texturas, blur, vídeo ni movimiento ambiental. La profundidad procede
de espacio negativo, escala breve, sombra acotada y un halo estático bajo la
portada seleccionada.

## Chrome del lector y Libro

La barra superior y la dock inferior son overlays: mostrarlas u ocultarlas no
modifica las restricciones del visor. En compacto/medio el panel también es
overlay con scrim; en expandido resta siempre su ancho y el visor se recentra
en el área restante. Abrir, cambiar o redimensionar el panel no reconstruye el
`RepaintBoundary` cacheado por configuración de documento.

Biblioteca limita selección a escala 1,035, elevación 6 px y halo sin spread;
el foco blanco de 2 px es independiente del borde violeta. Ajustes alinea sus
controles en 240 px dentro de contenido legible y la paleta usa filas de 52 px
con grupos separados para comandos y documentos recientes.

Los seis `ReaderTheme` resuelven el canvas y una paleta clara/oscura para el
chrome en Documento y Libro. Libro conserva `coverPolicy`, `initialSide` y RTL;
el ajuste calcula página o ancho sobre el pliego completo y nunca cambia de
modo por un breakpoint.

## Excepciones permitidas

- El fondo y la sombra de la página PDF pueden variar con el tema para mantener
  legibilidad del papel sin cambiar el contenido.
- Una portada puede reservar un tamaño físico distinto al de una fila cuando la
  composición lo necesita; el tamaño sigue saliendo de `AppSizes` o de los
  tokens de densidad.
- Los colores `danger`, `warning` y `success` son estados semánticos y siempre
  deben acompañarse de texto, icono o borde; nunca son el único indicador.

## Contrato de accesibilidad

- El foco usa `focusRing` y un borde de 2 px en los controles globales.
- Los objetivos de escritorio conservan al menos 40×40 logical px; el tamaño
  táctil disponible es 48×48.
- El texto normal y los componentes no textuales se validan en
  `test/accessibility_audit_test.dart`.
- La matriz de color alto contraste reemplaza los tokens por blanco/negro y
  foco fuerte sin depender de un tema personalizado.

Los cambios visuales deben actualizar este documento o justificar la excepción
en el mismo cambio. Las capturas y la matriz nativa siguen siendo evidencia
externa: este contrato no las sustituye.

## Contexto y chrome progresivo 0.6

La portada seleccionada abre una banda de contexto, no otra tarjeta. La banda
reserva una acción primaria y como máximo dos secundarias. En compacto se
comporta como superficie inferior descartable; al filtrar fuera de la selección
explica el cambio y permite recuperar el resultado sin perder el ID.

Cuando el autoocultado entra en reposo, la barra superior desaparece y la dock
se reduce a posición/progreso. Conserva 40 px accionables y revela los grupos
completos con puntero, toque o teclado; el cambio nunca modifica las
restricciones del visor. El panel acoplado usa un rail vertical de 52 px y el
overlay conserva la barra horizontal.

En Ajustes expandido, `settingsPreviewWidth` reserva 320 px a una muestra viva
persistente que reutiliza `YamaToolbarGroup` y `YamaIconButton`. La muestra
refleja preferencias sin abrir un documento ni duplicar el motor del lector.

## Retícula editorial 0.7

`AppTypeScale` explicita título de pantalla/sección/documento, encabezado
contextual, cuerpo, control, metadato, auxiliar, cifra y atajo. El mínimo
esencial es 12 px y cifras, páginas, zoom, porcentajes y atajos son tabulares.

Biblioteca usa una cabecera de 84 px y hace visible el número de filtros. Su
inspector es una banda por columnas; los estados usan icono, texto y tono. El
lector presenta una superficie continua arriba y una única dock abajo; los
divisores internos sustituyen cápsulas independientes. En expandido la función
primaria permanece centrada.

El rail acoplado mide 60 px y usa indicador vertical. Ajustes reserva 236 px a
la navegación y 320 px a la preview; sus grupos son Preferencias, Mantenimiento
y Privacidad. La paleta puede crecer a 780 px y reserva 228 px a la preview del
resultado solo en expandido. Ninguna de estas superficies crea un nivel de
elevación adicional ni entra en la identidad del visor PDF.
