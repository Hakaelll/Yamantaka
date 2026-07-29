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

## ADR-010 — Tokens visuales y política responsive

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: la primera interfaz mezclaba valores directos, tarjetas genéricas
  y breakpoints solapados de 900/999 px; esto impedía razonar sobre contraste,
  densidad y legibilidad de páginas enfrentadas.
- Opciones: incorporar un design-system externo, mantener valores por widget o
  centralizar un sistema pequeño sobre las primitivas de Material.
- Decisión: usar `ThemeExtension<AppColorTokens>` y clases inmutables para
  espacios, tamaños, radios y movimiento; definir clases no solapadas `<680`,
  `680–1099` y `>=1100`; permitir vista enfrentada solo si quedan al menos
  340 px legibles por página tras considerar panel, márgenes, altura y aspecto.
- Motivo: mantiene el producto ligero, hace claro/oscuro/e-ink coherentes y
  permite probar la política sin acoplarla a un widget.
- Consecuencias: los widgets visuales consumen tokens semánticos y las nuevas
  superficies deben superar pruebas en las fronteras exactas.
- Riesgos: los breakpoints simulados no sustituyen escalado nativo, táctil ni
  hardware e-ink; esas validaciones permanecen en YAM-022 a YAM-024.

## ADR-011 — Interacción de biblioteca sin índice adicional

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: la biblioteca necesita filtro, orden y densidad configurables sin
  aumentar consumo, dependencias ni superficie de datos privados.
- Opciones: incorporar un buscador indexado, consultar SQLite por cada tecla,
  cargar un paquete de componentes o transformar la lista acotada en memoria.
- Decisión: filtrar y ordenar los 40 recientes ya cargados; mantener la consulta
  efímera; modelar densidad cómoda/compacta como preferencia tipada y
  `ThemeExtension`; decodificar portadas al tamaño físico de su caja según DPR.
- Motivo: el límite de recientes hace que una pasada local sea determinista y
  barata; los tokens reutilizan Material y la decodificación evita bitmaps de
  interfaz sobredimensionados.
- Consecuencias: `Ctrl+F` pertenece a la biblioteca fuera del lector; el cambio
  de densidad reconstruye el tema, pero no el contenido PDF ni su zoom; no se
  añade ninguna dependencia ni tabla.
- Riesgos: si el límite de biblioteca crece varios órdenes de magnitud habrá que
  medir antes de conservar este enfoque; DPI simulado no reemplaza el perfil
  nativo pendiente.

## ADR-012 — Estado de lectura, panel y reconstrucciones

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: YAM-030/032 necesitan filtros y resize sin añadir tablas, consultas
  por tecla, escrituras por píxel ni renders PDF redundantes.
- Decisión: derivar el estado desde `reading_progress` en una pasada; persistir
  solo el enum de estado, nunca el texto; mantener panel 280–480 px como estado
  de vista y guardar con debounce de 240 ms; cachear el widget del adaptador por
  configuración lógica de render.
- Motivo: el conjunto está acotado a 40 recientes y Flutter ya ofrece las
  primitivas de selector, foco, resize y composición.
- Consecuencias: no cambia el esquema SQLite ni `pubspec`; solo se monta una
  sección lateral y el visor no se reconstruye por chrome/panel.
- Riesgos: una biblioteca futura mucho mayor o multi-ventana requerirá volver a
  medir y separar preferencias por ventana.

## ADR-013 — Atajos mediante opciones predefinidas

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: capturar combinaciones arbitrarias multiplica conflictos de layout,
  teclado/plataforma y accesibilidad.
- Decisión: permitir únicamente comandos importantes y activadores
  predefinidos; validar conflictos, descartar JSON inválido, reservar Escape,
  Inicio, Fin y navegación básica; resolver tooltips/menús/paleta desde el mismo
  registro.
- Motivo: ofrece personalización útil y reversible sin un editor global de
  teclas ni otra dependencia.
- Consecuencias: el shell consume el override de Abrir y el lector consume los
  demás; restaurar defaults equivale a guardar `{}`.
- Riesgos: no se cubren todos los layouts físicos; nuevas opciones exigen
  prueba de conflicto y etiqueta localizada.

## ADR-014 — Copia local JSON validada

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: una copia directa de SQLite es compacta, pero dificulta validar
  compatibilidad, previsualizar y restaurar ajustes de otro almacén.
- Opciones: copiar DB, ZIP propietario, JSON versionado o paquete externo.
- Decisión: JSON explícito `yamantaka-local-backup` versión 1, tablas/columnas
  allowlist, escalares validados, recuentos de vista previa, prebackup automático
  e importación transaccional.
- Motivo: es legible, probado y no necesita dependencia ni esquema nuevo.
- Consecuencias: incluye rutas/metadatos sensibles, pero declara
  `containsPdfFiles: false`; nunca incluye el contenido PDF.
- Riesgos: archivos grandes de historial pueden crecer; límites y versiones
  futuras deben validarse antes de mutar.

## ADR-015 — Historial mínimo y recuperación opt-in

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: recuperar contexto e historial aporta continuidad, pero no debe
  convertirse en analítica ni bucle de apertura.
- Decisión: reutilizar `reading_sessions`, limitar 50–1000, permitir desactivar
  y limpiar; persistir última ruta/contexto de UI solo localmente; intentar
  restauración una vez y limpiar la ruta tras ausencia/fallo.
- Motivo: aprovecha datos ya existentes y evita tabla, eventos personales y
  procesos en segundo plano.
- Consecuencias: “Iniciar siempre en biblioteca” tiene prioridad; una ruta
  explícita de línea de comandos gana a cualquier restauración.
- Riesgos: un cierre entre debounces recupera el último estado estable, no el
  evento más reciente.

## ADR-016 — Identidad segura para relocalización

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: “Localizar” no debe asociar silenciosamente otro PDF al registro
  ausente.
- Decisión: inspeccionar el archivo elegido y exigir la huella parcial ya
  almacenada; si no coincide, no se muta la biblioteca. Un archivo disponible
  se marca visualmente como cambiado cuando tamaño o modificación difieren.
- Motivo: mantiene continuidad para un movimiento real y evita mezclar
  progreso/marcadores de documentos distintos.
- Consecuencias: una versión editada debe abrirse normalmente y se registrará
  como actualización por ruta o documento distinto según identidad.
- Riesgos: una huella parcial no es prueba criptográfica del contenido
  completo, pero es suficiente para la relocalización local y acotada actual.

## ADR-017 — Diagnóstico Profile sin datos privados

- Fecha: 2026-07-29
- Estado: aceptada
- Contexto: YAM-011/034 requiere caché, frames y memoria medibles, mientras
  privacidad prohíbe telemetría y rutas/títulos en logs.
- Decisión: generar fixtures sin copyright; devolver contadores enteros del
  motor solo en memoria; escribir el informe desde el driver de prueba bajo
  `build/`; redaccionar la traza de `PdfDocumentRef` del fork; hacer que
  `AppLogger` retenga solo `runtimeType` y nunca el texto/stack de excepciones.
- Motivo: hace el perfil reproducible y auditable sin instrumentación remota ni
  datos de usuario.
- Consecuencias: el JSON declara `contains_paths_or_titles: false`; las cifras
  siempre incluyen modo, DPI, tamaño y número de pasadas.
- Riesgos: un único equipo y proceso no representa otras GPU ni una prueba de
  fugas sostenida; YAM-041/048 conservan esa ampliación.
