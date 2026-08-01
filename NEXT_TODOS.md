# Próximas tareas de Yamantaka

Este documento representa el estado operativo del proyecto.

## Estados

- `TODO`: todavía no iniciado.
- `IN_PROGRESS`: en desarrollo.
- `BLOCKED`: no puede continuar por una dependencia externa o técnica.
- `REVIEW`: implementación terminada pendiente de validación.
- `DONE`: criterios de aceptación comprobados.

## Prioridades

- `P0`: bloquea una versión utilizable o corrige un problema crítico.
- `P1`: mejora importante para calidad, estabilidad o experiencia.
- `P2`: mejora planificada que no bloquea la versión actual.
- `P3`: exploración o función futura.

## Reglas

- Una tarea solo pasa a `DONE` cuando sus criterios de aceptación han sido
  comprobados.
- Las tareas visuales requieren inspección manual o evidencia visual.
- Las tareas de rendimiento requieren mediciones reales.
- Las tareas multiplataforma requieren validación en la plataforma indicada.

## Resumen del proyecto

- Primer hito funcional de Windows: completado.
- Visor PDF real: operativo.
- Modo libro: geometría, navegación y zoom rehechos; pendiente la matriz nativa
  ampliada de DPI/Narrador.
- Persistencia local: operativa.
- Pruebas automáticas: operativas.
- Rendimiento avanzado: perfil reproducible Windows/Profile completado.
- Android e iOS: pendientes de validación.
- Empaquetado público: pendiente.
- Sistema visual: segunda generación implementada y validada en Windows.
- Interfaz ligera: filtro, orden, densidad y presupuesto de miniaturas
  implementados sin dependencias nuevas; estados de lectura, barra contextual,
  panel perezoso y acciones locales ampliados.
- Endurecimiento 2026-08-01: ownership de imagenes de cache, cancelacion
  nativa, restauracion de historial, slots compactos y perfiles de escritorio
  16:9/16:10/Full HD/2K implementados; verificacion nativa continua en REVIEW.

| Área | Estado | Próximo objetivo |
| --- | --- | --- |
| Motor PDF | Operativo | Eliminar parche cuando upstream lo permita |
| Biblioteca | Estados/filtro/densidad y acciones locales validados | Accesibilidad nativa restante |
| Lector | Barra contextual, panel y recuperación validados | DPI y Narrador restantes |
| Modo libro | Review | Validación visual y refinamiento |
| Windows | Primer hito validado | Endurecimiento y empaquetado |
| Android | Pendiente | Instalar toolchain y validar |
| iOS | Pendiente | Validar en macOS |

Siguientes objetivos prioritarios: **YAM-050**, resolver selección y geometría
nativas; **YAM-062**, implementar split con dos visores medidos; y **YAM-041**,
completar la matriz nativa de
Narrador y DPI para cerrar las tareas visuales que siguen en `REVIEW`.

## YAM-001 — Inspección inicial

- Prioridad: P0
- Estado: DONE
- Dependencias: ninguna
- Objetivo: conocer el punto de partida y preparar el toolchain real.
- Alcance: repositorio, instrucciones, Flutter, Windows y Visual Studio.
- Criterios de aceptación: estructura e instrucciones inspeccionadas; estado
  de Flutter, Windows y Visual Studio registrado.
- Validación: inspección inicial completada; SDK Flutter local instalado y
  excluido de Git.
- Archivos relacionados: `README.md`, `NEXT_TODOS.md`, `.gitignore`
- Riesgos o limitaciones: el SDK local no está añadido al `PATH` global.
- Notas técnicas: el repositorio inicial no contenía una aplicación funcional.

### Validación completada

- [x] Repositorio e instrucciones inspeccionados.
- [x] Flutter 3.44.8 y Dart 3.12.2 disponibles.
- [x] Toolchain Windows identificado.

## YAM-002 — Decisiones y diseño inicial

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-001
- Objetivo: fijar motor, persistencia, estado, paquetes y dirección visual.
- Alcance: ADR, arquitectura modular, privacidad y primer sistema de diseño.
- Criterios de aceptación: alternativas, licencias, riesgos y arquitectura
  documentados antes del visor completo.
- Validación: ADR-001 a ADR-009 y documentación arquitectónica inspeccionados.
- Archivos relacionados: `DECISIONS.md`, `DESIGN.md`, `ARCHITECTURE.md`
- Riesgos o limitaciones: el sistema visual inicial requería centralización.
- Notas técnicas: `pdfrx`/PDFium queda detrás de contratos propios.

## YAM-003 — Esqueleto multiplataforma

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-002
- Objetivo: crear la aplicación Flutter y sus servicios base.
- Alcance: Windows, Android, iOS, configuración, tema, logging y errores.
- Criterios de aceptación: arranque, composición e interfaces analizadas sin
  fallos.
- Validación: `flutter analyze` aprobado el 2026-07-28 y revalidado al inicio
  de la segunda iteración el 2026-07-29.
- Archivos relacionados: `lib/app`, `lib/core`, `windows`, `android`, `ios`
- Riesgos o limitaciones: Android/iOS aún no tienen toolchain validado.
- Notas técnicas: el nombre visible vive en `AppConfig.productName`.

## YAM-004 — SQLite y biblioteca funcional

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003
- Objetivo: mantener biblioteca, historial y progreso exclusivamente en local.
- Alcance: cinco tablas, migración, recientes, ausentes, continuar y portadas.
- Criterios de aceptación: repositorios probados; documentos sin duplicados;
  portada invalidada cuando cambia la huella.
- Validación: pruebas SQLite y widgets aprobadas dentro de las 34 pruebas
  históricas.
- Archivos relacionados: `lib/storage`, `lib/features/library`,
  `test/database_repositories_test.dart`, `test/library_widgets_test.dart`
- Riesgos o limitaciones: la relocalización avanzada sigue siendo manual.
- Notas técnicas: identidad mediante ruta normalizada, tamaño, fecha y hash
  parcial.

## YAM-005 — Apertura y motor PDF

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003
- Objetivo: abrir, renderizar e inspeccionar PDF reales sin bloquear la UI.
- Alcance: selector, drop, argumento, PDFium, texto, búsqueda, índice y enlaces.
- Criterios de aceptación: PDF real visible; cierre libera el documento;
  errores y contraseña tipados.
- Validación: bitmap, `ui.Image`, lectura de píxeles, pintura y cierre aprobados
  en Windows.
- Archivos relacionados: `lib/pdf_engine`, `lib/features/reader`,
  `test/pdf_engine_real_test.dart`, `integration_test/reader_integration_test.dart`
- Riesgos o limitaciones: parche local temporal documentado en YAM-014.
- Notas técnicas: PDFium se consume exclusivamente mediante
  `PdfDocumentEngine`/`PdfViewerAdapter`.

## YAM-006 — Navegación y restauración

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004, YAM-005
- Objetivo: conservar continuidad de lectura entre sesiones.
- Alcance: anterior/siguiente, página, zoom, ajuste, disposición, rotación y
  scroll relativo.
- Criterios de aceptación: página, zoom y offset persistidos con debounce y
  restaurados.
- Validación: widgets verifican persistencia; integración nativa verifica
  zoom, avance, retroceso, cierre y reapertura.
- Archivos relacionados: `lib/features/reader`,
  `test/reader_widgets_test.dart`, `integration_test/reader_integration_test.dart`
- Riesgos o limitaciones: el offset es una fracción aproximada, no una
  coordenada tipográfica.
- Notas técnicas: la escritura no ocurre por cada píxel.

## YAM-007 — Modo libro

- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-006
- Objetivo: ofrecer una lectura silenciosa, adaptable y prolongada.
- Alcance: una/dos páginas, portada sola, LTR/RTL, seis canvas, márgenes,
  laterales, ocultación y pantalla completa.
- Criterios de aceptación: todos los modos funcionales y evidencia visual
  nativa en tamaños/DPI representativos.
- Validación: navegación pura y widgets aprobados para 1/2/par/impar,
  Documento↔Libro, teclado completo y campos de texto; integración Windows
  2026-07-30 con PDFium real confirma por píxeles portada/pliego centrados, lomo
  compacto, zoom mínimo, ajuste, fullscreen y transición 1→2–3.
- Archivos relacionados: `lib/features/reader/presentation`,
  `lib/pdf_engine/presentation/reader_viewport_metrics.dart`,
  `test/reader_models_test.dart`, `test/reader_viewport_metrics_test.dart`,
  `test/reader_widgets_test.dart`, `integration_test/reader_integration_test.dart`,
  `docs/screenshots`
- Riesgos o limitaciones: falta matriz nativa manual a 100/150/200 % y Narrador;
  125 % ya tenía inspección de la iteración visual anterior.
- Notas técnicas: `ReadingUnitNavigation` centraliza índices/indicadores y
  `ReaderViewportMetrics` layout/fit/centrado/paneo. No cambia a `DONE` hasta
  completar YAM-041.

### Subtareas

- [x] Compactar y centrar portada/pliegos con lomo acotado y bordes alineados.
- [x] Unificar navegación visible/teclado y mostrar `2–3 / total`.
- [x] Normalizar zoom, paneo, ajuste y resize contra la unidad activa.
- [x] Probar dominio, widgets y PDFium real con aserciones por píxeles.
- [ ] Completar DPI nativo 100/150/200 % y Narrador mediante YAM-041.

## YAM-008 — Búsqueda, panel y marcadores

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004, YAM-005
- Objetivo: localizar contenido y conservar puntos de lectura.
- Alcance: búsqueda cancelable, navegación de resultados, panel único y
  marcadores con etiqueta.
- Criterios de aceptación: búsqueda PDFium y persistencia SQLite recuperable.
- Validación: búsquedas, cancelación, documento sin texto y marcador restaurado
  están cubiertos por pruebas.
- Archivos relacionados: `lib/features/search`, `lib/features/bookmarks`,
  `lib/features/reader/presentation/reader_side_panel.dart`
- Riesgos o limitaciones: no existe OCR.
- Notas técnicas: el panel muestra una sola sección cada vez.

## YAM-009 — Comandos, atajos y ajustes

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-006
- Objetivo: centralizar entrada y preferencias sin recargar la interfaz.
- Alcance: registro de comandos, atajos, `Ctrl+K`, apariencia, lectura, caché y
  privacidad.
- Criterios de aceptación: atajos solicitados, preferencias persistentes y
  perfil de tinta electrónica tipado.
- Validación: registro, filtrado/ejecución de paleta y ajustes globales
  aprobados por pruebas.
