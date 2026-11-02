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
- `field` (10 px) unifica entradas y menús.
- `floating`/`panel`/`dialog` (12/14/16 px) escalan con la superficie.
- `capsule` solo agrupa acciones, selector Documento/Libro y dock; no se aplica
  a filas de contenido ni páginas.

Los iconos globales usan objetivos circulares de 40 px. Selección, hover,
presión y foco combinan superficie, borde y semántica; el color no actúa solo.

## Identidad de Biblioteca

`AppTheme.library` utiliza la paleta global de la aplicación. En modo oscuro,
el fondo negro `#0B0B0B` y las superficies grises neutras sustituyen el matiz
azul anterior; en claro se conserva el papel cálido. `librarySelectionAccent`
reserva el violeta `#B58AFF` para el archivo seleccionado. Lector y Ajustes
consumen la misma base neutral sin alterar los colores del PDF.

No hay texturas raster, blur, vídeo ni movimiento ambiental. La profundidad
procede de espacio negativo, escala breve, sombra acotada, perspectiva del
libro y un halo estático bajo la portada seleccionada. Las cubiertas de reserva
usan seis familias vectoriales deterministas; las miniaturas reales ocupan la
tapa completa y reciben el mismo lomo, canto de páginas y acabado superficial.
El paralaje responde únicamente al puntero y vuelve a una tapa alineada a
píxeles; así los bordes largos no sufren escalonado en Windows. La portada
activa aplica una respuesta amortiguada en dos ejes: el centro ofrece más
resistencia y los extremos generan mayor inclinación y un desplazamiento breve.
Tapa y canto comparten la transformación para conservar la sensación de peso;
un gradiente radial de opacidad limitada desplaza su centro con la misma señal
y sugiere un acabado mate sin introducir otro controlador. Movimiento reducido
lo convierte en volumen y luz estáticos con duraciones cero. En
oscuro y solo en ancho expandido, una filigrana raster de marca al 3,2 % ocupa
el fondo; se omite con alto contraste o texto superior al 130 %.

La selección abre un contexto flotante de máximo 880 px con miniatura del libro,
metadatos, progreso y una acción primaria. La continuidad de sesiones no
reserva una fila: vive en un menú con contador y acciones equivalentes.

## Chrome del lector y Libro

La barra superior se divide en dos cápsulas flotantes —documento y
herramientas—. La primera reserva un ancho estable y adaptable para que los
títulos reales no colapsen alrededor de la flecha; la dock inferior sigue
siendo un overlay. Mostrarlas u ocultarlas no modifica las restricciones del
visor. En compacto/medio el panel también es overlay con scrim; en expandido
resta siempre su ancho y el visor se recentra en el área restante. Abrir,
cambiar o redimensionar el panel no reconstruye el
`RepaintBoundary` cacheado por configuración de documento. Cambiar de sección
mantiene la superficie estable y cruza solo su contenido durante 120 ms.

Los avisos usan una superficie flotante neutral de 420 px con borde y cierre
vertical. Un aviso nuevo sustituye al anterior; los pasivos tienen duración
acotada y las acciones conservan tiempo para deshacer.

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

## Archivo de precisión 0.8

Biblioteca reserva cuadrícula para reconocimiento y lista para comparación. La
primera añade títulos sin bajar opacidad ni saturación; la segunda usa cifras
tabulares y colapsa estado, progreso y páginas dentro de la fila cuando el texto
supera 130 % y no hay 1400 px útiles. El cambio es efímero y ambas vistas
conservan selección, foco, doble clic, teclado y menú contextual.

`AppMenuItemContent` normaliza icono, etiqueta, atajo, check y peligro;
`AppDialogHeader` une identidad y cierre; `AppDangerZone` separa alcance,
consecuencia y acción destructiva. El color de peligro siempre acompaña texto e
icono. La búsqueda global divide lista y preview solo en expanded; la lista
continúa siendo la fuente de selección.

El estado inicial de Biblioteca usa el asset raster
`assets/branding/yamantaka-logo-purple.png`, generado desde el SVG canónico
`assets/branding/yamantaka-logo.svg`. La marca corporativa representa un libro
abierto mediante dos planos geométricos y una capa inferior de páginas sobre
violeta. Sustituye al emblema anterior por petición del propietario del
2026-11-02; DESIGN.md conserva la dirección canónica. No se convierte en fondo,
textura ni decoración persistente.

## Instrumentación compacta 0.9

- barra del lector: 56 px, o 52 px en compacto; margen superior 8 px;
- target de escritorio: 40×40 px; iconografía: 18 px; foco: 2 px;
- dock activa: 48 px; dock en reposo: 38 px;
- cabecera y rail del panel: 52 px;
- cabecera de Biblioteca: 76 px;
- fila de resultado/paleta: 48 px;
- fila de Biblioteca: 52 px a 100 %, desde 64 px por encima de 130 %;
- fila de Ajustes: 52 px cómoda y 44 px compacta, con control de 40 px;
- radios de control/panel/diálogo: 10/12/14 px.

El orden de estados permanece `focus > pressed > selected > hover > rest`.
Disabled combina icono y color; alto contraste elimina sombras; movimiento
reducido o desactivado elimina la transición grid/list. Los márgenes del papel
son 12/16/24/32 px por lado corto y se calculan dentro del visor, no en su clave.
