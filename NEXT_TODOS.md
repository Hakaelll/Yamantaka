# Próximas tareas

Documento operativo principal. `DONE` solo significa que sus criterios fueron
ejecutados o inspeccionados. `REVIEW` indica implementación terminada pendiente
de validación nativa o manual.

## YAM-001 — Inspección inicial

- Descripción: inspeccionar el repositorio, instrucciones y toolchains.
- Prioridad: P0
- Estado: DONE
- Dependencias: ninguna
- Criterios de aceptación: estructura e instrucciones inspeccionadas; estado de
  Flutter, Windows y Visual Studio registrado.
- Archivos relacionados: `README.md`, `NEXT_TODOS.md`
- Notas técnicas: repositorio inicial sin aplicación; se instaló un SDK Flutter
  local ignorado por Git.

## YAM-002 — Decisiones y diseño

- Descripción: definir motor, persistencia, estado, paquetes y sistema visual.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-001
- Criterios de aceptación: alternativas, licencias, riesgos, arquitectura y
  diseño documentados antes de completar la interfaz.
- Archivos relacionados: `DECISIONS.md`, `DESIGN.md`, `ARCHITECTURE.md`
- Notas técnicas: `pdfrx`/PDFium queda detrás de contratos propios.

## YAM-003 — Esqueleto multiplataforma

- Descripción: crear Flutter Windows/Android/iOS y servicios base.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-002
- Criterios de aceptación: configuración central, tema, logging, errores,
  composición y análisis estático sin fallos.
- Archivos relacionados: `lib/app`, `lib/core`, `windows`, `android`, `ios`
- Notas técnicas: `flutter analyze` aprobado el 2026-07-28.

## YAM-004 — SQLite y biblioteca

- Descripción: implementar persistencia local y biblioteca ligera.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003
- Criterios de aceptación: cinco tablas, migración, recientes sin duplicados,
  ausentes, continuar, progreso y portada diferida probados.
- Archivos relacionados: `lib/storage`, `lib/features/library`,
  `test/database_repositories_test.dart`, `test/library_widgets_test.dart`
- Notas técnicas: ruta normalizada más tamaño, fecha y hash parcial; las
  portadas se invalidan si cambia la huella.

## YAM-005 — Apertura y motor PDF

- Descripción: selector, drop, argumento y operaciones reales de PDFium.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003
- Criterios de aceptación: PDF real abre y renderiza; texto/búsqueda funcionan;
  la primera página se muestra en Windows; cierre libera el documento.
- Archivos relacionados: `lib/pdf_engine`, `lib/features/reader`,
  `test/pdf_engine_real_test.dart`, `integration_test/reader_integration_test.dart`
- Notas técnicas: integración Windows aprobada con bitmap, `ui.Image`, lectura
  de píxeles, pintura del visor y cierre. El parche local de pdfrx queda
  registrado en YAM-014.

## YAM-006 — Navegación y restauración

- Descripción: navegación, zoom, disposiciones y posición persistente.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004, YAM-005
- Criterios de aceptación: anterior/siguiente/ir a página, zoom, ajustes,
  continuo/único, rotación y restauración de página/zoom/offset.
- Archivos relacionados: `lib/features/reader`,
  `test/reader_widgets_test.dart`, `integration_test/reader_integration_test.dart`
- Notas técnicas: widgets verifican persistencia de zoom/offset; integración
  nativa verifica zoom, avance, retroceso, cierre, reapertura, página y
  marcador.

## YAM-007 — Modo libro

- Descripción: experiencia de lectura silenciosa y adaptable.
- Prioridad: P0
- Estado: REVIEW
- Dependencias: YAM-006
- Criterios de aceptación: una/dos páginas, portada sola, LTR/RTL, seis fondos,
  márgenes, controles ocultables, progreso, laterales y pantalla completa.
- Archivos relacionados: `lib/features/reader/presentation`,
  `test/reader_models_test.dart`, `test/reader_widgets_test.dart`
- Notas técnicas: los widgets y emparejado están probados; falta inspección
  visual nativa a varios DPI y anchos. La captura Windows verifica que la
  portada queda sola en vista enfrentada.

## YAM-008 — Búsqueda, panel y marcadores

- Descripción: búsqueda local, navegación auxiliar y marcadores.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004, YAM-005
- Criterios de aceptación: búsqueda cancelable, anterior/siguiente, estado sin
  texto, panel de una sección y marcador persistente.
- Archivos relacionados: `lib/features/search`, `lib/features/bookmarks`,
  `lib/features/reader/presentation/reader_side_panel.dart`
- Notas técnicas: búsqueda PDFium y persistencia SQLite verificadas; integración
  nativa de restauración incluida para YAM-012.

## YAM-009 — Comandos, atajos y ajustes

- Descripción: entrada centralizada y configuración mínima.
- Prioridad: P1
- Estado: DONE
- Dependencias: YAM-006
- Criterios de aceptación: registro central, atajos solicitados, `Ctrl+K`,
  ajustes, caché configurable, movimiento reducido y sesiones desactivables.
