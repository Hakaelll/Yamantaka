# Pruebas

## Estado verificado — 2026-07-30

- `dart format`: código Dart formateado.
- `flutter analyze`: aprobado sin incidencias.
- `flutter test --reporter compact`: 113 pruebas aprobadas.
- Integración Windows: lector, biblioteca y perfil aprobados individualmente
  sobre PDFium real; el agregado encontró la incidencia de arranque del runner
  documentada más abajo.
- `flutter build windows --release`: aprobado; genera
  `build\windows\x64\runner\Release\yamantaka.exe`.
- Smoke Release: ventana visible y receptiva, tema claro restaurado y cierre
  limpio mediante la ventana; el ejecutable final abrió el fixture por
  argumento y mantuvo cero conexiones TCP.
- Inspección visual nativa: Biblioteca y Ajustes en composición compacta y
  ancha, tema claro/oscuro y escala de Windows 125 %.
- Interfaz ligera: filtro/orden de biblioteca capturados en Release a 1200×800
  y 800×760 físicos (mínimo nativo); bundle verificado sin dependencias nuevas.
- Perfil Windows: cinco pasadas Profile con 40 documentos, PDF de 100 páginas,
  fixture vectorial pesado, caché/deduplicación, RSS y frames.
- Refinamiento del lector: dominio/widgets cubren portada, pliegos pares e
  impares, lomo, alineación, centrado/paneo, ajuste, selector y teclado completo;
  la integración Windows aislada vuelve a aprobar y valida por píxeles dos
  páginas próximas y el conjunto centrado.

No hay pruebas omitidas marcadas como aprobadas.

## Comandos

```powershell
dart format --output=none --set-exit-if-changed `
  lib test integration_test tool test_driver
flutter analyze
flutter test --reporter compact
flutter test integration_test/reader_integration_test.dart -d windows
flutter build windows --release
```

Las capturas del lector, búsqueda y paleta se regeneran con:

```powershell
flutter test integration_test -d windows `
  --dart-define=YAMANTAKA_CAPTURE_PATH=docs/screenshots/windows-reader-document.png
```

La biblioteca por estado se captura con:

```powershell
flutter test integration_test/library_visual_test.dart -d windows `
  --dart-define=YAMANTAKA_LIBRARY_CAPTURE_PATH=docs/screenshots/windows-library-reading-states.png
```

El perfil reproducible se ejecuta con:

```powershell
dart run tool/create_performance_fixtures.dart build/performance/fixtures
flutter drive --driver=test_driver/performance_driver.dart `
  --target=integration_test/performance_profile_test.dart `
  -d windows --profile
