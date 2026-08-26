# Iteración visual 0.9: lector compacto de precisión

Estado: implementada; la sección de verificación se completa con la evidencia y
el release local generados después del último cambio.

## Intención

Yamantaka 0.9 reduce el aspecto de interfaz web y devuelve área y atención al
documento. La revisión cambia densidad, jerarquía y transiciones, no el motor
PDF, los datos persistidos, la política local-first ni la identidad del visor.

## Diagnóstico de 0.8

La base ya ofrecía buen contraste, seis temas, overlay/docking correctos,
selección semántica, empty states específicos y foco visible. La barra superior
seguía ocupando 64 px y comenzaba en el borde; el dock podía alcanzar 56 px; el
rail medía 60 px; la cabecera de Biblioteca 84 px y las filas normales 58 px.
Además, alternar grid/list reiniciaba explícitamente el offset de scroll. El
margen del papel se derivaba del tamaño antes de la caché y podía reconstruir el
visor al cruzar un escalón responsive.

## Contrato 0.9

| Superficie | Escritorio | Compacto/reposo |
|---|---:|---:|
| Barra superior | 56 px | 52 px |
| Dock inferior | 48 px | 38 px en reposo |
| Target/icono | 40 px / 18 px | 40 px / 18 px |
| Cabecera/rail del panel | 52 px | barra horizontal de 48 px |
| Cabecera de Biblioteca | 76 px | 76 px |
| Resultado/paleta | 48 px | 48 px |
| Fila de Biblioteca | 52 px | ≥64 px con texto >130 % |

La barra deja 8 px constantes sobre una superficie de radio 10 px. El título usa
el nombre real, una línea, ellipsis y tooltip. Atrás/adelante solo se montan si
existe la acción. En anchos estrechos, herramientas secundarias viven en menús.

La dock mantiene la navegación en el centro geométrico en expanded. En compacto
prioriza anterior/siguiente, posición y zoom; ajuste, tema y otras opciones se
agrupan. Reposo/actividad son overlays sobre idéntica geometría PDF.

## Canvas e identidad

El margen efectivo sigue los escalones 12/16/24/32 px por lado corto. Se calcula
en `_PdfrxViewerState` desde sus constraints; el shell entrega únicamente la
preferencia estable. Por tanto, tamaño, breakpoint, panel, chrome, hover, foco y
menús no forman parte de `_viewerCacheKey`. El panel docked reduce la columna y
el visor se centra dentro de ella sin reconstruir el documento.

La sombra del papel usa blur 7 px y offset 1 px (4 px en gris); alto contraste,
negro, oscuro y tinta electrónica conservan su tratamiento sin transformar
píxeles. El modo Libro mantiene canal central, alineación y emparejado.

## Resto del producto

El panel reduce cabecera y rail a 52 px y conserva indicador, icono, contraste y
resize handle de 10/12 px. Biblioteca mantiene intacto
`yamantaka-logo-purple.png`, reduce cabecera y filas y añade tooltip al título de
lista. Grid/list conserva selección, foco, filtros y scroll; la transición es una
opacidad de 120 ms y se vuelve inmediata con movimiento reducido.

Búsqueda y paleta usan filas de 48 px y cabeceras compartidas más compactas.
Ajustes reduce filas a 52/44 px con controles de 40 px. Paneles y diálogos usan
radios 12/14 px. Se mantienen texto esencial ≥12 px, cifras tabulares, foco 2 px
y el orden `focus > pressed > selected > hover > rest`.

## Verificación y límites manuales

La aceptación incluye formato, diff check, análisis, suite completa, golden 0.9,
matrices widget, inspección de PNG, build Windows Release, smoke sin red, ZIP,
manifiesto, hashes e inventario de firma. Narrator, varios monitores y DPI físico
100/125/150/175/200 % quedan pendientes de prueba manual real; una captura de
widget no los sustituye. No se publica, instala globalmente ni firma sin
certificado.