- Archivos relacionados: `lib/app/shortcuts`,
  `lib/features/command_palette`, `lib/features/settings`
- Riesgos o limitaciones: la personalización se limita deliberadamente a
  opciones prevalidadas; no existe captura arbitraria de teclas.
- Notas técnicas: idioma, geometría de ventana y overrides JSON también se
  persisten; YAM-037 documenta la ampliación.

## YAM-010 — Pruebas automáticas

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004 a YAM-009
- Objetivo: proteger dominio, persistencia, widgets y flujo PDF nativo.
- Alcance: unitarias, widgets, PDFium real e integración Windows.
- Criterios de aceptación: suite estándar e integración aprobadas sin fixtures
  con copyright.
- Validación: línea base de 34 pruebas preservada y ampliada a 66; captura con
  píxeles reales e integración revalidadas el 2026-07-29.
- Archivos relacionados: `test`, `integration_test`, `TESTING.md`
- Riesgos o limitaciones: la matriz visual amplia solo está automatizada en
  parte; DPI y tecnología asistiva siguen siendo manuales.
- Notas técnicas: el PDF textual y el AES-128 se generan de forma controlada.

### Validación completada

- [x] `flutter analyze`.
- [x] Pruebas unitarias.
- [x] Pruebas de widgets.
- [x] Integración nativa Windows.
- [x] PDF corrupto, vacío y contraseña real.

## YAM-011 — Rendimiento y caché

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-005
- Objetivo: limitar recursos y medir sesiones reales.
- Alcance: LRU por bytes, render bajo demanda, deduplicación, cancelación y
  métricas.
- Criterios de aceptación: cachés probadas y mediana/p95/memoria con PDF largo.
- Validación: cinco pasadas Windows/Profile con PDF de 100 páginas y fixture
  vectorial pesado; mediana/p95, frames, RSS, caché, deduplicación y cierre
  registrados el 2026-07-29 en `PERFORMANCE.md`.
- Archivos relacionados: `lib/pdf_engine/cache`, `PERFORMANCE.md`
- Riesgos o limitaciones: una página extrema puede superar temporalmente el
  presupuesto; todavía no hay teselas.
- Notas técnicas: el RSS mínimo tras volver a biblioteca quedó 3,1 MiB por
  debajo del pico, aunque el asignador del sistema no vuelve a la línea base;
  una página extrema continúa justificando una futura evaluación de teselas.

## YAM-012 — Validación Windows del primer hito

- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003 a YAM-010
- Objetivo: demostrar que el MVP se compila y ejecuta en el destino principal.
- Alcance: doctor, análisis, tests, integración, Release y smoke.
- Criterios de aceptación: ejecutable receptivo con PDF real, persistencia local
  y sin conexiones TCP.
- Validación: Build Tools 2022 17.14.37, SDK 10.0.26100.0, 34 pruebas,
  integración, captura y Release aprobados.
- Archivos relacionados: todo el repositorio, `TESTING.md`, `PERFORMANCE.md`
- Riesgos o limitaciones: no equivale a la matriz ampliada de YAM-015.
- Notas técnicas: Modo de desarrollador de Windows activo.

## YAM-013 — Validación móvil

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-012
- Objetivo: comprobar compilación, entrada y almacenamiento en Android/iOS.
- Alcance: Android/tables Android; iOS/iPadOS en macOS.
- Criterios de aceptación: build y ejecución por plataforma, permisos y flujo
  táctil verificados.
- Validación: no ejecutada.
- Archivos relacionados: `android`, `ios`, `ROADMAP.md`
- Riesgos o limitaciones: Android SDK ausente; iOS requiere macOS/Xcode.
- Notas técnicas: no declarar soporte validado antes de disponer del toolchain.

## YAM-014 — Retirar parche local de pdfrx

- Prioridad: P1
- Estado: TODO
- Dependencias: versión upstream corregida de `pdfrx`
- Objetivo: volver al paquete hospedado sin recuperar la pantalla en blanco.
- Alcance: eliminar `third_party/pdfrx`, restaurar lockfile y repetir gates.
- Criterios de aceptación: análisis, suite vigente, integración, captura de
  píxeles y Release aprobados con upstream.
- Validación: no ejecutable hasta que exista una versión candidata.
- Archivos relacionados: `third_party/pdfrx`, `pubspec.yaml`, `pubspec.lock`,
  `DECISIONS.md`
- Riesgos o limitaciones: divergencia de upstream.
- Notas técnicas: el parche MIT mantiene el codec hasta `getNextFrame()` y
  reporta errores antes silenciados. En esta iteración se eliminó además una
  traza de upstream que exponía la ruta local; retirar el fork sigue dependiendo
  de una versión hospedada validable.

## YAM-015 — Endurecimiento Windows ampliado

- Prioridad: P1
- Estado: TODO
- Dependencias: YAM-011, YAM-012
- Objetivo: validar entradas, DPI, accesibilidad y memoria más allá del smoke.
- Alcance: escaneado, grande, Unicode/ruta larga, selector/drop, modos, atajos,
  DPI, lector de pantalla, limpieza y relocalización.
- Criterios de aceptación: matriz registrada con equipo, build, documentos,
  mediana y p95 reales.
- Validación: corrupto, vacío, contraseña y smoke pequeño ya cubiertos; resto
  pendiente.
- Archivos relacionados: `TESTING.md`, `PERFORMANCE.md`, `docs/screenshots`
- Riesgos o limitaciones: necesita PDFs controlados y tiempo de prueba manual.
- Notas técnicas: no mezclar evidencia automatizada con validación manual.

## YAM-016 — Empaquetado y firma de Windows

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-015
- Objetivo: preparar una distribución verificable.
- Alcance: MSIX o instalador ligero, identidad, firma, instalación y
  desinstalación.
- Criterios de aceptación: paquete firmado; conservación/eliminación optativa
  de datos probada.
- Validación: no ejecutada.
- Archivos relacionados: `windows`, `README.md`, `DECISIONS.md`, `PRIVACY.md`
- Riesgos o limitaciones: requiere certificado y decisión de distribución.
- Notas técnicas: el bundle actual no está firmado y no es una publicación.

## YAM-017 — Sistema visual de segunda generación

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-007, YAM-009, YAM-012
- Objetivo: crear un sistema coherente, moderno, minimalista y reutilizable.
- Alcance: `DESIGN.md`, tokens, tipografía, color, bordes, iconografía,
  densidades, claro/oscuro y seis canvas de lectura.
- Criterios de aceptación: tokens documentados/aplicados; interfaz coherente;
  capturas en varios anchos; análisis y pruebas aprobados.
- Validación: línea base de 34 pruebas y capturas 1200/700; revalidada con 66
  pruebas, integración Windows, Profile, Release y capturas contextuales.
- Archivos relacionados: `DESIGN.md`, `lib/app/theme`, componentes visuales,
  `docs/screenshots`
- Riesgos o limitaciones: contraste formal y DPI distintos de 125 % continúan
  en YAM-022.
- Notas técnicas: no se añaden paquetes visuales.

### Subtareas

- [x] Reescribir `DESIGN.md`.
- [x] Centralizar colores, espacios, tamaños, radios y movimiento.
- [x] Aplicar claro/oscuro y fallback de tinta electrónica.
- [x] Corregir política responsive 900/999.
- [x] Capturar y revisar resultados finales.

## YAM-018 — Refinamiento visual de la biblioteca

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-017
- Objetivo: hacer de la biblioteca una entrada editorial centrada en continuar.
- Alcance: encabezado, continuar, recientes, portadas, vacío, carga, error y
  ausentes.
- Criterios de aceptación: continuar dominante; abrir localizable; teclado;
  composición estrecha; lista sin tarjetas; miniaturas progresivas.
- Validación: widgets a 420/679/680/1099/1100 px, estado ausente, cambiado,
  vacío, filtro recuperable, `Ctrl+F`, acción de información y selector por
  estado; capturas ancha/compacta/claro/oscuro a 125 %. Falta recorrido nativo
  completo por Tab/Shift+Tab y Narrador.
- Archivos relacionados: `lib/features/library/presentation`,
  `test/library_widgets_test.dart`, `docs/screenshots`
- Riesgos o limitaciones: las portadas asíncronas no deben bloquear la carga.
- Notas técnicas: la segunda generación elimina tarjetas elevadas y mantiene
  la acción principal en el app bar y el bloque editorial.

### Subtareas

- [x] Composición editorial expandida.
- [x] Variante compacta deliberada.
- [x] Estados carga/vacío/error/ausente.
- [x] Pruebas de overflow.
- [ ] Recorrido manual completo de foco y teclado.

## YAM-019 — Refinamiento visual del lector

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-017
- Objetivo: reducir ruido y aumentar la sensación de lectura.
- Alcance: barra, navegación, zoom, progreso, modos, controles contextuales,
  fullscreen y ocultación.
- Criterios de aceptación: PDF dominante; acciones accesibles; modo libro
  desaparece; fluidez/memoria sin regresión significativa.
- Validación: pruebas de widgets a seis anchos, reglas de legibilidad y e-ink;
  integración Windows del lector completo con PDFium; capturas de documento,
  búsqueda y paleta; smoke Release sin regresión observada.
- Archivos relacionados: `lib/features/reader/presentation`,
  `lib/pdf_engine/adapters/pdfrx_viewer_adapter.dart`, pruebas/capturas
- Riesgos o limitaciones: overlays no deben desplazar ni bloquear el PDF.
- Notas técnicas: la vista enfrentada conserva portada y dirección actuales.

### Subtareas

- [x] Barra superior adaptable.
- [x] Control inferior compacto.
- [x] Política legible de vista enfrentada.
- [x] Sombra según canvas/e-ink.

## YAM-020 — Paneles, diálogos y paleta de comandos

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-017
- Objetivo: unificar superficies temporales y su foco.
- Alcance: panel, búsqueda, marcadores, índice, miniaturas, paleta, diálogos,
  menús y tooltips.
- Criterios de aceptación: una capa secundaria; dimensiones coherentes; Escape;
  foco restaurado; teclado completo; movimiento reducido.
- Validación: búsqueda, panel redimensionable y paleta capturados con PDFium
  real; arriba/abajo/Enter/Escape, foco inicial, sección única y movimiento
  reducido probados. Falta recorrido nativo completo de Tab y restauración de
  foco con Narrador.
- Archivos relacionados: `reader_side_panel.dart`, `command_palette.dart`,
  tema y pruebas
