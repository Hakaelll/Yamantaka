# Persistencia local 0.12

Yamantaka usa SQLite v10. `documents` conserva la identidad estable por archivo;
un cambio de ruta exacto actualiza esa identidad sin desprender progreso,
anotaciones o metadatos.

La migración v10 es transaccional e idempotente y añade:

- `annotations.review_state`;
- consulta, agrupación, orden, densidad, columnas y Foco en `workspaces`;
- estado, bloqueo y siguiente acción en `workspace_tabs`;
- `document_relations`, `document_revision_groups` y miembros;
- `automation_rules` y `operation_journal`.

Las relaciones usan FK y no son propietarias de los PDF. Los grupos de revisión
solo guardan membresía y revisión actual. Las reglas solo serializan enums y
valores guiados; el journal agrega cantidades, estado, error seguro y undo.

La sesión atómica se guarda en `app_metadata` como JSON versionado. Una escritura
sucia precede a la actividad y el cierre normal la marca limpia. Las vistas de
Biblioteca y espacios usan paginación máxima de 80; el escaneo de reglas usa
keyset pagination hasta 100.000 y lotes de 80.

El backup local versión 8 incluye estas tablas y campos, pero nunca PDF, cachés,
texto indexado ni miniaturas. La restauración valida esquema, hashes y conflictos
en ensayo antes de reemplazar datos dentro de una transacción.
