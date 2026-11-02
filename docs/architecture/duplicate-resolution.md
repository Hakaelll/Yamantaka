# Resolución local de duplicados

La resolución opera únicamente sobre registros locales. No abre un PDF para
escribirlo, no mueve archivos y no elimina el archivo original.

## Clasificación

- `exactFingerprint`: se puede proponer la fusión, pero sigue requiriendo una
  acción explícita y un prebackup.
- `probableMetadata`: se muestra como propuesta y siempre requiere confirmar.
- `samePath`: se trata como conflicto de registro, no como permiso para
  fusionar automáticamente.
- `none`: no se ofrece la operación.

## Precedencia

El registro que conserva el usuario mantiene su ruta, huella y origen de
metadatos. Un campo local no vacío del registro conservado tiene precedencia;
un campo vacío se completa con el valor entrante. Los conflictos se devuelven
para que la interfaz los explique. El progreso máximo, favoritos, marcadores,
historial, etiquetas, colecciones, preferencias y anotaciones se conservan sin
duplicar claves. Una portada manual tiene precedencia sobre una automática.

La búsqueda se invalida para ambos registros y queda lista para reconstrucción;
así nunca se publica un resultado que siga apuntando al registro retirado.

La transacción empieza solo después de que el prebackup haya terminado. Si la
transacción falla, se hace rollback; el backup permite recuperar el estado
anterior incluso si el fallo ocurre después del commit local.

## Native Rust checkpoint

The [Rust consolidation store](../validation/rust-duplicate-consolidation-storage-2026-11-02.md) now uses freshly verified full SHA-256 proofs, a dependent-record revision, a pre-merge v9 backup and one SQL transaction. It preserves native preferences, manual covers, annotation provenance and incomplete historical sessions. Native discovery remains read-only until confirmation, live-editor/session retirement, continuity reconciliation and undo are connected. The historical Flutter partial-fingerprint classifier above does not authorize native exact-duplicate merges.