- Riesgos o limitaciones: el foco de Flutter/Windows requiere validación nativa.
- Notas técnicas: la selección explícita vive en la paleta; los paneles
  compacto/medio usan scrim y restituyen el foco solicitado antes de abrir.

### Subtareas

- [x] Panel responsive con scrim.
- [x] Selección/Enter/arriba/abajo en paleta.
- [x] Diálogos y tooltips tematizados.
- [x] Escape automatizado.
- [ ] Tabulación, foco restaurado y lector de pantalla en Windows.

## YAM-021 — Estados visuales y retroalimentación

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-017
- Objetivo: explicar operaciones y errores sin añadir ruido.
- Alcance: carga, apertura, render, búsqueda, vacío, sin texto, ausente, error,
  contraseña y avisos reversibles.
- Criterios de aceptación: feedback inmediato; mensajes accionables; color no
  exclusivo; sin spinner indefinido sin contexto.
- Validación: pruebas de biblioteca vacía/ausente, lector en error, búsqueda
  vacía/con resultado, perfil e-ink e integración real; contraseña real
  validada en motor y diálogo actualizado con error contextual.
- Archivos relacionados: biblioteca, lector, panel, `app.dart`, pruebas
- Riesgos o limitaciones: demasiados avisos pueden competir con la lectura.
- Notas técnicas: los `AppFailure` existentes se conservan.

### Subtareas

- [x] Sustituir spinners aislados por estados con texto y geometría estable.
- [x] Diferenciar vacío, ausente, sin texto, sin resultados y error.
- [x] Añadir acción recuperable cuando existe.
- [x] Mantener feedback local mediante snackbars.

## YAM-022 — Validación visual y regresión

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-017 a YAM-021
- Objetivo: validar sistemáticamente la calidad visual.
- Alcance: capturas, DPI 100/125/150/200 %, tres anchos, claro/oscuro, seis
  canvas, texto, foco, contraste y movimiento reducido.
- Criterios de aceptación: matriz en `docs/screenshots`; sin overflow, cortes,
  saltos ni controles inaccesibles; golden tests solo donde sean estables.
- Validación: capturas a 125 % en ancho compacto/ancho y tema claro/oscuro;
  integración a DPR 1 para lector/búsqueda/paleta/modo libro; fronteras,
  overflow, movimiento reducido y contraste WCAG AA automatizable aprobados.
  Quedan DPI nativo 100/150/200 %, matriz comparable de seis canvas y Narrador.
- Archivos relacionados: `docs/screenshots`, `TESTING.md`, pruebas visuales
- Riesgos o limitaciones: DPR simulado no sustituye DPI nativo de Windows.
- Notas técnicas: `TESTING.md` registra entorno/método y
  `tool/capture_windows_window.ps1` evita virtualización DPI.

### Subtareas

- [x] Capturas de Biblioteca, lector, búsqueda, paleta y Ajustes.
- [x] Claro/oscuro y responsive automatizado.
- [x] Captura nativa a 125 %.
- [ ] DPI nativo 100/150/200 %.
- [x] Contraste formal de pares semánticos.
- [ ] Seis canvas comparables y lector de pantalla.

## YAM-023 — Preparación visual para tablet

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-017, YAM-019
- Objetivo: separar composición de escritorio y táctil.
- Alcance: breakpoints, 48 × 48, controles inferiores, orientación, gestos,
  página única y enfrentada horizontal.
- Criterios de aceptación: nada esencial depende de hover; navegación
  independiente de entrada; composición cambia sin duplicar dominio.
- Validación: breakpoints, política de página única/enfrentada, controles
  compactos y ausencia de dependencia funcional de hover probados en widgets;
  dispositivo/emulador y orientación real pendientes.
- Archivos relacionados: tema, presentación del lector, `android`
- Riesgos o limitaciones: no declarar UX táctil validada desde Windows.
- Notas técnicas: la segunda generación deja tokens de 48 px y política
  preparada sin afirmar UX táctil validada.

### Subtareas

- [x] Separar clase de ventana y dominio de navegación.
- [x] Mantener acciones esenciales fuera de hover.
- [x] Preparar control de 48 × 48 y composición compacta.
- [ ] Validar táctil, gestos y orientaciones en dispositivo/emulador.

## YAM-024 — Perfil de tinta electrónica completo

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-013, YAM-023
- Objetivo: convertir el perfil tipado en experiencia e-ink verificada.
- Alcance: cero animación, alto contraste, opacidad plena, página única,
  navegación paginada, refresco completo y botones físicos.
- Criterios de aceptación: comportamiento medido en dispositivo o emulador
  apropiado.
- Validación: no ejecutada; no debe implementarse completamente sin hardware.
- Archivos relacionados: ajustes, lector, Android, documentación
- Riesgos o limitaciones: refresco y botones dependen del fabricante.
- Notas técnicas: esta iteración solo puede reforzar el fallback visual
  estándar sin simular capacidades físicas.

### Subtareas

- [x] Forzar canvas blanco, página única, cero sombra y movimiento reducido.
- [x] Probar el fallback a nivel de configuración del visor.
- [ ] Medir refresco completo y ghosting.
- [ ] Mapear botones físicos por fabricante.
- [ ] Validar en hardware o emulador e-ink apropiado.

## YAM-025 — Biblioteca instantánea y orientada a teclado

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-018
- Objetivo: localizar y ordenar documentos sin abandonar la biblioteca ni
  introducir una base de búsqueda secundaria.
- Alcance: filtro local por título, autor y archivo; orden por última apertura,
  título y progreso; contador de resultados; estado vacío recuperable; acceso
  directo con `Ctrl+F`.
- Criterios de aceptación: filtrado y orden deterministas; el foco llega al
  campo desde teclado; limpiar restaura la lista; cero consultas de red o
  escrituras adicionales; composición sin overflow en 420/680/1100 px.
- Validación: filtro/orden cubiertos por pruebas deterministas; `Ctrl+F`, foco,
  limpiar y estado sin coincidencias aprobados en widgets; composición
  comprobada a 420/680/1100 px y capturada en Release ancho/compacto.
- Archivos relacionados: biblioteca, comandos, pruebas de widgets
- Riesgos o limitaciones: la biblioteca conserva un máximo de 40 recientes, por
  lo que el filtrado en memoria es más simple y rápido que crear otro índice.
- Notas técnicas: normalización local insensible a mayúsculas; no se registra
  ni persiste el texto buscado.

### Subtareas

- [x] Añadir filtro y orden responsive.
- [x] Añadir `Ctrl+F`, limpiar y estado sin coincidencias.
- [x] Cubrir dominio, teclado y fronteras responsive.

## YAM-026 — Densidad de interfaz adaptable y persistente

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-017, YAM-018
- Objetivo: permitir una interfaz cómoda o compacta sin duplicar componentes ni
  reducir la legibilidad del documento.
- Alcance: preferencia tipada, persistencia, tokens de densidad, tema Material,
  filas de biblioteca y ajustes.
- Criterios de aceptación: cambio inmediato y reversible; controles esenciales
  conservan un objetivo mínimo de 40 px en escritorio; la preferencia sobrevive
  al reinicio; el lector no cambia el tamaño del contenido PDF.
- Validación: preferencia tipada guardada/restaurada y fallback probado; tokens
  cómoda/compacta aplicados a tema, biblioteca y ajustes; objetivo de 40 px
  protegido por prueba y PDF no modificado por la densidad.
- Archivos relacionados: tema, ajustes, persistencia, biblioteca, pruebas
- Riesgos o limitaciones: densidad compacta no equivale a escala tipográfica ni
  debe aplicarse como zoom al documento.
- Notas técnicas: usar `ThemeExtension` y constantes; ninguna dependencia
  visual nueva.

### Subtareas

- [x] Modelar y persistir densidad cómoda/compacta.
- [x] Aplicar tokens compartidos a superficies frecuentes.
- [x] Verificar objetivos, tema y restauración.

## YAM-027 — Portadas y listas con presupuesto de memoria

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-011, YAM-018
- Objetivo: mantener la biblioteca fluida evitando decodificar miniaturas por
  encima de su tamaño visible.
- Alcance: `cacheWidth`/`cacheHeight` según DPR, calidad de filtrado apropiada,
  aislamiento de repintado, lista virtualizada y ausencia de efectos costosos.
- Criterios de aceptación: cada portada solicita un bitmap acotado por su caja
  física; fallos siguen mostrando fallback; no se añade caché paralela; la lista
  continúa usando slivers con extensión conocida cuando sea aplicable.
- Validación: `ResizeImage` solicita 88×120 px para una portada visible de
  44×60 a DPR 2; calidad baja, fallback, `RepaintBoundary` y slivers
  preservados; integración y Release aprobados.
- Archivos relacionados: biblioteca, caché de portadas, pruebas de widgets
- Riesgos o limitaciones: el motor conserva la caché de archivo existente; esta
  tarea limita la imagen decodificada en memoria de interfaz.
- Notas técnicas: priorizar `FilterQuality.low` para miniaturas pequeñas.

### Subtareas

- [x] Acotar decodificación por DPR.
- [x] Aislar repintado de portadas.
- [x] Añadir una prueba que proteja el presupuesto.

## YAM-028 — Interacción coherente con foco y movimiento reducido

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-020, YAM-025
- Objetivo: hacer predecibles las superficies temporales y la búsqueda para
  teclado y perfiles sin animación.
- Alcance: foco inicial, Escape, anuncios semánticos de resultados, navegación
  de la paleta sin animación cuando se solicita y tooltips de atajos.
- Criterios de aceptación: ninguna animación programática permanece en la
  paleta con movimiento reducido; resultados y estados vacíos tienen semántica;
  teclado y ratón ejecutan la misma acción.
- Validación: foco local y `Ctrl+F` aprobados; contador semántico y estado vacío
  implementados; scroll de paleta inmediato con movimiento reducido, Escape y
  ejecución por teclado conservados; transición global de tema queda en cero.
- Archivos relacionados: paleta, biblioteca, lector, pruebas de widgets
- Riesgos o limitaciones: la validación con lector de pantalla nativo sigue
  perteneciendo a YAM-020/YAM-022.
- Notas técnicas: no simular tecnología asistiva nativa con una prueba widget.

### Subtareas

- [x] Propagar movimiento reducido a la paleta.
- [x] Completar foco y semántica de búsqueda.
- [x] Probar navegación sin animación y restauración funcional.

## YAM-029 — Presupuesto de regresión para interfaz ligera

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-025 a YAM-028
- Objetivo: impedir que las mejoras visuales degraden tamaño, reconstrucciones o
  estabilidad responsive.
