# Auditoría de simplificación de Biblioteca

Fecha: 2026-09-08. Checkpoint recuperable: `d4d86a263acf1c1aeb5f6ce2e187b7a76163c6e9`
(`main` limpio antes de editar).

## Alcance y evidencia

La auditoría combina la ejecución del bundle Windows actual, las capturas de
widgets y del flujo PDFium real conservadas en
`build/visual-evidence/v0.15.0/`, y los contratos descritos en `TESTING.md`.
Las capturas cubren claro, oscuro, 640×480, 1280×720, texto al 200 %, listas,
cuadrículas, filtros, paneles y diálogos. La ejecución nativa disponible no
acredita Narrator, varios monitores ni escalas físicas distintas de la del host.

## Inventario breve

| Pantalla o estado | Esencial | Útil | Secundario | Redundante / relleno | Dudoso o no verificable |
| --- | --- | --- | --- | --- | --- |
| Biblioteca vacía | Explicación breve y `Abrir PDF` | — | — | Nombre y logo visibles sin función operativa | La identidad permanece en metadatos accesibles, no como decoración |
| Biblioteca con 1 documento | Documento, título, estado y apertura | Búsqueda y cambio de vista | Inspector tras seleccionar | Inspector abierto automáticamente sin intención | DPI físico y Narrator |
| Biblioteca con 2–3 documentos | Catálogo, selección y apertura | Búsqueda, vista y filtro | Sesión reciente plegada | Recuento repetido en cabecera y barra | Solapamiento del inspector en todos los títulos reales largos |
| Biblioteca con decenas | Retícula/lista virtualizada y consulta SQL | Orden, filtros y agrupación | Selección múltiple | Icono enmarcado junto al recuento, sin significado adicional | Estabilidad perceptual con 100 000 portadas reales |
| Cuadrícula | Portada, título, selección y foco | Autor/estado cuando existen | Hover informativo | — | Portadas fotográficas de proporciones extremas |
| Lista | Título, estado, progreso y foco | Páginas y última apertura | Metadatos que colapsan con texto ampliado | — | — |
| Búsqueda global | Campo, tipos, resultados, selección y abrir | Estado del índice y preview | Filtros por tipo | — | Corpus personal grande no usado por privacidad |
| Filtros, orden y colecciones | Filtros activos y restablecer | Facetas, agrupación y orden | Acceso a colecciones | Las mismas rutas aparecen en rail y menú superior expandido | — |
| Inspector de documento | Título, estado, progreso, acción contextual y cerrar | Favorito e información | Datos editoriales | Apertura automática con un único documento | Superposición física a escalas no disponibles |
| Lector normal / fullscreen | Documento, navegación, página y salida | Zoom, búsqueda y vista | Menú de acciones infrecuentes | — | Caption/taskbar fullscreen y DPI físicos fuera del host |
| Índice, búsqueda y anotaciones | Identidad del panel, cerrar, resultados y salto | Filtros y acciones de nota | Pin y orden | — | Narrator físico |
| Ajustes | Sección, controles y consecuencias importantes | Búsqueda y preview | Restablecer sección | — | Escala física 200 % |
| Diálogos, menús y avisos | Título, consecuencia y acción | Progreso/cancelación | Detalle técnico bajo demanda | — | Avisos nativos del sistema |
| Restauración de sesión | Documento, página y decisión | Motivo de recuperación | Detalle de diagnóstico | — | Cierre inesperado físico durante E/S concurrente |

## Oportunidades priorizadas

1. **Alto impacto, alta confianza, bajo riesgo:** conservar el recuento junto a
   búsqueda y filtros y retirarlo de la cabecera.
2. **Alto impacto, alta confianza, bajo riesgo:** no seleccionar automáticamente
   el único documento; mostrar su inspector solo tras una acción del usuario.
3. **Impacto medio, alta confianza, bajo riesgo:** ocultar Ajustes en la cabecera
   cuando el rail expandido ya ofrece la misma acción.
4. **Impacto medio, alta confianza, riesgo bajo:** centrar el rail expandido en
   encontrar y organizar documentos; llevar revisión de anotaciones y problemas
   al menú secundario y evitar duplicar allí las rutas principales del rail.
5. **Impacto medio, alta confianza, riesgo bajo:** quitar la caja e icono
   decorativos del resumen de resultados; el texto y la región viva conservan
   su función.
6. **Impacto medio, confianza media, riesgo medio:** agrupar selección múltiple
   y facetas bajo demanda. Implementado en la revisión `cover-first` con un menú
   etiquetado, tooltips, teclas y estados comprobables.
7. **Impacto alto, confianza media, riesgo medio:** convertir el inspector
   flotante en una región que reserve altura solo al seleccionarse. Se pospone:
   puede introducir saltos de layout y reconstrucciones en una ruta con una
   regresión de rendimiento ya conocida.

Esta pasada implementa únicamente los puntos 1–5. A petición del usuario,
también sustituye el matiz azul del tema oscuro global por negro y grises
neutros, conservando el violeta funcional. Lector, Ajustes, búsqueda y
persistencia no cambian de estructura.

## Resultado verificado

- Análisis estático limpio, 326 pruebas y build Windows Release aprobados.
- Flujo PDFium real aprobado con restauración en página 2, temas claro/oscuro y
  PDF sintético byte-idéntico; no usa datos personales.
- Matrices de Biblioteca, Lector y Ajustes regeneradas en
  `build/visual-evidence/library-simplification-after/`, con once comparaciones
  antes/después revisadas.