```

Las capturas DPI-aware de una ventana Release se generan con
`tool/capture_windows_window.ps1`.

## Cobertura automatizada

- Dominio: progreso, límites, portada, pares enfrentados, página impar, RTL,
  indicadores de pliego y normalización de viewport.
- Responsive: fronteras exactas 679/680/1099/1100 px, padding por clase,
  composición del lector a 640/679/680/1099/1100/1440 px y filas de biblioteca
  a 420/679/680/1099/1100 px.
- Legibilidad: la doble página se desactiva por ancho, panel o altura; exige
  340 px útiles por página.
- Temas: tokens semánticos claro/oscuro, seis canvas y fallback e-ink blanco,
  sin sombra, con una página y movimiento reducido.
- Plataforma: normalización de rutas y argumento `.pdf`.
- Errores: extensión no compatible, archivo ausente, PDF vacío/corrupto,
  contraseña requerida/errónea y error recuperable del lector.
- Persistencia: esquema SQLite estricto, migración, recientes, preferencias,
  marcadores, sesiones, huella/relocalización, portada, densidad y ajustes
  globales; estado de lectura, panel/chrome, recuperación y atajos.
- Copias locales: formato/versionado/tipos, previsualización, round-trip,
  prebackup automático y rechazo de importación sin mutación.
- Historial: orden y límite, IDs monotónicos, borrado individual/todos,
  activación y retención configurables.
- Caché: LRU por presupuesto de bytes, recorte de portadas y decodificación de
  miniaturas acotada por caja física/DPR.
- Búsqueda: resultados, navegación, documento sin texto y cancelación.
- Comandos: registro único, atajos configurables/conflictos/reset, filtrado,
  recientes, vacío, selección con flechas/Enter y scroll inmediato con
  movimiento reducido.
- Widgets: biblioteca vacía/ausente, lector, progreso, panel, seis temas,
  márgenes, selector Documento/Libro, flechas, `PageUp`/`PageDown`,
  `Home`/`End`, espacio/Mayús+espacio, exclusión de campos de texto, `Ctrl+F`,
  filtro/orden, densidad, Escape, perfil e-ink y ausencia de overflow.
- Motor real: PDF legal generado durante el test, apertura PDFium, bitmap
  visible, extracción, búsqueda y AES-128 con contraseña.
- Integración Windows: lector real completo, pintura de página, chrome,
  búsqueda desde el panel, paleta, zoom, avance/retroceso, marcador, cierre,
  reapertura y restauración.
- Rendimiento: 40 PDF, 100 páginas, carga vectorial, apertura/render repetidos,
  panel/resize/zoom, caché/hit/deduplicación, cancelación, frames y RSS sin
  datos privados.

## Matriz visual ejecutada

| Superficie | Entorno/tamaño | Tema | Resultado y evidencia |
| --- | --- | --- | --- |
| Biblioteca | Release, 1200×800 físicos, Windows 125 % | Claro | Aprobada; `windows-library-wide.png` |
| Biblioteca | Release, ancho compacto limitado por mínimo nativo | Claro | Aprobada; `windows-library-compact.png` |
| Biblioteca con filtro | Release, 1200×800 físicos, Windows 125 % | Claro | Aprobada; `windows-library-filter-wide.png` |
| Biblioteca con filtro | Release, 800×760 físicos, mínimo nativo, Windows 125 % | Claro | Aprobada; `windows-library-filter-compact.png` |
| Biblioteca por estado | Integración Windows, 1200×800 lógicos/DPR 1 | Claro | Aprobada; `windows-library-reading-states.png` |
| Biblioteca “En curso” | Integración Windows, 1200×800 lógicos/DPR 1 | Claro | Aprobada; `windows-library-reading-states-in-progress.png` |
| Biblioteca por estado compacta | Integración Windows, 640×760 lógicos/DPR 1 | Claro | Aprobada; `windows-library-reading-states-compact.png` |
| Biblioteca por estado compacta | Integración Windows, 640×760 lógicos/DPR 1 | Oscuro | Aprobada; `windows-library-reading-states-compact-dark.png` |
| Ajustes | Release, 1200×800 físicos, Windows 125 % | Claro | Navegación lateral aprobada |
| Ajustes | Release, ancho compacto limitado por mínimo nativo | Claro | Secciones apiladas aprobadas |
| Ajustes | Release, 1200×800 físicos, Windows 125 % | Oscuro | Contraste y selección aprobados |
| Lector | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | PDFium, chrome y portada individual aprobados |
| Vista enfrentada | Integración Windows, 1200×800 lógicos/DPR 1 | Marfil | Pareja completa, legible y centrada |
| Modo libro | Integración Windows, 1200×800 lógicos/DPR 1 | Marfil | Portada única centrada, controles ocultos y PDF dominante |
| Buscar | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Resultado real, panel y navegación aprobados |
| Paleta | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Overlay, selección y atajos visibles aprobados |
| Lector contextual | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Barra jerárquica y PDF dominante; `windows-reader-contextual.png` |
| Panel redimensionado | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Panel acoplado sin tapar chrome/PDF; `windows-reader-contextual-panel-resized.png` |
| Información documental | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Diálogo estable y ruta de fixture anonimizada; `windows-reader-contextual-info.png` |
| Lector rediseñado | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | PDF centrado y barra de pestañas retirada; `windows-reader-redesign.png` |
| Menú del lector | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Menú compacto con iconos, grupos y atajos; `windows-reader-redesign-menu.png` |
| Miniaturas del lector | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Texto legible, escala ligada al panel/DPR y lienzo restante centrado; `windows-reader-redesign-thumbnails.png` |
| Zoom mínimo | Integración Windows, 1200×800 lógicos/DPR 1 | Claro/marfil | Página completa y centro horizontal verificado por píxeles; `windows-reader-redesign-minimum-zoom.png` |
| Pantalla completa | Integración Windows, 1200×800 lógicos/DPR 1 | Marfil | Solo lienzo y documento, sin chrome/panel/progreso; `windows-reader-redesign-fullscreen.png` |
| Pliego de libro | Integración Windows, 1200×800 lógicos/DPR 1 | Marfil | Páginas 2–3 próximas, borde superior alineado, lomo acotado y conjunto centrado comprobado por píxeles; `windows-reader-redesign-book-spread.png` |

Las capturas viven en `docs/screenshots`. Las variantes con sufijos
`-search`, `-facing`, `-palette` y `-book` registran los estados del lector.
La variante compacta nativa queda
limitada por el mínimo de ventana de 640 px lógicos; las fronteras menores se
validan además mediante widgets.

## Matriz manual de accesibilidad nativa

`Resultado observado = Pendiente` significa que el caso no se ha ejecutado con
tecnología asistiva real y mantiene YAM-033 en `REVIEW`.

| Pantalla | Acción | Resultado esperado | Resultado observado | Build | DPI | Tema | Tecnología |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Biblioteca | Tab/Shift+Tab por abrir, ajustes, filtro, estado, orden, filas e información | Orden lógico, foco visible, sin trampa | Pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador + teclado |
| Biblioteca | Ctrl+F, escribir, limpiar y Escape | Campo anunciado; recuento una vez; colección recuperada | Automatización aprobada; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador + teclado |
| Ficha documental | Abrir, recorrer, cancelar y confirmar | Foco inicial/restaurado; riesgo y “PDF no se borra” anunciados | Automatización parcial; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador |
| Lector | Recorrer barras; Enter/Space; ocultar/revelar chrome | Página/zoom/modo/selección anunciados; foco visible | Widgets aprobados; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro + seis canvas | Narrador + teclado |
| Panel | Abrir secciones, redimensionar con flechas, Escape | Sección única; ancho/valor anunciado; foco vuelve al origen | Widgets aprobados; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador + teclado |
| Paleta | Abrir, buscar, flechas, Enter, Escape | Foco en búsqueda; selección única; cierre devuelve foco | Widgets/integración aprobados; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador + teclado |
| Ajustes | Recorrer secciones, dropdowns, historial y copias | Etiquetas/estado completos; diálogos sin trampa | Contraste/semántica automatizados; Narrador pendiente | Release | 100/125/150/200 % | Claro/oscuro | Narrador + teclado |

Antes de cada fila se registra ejecutable, commit, escala real de Windows,
resolución, tema y documento fixture. El recorrido se repite tras cada hallazgo.

## Matriz pendiente general

1. Selector nativo, drop y apertura con rutas largas/Unicode.
2. PDF escaneado, grande y variantes patológicas adicionales.
3. Continuo, único y enfrentado en documentos largos; LTR/RTL, rotación, F11,
   ocultación y clic lateral en sesión manual sostenida.
4. Búsqueda larga/cancelada y documento real sin texto.
5. Mover un PDF, observar ausente y completar relocalización manual.
6. Tabulación completa, restauración de foco y Narrador según la tabla anterior.
7. Escala nativa Windows 100 %, 150 % y 200 %; 125 % ya ejecutado.
8. Los seis canvas en capturas nativas comparables; el contraste formal
   automatizable ya está aprobado.
9. Limpieza manual de caché/historial/datos conservando el PDF original.
10. Cierre durante búsqueda/render, procesos repetidos y perfil de 1000 páginas.
11. Orientaciones y entrada táctil en Android/iOS.
12. Perfil e-ink y botones físicos en hardware o emulador apropiado.

Cada ejecución restante debe registrar fecha, build, equipo, escala y PDF. Un
caso pendiente nunca se presenta como aprobado.

## Tercera iteración — evidencia 2026-07-29

La suite añade cobertura de:

- esquema v2/FTS5, migración v1→v2, copia previa, backup v1/v2 e integridad;
- anotaciones CRUD, filtros, rectángulos normalizados, rotación, huella,
  revisión/undo y exportación Markdown/JSON;
- colecciones, vistas inteligentes y relaciones sin borrar documentos;
- contextos documentales activo/warm/suspendido, máximo, close/reopen y
  workspaces/split serializado, sin franja visual sobre el lector;
- historial atrás/adelante y frontera exacta de split;
- búsqueda unificada por metadato, página y anotación, además de clear FTS;
- safe mode tras tres arranques y diagnóstico sin datos privados;
- argumentos múltiples deduplicados.

La integración Windows aislada aprueba lector, rendimiento y biblioteca. El
comando agregado `flutter test integration_test -d windows` aprobó biblioteca,
pero el runner no pudo iniciar después las apps de perfil y lector, antes de
cualquier aserción. Lector volvió a aprobar aislado y el perfil aprobó aislado y
con `flutter drive --profile`. Por ello no se oculta la incidencia ni se
clasifica como regresión funcional.

| Evidencia nueva | Tamaño/tema | Resultado |
| --- | --- | --- |
| `windows-reader-third-generation-annotations.png` | 1200×800, claro/marfil | Panel, filtro, nota y salto aprobados |
| `windows-library-third-generation-collections.png` | 1200×800, claro | Selector de colección y smart views aprobados |
| `windows-library-third-generation-collection-filtered.png` | 1200×800, claro | Dos documentos correctos, sin overflow |
| `windows-library-third-generation-compact.png` | 640×760, claro | Jerarquía y controles accesibles |
| `windows-library-third-generation-compact-dark.png` | 640×760, oscuro | Contraste y densidad aprobados visualmente |

Pendientes manuales explícitos: selección geométrica del PDF, split con dos
visores, comparación, Narrador, DPI nativo 100/150/200 %, paths Unicode/largos,
permisos/reparse points de carpetas y otro equipo/GPU.

## Biblioteca de artefactos — evidencia 2026-07-29

Cobertura automatizada añadida:

- preferencia razonable por página uno, portada posterior evidente y fallback;
- persistencia manual, no sobrescritura, invalidación automática y favorito;
- migración/esquema v3 e integridad;
- jitter estable y acotado, separación para 100 objetos y orden de flechas;
- búsqueda temporal, filtro combinado, título vacío y archivo ausente;
- composición compacta/680/1100, claro/oscuro y contraste AA.
- centrado horizontal de la biblioteca, menú compacto sin pestañas y aplicación
  del tema semántico a superficies globales;
- escalas mínimas de portada, versión de caché y miniaturas laterales
  dependientes de ancho/DPI.
- iconos de Android, iOS y Windows derivados directamente de la imagen original
  entregada, sin reinterpretarla ni insertarla dentro de la interfaz.

Comandos de validación:

```powershell
flutter analyze
flutter test
flutter test integration_test/library_visual_test.dart -d windows `
  --dart-define=YAMANTAKA_LIBRARY_CAPTURE_PATH=docs/screenshots/windows-library-artifacts.png
flutter build windows --release
```