- Alcance: gates de formato/análisis/tests, ausencia de nuevas dependencias,
  límites de recientes/miniaturas, fronteras de ancho y build Release.
- Criterios de aceptación: suite completa aprobada; build Release aprobado; sin
  overflow; ninguna dependencia de producción nueva; documentación registra
  decisiones y límites verificables.
- Validación: formato, análisis, 66 pruebas, integración Windows con PDFium,
  perfil y Release aprobados; bundle de 19 archivos y 39,19 MiB; `pubspec.yaml` y
  `pubspec.lock` sin cambios; capturas Release revisadas sin overflow.
- Archivos relacionados: `pubspec.yaml`, pruebas, `TESTING.md`,
  `PERFORMANCE.md`, `DECISIONS.md`
- Riesgos o limitaciones: el perfil actual cubre 100 páginas y carga vectorial;
  1000 páginas, DPI múltiples y GPU distintas continúan en YAM-015/YAM-046.
- Notas técnicas: los gates funcionales no se presentan como benchmark.

### Subtareas

- [x] Ampliar pruebas focalizadas.
- [x] Ejecutar gates completos y Release.
- [x] Registrar tamaño y ausencia de dependencias nuevas.

## YAM-030 — Biblioteca por estado de lectura

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-025
- Objetivo: encontrar con rapidez qué empezar, continuar o terminar sin
  convertir la biblioteca en un gestor pesado.
- Alcance: filtros `Todos`, `En curso`, `Sin empezar` y `Terminados`; recuentos
  locales; combinación con texto/orden; persistencia opcional del último filtro.
- Criterios de aceptación: clasificación derivada del progreso existente; una
  sola pasada sobre un máximo de 40 recientes; teclado completo; estado vacío
  recuperable; sin nuevas tablas ni dependencias.
- Validación: evaluación combinada texto/estado/orden y recuentos probada; el
  selector funciona a 420/680/1100 px, expone semántica, conserva el estado y
  no conserva la consulta; capturas Windows claro/oscuro inspeccionadas.
- Archivos relacionados: `library_screen.dart`,
  `app_settings_repository.dart`, pruebas y capturas de biblioteca
- Riesgos o limitaciones: el 100 % es una señal de navegación, no una
  certificación de que la persona terminó de leer.
- Notas técnicas: una sola iteración filtra texto y cuenta estados; solo se
  ordena la lista resultante. El bloque Continuar excluye 0 %, 100 % y ausentes.

### Subtareas

- [x] Modelar filtros combinables y estables.
- [x] Diseñar selector responsive sin aumentar el app bar.
- [x] Probar combinaciones, foco, semántica y 420/680/1100 px.

## YAM-031 — Barra del lector contextual y eficiente

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-019, YAM-028
- Objetivo: mostrar solo acciones relevantes al estado actual y trasladar las
  secundarias a un overflow consistente.
- Alcance: prioridades por ancho, estado activo visible, menú único de acciones
  secundarias, tooltips con atajo y personalización mínima persistente.
- Criterios de aceptación: PDF conserva el área dominante; ningún control salta
  de posición durante render/búsqueda; acciones disponibles en teclado/paleta;
  máximo una reconstrucción de chrome por cambio de estado lógico.
- Validación: jerarquía compacta/media/expandida y overflow único aprobados;
  grupos modo/panel persistentes; estados activos tienen forma/semántica;
  tooltips muestran atajos vigentes; una prueba demuestra que panel, resize y
  estado del visor no reconstruyen el adaptador PDF.
- Archivos relacionados: `reader_top_bar.dart`,
  `reader_bottom_controls.dart`, `reader_screen.dart`, comandos y ajustes
- Riesgos o limitaciones: demasiada personalización aumenta soporte; limitar a
  mostrar/ocultar grupos predefinidos.
- Notas técnicas: el visor queda cacheado por configuración lógica; barra y
  controles inferiores escuchan únicamente el estado que consumen.

### Subtareas

- [x] Definir jerarquía de acciones por clase de ventana.
- [x] Implementar overflow y estado activo accesible.
- [x] Proteger reconstrucciones y cubrir atajos existentes.

## YAM-032 — Panel lateral redimensionable y perezoso

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-020, YAM-029
- Objetivo: adaptar miniaturas, índice, búsqueda y marcadores al documento sin
  cargar contenido de secciones invisibles.
- Alcance: ancho redimensionable con límites, restauración local por ventana,
  sección única montada, colapso rápido y divisor accesible por teclado.
- Criterios de aceptación: 280–480 px; el PDF recibe el ancho restante sin
  overflow; se construye solo la sección activa; ancho persistido con debounce;
  movimiento reducido respetado.
- Validación: ratón, flechas y semántica de incremento/decremento probados;
  límites 280–480, debounce de 240 ms, restauración y cierre por Escape
  aprobados; solo la sección activa aparece en árbol y el adaptador PDF mantiene
  un único build durante el arrastre.
- Archivos relacionados: `reader_side_panel.dart`, `reader_screen.dart`,
  ajustes, persistencia y pruebas de widgets
- Riesgos o limitaciones: el redimensionado continuo no debe escribir por cada
  píxel ni provocar renders PDF redundantes.
- Notas técnicas: las miniaturas visibles reutilizan futuros por índice y
  conservan el scroll mediante `PageStorage`.

### Subtareas

- [x] Añadir divisor y límites tipados.
- [x] Persistir ancho con debounce y restauración segura.
- [x] Instrumentar builds/renders durante el arrastre.

## YAM-033 — Accesibilidad nativa de interfaz completa

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-018, YAM-020, YAM-028
- Objetivo: cerrar la validación que mantiene varias tareas visuales en review.
- Alcance: recorrido Tab/Shift+Tab, orden de foco, restitución tras diálogos,
  nombres/estados semánticos, Narrador de Windows y contraste formal.
- Criterios de aceptación: matriz nativa grabada para biblioteca, lector,
  paneles, paleta y ajustes; ninguna trampa de foco; anuncios sin duplicados;
  contraste de texto/controles documentado.
- Validación: pares semánticos cumplen WCAG AA automatizable; botones de icono,
  estados, progreso, selector, resultados, divisor y superficies temporales
  tienen nombres/valores; Enter/Space/Escape, foco inicial y navegación de
  paleta están probados. Narrador y recorrido nativo completo siguen pendientes.
- Archivos relacionados: todas las superficies, `TESTING.md`, capturas
- Riesgos o limitaciones: las pruebas widget ayudan, pero Narrador y foco
  nativo requieren ejecución manual en Windows.
- Notas técnicas: esta tarea puede cerrar YAM-018/YAM-020 y parte de YAM-022.

### Subtareas

- [x] Automatizar contraste, etiquetas, roles, estados y teclado.
- [x] Documentar una matriz manual reproducible.
- [ ] Ejecutar recorrido nativo con Narrador.
- [ ] Registrar resultado observado y corregir cualquier hallazgo nativo.

## YAM-034 — Perfil de rendimiento de la interfaz

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-011, YAM-029 a YAM-032
- Objetivo: convertir los presupuestos estructurales en mediciones repetibles de
  fluidez, memoria y reconstrucciones.
- Alcance: biblioteca con 40 portadas, filtro rápido, cambio de densidad,
  apertura/cierre de panel, 100 páginas y retorno a biblioteca.
- Criterios de aceptación: mediana/p95 de interacción, frames lentos, pico de
  memoria y memoria liberada registrados en Profile/Release; comparación contra
  la línea base; sin rutas ni títulos en logs.
- Validación: cinco pasadas Windows/Profile a 125 % con 40 documentos, PDF de
  100 páginas y PDF vectorial pesado. Biblioteca 123,420/135,266 ms y lector
  83,997/87,589 ms (mediana/p95 de la secuencia); frames build 3,294/6,360 ms y
  2,684/5,570 ms; cero presupuestos perdidos; pico 283,3 MiB y 3,1 MiB
  liberados respecto al mínimo tras retorno.
- Archivos relacionados: `performance_profile_test.dart`,
  `performance_driver.dart`, generador de fixtures y `PERFORMANCE.md`
- Riesgos o limitaciones: un único equipo no representa todas las GPU/DPI.
- Notas técnicas: el JSON no contiene rutas/títulos; la instrumentación del
  motor solo expone contadores locales consultables y no emite telemetría.

### Subtareas

- [x] Crear fixtures reproducibles de biblioteca, documento largo y pesado.
- [x] Registrar cinco pasadas, apertura/render, caché, RSS y frames a 125 %.
- [x] Conservar 100/200 % como matriz nativa pendiente, sin simularla.

## YAM-035 — Información y acciones del documento

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-030, YAM-031
- Objetivo: consultar y actuar sobre un documento sin convertir Yamantaka en
  gestor de archivos.
- Alcance: ficha accesible desde lector y biblioteca; título, autor, archivo,
  páginas, tamaño, última apertura, progreso, página, estado, ruta, modificación
  y presencia; copiar, mostrar, abrir/continuar, localizar, reiniciar y retirar.
- Criterios de aceptación: todos los datos proceden del estado local; retirar no
  borra el PDF; las acciones destructivas confirman; relocalizar exige la misma
  huella; un archivo cambiado o ausente se diferencia sin depender del color.
- Validación: acceso independiente desde fila probado; diálogo integrado en la
  aplicación; cambio/ausencia/error de portada tienen icono y texto; integración
  y build Windows aprobados.
- Archivos relacionados: `app.dart`, biblioteca, repositorio de documentos,
  barra/paleta del lector y pruebas
- Riesgos o limitaciones: una modificación legítima cambia la huella; puede
  abrirse como versión nueva, pero no se fuerza una relocalización ambigua.
- Notas técnicas: “Mostrar” delega al explorador del sistema; copiar ruta solo
  ocurre por acción explícita.

### Subtareas

- [x] Crear una ficha local compartida por biblioteca y lector.
- [x] Añadir acciones seguras y confirmaciones explícitas.
- [x] Detectar ausencia/cambio y corregir el flujo de relocalización.

## YAM-036 — Selector rápido de documentos

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-009, YAM-031
- Objetivo: cambiar entre documentos recientes sin abandonar el teclado.
- Alcance: los ocho recientes se integran en la paleta existente; búsqueda por
  título, autor y archivo; progreso discreto; abrir otro PDF y continuar.
- Criterios de aceptación: una sola paleta/consulta; flechas y Enter ejecutan;
  resultados limitados; ningún índice, red, persistencia de consulta ni PDF
  adicional abierto.