- Smoke offline aprobado durante ocho segundos, sin conexiones TCP/UDP. El
  benchmark de catálogo aprueba 5 000, 25 000 y 100 000 documentos, limita la
  página materializada a 80 y no recorre el sistema de archivos en la primera
  carga.
- El gate agregado queda en `failed` solo porque la matriz nativa no dispone de
  confirmación física de 96 DPI/100 %. Conserva cuatro capturas candidatas, pero
  no acredita DPI, Narrator ni varios monitores. La build tampoco está firmada.
- Las regresiones de rendimiento de 0.15 continúan abiertas: Biblioteca 6986
  frente a 4672 µs (+49,53 %) y lector 8078 frente a 6760 µs (+19,50 %). Esta
  pasada no consumió ninguno de los dos experimentos permitidos sin una nueva
  hipótesis respaldada por trazas.

## Mejora adicional de visibilidad

La revisión posterior reduce todavía más el peso cromático: selección y
superficies activas oscuras pasan a gris neutro, y las portadas generadas usan
la superficie secundaria en lugar de teñir toda la cubierta con el acento. El
violeta queda limitado al canto, indicadores y acciones.

La selección de una tarjeta añade una marca visible y mantiene el título en el
color de texto primario con mayor peso. En lista, el foco de teclado deja de ser
una línea lateral y rodea la fila completa con un contorno de 2 px. Así, foco y
selección no dependen solo del color y siguen siendo distinguibles sin añadir
controles permanentes.

La matriz posterior queda en `build/visual-evidence/visibility-pass/`. Se
revisaron Biblioteca oscura, selección clara, Biblioteca a 640×480 con texto al
200 %, Ajustes oscuro, Lector oscuro y Navegador oscuro. El análisis estático,
las 326 pruebas y la build Windows Release vuelven a pasar después de esta
segunda revisión.

## Biblioteca `cover-first` y movimiento

La revisión del 8 de septiembre toma como principio del navegador de Memory
Card de PS2 que cada elemento guardado sea el objeto visual reconocible. No
copia su estética: traduce esa jerarquía a documentos PDF. La Biblioteca deja
de mostrar nombre y logo, elimina el rail persistente y agrupa vista, selección
múltiple y facetas en un menú. Búsqueda, filtros, importación, colecciones,
espacios, anotaciones, problemas y Ajustes siguen accesibles bajo demanda.

Las portadas crecen hasta 220 px y la retícula usa 3/5/6/8 columnas según el
ancho. Las portadas generadas ya contienen su título, por lo que no lo repiten
debajo; una portada real conserva una leyenda truncada. `Continuar leyendo` se
reduce a una sola fila con título, progreso y acción, de modo que el catálogo
empieza antes y mantiene el protagonismo.

El modo `Animaciones completas` usa una entrada y respuesta de foco/hover o
selección de 160 ms (`easeOutCubic`) limitada a la portada activa. `Movimiento
reducido` fija transformaciones, entrada y sombras animadas en estado estático.
También prevalecen la preferencia de Windows y el alto contraste. El valor
histórico `none` se sigue leyendo por compatibilidad, pero la interfaz presenta
solo los dos modos solicitados.

La matriz específica queda en `build/visual-evidence/cover-first-library/` y
cubre claro, negro, 640×480, texto al 200 %, alto contraste, movimiento
reducido, selección, lista y Ajustes. Las pruebas focalizadas (45 y después 16
casos tras compactar continuidad), el análisis estático, las 326 pruebas y la
build Windows Release pasan. El flujo nativo con PDFium real también pasa,
restaura la página 2 y deja el PDF sintético byte-idéntico.

El gate final aprueba 13 de 14 fases sin omisiones. Solo falla la matriz nativa
porque el host no confirma físicamente sus 96 DPI/100 %; no se fabricó esa
confirmación. La inspección mediante `computer-use` tampoco pudo iniciarse
porque el runtime de esta sesión no expuso aplicaciones Windows. Las capturas
automatizadas son evidencia válida de composición, pero no certifican Narrator,
varios monitores ni DPI físicos. El bundle conserva nueve binarios sin firma y
no es publicable.

## Cubiertas tridimensionales

La revisión posterior sustituye las cubiertas de reserva planas por libros
vectoriales con perspectiva suave, lomo, bloque posterior y sombra de contacto.
Cada documento recibe de forma determinista una de seis paletas y uno de seis
emblemas geométricos; no se inventan metadatos y la identidad permanece estable
entre ejecuciones. El número de páginas mostrado procede del documento.

Cuando existe una carátula real, la misma figura tridimensional reutiliza esa
imagen como frente del libro. No se añaden bitmaps, paquetes ni dependencias:
el volumen, el patrón y los emblemas se dibujan con widgets y `CustomPainter`.
Cada cubierta queda aislada por `RepaintBoundary`. Alto contraste usa blanco y
negro sin patrón ni profundidad decorativa; movimiento reducido conserva el
objeto estático y elimina sus interpolaciones.

La matriz revisada queda en
`build/visual-evidence/three-dimensional-covers/library/` y cubre claro, negro,
selección, alto contraste, movimiento reducido y 640×480 con texto al 200 %.
El análisis estático, las 326 pruebas y la build Windows Release pasan. El ZIP
de prueba mide 19.683.410 bytes (18,77 MiB), 27.342 bytes menos que el ZIP 0.15
anterior, y el smoke de ocho segundos confirma cero conexiones de red.
