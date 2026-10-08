# Memoria viva y scheduler Rust

Plan original, 2026-10-08; avance de implementación, 2026-10-09.

La primera fase ya está implementada: gobernador compartido, reservas atómicas,
leases RAII y buffer RGBA privado. PDFium escribe directamente en un buffer
reservado antes de FFI; `Arc` fuera de caché sigue contabilizado hasta el último
drop. Las pruebas nativas comprueban cierre, expulsión y bloqueo de admisión con
un consumidor retenido, además de orden de canales/alfa. Las regiones nativas ya
reservan únicamente una tesela y se contrastan con renders enteros rotados.

El presentador wgpu está conectado y pasó readback nativo DX12, liberación por
fences y rechazo atómico en Intel Iris Xe. El workspace integrado pasó 63 pruebas,
formato y Clippy. Las nuevas capturas conservan exactamente los píxeles del lector
revisado; condiciones y límites están en `docs/validation/rust-native-page-resources.md`.
La auditoría siguiente describe el punto
de partida histórico; no representa todos los defectos todavía actuales. Texto
retenido, tickets asíncronos, mailbox acotado, Glow y recuperación del dispositivo
siguen pendientes. No se declara cumplido el presupuesto global completo.

## Auditoría

`rust/pdf/src/cache.rs` descuenta entradas expulsadas aunque sobrevivan sus `Arc`; `worker.rs` limita cachés a 72/24/8 MiB, pero reserva bitmap antes de contabilizarlo. `as_rgba_bytes()` copia el buffer nativo y puede reservar otra conversión. `RenderedPage.rgba` permite ownership sin presupuesto. El límite preliminar de 192 MiB tampoco coincide con las categorías. La deduplicación alcanza únicamente solicitudes pendientes; la prioridad no evita un render nativo largo.

`runtime.rs` espera `PdfEngine` síncrono dentro del actor SQL; un PDF bloquea consultas, texto e importaciones. `Update` usa canal ilimitado y `Page` pierde el token del consumidor. `app.rs` convierte a `ColorImage`, carga una textura adicional y marca `requested` antes de admitir la solicitud: un rechazo puede impedir reintentar. `TextureHandle::drop` encola liberación; no demuestra finalización GPU.

Fuentes primarias locales contrastadas: pdfium-render 0.9.4 (`bitmap.rs`, `page.rs`, `render_config.rs`); egui-wgpu/epaint 0.36.2 (`renderer.rs`, `texture_handle.rs`); wgpu 30.0.1 (`queue.rs`).

## Contrato de memoria

Añadir `domain::memory`: `MemoryGovernor` compartido por PDF/presentación, `ByteLease` RAII no clonable y `TrackedBytes` privado. Una reserva comprueba **atómicamente** total configurable, categoría y sublímites CPU/GPU/transitorios; no suma los sublímites nuevamente al total. Preservar máximos iniciales Pages=72, Tiles=24, Text=8 MiB, ahora para todas sus representaciones vivas. Miniaturas pertenecen a Pages; reservar también atlas/UI/staging bajo Other y el total. No presupuestar por pestaña.

`Arc<RenderedPage>` contiene buffer y lease: caché, eventos y consumidores comparten una única contabilización, liberada con el último propietario. Texto comparte `Arc<TrackedText>` en lugar de clonar `String`. Expulsar LRU elimina propiedad; el gobernador vuelve a comprobar bytes efectivos, nunca supone que expulsar libera memoria. El controlador puede pedir que consumidores ocultos abandonen imágenes. Los visibles permanecen fijados. Disminuir configuración inicia presión; no revoca leases ni declara cumplimiento inmediato.

Reemplazar `RenderedPage.rgba` público por acceso `pixels()` inmutable; constructor restringido exige lease, stride y formato. `pdf::lib.rs` exporta configuración/admisión; `main.rs` crea un único gobernador.

`RenderPlan` valida con operaciones comprobadas dimensiones, stride, región, rotación canónica y capacidades reales; máximo por eje `min(device.max_texture_dimension_2d,16384)`. En teselas, dimensiones representan salida local y escala completa se expresa separadamente. Reservar antes de `try_reserve_exact`; reconciliar capacidad real antes de publicar. Fallos liberan reservas parciales.

Ruta preferida: buffer BGRA reservado, `PdfBitmap::from_bytes`, `render_into_bitmap_with_config`, destruir bitmap **antes** de acceder nuevamente al buffer, convertir canales in situ y publicar RGBA. Usar fondo opaco blanco; probar color/alfa. Nunca ejecutar `as_rgba_bytes` en esa ruta. Si un adaptador exige bitmap y RGBA separados, reservar ambos simultáneamente antes de llamar a PDFium. Pools conservan leases incluso inactivos y se purgan bajo presión.