- Archivos relacionados: `lib/app/shortcuts`,
  `lib/features/command_palette`, `lib/features/settings`
- Notas técnicas: incluye dirección de lectura y perfil inicial de tinta
  electrónica como valores tipados, persistencia de idioma y geometría de
  ventana.

## YAM-010 — Pruebas automáticas

- Descripción: cubrir dominio, persistencia, widgets y flujo PDF nativo.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-004 a YAM-009
- Criterios de aceptación: unitarias y widgets aprobados; integración Windows
  abre, muestra, navega, cierra, reabre, restaura y recupera marcador.
- Archivos relacionados: `test`, `integration_test`, `TESTING.md`
- Notas técnicas: 34 pruebas estándar y la integración nativa Windows
  aprobadas; la captura verifica píxeles reales de la superficie Flutter.

## YAM-011 — Rendimiento y caché

- Descripción: limitar recursos y preparar medición reproducible.
- Prioridad: P1
- Estado: REVIEW
- Dependencias: YAM-005
- Criterios de aceptación: LRU por bytes probada, render bajo demanda,
  deduplicación/cancelación inspeccionadas y métricas Release ejecutadas.
- Archivos relacionados: `lib/pdf_engine/cache`, `PERFORMANCE.md`
- Notas técnicas: caché, cancelación activa de render y logs de desarrollo
  operativos; build Release y smoke medido disponibles. Mediana/p95, fluidez y
  memoria de un PDF largo siguen pendientes.

## YAM-012 — Validación Windows

- Descripción: validar el primer hito en el destino principal.
- Prioridad: P0
- Estado: DONE
- Dependencias: YAM-003 a YAM-010
- Criterios de aceptación: `flutter doctor -v`, análisis, tests, integración,
  build Release y smoke manual del ejecutable aprobados.
- Archivos relacionados: todo el repositorio
- Notas técnicas: Modo de desarrollador activo; doctor aprueba Build Tools
  2022 17.14.37 y Windows SDK 10.0.26100.0. Formato, análisis, 34 pruebas,
  integración, captura, Release, argumento PDF, proceso receptivo, SQLite
  local, cero conexiones TCP y cierre limpio aprobados. La matriz ampliada se
  separa en YAM-015.

## YAM-013 — Validación móvil

- Descripción: instalar toolchain Android y comprobar diseño táctil; iOS en macOS.
- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-012
- Criterios de aceptación: build, ejecución y pruebas de almacenamiento/entrada
  por plataforma.
- Archivos relacionados: `android`, `ios`, `ROADMAP.md`
- Notas técnicas: `flutter doctor -v` no encuentra Android SDK; iOS requiere
  macOS/Xcode.

## YAM-014 — Retirar parche local de pdfrx

- Descripción: volver al paquete hospedado cuando upstream corrija el codec
  asíncrono de imágenes en Windows.
- Prioridad: P1
- Estado: TODO
- Dependencias: nueva versión de pdfrx
- Criterios de aceptación: eliminar `third_party/pdfrx`, restaurar la
  dependencia hospedada y aprobar análisis, 34 pruebas, integración Windows,
  captura de píxeles y build Release.
- Archivos relacionados: `third_party/pdfrx`, `pubspec.yaml`, `pubspec.lock`,
  `DECISIONS.md`
- Notas técnicas: el parche MIT mantiene el codec hasta `getNextFrame()` y
  reporta errores de preview que upstream 2.4.7 descarta silenciosamente.

## YAM-015 — Endurecimiento Windows ampliado

- Descripción: completar la matriz manual y de rendimiento más allá del primer
  hito.
- Prioridad: P1
- Estado: TODO
- Dependencias: YAM-011, YAM-012
- Criterios de aceptación: probar PDF escaneado/grande/Unicode/ruta larga,
  selector y drop, todos los modos/atajos, DPI 100/125/200 %, lector de
  pantalla, limpieza/relocalización y memoria al recorrer/cerrar 100 páginas;
  registrar equipo, build, documentos, mediana y p95 sin falsear resultados.
- Archivos relacionados: `TESTING.md`, `PERFORMANCE.md`, `docs/screenshots`
- Notas técnicas: corrupto, vacío y contraseña ausente/errónea/correcta ya
  tienen cobertura automática real; el smoke Release pequeño ya está medido.

## YAM-016 — Empaquetado y firma de Windows

- Descripción: preparar una distribución verificable para usuarios.
- Prioridad: P2
- Estado: TODO
- Dependencias: YAM-015
- Criterios de aceptación: elegir MSIX o instalador ligero, fijar identidad y
  actualizaciones explícitas, firmar binarios/paquete, probar instalación,
  desinstalación y conservación/eliminación optativa de datos.
- Archivos relacionados: `windows`, `README.md`, `DECISIONS.md`, `PRIVACY.md`
- Notas técnicas: el bundle Release actual ocupa 39,0 MiB en 18 archivos y no
  está firmado; es un artefacto de desarrollo, no una publicación.