- Validación: filtrado y ejecución por Enter probados; integración Windows
  confirma acciones y documentos recientes dentro del mismo overlay.
- Archivos relacionados: `command_palette.dart`, `app_commands.dart`,
  `reader_screen.dart`, pruebas de paleta
- Riesgos o limitaciones: se limita a ocho recientes para conservar escaneo,
  navegación y composición constantes.
- Notas técnicas: un fallo al abrir otro PDF conserva el documento actual.

### Subtareas

- [x] Unificar comandos y documentos recientes en el mismo modelo visual.
- [x] Añadir búsqueda y navegación de teclado compartidas.
- [x] Proteger el lector actual frente a una apertura fallida.

## YAM-037 — Atajos configurables

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-009, YAM-031
- Objetivo: personalizar atajos importantes sin permitir combinaciones
  imposibles ni duplicar el registro de comandos.
- Alcance: abrir, buscar, paleta, ir a página, panel, página siguiente/anterior,
  zoom, pantalla completa, chrome y marcador; opciones predefinidas y reset.
- Criterios de aceptación: conflicto/restricción detectados; Escape, Inicio, Fin
  y flechas básicas reservados; persistencia JSON validada; tooltips, menús,
  paleta y shell usan la combinación resuelta; defaults reversibles.
- Validación: conflictos, entrada inválida, round-trip, restauración y bindings
  probados; navegación/zoom/panel/chrome comparten las acciones del registro.
- Archivos relacionados: `app_commands.dart`, ajustes, lector, persistencia y
  pruebas del registro
- Riesgos o limitaciones: no se admite captura arbitraria de teclas; la lista
  segura evita diferencias de teclado y soporte ilimitado.
- Notas técnicas: las flechas y Space permanecen como navegación básica además
  del atajo configurable de página.

### Subtareas

- [x] Ampliar el registro sin crear rutas de ejecución paralelas.
- [x] Validar/persistir asignaciones y conflictos.
- [x] Propagar etiquetas actuales a tooltips, menús y paleta.

## YAM-038 — Copia de seguridad local

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-004, YAM-009
- Objetivo: exportar y restaurar datos locales de forma explícita y auditable.
- Alcance: JSON versionado con ajustes, biblioteca, progreso, marcadores,
  sesiones, preferencias y metadatos; vista previa e importación transaccional.
- Criterios de aceptación: nunca incluye bytes PDF; valida formato, versión,
  tablas, columnas, tipos y recuentos antes de mutar; crea copia previa; informa
  errores y no usa red.
- Validación: round-trip de datos/ajustes, previsualización, copia de seguridad
  previa y rechazo sin mutación de un archivo inválido probados.
- Archivos relacionados: `local_backup_service.dart`, ajustes, pruebas y
  documentación de privacidad
- Riesgos o limitaciones: contiene metadatos y rutas locales sensibles; debe
  almacenarse con el mismo cuidado que la biblioteca.
- Notas técnicas: el esquema SQLite sigue en versión 1; la restauración usa una
  transacción y orden compatible con claves foráneas.

### Subtareas

- [x] Definir formato JSON y validación exhaustiva.
- [x] Añadir exportación, vista previa, confirmación e importación segura.
- [x] Probar round-trip y rechazo sin sobrescritura silenciosa.

## YAM-039 — Historial de lectura local

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-004, YAM-038
- Objetivo: retomar actividad reciente sin convertir sesiones en analítica.
- Alcance: lista agrupada en hoy/esta semana/anteriores; página final y duración
  ya disponible; limpiar una/todas; desactivar y limitar entre 50 y 1000.
- Criterios de aceptación: orden descendente y límite estable; limpieza
  confirmada; desactivar impide nuevas sesiones; ningún cálculo conductual,
  red, identificador externo o consulta privada.
- Validación: orden, límite, IDs monotónicos, borrado individual/todos y
  preferencias probados; superficie local integrada en Ajustes.
- Archivos relacionados: repositorio de sesiones, lector, ajustes y pruebas
- Riesgos o limitaciones: sesiones antiguas importadas pueden carecer de una
  duración final si el cierre anterior fue abrupto.
- Notas técnicas: no se añade tabla; se reutiliza `reading_sessions`.

### Subtareas

- [x] Exponer historial acotado y operaciones de limpieza.
- [x] Añadir agrupación, activación y límite configurables.
- [x] Evitar colisiones temporales en sesiones rápidas.

## YAM-040 — Recuperación de sesión y cierre seguro

- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-006, YAM-032
- Objetivo: volver al contexto de lectura tras cierre o reinicio sin crear
  bucles cuando el archivo ya no es utilizable.
- Alcance: documento opt-in, página, zoom, disposición, offset, panel, sección,
  ancho y chrome; opción de iniciar siempre en biblioteca.
- Criterios de aceptación: una sola tentativa por arranque; ruta explícita tiene
  prioridad; ausencia/fallo limpia el intento; inicio en biblioteca bloquea la
  reapertura; cerrar desde la aplicación limpia el documento pendiente.
- Validación: política pura probada para ruta explícita, desactivación, inicio
  en biblioteca, ruta vacía/ausente y caso válido; preferencias de lector/panel
  cubiertas por repositorios y widgets.
- Archivos relacionados: `app.dart`, ajustes, lector, persistencia y
  `session_recovery_test.dart`
- Riesgos o limitaciones: un cierre del proceso durante el debounce puede
  recuperar el último estado estable, no necesariamente el último píxel.
- Notas técnicas: el PDF nunca se copia ni se mantiene abierto entre procesos.

### Subtareas

- [x] Persistir el contexto mínimo de UI con debounce.
- [x] Implementar política opt-in y prioridad de biblioteca.
- [x] Añadir guardas contra archivos ausentes, corruptos y bucles.

## YAM-041 — Laboratorio nativo de accesibilidad y DPI

- Prioridad: P1
- Estado: TODO
- Dependencias: YAM-007, YAM-018, YAM-020, YAM-022, YAM-033
- Objetivo: cerrar con evidencia nativa las tareas que no pueden validarse desde
  widgets ni DPR simulado.
- Alcance: script/checklist guiado para 100/125/150/200 %, claro/oscuro,
  cómoda/compacta, seis canvas, Tab/Shift+Tab y Narrador en biblioteca, lector,
  panel, paleta, ficha documental, copias e historial.
- Criterios de aceptación: cada caso registra build, escala, tema, acción,
  resultado esperado/observado, captura y tecnología asistiva; ningún foco
  atrapado, anuncio duplicado, corte u objetivo inaccesible.
- Validación: pendiente; solo 125 % y automatización están disponibles.
- Archivos relacionados: `TESTING.md`, `docs/screenshots`, scripts de captura
- Riesgos o limitaciones: requiere cambio de escala de sesión y operación manual
  de Narrador; no debe automatizarse fingiendo equivalencia.
- Notas técnicas: siguiente objetivo prioritario. Sus hallazgos deben corregirse
  antes de pasar YAM-007/018/020/022/033 a `DONE`.

### Subtareas

- [ ] Preparar una ejecución limpia y nomenclatura de evidencias.
- [ ] Completar cuatro DPI y dos temas con densidades/canvas.
- [ ] Recorrer Narrador y teclado, corregir y repetir casos fallidos.

## YAM-042 — Teselas adaptativas para páginas extremas

- Prioridad: P1
- Estado: TODO
- Dependencias: YAM-011, YAM-034
- Objetivo: impedir picos desproporcionados cuando una página rasterizada
  excede por sí sola el presupuesto LRU.
- Alcance: decisión por tamaño físico/zoom, regiones visibles con prefetch
  acotado, cancelación por generación, composición sin costuras y fallback a
  página completa para casos baratos.
- Criterios de aceptación: página extrema controlada no retiene un bitmap mayor
  que el presupuesto definido; scroll/zoom no muestra huecos persistentes;
  solicitudes obsoletas se cancelan/deduplican; páginas normales no regresan.
- Validación: pendiente; requiere fixtures raster/vector extremos y perfil
  comparativo en Profile.
- Archivos relacionados: contrato `PdfRenderRegion`, motor, adaptador, caché,
  pruebas e `PERFORMANCE.md`
- Riesgos o limitaciones: teselas elevan complejidad, bordes y número de renders;
  solo se implementarán si la medición justifica el coste.
- Notas técnicas: mantener un único caché por bytes y una clave con región,
  escala y rotación; no crear otra caché de imágenes.

### Subtareas

- [ ] Fijar umbral desde mediciones de páginas extremas.
- [ ] Prototipar planificador cancelable y composición estable.
- [ ] Comparar memoria, p95, calidad visual y bundle con página completa.

## YAM-043 — Notas locales ligadas a página

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-008, YAM-038
- Objetivo: ampliar marcadores con notas breves sin modificar el PDF.
- Alcance: texto local opcional, edición desde marcador/ficha, búsqueda por
  metadatos en la paleta, exportación/importación y limpieza explícita.
- Criterios de aceptación: límite de longitud; autosave con debounce; navegación
  por teclado; nota nunca se registra ni se incrusta en el archivo; migración y
  backup versionados; 40 documentos no requieren índice nuevo.
- Validación: cubierta por migración v2, repositorio, backup v2, exportación,
  tests unitarios y captura Windows; queda revisión manual de edición extensa.
- Archivos relacionados: dominio/repositorio de marcadores, panel, paleta,
  backup, migración y privacidad
- Riesgos o limitaciones: las notas son datos sensibles y elevan el impacto de
  una copia local; deben estar claramente incluidas en exportación/limpieza.
- Notas técnicas: evitar un editor rico o sistema general de anotaciones.

### Subtareas

- [ ] Diseñar migración mínima y límite de texto.
- [ ] Añadir edición accesible y búsqueda local acotada.
- [ ] Extender backup, privacidad y pruebas de recuperación.

## YAM-044 — Colecciones locales manuales

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-030, YAM-038
- Objetivo: agrupar documentos sin carpetas virtuales pesadas ni mover archivos.
- Alcance: pocas colecciones nombradas, asignación desde ficha, filtro de
  biblioteca, estado vacío, backup y limpieza.
- Criterios de aceptación: relación SQLite mínima; ningún escaneo de disco;
  filtro se combina en una pasada con texto/estado/orden; nombres limitados y
  accesibles; borrar una colección no borra documentos ni PDF.
- Validación: cubierta por repositorio SQLite, filtros, backup, tests y capturas
  Windows; queda revisión manual de gestión completa y teclado.
