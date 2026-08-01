# Trazabilidad de `NEXT_TODOs.md`

Este documento separa implementación local de evidencia de gate. No marca una
tarea como terminada solo porque exista un contrato o un test puro: las
casillas de `NEXT_TODOs.md` permanecen abiertas hasta que la validación que allí
se exige produzca un artefacto reproducible.

## Código y herramientas preparados

| Tarea | Implementación disponible | Evidencia pendiente |
| --- | --- | --- |
| YAM-100 | `tool/verify_windows.ps1`, self-check y reporte acumulado | Checkout limpio con Pub y toolchain Windows disponibles |
| YAM-101 | `tool/run_visual_matrix.ps1` y manifiesto de escenarios | Capturas en cada resolución y DPI físico |
| YAM-102 | `tool/verify_visual_regression.ps1` y `docs/verification/visual-manifest.json` | Baselines/candidatos Release revisados |
| YAM-103 | Generador de corpus sintético con hashes, expectativas y límites | Ejecutar PDFium, repetir apertura/cierre y medir recursos |
| YAM-104 | `LifecycleTracker`, leases idempotentes, integración en motor/jobs y `docs/architecture/ownership.md` | Soak nativo de 100 ciclos y handles |
| YAM-105 | `JobCoordinator` con prioridad, deduplicación, generaciones, cancelación, pausa de background y categorías para portadas/importación/índice | Integrar backup/recovery/render visible y validar fairness |
| YAM-106 | ADR y parche local conservado | Bloqueada hasta una release upstream verificable |
| YAM-110–114 | Planificador de teselas por regiones ya consumible por el motor, `MemoryGovernor`, prefetch, apertura progresiva y anclas de viewport | Activación completa en visor real, A/B y matriz de PDF extremos |
| YAM-115–116 | Modelo tipado de selección/geometría y persistencia existente fuera del PDF | Selección nativa PDFium y validación visual; actualmente no expuesta |
| YAM-117–119 | Búsqueda cancelable con descarte de resultados obsoletos, navegación canónica y panel existente | Widgets, Narrador y perfil de scroll |
| YAM-120–124 | Sesión de dos paneles, sincronización sin bucles, registro de ventanas, política de impresión y ADR | Dos visores/ventanas y API de impresión Windows reales |
| YAM-130–139 | Consulta paginada, benchmark de 5.000 documentos, cola de importación persistente, watcher de raíces autorizadas, resolución transaccional de duplicados, editor de metadatos, thumbnails/index/jobs, dominios de organización/workspace y acciones masivas conectadas a la biblioteca (`docs/architecture/bulk-library-operations.md`) | Suite SQLite/GUI completa, reanudación nativa y evidencia de 5.000 documentos |
| YAM-140–149 | Tokens semánticos, breakpoints, alto contraste, movimiento reducido, e-ink, `docs/architecture/visual-system.md`, catálogo local y layouts existentes | Auditoría visual, strings completos y recorrido manual con Narrador |
| YAM-150–156 | Timeline de arranque, governor, coordinator, índices SQLite, benchmark, soak y políticas de actividad | Ejecución Release/Profile en dos equipos y umbrales medidos |
| YAM-157–161 | Backup v5 con manifest/hash, archivo parcial y backup previo recuperable, recovery policy, centro de almacenamiento y diagnóstico redactado | Round-trip de fallo/rollback, smoke de cero red y exportación nativa |
| YAM-170–177 | Pipeline de bundle/portable, protocolo de instancia única, geometría, catálogo es/en, checklist y soak | Certificado, VM limpia, asociación/desinstalación, dos equipos y periodo real de estabilidad |

## Verificación ejecutada en esta iteración

- `dart format` del código, tests y herramientas: correcto con el SDK local.
- Análisis Dart de todos los módulos puros nuevos: correcto.
- `dart --enable-asserts run tool/self_check.dart`: correcto.
- El benchmark de repositorio y el generador de corpus son ejecutables con el
  Dart local; el gate principal exige además una caché de Pub completa antes
  de declarar aprobada la suite Flutter.
- Parseo de los scripts PowerShell: correcto.
- `git diff --check`: sin errores de whitespace.
- `tool/verify_windows.ps1` ahora registra cada fase como `passed`, `failed` o
  `skipped`, normaliza el entorno `PATH` de Codex, acepta fases aisladas y
  termina procesos externos que superan el timeout configurado.
- El gate aislado ejecutado en `build/verification/gate-final-bulk` pasó higiene,
  self-check y formato; análisis, Pub, tests, corpus, benchmark, build,
  smoke, visual y DPI quedaron explícitamente `skipped` y no se consideran
  aprobados.
- El generador offline produjo nueve casos sintéticos en
  `build/verification/corpus-final`; el manifiesto quedó
  marcado sin datos personales y el SHA-256 de `normal.pdf` coincidió con
  `Get-FileHash`.
- El smoke de red registra TCP y UDP; el catálogo de `FailureKind` tiene una
  presentación segura exhaustiva y la observación de carpetas solo se activa
  mediante la casilla explícita de importación.
- La selección masiva tiene contrato, prueba de resumen parcial, widget de
  selección accesible y servicio por elemento; retirar limpia miniaturas sin
  tocar los PDF y archivar usa una colección local explícita.
- `tool/package_windows.ps1` valida las raíces de Pub antes del build y conserva
  un `release-failure.json` cuando falla; el primer intento de release quedó
  bloqueado por el paquete `test` no resoluble (Flutter exit 69) y la
  comprobación actual confirma raíces Pub inaccesibles, dejando
  `build/release-current/release-failure.json` sin publicar el bundle de
  `build/windows` histórico.

## Bloqueo reproducible

La caché local no contiene los paquetes declarados. `flutter pub get --offline`
no puede resolverlos y el intento de `flutter analyze --no-pub` intenta buscar
`test` en `pub.dev`. No se descargaron dependencias porque el flujo exige
consentimiento explícito para red. Por ello no se puede afirmar todavía que la
suite Flutter, PDFium, el build Windows, las capturas, el benchmark SQLite o el
soak nativo estén aprobados.

La retirada de `third_party/pdfrx` continúa bloqueada por YAM-106 según su ADR;
no se elimina el parche sin una release upstream equivalente y una comparación
A/B aprobada.

La caché de Pub tampoco permite producir en este entorno una release Windows
actual: `flutter pub get --offline` no resuelve los paquetes declarados y el
benchmark SQLite encuentra `sqlite3-3.5.0` inaccesible. El intento de resolución
con red fue rechazado; por tanto cualquier paquete generado a partir de un
bundle anterior se consideraría histórico, no evidencia de este checkout.