La revisión nativa debe comparar la captura base, colección, filtrada,
compacta y compacta oscura. Narrador y DPI 100/150/200 % continúan como
`REVIEW`; no se sustituyen por una aserción de widget.

## Iteracion 2026-08-01 — verificacion y limites del entorno

Cambios cubiertos por pruebas nuevas:

- `PdfOperationToken`: listener inmediato tras cancelacion, eliminacion de
  listener e idempotencia.
- `ReaderViewportMetrics`: modo top-aligned para `Ajustar ancho` y geometria
  estable en escritorios 1920x1080, 1920x1200 y 2560x1440.
- `LibraryArtifactLayout`: slots horizontales seguros en compactos y 8/10
  columnas para contenido Full HD/2K.
- `AppCommandRegistry`: Page Up/Page Down no se pueden reasignar.
- `PdfViewController`: restauracion de posicion y detach neutral sin estado
  residual.

Comandos ejecutados con el SDK local:

```powershell
dart format lib test integration_test tool test_driver
dart analyze lib/features/library/domain/library_artifact_layout.dart
dart analyze lib/pdf_engine/models/pdf_models.dart
dart analyze lib/pdf_engine/presentation/reader_viewport_metrics.dart
dart analyze lib/app/shortcuts/app_commands.dart
git diff --check
```

El formato y el analisis de los modulos independientes terminaron sin issues.
`flutter test --reporter compact`, `flutter analyze`, build Windows y las
capturas de integracion se intentaron con el SDK local, pero el entorno no
pudo resolver `test`/`flutter_lints` desde Pub (`socket error` hacia
`pub.dev`). La solicitud de acceso externo fue rechazada por la politica de
seguridad; no se hizo bypass ni se modifico el lockfile. Por tanto, los
resultados nativos, la matriz Narrador/DPI 100/125/150/200 y las diez capturas
de esta iteracion siguen en `REVIEW`.