- Archivos relacionados: biblioteca, repositorios, backup, ajustes y pruebas
- Riesgos o limitaciones: demasiadas etiquetas convertirían la interfaz en un
  gestor documental; no se incluyen reglas automáticas en esta tarea.
- Notas técnicas: medir antes de aumentar el límite actual de 40 recientes.

### Subtareas

- [ ] Validar el modelo colección-documento y su migración.
- [ ] Diseñar selector editorial compacto, sin nube de chips.
- [ ] Cubrir combinación de filtros, backup y borrado seguro.

## YAM-045 — Ventanas de lectura independientes

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-015, YAM-031, YAM-040
- Objetivo: permitir comparar o consultar dos PDF sin duplicar estado global.
- Alcance: estudio Windows de una ventana de biblioteca y hasta dos lectores,
  comandos/contexto por ventana, cierre, restauración opt-in y presupuesto de
  memoria agregado.
- Criterios de aceptación: cada ventana posee documento/panel/chrome/foco;
  cierre libera PDFium y listeners; atajos actúan solo en la ventana activa;
  memoria total permanece acotada.
- Validación: pendiente; primero requiere un prototipo y perfil nativo.
- Archivos relacionados: shell, `AppServices`, ventana, lector y rendimiento
- Riesgos o limitaciones: Flutter desktop multi-window puede exigir plugin o
  código nativo; no se añadirá dependencia sin ADR y medición de bundle.
- Notas técnicas: no implementar como dos visores invisibles en el mismo árbol.

### Subtareas

- [ ] Investigar soporte nativo y ownership de servicios.
- [ ] Prototipar ciclo de vida con dos fixtures y cierre repetido.
- [ ] Decidir mediante ADR antes de exponer UI estable.

## YAM-046 — Copias cifradas y retención

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-038
- Objetivo: ofrecer protección opcional para copias que contienen rutas,
  progreso, historial y futuras notas.
- Alcance: evaluación de cifrado autenticado local, derivación de clave,
  versión de contenedor, contraseña no persistida, política de retención y
  compatibilidad con JSON sin cifrar.
- Criterios de aceptación: formato autenticado y versionado; contraseña nunca en
  logs/almacén; error no muta datos; copia previa sigue disponible; licencia,
  mantenimiento y coste de cualquier dependencia documentados.
- Validación: pendiente.
- Archivos relacionados: backup, privacidad, decisiones, ajustes y pruebas
- Riesgos o limitaciones: criptografía propia es inaceptable y puede requerir
  una dependencia auditada; recuperación de contraseña no es posible.
- Notas técnicas: mantener exportación sin cifrar para interoperabilidad y
  advertir claramente su sensibilidad.

### Subtareas

- [ ] Evaluar primitivas de plataforma y paquetes mantenidos.
- [ ] Especificar contenedor, KDF, AEAD y migración.
- [ ] Añadir vectores de prueba, corrupción y contraseña errónea.

## YAM-047 — Localización preparada

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-017, YAM-033
- Objetivo: separar texto de presentación para añadir idiomas sin degradar
  accesibilidad ni tamaño.
- Alcance: inventario de cadenas, ARB/generación oficial de Flutter, plurales,
  fechas, tooltips, semántica y fallback español.
- Criterios de aceptación: español no cambia visualmente; una localización de
  prueba cubre expansión 30–50 % y RTL de texto sin alterar dirección del PDF;
  cero paquete de runtime innecesario.
- Validación: pendiente.
- Archivos relacionados: presentación, configuración Flutter, pruebas
  responsive y documentación
- Riesgos o limitaciones: internacionalizar todo de una vez genera diffs grandes;
  migrar por superficies con gates de captura.
- Notas técnicas: la dirección del idioma de UI y la dirección de lectura del
  documento son preferencias distintas.

### Subtareas

- [ ] Inventariar cadenas y definir convención de claves.
- [ ] Migrar shell/biblioteca/lector con fallback.
- [ ] Probar expansión, pluralización, semántica y RTL de UI.

## YAM-048 — Historial de perfiles y detector de regresiones

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-034
- Objetivo: comparar distribuciones de rendimiento sin convertir una ejecución
  aislada en umbral arbitrario.
- Alcance: normalizar JSON de perfil, conservar baselines versionadas sin datos
  privados, comparar hardware/build equivalentes y emitir informe local.
- Criterios de aceptación: identifica mediana/p95, frames, RSS, caché y bundle;
  rechaza comparaciones de DPI/modo/hardware incompatibles; no sube resultados;
  umbrales se fijan tras al menos tres baselines comparables.
- Validación: pendiente.
- Archivos relacionados: `test_driver`, `tool`, `PERFORMANCE.md` y CI futuro
- Riesgos o limitaciones: ruido térmico/antivirus/GPU puede producir falsos
  positivos; el informe debe mostrar distribución y entorno.
- Notas técnicas: conservar únicamente métricas, nunca rutas, títulos o
  consultas.

### Subtareas

- [ ] Versionar el esquema de resultados y anonimizar entorno.
- [ ] Crear comparador local con tolerancias justificadas.
- [ ] Reunir tres baselines antes de activar un gate.

## YAM-049 — Anotaciones locales normalizadas

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-043
- Objetivo: modelar notas, resaltados, subrayados y tachados sin modificar PDF.
- Alcance: CRUD SQLite, colores acotados, etiquetas, coordenadas normalizadas,
  rotación, estado huérfano/revisión, undo y navegación.
- Criterios de aceptación: validación geométrica y de longitud; borrado
  recuperable inmediato; relocalización por huella; ninguna escritura al PDF.
- Validación: tests de CRUD, filtros, rotación, huella y captura Windows.
- Archivos relacionados: `lib/features/annotations`,
  `lib/storage/repositories/annotation_repository.dart`, migración v2.
- Riesgos o limitaciones: el visor aún no entrega geometría de selección nativa.
- Notas técnicas: las coordenadas se guardan en rango 0–1 por página.

### Subtareas

- [x] Persistir modelo y relaciones de etiquetas.
- [x] Integrar controlador, panel e indicadores.
- [ ] Validar overlay real cuando exista geometría de texto.

## YAM-050 — Selección contextual sobre PDF

- Prioridad: P0
- Estado: BLOCKED
- Dependencias: YAM-049, YAM-014
- Objetivo: crear anotaciones desde texto/área seleccionados en el visor.
- Alcance: selección, menú contextual, copiar, color y nota asociada.
- Criterios de aceptación: geometría por glifo estable en zoom/rotación;
  teclado/táctil; copiar respeta privacidad; cancelación sin estado residual.
- Validación: bloqueada por falta de contrato de selección en el adaptador.
- Archivos relacionados: `PdfViewerAdapter`, lector y fork `pdfrx`.
- Riesgos o limitaciones: inferir cajas desde texto extraído sería incorrecto.
- Notas técnicas: no simular selección mediante coordenadas aproximadas.

### Subtareas

- [ ] Exponer selección y geometría desde el motor/visor.
- [ ] Diseñar menú accesible para ratón, teclado y tacto.
- [ ] Probar zoom, rotación, páginas enfrentadas y RTL.

## YAM-051 — Panel de anotaciones

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-049
- Objetivo: consultar, filtrar, editar y saltar entre anotaciones.
- Alcance: lista virtualizada, búsqueda local, filtros por tipo/color/etiqueta,
  edición, copia, borrado con undo y aviso de revisión.
- Criterios de aceptación: una sección montada; foco y semántica; navegación
  exacta; estados vacío/error; sin registrar contenido.
- Validación: unitarios e integración Windows con captura dedicada.
- Archivos relacionados: `reader_side_panel.dart`, controlador de anotaciones.
- Riesgos o limitaciones: falta validar Narrador y colecciones muy grandes.
- Notas técnicas: consultas acotadas por documento y orden estable.

### Subtareas

- [x] Implementar filtros, menú y navegación.
- [x] Añadir indicador por página y aviso de huérfanas.
- [ ] Ejecutar matriz manual Narrador/DPI.

## YAM-052 — Exportación de anotaciones

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-049, YAM-038
- Objetivo: exportar anotaciones por acción explícita sin incluir el PDF.
- Alcance: Markdown legible y JSON versionado con coordenadas normalizadas.
- Criterios de aceptación: selector de destino; formato estable; etiquetas,
  página, tipo y estado; fallo no muta datos; advertencia de sensibilidad.
- Validación: round-trip estructural unitario y flujo de guardado integrado.
- Archivos relacionados: `annotation_export_service.dart`, shell y privacidad.
- Riesgos o limitaciones: el archivo exportado contiene texto privado.
- Notas técnicas: no se exportan bytes, rutas internas ni índice FTS.

### Subtareas

- [x] Implementar Markdown y JSON v1.
- [x] Añadir acción de exportación y manejo seguro de errores.
- [ ] Validar manualmente permisos y rutas Unicode.

## YAM-053 — Contextos documentales acotados

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-040
- Objetivo: restaurar y alternar documentos sin multiplicar handles PDF ni
  memoria, y sin ocupar el lector con una franja adicional.
- Alcance interno: hasta 12 contextos, pin, reorder, duplicados,
  close/reopen y estados activo/caliente/suspendido.
- Criterios de aceptación: máximo dos calientes; resto suspendido; progreso
  visible; ausentes explícitos; cerrar libera documento.
- Validación: política y reapertura probadas; integración de cambio de PDF.
- Archivos relacionados: `lib/features/workspaces`, `lib/app/app.dart`.
- Riesgos o limitaciones: falta prueba sostenida con 12 PDF pesados.
- Notas técnicas: un solo contexto activo posee el visor concreto; desde
  2026-07-29 no existe barra de pestañas visible.

### Subtareas

- [x] Implementar dominio y controlador suspendible.
- [x] Retirar la barra reordenable visible y conservar apertura/workspaces.
- [x] Integrar apertura múltiple y cierre contextual.
- [ ] Perfilar ciclo de 12 documentos pesados.

## YAM-054 — Espacios de trabajo locales

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-053
- Objetivo: guardar y restaurar conjuntos de documentos y navegación.
- Alcance: nombre, orden, pin, documento activo, página, zoom, layout y split.
- Criterios de aceptación: guardado transaccional; duplicar/borrar; resumen de
  ausentes; restauración parcial segura; sin abrir todos los PDF a la vez.
- Validación: round-trip SQLite y diálogo Windows.
- Archivos relacionados: repositorio/controlador/diálogo de workspaces.
- Riesgos o limitaciones: el split se conserva como contrato, no se renderiza.
- Notas técnicas: se persisten IDs y contexto, nunca bytes PDF.

