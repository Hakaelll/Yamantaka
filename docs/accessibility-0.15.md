# Accesibilidad y revisión visual 0.15

Registro de alcance, 7 de septiembre de 2026. Los resultados del gate y de la
release se centralizan en `architecture/visual-iteration-0.15.md`.

## Evidencia automatizada

- `test/accessibility_audit_test.dart`: contraste de pares semánticos y nombres
  accesibles de superficies de estado. No equivale a una auditoría de Narrator.
- `test/iteration_014_test.dart`: controles, texto ampliado, RTL, movimiento
  reducido y superficies críticas compartidas.
- `test/iteration_015_test.dart`: navegación SQL de Biblioteca, reinicio de
  filtros, aislamiento de decisiones y foco/desactivación de controles.
- `test/iteration_015_review_test.dart`: menú al 200 % con atajo redistribuido;
  reglas locales legibles y traducidas en ES/EN, claro/oscuro, 640×480 al 200 %.
- `test/library_widgets_test.dart` y `test/reader_widgets_test.dart`: teclado,
  selección, foco, navegación, paneles y conservación del contexto.

Las matrices visuales explícitas cubren las ocho resoluciones de 640×480 a
3440×1440; claro/oscuro; texto 100/130/150/200 %; ES/EN; RTL; alto contraste y
movimiento reducido. Se combinan los casos de mayor riesgo, no todas las
permutaciones. Los manifiestos de cada familia enumeran los casos ejecutados.

## Revisión humana de imágenes

Se revisan composición, alineación, legibilidad, acciones alcanzables, foco y
estados no ideales. Las hojas de contacto sirven para comparar estados vecinos;
los PNG a tamaño real permiten inspeccionar detalles. El registro principal
describe los defectos encontrados y las correcciones posteriores.

Los fixtures utilizan contenido sintético. Los de lector y comparación pueden
usar un motor de prueba; demuestran la interfaz, no la rasterización de PDFium.
El gate, el perfil nativo y el smoke comprueban separadamente el motor real.
`integration_test/professional_polish_flow_test.dart` recorre lector, Biblioteca,
Ajustes, cambio de tema y restauración de página con PDFium real. Comprueba
Ctrl+F y que el PDF conserve su SHA-256; utiliza preferencias en memoria y
datos temporales. Sus imágenes son renders de integración Flutter, no capturas
del compositor ni pruebas de entrada de teclado física.

## Límites físicos

Texto ampliado en widgets no equivale a DPI de Windows. Un manifiesto de
capturas tampoco demuestra que Narrator anuncie correctamente un flujo real.
La sonda del gate registra el DPI que Windows comunica al proceso. Ese valor
puede estar virtualizado y, sin comprobación independiente del monitor, no
certifica por sí solo una escala física.

Narrator, varios monitores y las escalas físicas que no se hayan ejecutado
permanecen pendientes. Las capturas WGC con regiones obsoletas se excluyen
de la aceptación estética; no se reinterpretan como un resultado aprobado.
