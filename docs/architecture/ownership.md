# Ownership y ciclo de vida

Este inventario es el contrato de cierre de Yamantaka. Los identificadores de
documento, rutas, títulos, consultas y notas no aparecen en los diagnósticos;
solo se usan como claves efímeras dentro del proceso.

| Recurso | Propietario | Transferencia | Cierre |
| --- | --- | --- | --- |
| `AppDatabase` | `AppServices` | Ninguna | `AppServices.dispose` tras marcar cierre limpio |
| PDF nativo abierto | `PdfrxDocumentEngine` | `ReaderController` recibe un `OpenedPdfDocument` lógico | `closeDocument` o `dispose` del motor |
| imagen de render | `PdfrxDocumentEngine` y su LRU | La LRU retiene solo entradas cacheadas | `ui.Image.dispose`, incluido en eviction/clear |
| portada/miniatura en disco | `ThumbnailService` | `DocumentRepository` recibe solo metadata | `clear`, reemplazo atómico y `trimToLimit` |
| trabajo asíncrono | `JobCoordinator` | consumidores reciben `JobHandle` | cancelación por consumidor/generación y `dispose` |
| cuota lógica de memoria | `MemoryGovernor` | el llamador conserva `MemoryReservation` | `release` idempotente o eviction |
| listeners/timers de reader | `ReaderController` | no se exportan fuera del controlador | `dispose`, `closeDocument` y cancelación de generación |
| ventana | runner nativo + `window_manager` | solo callbacks de la ventana activa | `WindowListener` se desregistra en `dispose` |

Reglas obligatorias:

1. Una operación que cambia de documento captura una generación y descarta todo
   resultado tardío.
2. Un consumidor puede cancelar su interés sin cancelar a otros consumidores de
   un trabajo deduplicado; el trabajo se cancela cuando no queda ninguno.
3. Ningún callback de render toca un controlador después de `dispose`.
4. Los PDF originales nunca son propiedad de backup, cache, importación o
   desinstalación.
5. El contador de lifecycle es evidencia de cierre, no una aproximación de la
   memoria física del proceso.

La evidencia automática se obtiene con el campo `lifecycle` de diagnósticos,
`jobs`, `memory` y los tests de `test/lifecycle_tracker_test.dart`.
