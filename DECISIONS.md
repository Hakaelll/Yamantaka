# Registro de decisiones

## ADR-001 — Motor PDF inicial

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: se requieren Windows, Android e iOS, renderizado rápido, texto,
  búsqueda, índice, enlaces y contraseña sin construir un motor.
- Opciones: `pdfrx`, `pdfx`, `pdf_render`, SDK comerciales.
- Decisión: `pdfrx` 2.4.x/PDFium detrás de `PdfDocumentEngine`.
- Motivo: cubre todas las plataformas objetivo iniciales, búsqueda, selección,
  outline, enlaces, contraseña, vista enfrentada y renderizado multiproceso;
  está mantenido y usa licencia MIT.
- Consecuencias: binario nativo mayor que un visor básico y modo desarrollador
  obligatorio al compilar en Windows.
- Riesgos: PDFium puede contener vulnerabilidades; se fijan versiones y se
  revisan actualizaciones. Las API del paquete no atraviesan el dominio.

Fuentes: [pdfrx](https://pub.dev/packages/pdfrx) y
[licencia](https://pub.dev/packages/pdfrx/license).

## ADR-002 — Persistencia

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: datos privados y estructurados en un archivo local.
- Opciones: `sqlite3`, Drift, Isar, archivos JSON.
- Decisión: `sqlite3` 3.x con repositorios escritos a mano y migraciones SQL.
- Motivo: pequeño, directo, multiplataforma y empaqueta SQLite mediante native
  assets sin generadores.
- Consecuencias: las consultas se mantienen manualmente y deben probarse.
- Riesgos: la API es síncrona; el arranque y operaciones grandes se aíslan y
  nunca se invocan durante `build`.

Fuente: [sqlite3](https://pub.dev/packages/sqlite3).

## ADR-003 — Estado y dependencias

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: evitar un estado global y dependencias innecesarias.
- Opciones: Riverpod, Bloc, Provider, controladores `ChangeNotifier`.
- Decisión: controladores pequeños `ChangeNotifier`, composición explícita en
  `AppServices`.
- Motivo: API estable de Flutter, poco coste y separación suficiente para el
  primer hito.
- Consecuencias: la inyección es manual; si el grafo crece se reevaluará.
- Riesgos: listeners olvidados; cada pantalla es responsable de su ciclo de vida.

## ADR-004 — Selección, drop y ventana

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: selector nativo multiplataforma, drop de escritorio y F11 real.
- Opciones: implementar canales propios o paquetes mantenidos.
- Decisión: `file_selector` (publicado por flutter.dev), `desktop_drop` y
  `window_manager`, siempre detrás de servicios propios.
- Motivo: reducen código nativo y preservan portabilidad.
- Consecuencias: tres plugins pequeños; drop no se ofrece en iOS.
- Riesgos: las plataformas difieren en filtros y permisos.

Fuentes: [file_selector](https://pub.dev/packages/file_selector),
[desktop_drop](https://pub.dev/packages/desktop_drop) y
[window_manager](https://pub.dev/packages/window_manager).

## ADR-005 — Navegación

- Fecha: 2026-07-28
- Estado: aceptada
- Decisión: navegación declarativa mínima con estado de shell y `Navigator`.
- Motivo: solo existen biblioteca, lector y ajustes; un router externo no
  aporta valor todavía.
- Consecuencia: enlaces profundos se reevaluarán antes del port móvil.

## ADR-006 — Caché y miniaturas

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: un límite por número de páginas no controla documentos con imágenes
  gigantes y una caché ilimitada degrada sesiones largas.
- Opciones: caché del paquete, LRU por entradas, LRU por bytes o teselas.
- Decisión: visor virtualizado de `pdfrx`, LRU propia de 96 MiB por bytes
  estimados y portadas de baja resolución con máximo global configurable.
- Motivo: límites simples y medibles sin renderizar todo el documento.
- Consecuencias: hay dos presupuestos de memoria independientes; limpiar caché
  vacía ambos recursos controlados por Yamantaka y los archivos de portada.
- Riesgos: una única página extrema puede superar temporalmente el presupuesto;
  teselas se decidirán con mediciones, no antes.

## ADR-007 — Enlaces externos

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: los PDF son entradas no confiables y no deben provocar salidas
  silenciosas ni ejecutar esquemas arbitrarios.
- Opciones: deshabilitar todos los enlaces, delegar al comportamiento del motor
  o interceptar y confirmar.
- Decisión: navegar destinos internos; mostrar aviso para `http`, `https` y
  `mailto`; bloquear otros esquemas; abrir fuera de la aplicación solo tras
  confirmación mediante `url_launcher`.
- Motivo: mantiene utilidad sin convertir el lector en navegador ni ejecutar
  contenido incrustado.
- Consecuencias: `url_launcher` es una dependencia directa pequeña; la única red
  posible parte de una acción explícita en una aplicación externa.
- Riesgos: la URL puede apuntar a contenido malicioso, por lo que se muestra
  completa antes de confirmar.

## ADR-008 — Inventario mínimo de dependencias

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: el producto debe seguir siendo ligero, auditable y local-first.
- Decisión: mantener solo dependencias directas con una responsabilidad concreta
  y conservar sus versiones resueltas en `pubspec.lock`.

| Dependencia | Responsabilidad y motivo |
| --- | --- |
| `crypto` | SHA-256 parcial para identificar cambios o movimientos sin copiar el PDF. |
| `desktop_drop` | Entrada por arrastrar y soltar en escritorio. |
| `file_selector` | Selector de archivo nativo mantenido por Flutter. |
| `path` | Normalización y composición de rutas independiente de plataforma. |
| `path_provider` | Directorios privados de datos y caché de cada sistema. |
| `pdfrx` | Único backend PDF; 2.4.7 está aislado y parcheado localmente según ADR-009. |
| `shared_preferences` | Ajustes globales pequeños y asíncronos; no datos relacionales. |
| `sqlite3` | Biblioteca, progreso, preferencias por documento, sesiones y marcadores. |
| `url_launcher` | Entrega enlaces externos permitidos al sistema tras confirmación. |
| `window_manager` | Pantalla completa y geometría de ventana en escritorio. |

`integration_test` y `flutter_lints` solo participan en desarrollo. No existe
cliente HTTP, SDK de analítica, publicidad, cuenta, nube ni sincronización. Las
licencias transitivas se deben revisar desde el lockfile y los avisos generados
antes de cada distribución.

## ADR-009 — Parche local temporal de pdfrx

- Fecha: 2026-07-28
- Estado: aceptada
- Contexto: en Flutter 3.44/Skia para Windows, el helper asíncrono usado por
  pdfrx 2.4.7 libera el codec antes de que `getNextFrame()` entregue la imagen.
  PDFium genera un bitmap correcto, pero el visor queda en blanco.
- Opciones: afirmar soporte con pantalla en blanco, reescribir el visor,
  modificar la caché global de pub o aislar un parche reproducible.
- Decisión: incluir 599 KiB de fuente de pdfrx 2.4.7 bajo su licencia MIT en
  `third_party/pdfrx`; mantener el codec hasta completar el frame y reportar los
  errores de preview antes silenciados.
- Motivo: preserva el visor probado, todas sus plataformas objetivo y una build
  reproducible, sin depender de una modificación fuera del repositorio.
- Consecuencias: `pubspec.yaml` usa una dependencia `path`; el parche y su
  licencia deben acompañar el producto.
- Riesgos: divergencia de upstream. YAM-014 obliga a retirar el fork cuando una
  versión hospedada supere análisis, integración, captura y Release.

El detalle exacto está en `third_party/pdfrx/PATCHES.md`.