PDFium/driver mantienen asignaciones internas no expuestas: este límite cubre recursos controlados, no garantiza RSS/VRAM físicos. Registrar separadamente estimaciones de documentos y métricas del proceso; presión externa reduce admisión. No afirmar memoria total exacta.

## Scheduler y runtime

Cambiar `PdfEngine` a tickets no bloqueantes. Un solo trabajador posee todos los handles PDFium. Estado de scheduler compartido bajo mutex corto: mapa pendiente/en ejecución, consumidores, prioridad efectiva y secuencia FIFO. Admisión limita conjuntamente jobs, consumidores y slots de respuesta a 128; reserva carril de control independiente para close/shutdown. Despertar mediante `try_send`; nunca esperar con mutex tomado.

Deduplicar pendientes **y en ejecución** por documento+epoch de apertura, página, región, escala física, rotación y generación. Cada consumidor conserva ID/token/viewport; cancelar uno no cancela otros. Si todos cancelan, retirar pendiente; durante llamada nativa finalizar y descartar. Epoch evita reutilizar resultados tras cierre/reapertura. Completar cada ticket una vez mediante publicación no bloqueante.

Orden: control, visible, próxima, prefetch, miniatura; texto tiene carril propio. Tras ocho unidades foreground pendientes ceder una a trabajo inferior; rotar documentos/consumidores dentro del nivel. Es garantía por unidades, no por milisegundos. Dividir inspección de 5.000 páginas en lotes y refinamiento en teselas; ninguna llamada se presume interrumpible. Generación nueva invalida trabajo anterior antes de reservar.

Separar actor de repositorio y coordinación PDF; importación continúa por estados tras cada ticket, nunca espera PDF dentro del actor SQL. Mantener máximo dos documentos nativos: cerrar víctima antes de abrir tercero; importación espera si ambos están fijados. Doce pestañas guardan contexto, no doce handles/bitmaps. Consultas paginadas conservan máximo 80 documentos.

Sustituir canal ilimitado por mailbox acotado con slots reservados por ticket; coalescer solo resultados reemplazables por viewport/generación. `Update::Page` incluye consumidor/token/epoch. Eventos terminales de escrituras no se pierden. Cierre cancela admisión, termina llamada nativa, completa tickets y hace join; respuestas nunca requieren drenaje UI para avanzar.

## Presentación y recuperación

Añadir `GpuPage` privado con textura, registro egui y lease; reservar textura y staging alineado antes de crear recursos. Preferir `RenderState`/`register_native_texture` para evitar `ColorImage`; staging explícito conserva lease hasta completar submission. Mantener imagen anterior mientras nuevo upload se confirma; ambas cuentan.

Retirar registro al siguiente inicio de `logic`, tras enviar el último frame que lo usa; `free_texture` elimina referencias del renderer. Un submit de barrera y `on_submitted_work_done` retienen textura/lease hasta finalización. Poll únicamente mientras haya retiros; callback solo libera/señala, nunca adquiere mutex del renderer. `Drop` delega retiro al propietario de presentación; cierre drena GPU antes de destruir renderer. Pérdida de dispositivo invalida epoch GPU y reconstruye previews; liberar leases tras destruir el dispositivo anterior.

UI marca solicitud admitida; `QueueFull`/`BudgetDenied` limpian pendiente. Reintentar por evento de capacidad o demora limitada, sin bucle de repintado. Reducir dimensiones según pico CPU+staging+GPU disponible; si no cabe reemplazo mantener preview y esperar. Refinar al estabilizar navegación. Glow requiere adaptador equivalente y prueba de recuperación.

## Fases y aceptación

1. Gobernador/buffers: retener Arc fuera de caché, overflow, fallo de asignación y rollback; comprobar contadores al último drop.
2. Tickets/mailbox: pausar UI, saturar cola, deduplicar durante render, cancelar consumidores independientemente, verificar fairness, SQL durante PDF y cierre sin deadlock.
3. GPU: retrasar fence artificialmente; confirmar pico doble imagen, staging y lease retenido; probar pérdida/recreación y cierre real.
4. Teselas: comparar con render completo rotado y bordes; validar formularios. Bindings locales deshabilitan form data con clip/transform: mantener preview completa hasta resolver overlay sin modificar originales. Secuencia 12 pestañas/2 calientes/80 filas y zoom rápido debe terminar con recursos liberados y resultados obsoletos descartados.

Firmas propuestas:

```rust
fn try_reserve(&self, costs: &[Cost]) -> Result<ReservationSet>;
fn try_render(&self, request: RenderRequest) -> Result<RenderTicket>;
fn cancel_consumer(&self, id: ConsumerId);
fn begin_close(&self, id: DocumentEpoch) -> CloseTicket;
fn try_upload(&mut self, page: Arc<RenderedPage>) -> Result<GpuPage>;
fn retire(&mut self, page: GpuPage);
fn drain_updates(&mut self, max_events: usize);
```
