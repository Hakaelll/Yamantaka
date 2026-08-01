# Acciones masivas de biblioteca

La biblioteca ofrece un modo de selección accesible desde la barra de comandos.
La selección mantiene IDs locales en `LibrarySelection`, conserva el orden de
entrada y permite seleccionar rangos con Shift o alternar elementos con Ctrl o
Cmd. Escape limpia la selección y devuelve el foco a la biblioteca.

`LibraryBulkOperationService` procesa cada documento de forma aislada y
devuelve un `BulkOperationSummary` con un `BulkItemResult` por elemento. Un
fallo no cancela los demás elementos y la UI muestra el resultado individual.
Las acciones de retirar requieren confirmación y solo eliminan el registro
local; el PDF original nunca se borra ni se mueve.

Las acciones disponibles son:

- favoritos y etiquetas locales;
- añadir o quitar de una colección;
- archivar en la colección local reservada `Archivado`;
- regenerar portadas sin modificar el PDF;
- retirar de la biblioteca con confirmación.

Las operaciones reciben únicamente IDs y, cuando hace falta, un `targetId` de
colección o etiqueta. No se guardan rutas, consultas ni contenido del PDF en la
cola. La retirada limpia las miniaturas propias antes de eliminar el registro;
si esa limpieza falla, el resultado de ese elemento es fallido y el PDF queda
intacto.