### Subtareas

- [x] Persistir snapshot transaccional.
- [x] Añadir gestión y resumen de restauración.
- [ ] Validar 100 workspaces y conflictos de archivos movidos.

## YAM-055 — Historial de navegación significativo

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-008, YAM-049
- Objetivo: volver/avanzar entre saltos significativos dentro del documento.
- Alcance: resultados, índice, marcadores, anotaciones, page dialog y atajos.
- Criterios de aceptación: sin entradas por scroll; deduplicación; máximo
  acotado; Alt+izquierda/derecha; limpiar al cambiar documento.
- Validación: tests de pila, deduplicación y acciones integradas.
- Archivos relacionados: `navigation_history_controller.dart`, lector.
- Riesgos o limitaciones: historial efímero, no se restaura entre procesos.
- Notas técnicas: el controlador de lectura sigue siendo dueño de la página.

### Subtareas

- [x] Implementar pilas atrás/adelante.
- [x] Registrar saltos de todas las superficies relevantes.
- [ ] Ejecutar recorrido manual completo con teclado.

## YAM-056 — Navegación visual de páginas

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-055
- Objetivo: saltar rápido en documentos largos sin render por píxel.
- Alcance: scrubber de página, etiqueta de sección e ingreso numérico.
- Criterios de aceptación: solo aparece con 20+ páginas; no dispara renders
  continuos; rango accesible; confirma un salto significativo.
- Validación: política UI y navegación de integración.
- Archivos relacionados: diálogo de ir a página, índice y lector.
- Riesgos o limitaciones: preview de miniatura durante arrastre está pendiente.
- Notas técnicas: usa estado local hasta confirmar.

### Subtareas

- [x] Añadir slider acotado y etiqueta contextual.
- [ ] Añadir preview diferido con presupuesto medido.
- [ ] Validar documento de 1000 páginas.

## YAM-057 — Índice local de biblioteca

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-008, YAM-049
- Objetivo: indexar metadatos, texto y anotaciones solo por opt-in.
- Alcance: FTS5, estado incremental, pausa/cancelación, reconstrucción y borrado.
- Criterios de aceptación: secuencial; cierra cada PDF; no OCR; errores por
  documento; borrar elimina FTS; UI explica coste y privacidad.
- Validación: migración/FTS, búsqueda, limpieza y benchmark de 500 documentos.
- Archivos relacionados: repositorio/controlador/diálogo de búsqueda global.
- Riesgos o limitaciones: el opt-in no persiste y debe reactivarse al arrancar.
- Notas técnicas: el índice es reconstruible y no entra en backups.

### Subtareas

- [x] Crear FTS5 y estado incremental.
- [x] Añadir pausa, cancelación, rebuild y clear.
- [ ] Perfilar extracción de corpus real grande.

## YAM-058 — Búsqueda unificada

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-057
- Objetivo: consultar documentos, páginas, anotaciones, colecciones y etiquetas.
- Alcance: Ctrl+Shift+F, filtros por tipo, paginación, snippets y apertura.
- Criterios de aceptación: consulta escapada; resultados limitados; salto a
  documento/página; ausencia de índice explicada; consulta no persistida.
- Validación: tests FTS por tipo e integración de apertura.
- Archivos relacionados: `lib/features/search`, shell y biblioteca.
- Riesgos o limitaciones: snippets FTS pueden contener texto privado en pantalla.
- Notas técnicas: ninguna consulta se registra ni exporta.

### Subtareas

- [x] Implementar modelos, repositorio y diálogo unificado.
- [x] Integrar apertura y navegación al resultado.
- [ ] Validar corpus multilingüe, signos y RTL.

## YAM-059 — Colecciones, etiquetas y vistas inteligentes

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-044, YAM-057
- Objetivo: organizar documentos localmente sin moverlos.
- Alcance: colecciones manuales, pin/reorder, etiquetas reutilizables, merge,
  relaciones y ocho vistas inteligentes.
- Criterios de aceptación: borrar colección/etiqueta no borra PDF; filtros
  combinables; nombres validados; backup v2; vacío explícito.
- Validación: tests de repositorio/smart views y capturas filtradas.
- Archivos relacionados: `lib/features/organization`, biblioteca y backup.
- Riesgos o limitaciones: gestión avanzada de etiquetas aún no tiene UI completa.
- Notas técnicas: vistas inteligentes son consultas locales deterministas.

### Subtareas

- [x] Crear esquema y repositorio.
- [x] Integrar selector de colecciones en biblioteca.
- [ ] Completar UI de merge/reorder/asignación masiva.

## YAM-060 — Importación recursiva de carpeta

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-004
- Objetivo: previsualizar e importar PDF de una carpeta sin bloquear la UI.
- Alcance: recorrido cancelable, límite 5000, duplicados, inválidos, ausentes y
  progreso secuencial.
- Criterios de aceptación: no sigue archivos no PDF; preview antes de mutar;
  cancelación; cada handle se cierra; errores aislados.
- Validación: controlador y diálogo integrados; pendiente matriz de permisos.
- Archivos relacionados: `folder_import_controller.dart`, diálogo y biblioteca.
- Riesgos o limitaciones: enlaces/reparse points y árboles extremos requieren
  validación nativa específica.
- Notas técnicas: no hay watcher ni escaneo automático de fondo.

### Subtareas

- [x] Implementar scan/preview/import cancelables.
- [x] Añadir estados y progreso.
- [ ] Probar Unicode, rutas largas, permisos y reparse points.

## YAM-061 — Recuperación asistida de documentos

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-016, YAM-060
- Objetivo: relocalizar ausentes sin asociar silenciosamente otro contenido.
- Alcance: candidatos por huella, nombre, tamaño y modificación; resumen.
- Criterios de aceptación: solo huella exacta autoasocia; ambiguo pide decisión;
  baja confianza no muta; operación reversible por backup.
- Validación: scoring unitario y acción integrada.
- Archivos relacionados: `document_recovery_service.dart`, biblioteca.
- Riesgos o limitaciones: la búsqueda de candidatos depende de carpeta elegida.
- Notas técnicas: el score no sustituye la verificación de identidad.

### Subtareas

- [x] Implementar ranking y umbrales conservadores.
- [x] Integrar resumen sin rutas en logs.
- [ ] Diseñar selector explícito para candidatos ambiguos.

## YAM-062 — Vista dividida con dos lectores

- Prioridad: P0
- Estado: BLOCKED
- Dependencias: YAM-053, YAM-034
- Objetivo: mostrar dos documentos o posiciones simultáneamente.
- Alcance: horizontal/vertical, foco activo, ratio, sincronización opcional y
  ownership independiente.
- Criterios de aceptación: dos visores reales; atajos al panel activo; cierre
  libera ambos; límite adaptativo exacto; memoria y frames dentro de presupuesto.
- Validación: solo contrato/política y frontera 1100 px están probados.
- Archivos relacionados: `workspace_models.dart`, shell y motor PDF.
- Riesgos o limitaciones: duplicar el visor actual sin ownership causaría fugas.
- Notas técnicas: no se anuncia en UI hasta superar perfil nativo.

### Subtareas

- [x] Definir estado, orientación, ratio y política.
- [ ] Implementar ownership de dos controladores/handles.
- [ ] Perfilar resize, sincronización y cierre repetido.

## YAM-063 — Comparación sincronizada

- Prioridad: P1
- Estado: BLOCKED
- Dependencias: YAM-062
- Objetivo: comparar ediciones con navegación proporcional o por página.
- Alcance: vínculo/desvínculo, offset, páginas de distinto tamaño y foco.
- Criterios de aceptación: sincronización no recursiva; pausa manual; indicador
  visible; comportamiento definido ante recuentos distintos.
- Validación: bloqueada hasta existir split real.
- Archivos relacionados: workspaces, lectores y navegación.
- Riesgos o limitaciones: correspondencia semántica no se puede inferir.
- Notas técnicas: no incluye diff visual ni análisis de contenido.

### Subtareas

- [ ] Elegir política proporcional/página mediante prueba de usuario.
- [ ] Añadir guardas contra feedback de scroll.
- [ ] Validar pares de documentos heterogéneos.

## YAM-064 — Perfiles de lectura

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-009, YAM-024
- Objetivo: aplicar configuraciones coherentes por intención.
- Alcance: Lectura, Estudio, Presentación, Noche, eInk y Comparación.
- Criterios de aceptación: aplicación atómica; nombres accesibles; fallback;
  safe mode ignora personalizados; sin alterar el PDF.
- Validación: modelos/controlador y selector en ajustes.
- Archivos relacionados: `lib/features/profiles`, ajustes y shell.
- Riesgos o limitaciones: CRUD de perfiles personalizados queda pendiente.
- Notas técnicas: los built-ins son inmutables y no requieren esquema complejo.

### Subtareas

- [x] Definir y aplicar seis perfiles base.
- [ ] Añadir CRUD/duplicado/importación de perfiles personalizados.
- [ ] Probar combinaciones con densidad y movimiento reducido.

## YAM-065 — Modo estudio

- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-051, YAM-064
- Objetivo: concentrar índice, anotaciones y lectura en una sesión de estudio.
- Alcance: composición de tres zonas, atajos, sesión y recuperación.
- Criterios de aceptación: paneles colapsables; foco lógico; documento domina;
  una sola instancia por panel; restauración local.
- Validación: perfil Estudio y paneles existen, composición dedicada pendiente.
- Archivos relacionados: lector, perfiles, paneles y workspaces.
- Riesgos o limitaciones: tres zonas pueden degradar legibilidad en ancho medio.
- Notas técnicas: usar los paneles existentes, no duplicar controladores.

### Subtareas

- [ ] Diseñar composición expandida/compacta.
- [ ] Añadir sesión y atajos propios.
- [ ] Perfilar documento largo con ambos paneles.

## YAM-066 — Presentación endurecida

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-007, YAM-064
- Objetivo: aplicar una experiencia de presentación mínima y estable.
- Alcance: perfil, página única, chrome oculto, navegación y pantalla completa.
- Criterios de aceptación: salida evidente; controles por teclado; cursor/chrome
  recuperables; no modificar progreso accidentalmente si se desactiva.
- Validación: perfil, modo libro y pantalla completa automatizados.
- Archivos relacionados: perfiles, lector y servicio de ventana.
- Riesgos o limitaciones: keep-awake y ocultación temporal del cursor pendientes.
- Notas técnicas: no añadir plugin sin ADR y medición.

### Subtareas

- [x] Añadir perfil Presentación.
- [ ] Implementar cursor/keep-awake con API nativa evaluada.
- [ ] Ejecutar sesión manual multi-monitor.

## YAM-067 — Diagnóstico local seguro

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-011, YAM-034
- Objetivo: inspeccionar salud y rendimiento sin exponer contenido privado.
- Alcance: versión, plataforma, esquema, integridad, caché, conteos, motor,
  verificar, optimizar, limpiar y exportar informe.
- Criterios de aceptación: rutas enmascaradas; sin títulos/consultas/notas;
  acciones destructivas confirmadas; copia/exportación explícitas.
- Validación: test de redacción, integridad y diálogo integrado.
- Archivos relacionados: `lib/features/diagnostics`, ajustes y base.
- Riesgos o limitaciones: abrir carpeta delega en el sistema por acción manual.
- Notas técnicas: todos los campos exportables viven en allowlist.

### Subtareas

- [x] Implementar informe redaccionado y acciones.
- [x] Añadir copia/exportación y confirmaciones.
- [ ] Revisar reporte en todas las plataformas objetivo.

## YAM-068 — Modo seguro de arranque

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-040, YAM-067
- Objetivo: recuperarse de cierres repetidos sin bucle de restauración.
- Alcance: contador de arranques no limpios, umbral tres, preferencias por
  defecto y omisión de sesión.
- Criterios de aceptación: tercer arranque entra en safe mode; cierre limpio
  reinicia contador; banner/indicador; no borra datos.
- Validación: prueba de tres reinicios y composición del shell.
- Archivos relacionados: `AppDatabase`, `AppServices`, app y biblioteca.
- Riesgos o limitaciones: no distingue crash de cierre forzado por el usuario.
- Notas técnicas: estado mínimo en `app_metadata`, sin telemetría.

### Subtareas

- [x] Registrar arranque y cierre limpios.
- [x] Evitar restauración y temas personalizados en modo seguro.
- [ ] Añadir acción UI para salir/reintentar de forma explícita.

## YAM-069 — Integridad y migraciones recuperables

- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-038, YAM-067
- Objetivo: migrar datos privados de forma atómica y recuperable.
- Alcance: esquema v2, copia pre-migración, transacción, integrity check y
  compatibilidad de backup v1/v2.
- Criterios de aceptación: fallo revierte; copia previa existe; FK válidas;
  datos v1 sobreviven; índice reconstruible no se copia.
- Validación: migración v1→v2, backup y `PRAGMA integrity_check` automatizados.
- Archivos relacionados: `app_database.dart`, `local_backup_service.dart`.
- Riesgos o limitaciones: `VACUUM INTO` depende de espacio libre suficiente.
- Notas técnicas: nunca se reemplaza una DB sin validación posterior.

### Subtareas

- [x] Implementar migración transaccional y copia previa.
- [x] Extender backup y clear policies.
- [ ] Simular falta de espacio/corrupción en Windows.

## YAM-070 — Sistema visual de tercera generación

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-017, YAM-033
- Objetivo: integrar superficies avanzadas sin perder ligereza editorial.
- Alcance: tabs, panel de anotaciones, colecciones, estados compactos/oscuros,
  semántica y feedback.
- Criterios de aceptación: PDF domina; sin overflow; jerarquía clara; foco,
  tooltips y objetivos; cero paquete visual nuevo.
- Validación: 14 capturas reproducibles revisadas visualmente en Windows.
- Archivos relacionados: presentación, tokens, `docs/screenshots`.
- Riesgos o limitaciones: DPI 100/150/200 y Narrador siguen manuales.
- Notas técnicas: se reutilizan tokens y primitivas Material existentes.

### Subtareas

- [x] Capturar lector y biblioteca en estados nuevos.
- [x] Revisar ancha, filtrada, compacta y oscura.
- [ ] Completar matriz nativa de accesibilidad/DPI.

## YAM-071 — Ayuda contextual y privacidad visible

- Prioridad: P2
- Estado: REVIEW
- Dependencias: YAM-067
- Objetivo: explicar funciones sensibles desde su contexto.
- Alcance: ayuda en ajustes, índice local, backups, recuperación y diagnóstico.
- Criterios de aceptación: texto breve; riesgos antes de la acción; sin web
  obligatoria; accesible por teclado y lectores de pantalla.
- Validación: textos y navegación integrados en ajustes/diálogos.
- Archivos relacionados: ajustes, diálogos y `PRIVACY.md`.
- Riesgos o limitaciones: falta un centro de ayuda completo y localizable.
- Notas técnicas: no duplicar manuales extensos dentro de la UI.

### Subtareas

- [x] Añadir ayuda a acciones nuevas de mayor riesgo.
- [ ] Completar inventario de tooltips/semántica.
- [ ] Validar lenguaje con prueba de usuario.

## YAM-072 — Integración Windows y apertura múltiple

- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-015, YAM-053
- Objetivo: aceptar varios PDF desde argumentos/drop y registrarlos como
  contextos documentales suspendibles.
- Alcance: deduplicación, máximo acotado, primera activa y errores aislados.
- Criterios de aceptación: argumentos múltiples conservan orden; drop múltiple
  no bloquea; cada documento valida; no se abren conexiones.
- Validación: bootstrap unitario, integración de drop y build Windows.
- Archivos relacionados: bootstrap, shell, biblioteca y runner Windows.
- Riesgos o limitaciones: asociación de archivos/Jump List/menú contextual del
  instalador no están configurados.
- Notas técnicas: empaquetado y firma permanecen en YAM-016.

### Subtareas

- [x] Aceptar y deduplicar múltiples rutas.
- [x] Integrar apertura acotada en contextos internos.
- [ ] Configurar y validar asociaciones en paquete firmado.

## YAM-073 — Gate de rendimiento de funciones avanzadas

- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-048, YAM-053, YAM-057, YAM-069
- Objetivo: medir búsqueda, colecciones, workspaces, frames, RSS y bundle.
- Alcance: tres procesos sintéticos de 500 documentos y cinco pasadas Profile
  con PDF reales controlados.
- Criterios de aceptación: mediana/p95; integridad ok; cero handles pendientes;
  informe sin datos privados; Release y ausencia de TCP verificadas.
- Validación: tres JSON avanzados y `performance-profile.json`; falta otro
  equipo/GPU para convertir cifras en umbral.
- Archivos relacionados: `tool/benchmark_advanced_features.dart`,
  `integration_test/performance_profile_test.dart`, `PERFORMANCE.md`.
- Riesgos o limitaciones: un único equipo no demuestra ausencia de fugas.
- Notas técnicas: conservar métricas bajo `build/`, nunca rutas o títulos.

### Subtareas

- [x] Ejecutar tres procesos y cinco pasadas Profile.
- [x] Verificar integridad, frames, caché y liberación de recursos.
- [ ] Comparar en otro hardware antes de activar gate regresivo.

## YAM-074 — Biblioteca de artefactos y portadas inteligentes locales

- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-018, YAM-027, YAM-069, YAM-070
- Objetivo: sustituir la lista convencional por una colección espacial sobria
  sin perder apertura, progreso, organización, persistencia ni accesibilidad.
- Alcance: layout determinista, artefacto visual, navegación/foco, overlays,
  búsqueda temporal, filtros combinados, favorito, detección local de portada,
  dos miniaturas, selección manual, esquema v3 y emblema SVG.
- Criterios de aceptación: posiciones estables; cero solapamiento inválido;
  teclado/ratón/contextual completos; manual no sobrescrita; caché invalidada;
  importación no bloqueada; claro/oscuro/compacto sin overflow; build Windows.
- Validación: scoring, fallback, metadata, migración, 100 layouts, navegación,
  filtros, widgets, análisis, suite, integración visual y build; Narrador/DPI
  100/150/200 % requieren revisión nativa.
- Archivos relacionados: `features/library`, `thumbnail_service.dart`,
  `app_database.dart`, `document_repository.dart`, tokens, branding y pruebas.
- Riesgos o limitaciones: heurística sin comprensión semántica; perfil sostenido
  con cientos de portadas reales y matriz Narrador/DPI permanecen pendientes.
- Notas técnicas: seis previews a baja escala, cola serial cancelable, FNV
  estable, slivers desde siete objetos, cero API/IA/OCR/dependencia nueva.

### Subtareas

- [x] Implementar esquema v3, metadata y selección manual dominante.
- [x] Implementar scoring visual/textual, fallback y caché pequeña/grande.
- [x] Implementar artefactos, layout, navegación, overlays y acciones.
- [x] Reemplazar buscador permanente y franja de pestañas por overlays y filtro
  compacto combinable.
- [x] Añadir pruebas unitarias, repositorio, widgets y fixture visual.
- [ ] Completar Narrador y DPI nativo 100/150/200 % para pasar a `DONE`.

## YAM-075 — Ownership de render y perfiles de escritorio

- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-034, YAM-070, YAM-074
- Objetivo: cerrar invalidacion de imagenes cacheadas, cancelacion de trabajos
  obsoletos y geometria segura para escritorios habituales.
- Alcance: `RenderedPdfPage` prestado, cache LRU por bytes, tokens nativos,
  miniaturas cancelables, historial con zoom/offset, `Ajustar ancho` top-align,
  compactos sin overflow y perfiles 16:9/16:10/1920x1080/2560x1440.
- Criterios de aceptacion: ningun consumidor dispone imagenes del motor;
  cancelar no publica resultados obsoletos ni deja PNG parcial; historial y
  resize conservan unidad/zoom/offset; columnas y slots caben en la ventana.
- Validacion: formato, analisis de modulos independientes, pruebas nuevas y
  `git diff --check` aprobados. Suite Flutter/Pub, build Windows, capturas,
  Narrador y DPI nativo requieren desbloquear dependencias/runner.
- Archivos relacionados: `pdf_engine`, `reader`, `features/library`, tokens y
  `test/pdf_operation_token_test.dart`.
- Riesgos o limitaciones: falta ejecutar el runner nativo en este entorno y no
  se debe marcar `DONE` solo con pruebas de widget.

### Subtareas

- [x] Separar imagen prestada de la propiedad de cache y liberar excedentes.
- [x] Propagar cancelacion a pdfrx y limpiar trabajos/archivos obsoletos.
- [x] Cubrir layout compacto, Full HD/2K, viewport y shortcuts reservados.
- [ ] Ejecutar suite Flutter completa, build Windows y diez capturas nuevas.
- [ ] Completar Narrador y DPI nativo 100/125/150/200 % para pasar a `DONE`.
