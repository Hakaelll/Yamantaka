# Changelog

## 0.17.0 — 2026-11-07

- Add contextual highlight, underline, copy and note actions when selecting PDF text.
- Add a Study toolbar for repeated marking, freehand drawing, arrows, rectangles and erasing. The eraser previews its target and protects written notes.
- Add page markers that open the matching note, page filtering and a direct Pending filter for review.
- Refine the library, reader and settings with more consistent controls, lighter menus and compact layouts.
- Include drawings in local v9 backups, with continued support for older libraries and backups.

## 0.16.0 — 2026-11-04

- Replace the active Flutter project with the native Rust Windows application.
- Preserve immersive 3D covers, local PDF reading, study and organization workflows.
- Keep SQLite v11, historical backup compatibility and independent reviewed migration.
- Add protected cover batches, robust metadata scheduling, authorized folder notifications, visible-page search highlights and Unicode passage selection.
- Archive Flutter before cutover and publish a complete portable Windows bundle with hashes, licenses and SBOM.
- Record the owner's final acceptance without claiming new agent-run tests or numerical performance measurements.


## 0.15.0 — 2026-09-07 (acabado editorial profesional)

- Unifica Biblioteca, lector y Ajustes en temas claro y oscuro compartidos,
  separando la apariencia de la interfaz del fondo del documento.
- Recompone Biblioteca con navegación lateral adaptativa, búsqueda etiquetada,
  continuidad contenida, recientes secundarios y portadas tipográficas estables.
- Mejora legibilidad del navegador y controles del lector; conserva filtros
  de anotaciones alcanzables a 640×480 y texto al 200 %.
- Amplía la vista previa de Ajustes y acota su composición ultrapanorámica.
- Pliega la sesión reciente para despejar el catálogo y mantiene sus acciones
  alcanzables con texto ampliado; corrige etiquetas y menús al 200 %.
- Traduce evidencias de duplicados y aísla elecciones por candidato y campo.
- Mantiene integridad documental, funcionamiento offline, SQLite v11 y marca.
- La evidencia y el estado de validación se registran en
  `docs/architecture/visual-iteration-0.15.md`; la release es solo local y sin firma.

## 0.14.0 — 2026-09-03 (coherencia visual y UX de escritorio)

- Consolida tamaños de icono, alturas de control, bordes, opacidades y curvas
  en tokens compartidos, sin alterar breakpoints ni la geometría de PDFium.
- Añade un marco adaptativo para diálogos críticos y un estado operativo común
  con fase, progreso, live region y cancelación accesible.
- Refina contraseña, recuperación de sesión, restore transaccional, duplicados
  y reglas locales; la garantía de contraseña en memoria permanece visible
  también tras un intento incorrecto y con texto al 200 %.
- Reorganiza Ajustes en Apariencia, Lectura, Accesibilidad, Biblioteca,
  Privacidad, Almacenamiento, Recuperación y Diagnóstico, en español e inglés.
- Añade contratos de componentes, taxonomía, RTL, alto contraste, movimiento
  reducido, teclado, semántica y los ocho tamaños de revisión; la suite alcanza
  314 pruebas.
- Genera 185 PNG ejecutados, 20 hojas de contacto y diez comparaciones 0.13/0.14
  con hashes; la marca permanece byte-idéntica a su referencia.
- La comparación de cinco series por familia termina `pass-with-watch`: cero
  regresiones fiables, cinco observaciones ruidosas y nueve invariantes sanos;
  cancela una recarga diferida, acelera snapshots y elimina ripple prolongado.
- Eleva a 300 s el timeout externo del gate porque el análisis limpio supera de
  forma reproducible los 180 s en este host.
- Completa las 14 fases del gate Windows sin omisiones, incluido smoke con cero
  conexiones y matriz nativa a los 120 DPI/125 % reales del monitor disponible.
- Prepara el release local `0.14.0+15` sin commit, tag, push, firma ni
  publicación.

## 0.13.0 — 2026-09-02 (resiliencia documental y confianza local)

- Formaliza el ciclo de apertura con generación, cancelación y estados tipados;
  limita las contraseñas a tres intentos en memoria y bloquea la restauración
  automática del documento que provocó un fallo.
- Amplía la sesión recuperable con huella, página, total, estado y cuarentena;
  verifica identidad antes de reabrir y conserva la relocalización exacta.
- Migra SQLite a v11 con procedencia de anotaciones, cuarentena documental,
  journal por fases y registro reversible de consolidaciones.
- Eleva el backup local a v9 con manifiesto, ensayo y plan detallado de altas,
  reemplazos, eliminaciones y conflictos, pre-backup y rollback transaccional.
- Añade revisión de duplicados exactos/probables con evidencias, impacto,
  decisiones por conflicto, mantener ambos, consolidación completa y undo; los
  PDF nunca se escriben ni se eliminan.
- Añade fixtures reales e idempotentes de migración v1–v11, carreras de
  apertura/cierre, interrupciones de restore, rutas Unicode/acotadas y 305
  pruebas; la matriz ejecutada reúne 179 PNG en 19 hojas y diez comparaciones.
- Compara cinco series avanzadas, cinco de Biblioteca y cinco nativas por
  versión; la serie 0.12/0.13 emparejada termina `pass` sin fallos, observaciones
  ni invariantes rotos y conserva aparte la divergencia histórica del host.
- Genera un SBOM CycloneDX 1.5 offline con versiones, hashes, licencias y
  procedencia de paquetes y binarios nativos.
- Prepara el release local `0.13.0+14` sin commit, tag, push, firma ni
  publicación.

## 0.12.0 — 2026-08-31 (continuidad, revisión y recuperación)

- Rediseña el inicio como centro compacto de continuidad con lectura activa,
  pestañas, fijados, pendientes, espacios, problemas y acciones de reanudación.
- Amplía espacios a vistas completas de Biblioteca y añade cola manual con
  estados, bloqueo, siguiente acción, reordenado por teclado/puntero y undo de
  eliminación; el modo Foco se recuerda por espacio.
- Añade comparación local lado a lado, sincronía opcional de página/zoom,
  intercambio de lados, marcas manuales y liberación determinista del segundo
  handle.
- Convierte anotaciones en una revisión global paginada con filtros, agrupación,
  edición inline, selección masiva, estados editoriales, etiquetas y exportación
  Markdown/JSON.
- Añade relaciones, backlinks y grupos de revisiones con selección explícita de
  revisión actual, sin fusionar ni borrar archivos.
- Introduce reglas locales allowlist con simulación, confirmación, cancelación,
  límites de 100.000 elementos, lotes de 80 y 30 segundos, journal, rollback y
  deshacer; no admite scripts ni red.
- Migra SQLite a v10, elimina N+1 en espacios/anotaciones/revisiones, añade
  sesión atómica, integridad y centro de problemas no modal.
- Añade caché LRU de ocho búsquedas recientes con TTL de un segundo e
  invalidación por mutación; la mediana caliente medida baja de 592 a 28 µs.
- Amplía a 287 pruebas y 172 capturas verificadas en 18 hojas de contacto; las
  cinco pasadas terminan `pass-with-watch`, sin regresión fiable ni invariantes
  fallidos.
- Prepara el release local `0.12.0+13` con manifiesto, hashes, informes, smoke y
  estado de firma, sin commit, tag, push, firma ni publicación.

## 0.11.0 — 2026-08-28 (navegación documental y continuidad)

- Añade un navegador unificado con `Ctrl+J` para índice, miniaturas, búsqueda
  interna, marcadores, anotaciones, historial y posiciones A/B; filtra todas
  las fuentes, usa lista + detalle en expandido y lista → detalle → retorno en
  compacto, conserva zoom y permite volver al origen con `Alt+←`.
- Conserva el panel lateral en `Ctrl+Mayús+J` y mantiene navegador, panel,
  chrome y breakpoints fuera de la identidad cacheada del visor PDF.
- Persiste contexto rico de sesión y documentos cerrados recientemente en el
  almacén local incluido en backup; restaura página, zoom, layout, modo, tema,
  scroll y sección del panel, limita a tres documentos fijados y añade acciones
  Continuar, Releer y Terminado.
- Expone `Cerrar sesión y conservar contexto` como acción inequívoca del lector
  y restaura el snapshot desde la banda de documentos cerrados recientemente.
- Añade metas locales opcionales por páginas o minutos, visibles como progreso
  tipográfico discreto, sin cuentas, nube, avisos ni gamificación.
- Conecta la selección real de `pdfrx` con resaltado, subrayado y tachado:
  conserva texto, contexto y rectángulos normalizados; una selección multipágina
  o sin geometría fiable degrada de forma explícita a nota de página.
- Generaliza la agrupación de Biblioteca por autor, colección, etiqueta o estado
  de lectura, con secciones plegables, selección y scroll estables, una sola
  barra masiva, consultas SQL acotadas y máximo de 80 filas materializadas.
- Cancela portadas recreables cuando una tarjeta virtualizada abandona el
  viewport y publica contadores locales redactados de decodificaciones
  iniciadas y evitadas.
- Reanuda el índice FTS5 desde un cursor persistente, preserva páginas ya
  completadas, reajusta prioridad para documento/página visibles y mantiene una
  cola secundaria acotada de documentos fijados o recientes.
- Separa métricas locales de búsqueda fría, caliente, filtrada y cancelada sin
  registrar consultas, rutas, títulos ni contenido del PDF.
- Amplía pruebas, matrices visuales y benchmarks comparables con la base 0.10;
  cubre también el diálogo de importación, prioridades cambiadas durante un
  índice activo y consultas solapadas; genera el release local `0.11.0+12` sin
  firma ni publicación.

## 0.10.0 — 2026-08-27 (continuidad y orientación local)

- Añade una franja condicional de Continuar leyendo con página, total,
  porcentaje, última apertura, acción primaria, Enter y menú contextual.
- Añade conmutador compacto de documentos con Ctrl+Tab/Ctrl+Shift+Tab y una
  política de dos handles PDFium calientes como máximo; los documentos
  suspendidos conservan página, zoom, layout, modo, scroll, tema y panel, y
  liberan recursos nativos. Ctrl+Enter abre un resultado en otra pestaña.
- Amplía Ir a página con número, porcentaje, inicio/fin, scrubber, historial,
  validación inline, Escape y retorno de foco.
- Amplía FTS5 con filtros por documento, texto, anotación, marcador, colección
  y etiqueta, relevancia con recencia como desempate, resaltado y estados
  cancelado, ausente y obsoleto; en compacto navega de lista a detalle con
  retorno predecible y el debounce evita refrescar fuentes derivadas por tecla.
- Mantiene el resultado por archivo en la cola de importación; cada duplicado
  permite abrir el existente, comparar huella/tamaño/fecha o ignorarlo, sin
  mover ni copiar PDF.
- Combina anotaciones y marcadores en un filtro ordenado por página; saltar a
  un resultado anuncia y marca temporalmente el destino sin modificar el PDF.
- Añade búsqueda de preferencias, alcance global explícito y reset por sección
  antes de las acciones de borrado total.
- Instrumenta bootstrap, base de datos, primera Biblioteca, primer PDF,
  consultas de Biblioteca y búsqueda con métricas locales sin datos privados.
- Prioriza en la cola incremental el documento y la página visibles, y mantiene
  la pausa de tareas no críticas durante interacción o ventana minimizada.
- Desacopla el repositorio de espacios de trabajo de los tokens Flutter para
  recuperar el benchmark Dart puro y mantener las restricciones en el dominio.
- Conserva el contrato visual 0.9, adapta paleta y previews a texto 150/200 % y
  genera golden 0.10, comparaciones 0.9/0.10 y release local `0.10.0+11` sin
  publicar ni firmar.

## 0.9.0 — 2026-08-26 (lector compacto de precisión)

- Reduce la barra superior del lector a 56 px en escritorio y 52 px en
  compacto, con targets de 40 px, iconos de 18 px, margen superior de 8 px y
  título real truncado; atrás/adelante solo aparecen cuando están disponibles.
- Limita la dock activa a 48 px y su reposo a 38 px, conserva navegación
  geométricamente centrada y mueve las acciones secundarias al menú compacto.
- Reduce rail y cabecera del panel a 52 px, acorta sombras de papel y chrome y
  adapta los márgenes a 12/16/24/32 px dentro del visor sin añadir tamaño,
  breakpoint, panel ni visibilidad del chrome a su identidad cacheada.
- Compacta cabeceras, filas de Ajustes, resultados y paleta; unifica estas
  últimas en 48 px y reduce radios de paneles y diálogos.
- Conserva selección, filtros y scroll al alternar cuadrícula/lista mediante una
  transición breve de opacidad, inmediata con movimiento reducido; las filas
  usan 52 px a texto 100 % y crecen desde 64 px por encima de 130 %.
- Añade aserciones geométricas para barra, dock, targets y estabilidad del visor,
  actualiza la golden 0.9 y prepara evidencia 0.8/0.9 y release local
  `0.9.0+10` sin publicar ni firmar.

## 0.8.0 — 2026-08-26 (archivo privado y lectura de precisión)

- Convierte Biblioteca en un archivo de escritorio con cuadrícula titulada y
  lista densa; ambas conservan selección, navegación, inspector y estados de
  ausencia, modificación, revisión y progreso.
- Expone una banda eliminable de filtros activos y adapta la lista al 150 % de
  texto sin comprimir columnas auxiliares hasta volverlas ilegibles.
- Reduce el estado inicial a la marca Yamantaka, una frase breve y `Abrir PDF`;
  recolorea píxel a píxel el símbolo original de azul a violeta `#B58AFF`, sin
  regenerar geometría, relieve, textura, alfa ni tamaños de icono.
- Refina barra, dock y panel del lector con radios de instrumento más sobrios,
  elimina el falso estado seleccionado del selector de tema y muestra la
  disposición de página actual como contexto sin cambiar la identidad del
  visor PDF.
- Eleva búsqueda global a lista + preview en expandido, fortalece selección y
  estados de carga, y normaliza iconos, atajos y acciones destructivas en
  paleta, menús, anotaciones y diálogos.
- Hace explícitos en metadatos el valor detectado, el override local, los cambios
  pendientes y la restauración por campo; guardar continúa siendo local y no
  modifica el PDF.
- Separa zonas peligrosas y errores recuperables en Ajustes, conserva una sola
  acción primaria por superficie y evita mostrar excepciones técnicas.
- Amplía la suite a 252 pruebas, mantiene una golden 0.8, genera 54 capturas y
  diez comparaciones 0.7/0.8, y prepara el release local `0.8.0+9` sin publicar
  ni firmar automáticamente.

## 0.7.0 — 2026-08-25 (retícula editorial de escritorio)

- Define escala tipográfica editorial, cifras tabulares y vocabulario completo
  de superficies normal, hover, pressed, selected, focused, disabled y loading.
- Recompone Biblioteca con cabecera jerárquica, contador explícito, número y
  reset visible de filtros, estados icónicos de portada e inspector por columnas
  con `Abrir`, `Continuar`, `Releer` o `Localizar`.
- Une navegación, zoom, ajuste, tema y marcador en una dock única; cohesiona la
  barra superior y refuerza rail, cabecera y resize del panel sin reconstruir ni
  desplazar el visor PDF.
- Agrupa Ajustes en preferencias, mantenimiento y privacidad; separa acciones
  destructivas y permite alternar reposo, activo, panel y e-ink en una preview
  local que no abre PDFium.
- Añade iconos específicos, metadatos y preview textual expandida a la paleta;
  conserva Enter, Escape, retorno de foco y separación de documentos recientes.
- Amplía pruebas a 247 casos y añade matrices reales de widgets para paleta,
  menús y diálogos, además de la golden normativa 0.7.
- Prepara el release local `0.7.0+8` sin publicación ni firma automática.

## 0.6.0 — 2026-08-24 (sala de lectura digital)

- Convierte la selección de Biblioteca en un inspector editorial responsive con
  título, autor, estado, progreso, última lectura y acción primaria contextual;
  favorito e información son las únicas acciones secundarias visibles.
- Conserva la selección al filtrar y explica cuándo el documento deja de formar
  parte del resultado, con recuperación directa y cierre por Escape; en compacto
  la superficie admite descarte gestual.
- Introduce chrome progresivo en el lector: el reposo mantiene una dock fina de
  posición/progreso y una interacción de teclado, toque o puntero revela
  navegación y zoom sin cambiar la geometría ni la clave del visor PDF.
- Mantiene las seis muestras accesibles de tema y añade rail vertical para las
  cinco secciones del panel acoplado, con feedback de ancho en hover, foco,
  teclado y arrastre.
- Eleva Ajustes a master-detail con previsualización viva persistente que
  reutiliza controles reales y responde a canvas, tema, densidad, chrome,
  progreso, movimiento y perfil e-ink.
- Amplía contratos de selección, dock en reposo, panel y preview; genera matrices
  0.6.0 comparables con la evidencia 0.5.0 y conserva la golden normativa del
  lector.
- Prepara el paquete local `0.6.0+7` con ZIP versionado, manifiesto por archivo y
  `SHA256SUMS.txt`; no publica y mantiene explícito el estado de firma.

## 0.5.0 — 2026-08-24 (refinamiento editorial y panel adaptativo)

- Consolida tamaños, sombras y duraciones semánticas para chrome, portadas,
  ajustes, menús y paleta, respetando movimiento reducido y alto contraste.
- Mantiene Biblioteca en `#080808`, convierte “Abrir PDF” en acción primaria,
  expone un restablecimiento de filtros visible y suaviza escala, elevación,
  halo y atenuación sin perder el violeta `#B58AFF` ni el foco blanco.
- Hace que el panel del lector reste siempre su ancho y recentre el PDF en
  ventanas expandidas; en compacto/medio continúa como overlay con scrim. El
  resize, pin, cierre, persistencia, foco y montaje de una sola sección se
  conservan sin reconstruir el visor.
- Refina Ajustes con navegación de escritorio sobria, contenido legible y una
  columna estable de controles; separa Comandos y Documentos recientes en la
  paleta con filas de altura compartida.
- Amplía contratos widget y matrices visuales desde 640×480 hasta 3440×1440,
  añade escenarios de panel overlay/acoplado y eleva la suite a 244 pruebas.
- Publica el paquete local de prueba `0.5.0+6`; los binarios nativos permanecen
  sin firma y no constituyen una distribución pública.

## 0.4.0 — 2026-08-23 (renderizado PDF progresivo y estable)

- Presenta cada página primero con una previsualización limitada a 110 DPI y
  refina después de 75 ms de reposo, con transición corta desactivada cuando
  el usuario reduce el movimiento y sin destellos blancos entre bitmaps.
- Introduce generaciones de render, cancelación por consumidor y prioridades
  visible/próxima/prefetch/miniatura; un resultado obsoleto nunca entra en
  caché y cerrar un PDF espera los trabajos nativos antes de liberar PDFium.
- Separa presupuestos LRU de páginas, regiones y texto, limita dimensiones y
  bytes de cada bitmap, responde a presión de memoria y expone diagnóstico
  local agregado de p50/p95, frames lentos, caché y renders progresivos.
- Virtualiza la vista continua mediante búsqueda binaria del rango próximo,
  prioriza páginas visibles, cancela trabajo fuera del anillo y pausa índice y
  enriquecimiento durante la interacción activa con el lector.
- Mejora la detección local de título, autor y hasta tres temas usando
  metadatos, cuatro páginas iniciales, títulos multilínea, autorías/editorías
  explícitas, nombres de archivo legibles y vocabularios ES/EN más específicos.
- Añade “Volver a analizar” para un documento o toda la biblioteca; conserva
  siempre los overrides manuales y excluye archivos ausentes.
- Amplía fixtures, pruebas de caché/cancelación/privacidad/enriquecimiento,
  documentación, benchmark y empaquetado reproducible de Windows.

## 0.3.0 — 2026-08-23 (biblioteca inteligente y lectura flexible)

- Enriquece cada PDF localmente con título y autor desde metadatos o portada y
  hasta tres temas reproducibles en español e inglés, con límites de 12 páginas
  y 64 KiB, cancelación, reintento y revisión de resultados dudosos.
- Añade filtros combinables y recuentos completos por autor, tema, estado y
  favorito, además de agrupación alfabética por autor y edición manual de temas.
- Eleva SQLite a esquema 8 y el backup local a formato 7, conservando sugerencias,
  procedencia/confianza, huella, versión de analizador y ajustes de pliego.
- Amplía la barra de progreso a 4 px: gris/blanca en documentos normales y
  violeta únicamente en el documento seleccionado.
- Sustituye Documento/Libro por los comandos Una página, Dos páginas y Continua
  dentro de la paleta `Ctrl+K`, sin un selector permanente y con persistencia
  por PDF.
- Incorpora Ajustar pliego para portada sola/emparejada, primera página a
  izquierda/derecha y restablecimiento de la recomendación.
- Abre la ventana maximizada sin perder la geometría normal almacenada y
  mantiene F11 como fullscreen real independiente.
- Actualiza pruebas, documentación y referencia visual de Windows; no añade
  OCR, IA, cuentas, red ni firma de binarios.

## 0.2.1 — 2026-08-23 (biblioteca monocromática)

- Sustituye la paleta azul-negra de Biblioteca por negros, blancos y grises
  neutros en fondos, superficies, texto, iconos, bordes y controles.
- Reserva el violeta `#B58AFF` exclusivamente para el borde, progreso y halo
  luminoso del archivo seleccionado; el resto de acciones conserva contraste
  monocromático y un foco blanco independiente del color.

## 0.2.0 — 2026-08-23 (revamp visual y paridad de Libro)

- Consolida un sistema visual profesional con botones circulares, controles
  segmentados en cápsula, grupos flotantes, paneles y diálogos de radios
  jerárquicos, sin nuevas dependencias ni efectos de blur.
- Preserva la biblioteca como una constelación oscura inspirada
  conceptualmente en navegadores de partidas de consola: portadas protagonistas,
  espacio negativo, barra superior limpia y selección con halo cian accesible.
- Recompone el chrome del lector en tres grupos flotantes, convierte los
  controles inferiores en una dock ligera y suaviza paneles, menús y estados.
- Da paridad real a Documento y Libro: el selector de tema está disponible en
  ambos modos y el lienzo, el pliego y el chrome respetan los seis entornos de
  lectura en layouts compactos y expandidos.
- Conserva portada individual, lado inicial y RTL al entrar o restaurar Libro;
  añade un canal central estrecho, semántica de pliego y ajuste sobre la unidad
  completa sin degradar silenciosamente a Documento.
- Añade navegación interna al panel lateral sin reconstruir el visor, limpia
  tarjetas redundantes de Ajustes y Ayuda y actualiza la referencia visual de
  Libro junto con pruebas de paridad, accesibilidad y responsive.
- Retira del bundle el emblema raster de 1254×1254 (2.089.291 bytes), que no
  tenía ninguna referencia en código, tests ni integración.
- Elimina el antiguo `ContinueReadingCard` y tres wrappers Yama sin consumidores
  después de verificar referencias y sustituir sus responsabilidades por las
  superficies activas del sistema.
- Endurece el gate de Windows para conservar la caché Pub aislada, aceptar
  rutas absolutas, cerrar capturas limpiamente y activar Documento/Libro por su
  control visible; aprueba 206 pruebas, build Release, smoke sin red, benchmark
  de 5.000 documentos y matriz nativa a escala real del 100 %.

## 0.1.0-dev — 2026-08-02 (lector editorial y fullscreen real)

- Sustituye el fullscreen de Windows por una transición Win32 sin frame que
  conserva y restaura estilo, monitor y geometría normal.
- Rehace Libro para abrir con 1–2 enfrentadas, avanzar por pliegos completos y
  mantener un lomo ligado a las páginas durante zoom y paneo.
- Elimina el velo superior transparente, centra selector y navegación respecto
  a toda la ventana y ordena las acciones de vista en un grupo independiente.
- Normaliza defaults y preferencias antiguas en dominio, SQLite y backups; el
  cierre del lector restaura fullscreen antes de liberar el documento.
- Mejora el menú contextual de biblioteca por teclado, el tratamiento de
  archivos ausentes y la resolución de portada seleccionada.
- Añade una referencia visual mantenida por prueba, corrige el gate y la matriz
  de capturas, y retira 57 imágenes y documentos generacionales obsoletos.
- Separa ejecución local y distribución: rechaza bundles incompletos,
  descargados o externos al árbol de build, y reserva `-RequireSignature` para
  validar todos los ejecutables y DLL con Authenticode; compilar o firmar no
  publica la app.
- Aprueba análisis, 204 pruebas y build Release de Windows. El arranque local se
  revalidó el 8 de agosto: la ventana permaneció receptiva y cerró normalmente;
  los nueve binarios siguen sin firma y no se presentan como release distribuible.

## 0.1.0-dev — 2026-08-01 (gates, ownership y almacenamiento local)

- Añade un coordinador único de trabajos con generaciones, prioridades,
  deduplicación y cancelación compartida para importación, portadas e índice.
- Endurece lifecycle, cuotas de memoria, apertura progresiva, seguridad de
  contraseñas/enlaces y diagnóstico local redactado.
- Añade manifest/hash de backup v5, escritura recuperable con archivo previo,
  notas locales por documento y límites de tamaño,
  catálogo de almacenamiento con limpieza explícita y modo portable.
- Añade protocolo de instancia única Windows, canal de activación, matriz
  visual, corpus sintético, benchmark, soak y empaquetado reproducible.
- Añade cola de importación persistente, observación de carpetas autorizadas,
  resolución transaccional de duplicados, edición local de metadatos e índice
  FTS5 que refleja los overrides.
- La suite Flutter/PDFium y los gates nativos siguen pendientes mientras no haya
  una caché de Pub completa y un host Windows con toolchain/certificado.
- Centraliza la presentación segura de `FailureKind`, corrige textos de error
  con Unicode y evita exponer mensajes privados en biblioteca, lector,
  búsqueda y apertura de documentos.
- La importación permite autorizar una raíz de observación de forma explícita;
  el watcher normaliza duplicados, pausa con el ciclo de vida y solo solicita
  revalidación.
- Conecta las acciones masivas de biblioteca a la UI con Ctrl/Shift, resumen
  por elemento, confirmación segura, etiquetas/colecciones, archivo local y
  retirada sin tocar el PDF; el coordinador también evita que el trabajo de
  fondo quede hambriento tras una ráfaga visible.

## 0.1.0-dev — 2026-08-01 (endurecimiento de viewport y render)

- Corrige el ownership de imagenes renderizadas: la cache LRU posee las
  `ui.Image` y los consumidores no pueden invalidarlas al generar miniaturas.
- Propaga cancelacion al token nativo de pdfrx, cancela miniaturas obsoletas y
  elimina PNG parciales si una portada no llega a persistirse.
- Restaura zoom y posicion en historial, protege navegacion con documento vacio
  y mantiene `Ajustar ancho` alineado al inicio del contenido.
- Ajusta biblioteca y portadas para DPI alto y escritorios 16:9/16:10,
  1920x1080 y 2560x1440; evita overflow en anchos compactos.
- Reserva Page Up/Page Down para navegacion, refuerza semantica de ajustes y
  añade pruebas de token, viewport, layout de escritorio y shortcuts.
- No se crea commit. La suite Flutter completa y las capturas nativas quedan
  condicionadas por la indisponibilidad local de dependencias de Pub.

## 0.1.0-dev — 2026-07-29 (biblioteca de artefactos)

- Extiende el rediseño monocromático al lector, paneles, menús, diálogos,
  ajustes y controles globales; elimina la franja visible de pestañas.
- Corrige el centrado del PDF al redimensionar o acoplar paneles y centra de
  forma simétrica las constelaciones pequeñas de la biblioteca.
- Regenera portadas en caché v3 a mayor resolución y adapta las miniaturas
  laterales al ancho/DPI con filtrado de alta calidad y retención acotada.
- Compacta el menú del lector, añade iconos y evita duplicar accesos al panel
  cuando ya están visibles en la barra superior.
- Mantiene centrada la página al alcanzar el zoom mínimo, limita ese extremo al
  ajuste de página y valida el centro mediante lectura de píxeles.
- Convierte pantalla completa en una vista inmersiva sin barras, panel,
  progreso ni indicadores; `Escape` restaura la interfaz.
- Corrige el modo oscuro aun durante un arranque seguro, convierte `←`/`→` en
  la navegación principal y hace que el modo libro avance por pliegos:
  portada, 2–3, 4–5 y sucesivos.
- Rehace el viewport del lector con pliegos compactos, lomo de 10–16 px,
  normalización de paneo/zoom, ajuste sobre la unidad completa, indicador
  `2–3 / total` y una sola navegación para botones y teclado.
- Sustituye el selector de modos por un control segmentado monocromático con
  forma, peso, subrayado, hover, semántica y tooltips explícitos.
- Biblioteca espacial monocromática con portadas flotantes, jitter estable,
  virtualización, selección, foco, hover, teclado y menú contextual.
- Búsqueda temporal y filtros/orden compactos; favorito persistente y
  restauración de selección, scroll y contexto de vista.
- Detección local sobre seis páginas, caché pequeña/grande, esquema SQLite v3 y
  selector manual de portada que el algoritmo no sobrescribe.
- Tokens claro/oscuro refinados e iconos de aplicación generados directamente
  desde la imagen original entregada, sin reinterpretarla ni introducirla como
  decoración dentro de la interfaz.
- Añade favicon `.ico`/`.png` e iconos web/PWA desde esa misma imagen original.
- Transición corta de opacidad/escala entre biblioteca y lector, respetando
  tanto la preferencia de animación como la reducción de movimiento del sistema.
- Nuevas pruebas de scoring, persistencia, layout, navegación, filtros,
  accesibilidad y migración; sin dependencias de runtime nuevas.

## 0.1.0-dev — 2026-07-29 (tercera iteración local)

- Esquema SQLite v2 recuperable para anotaciones, etiquetas, colecciones,
  workspaces, perfiles e índice FTS5; backup JSON v2 compatible con v1.
- Notas/anotaciones locales por página, panel filtrable, navegación, undo y
  exportación Markdown/JSON sin modificar el PDF.
- Pestañas acotadas y suspendibles, apertura múltiple y workspaces locales.
- Búsqueda global e indexación opt-in cancelable, sin OCR ni red.
- Colecciones/vistas inteligentes, importación de carpeta con preview y
  recuperación conservadora por huella.
- Historial de navegación, perfiles base, diagnóstico redaccionado y safe mode.
- 86 pruebas, integración Windows aislada, 14 capturas nuevas, perfil de cinco
  pasadas y tres procesos sintéticos avanzados.
- Selección geométrica y split/comparación real permanecen bloqueados y no se
  anuncian como funciones terminadas.

## 0.1.0-dev — 2026-07-29 (interfaz contextual y funciones locales)

- Biblioteca por estado de lectura con recuentos, combinación texto/orden en una
  pasada, selector responsive, persistencia solo del estado y consulta efímera.
- Corregido “Continuar leyendo” para excluir documentos sin empezar,
  terminados y ausentes; filas diferenciadas para progreso, cambio de archivo y
  error de portada.
- Barra contextual del lector con jerarquía por ancho, overflow único, estados
  activos accesibles, grupos visibles persistentes y tooltips dinámicos.
- Panel lateral redimensionable 280–480 px con ratón/teclado, semántica,
  debounce de 240 ms, sección única, futuros de miniatura reutilizados y visor
  PDF aislado de reconstrucciones de chrome.
- Paleta ampliada con ocho documentos recientes y cambio seguro que conserva el
  lector actual cuando otra apertura falla.
- Atajos configurables para abrir, búsqueda, paleta, página/panel, navegación,
  zoom, marcador, pantalla completa y chrome; conflictos, JSON, defaults y
  etiquetas se resuelven en el registro único.
- Ficha documental desde biblioteca y lector con metadatos/presencia, copiar,
  mostrar, relocalizar por huella, abrir/continuar, reiniciar y retirar sin
  borrar el PDF.
- Copias JSON locales versionadas con validación y vista previa, prebackup e
  importación transaccional; nunca incluyen archivos PDF.
- Historial local agrupado, acotado, desactivable y limpiable; recuperación
  opt-in de documento/página/zoom/layout/panel/chrome con guardas anti-bucle.
- Ajustes reorganizados en ocho secciones con previsualización visual ligera.
- Auditoría automatizada WCAG AA, semántica, teclado, foco, movimiento reducido
  y fronteras responsive; Narrador y DPI 100/150/200 % siguen en review.
- Perfil Windows/Profile de cinco pasadas con 40 PDF, 100 páginas y carga
  vectorial: cero frames fuera de presupuesto, caché/deduplicación verificadas y
  3,1 MiB liberados desde el pico observado al mínimo tras retorno; la retención
  de RSS del runtime queda documentada sin presentarla como ausencia de fugas.
- Suite estándar ampliada a 66 pruebas; análisis, integración PDFium, perfil y
  build Release aprobados. Bundle: 19 archivos y 39,19 MiB.
- Eliminadas del fork de pdfrx trazas debug que interpolaban la ruta local o
  detalles de fuentes embebidas; el logger central conserva solo el tipo de
  error y descarta detalles/stack que puedan contener rutas. No se añadieron
  dependencias ni tablas de producción.

## 0.1.0-dev — 2026-07-29 (segunda iteración visual)

- Reescrito `DESIGN.md` como sistema visual implementable de minimalismo
  editorial contemporáneo.
- Añadidos tokens semánticos de color, espacio, tamaño, radio, movimiento y
  breakpoints únicos `<680`, `680–1099` y `>=1100`.
- Rediseñadas Biblioteca, lector, controles, panel lateral, búsqueda, paleta,
  diálogos y Ajustes sin añadir dependencias.
- Añadidas navegación lateral de Ajustes, variante compacta, tema oscuro
  coherente y estados contextuales de carga, vacío, error, ausente y búsqueda.
- La biblioteca incorpora filtro local por título/autor/archivo, tres órdenes,
  contador accesible, estado sin coincidencias y acceso directo con `Ctrl+F`.
- Añadida densidad de interfaz cómoda/compacta mediante tokens tipados y una
  preferencia persistente, sin cambiar el zoom ni el tamaño del documento.
- Las portadas se decodifican al tamaño físico visible según DPR, usan calidad
  apropiada para miniaturas y quedan aisladas de repintados vecinos.
- Movimiento reducido elimina el scroll programático de la paleta y las
  transiciones globales de tema; la biblioteca mantiene foco local predecible.
- La vista enfrentada ahora exige 340 px legibles por página después de restar
  panel, márgenes, altura y relación de aspecto.
- Ajustar página/ancho y navegar en vista enfrentada encuadra la pareja
  completa; cambiar a modo libro reconstruye el layout y aísla una sola hoja.
- El perfil de tinta electrónica fuerza canvas blanco, página única, cero
  sombra y movimiento reducido como fallback; la validación física sigue
  pendiente.
- Ampliada la suite estándar de 34 a 52 pruebas con fronteras responsive,
  temas semánticos, overflow, e-ink, política de doble página, estados y
  teclado de la paleta/biblioteca, densidad y presupuesto de miniaturas.
- La integración Windows prueba el lector completo con PDFium real, búsqueda,
  paleta, captura, zoom, navegación, marcador y restauración; Escape se cubre
  en pruebas de widgets de panel y diálogo.
- Build Release y smoke nativo aprobados; capturas DPI-aware de Biblioteca,
  lector, búsqueda, paleta y Ajustes añadidas a `docs/screenshots`.
- La ampliación de interfaz no añade dependencias; el bundle Release verificado
  contiene 19 archivos y ocupa 39,07 MiB.

## 0.1.0-dev — 2026-07-28

- Inicialización Flutter para Windows, Android e iOS.
- Arquitectura, diseño, privacidad, decisiones, roadmap y plan de pruebas.
- Motor `pdfrx`/PDFium aislado para apertura, render, texto, búsqueda, índice,
  enlaces y contraseña.
- Biblioteca SQLite, recientes, ausentes, progreso, sesiones, preferencias,
  portadas e invalidación por huella.
- Visor continuo/único/enfrentado, modo libro, LTR/RTL, seis temas, navegación,
  zoom, rotación, pantalla completa y restauración de posición.
- Búsqueda cancelable, panel lateral, marcadores, paleta, atajos y ajustes.
- Márgenes por documento, restauración de geometría de ventana y cancelación
  activa de render al cerrar.
- LRU por bytes, límite de portadas y logging de rendimiento sin datos privados.
- 34 pruebas estándar aprobadas, incluidas regresiones PDFium, errores tipados
  y contraseña AES-128 con archivos legales generados.
- Integración Windows aprobada con render visible, zoom, avance/retroceso,
  búsqueda, marcador y restauración.
- Build Release Windows aprobado y ejecutado con un PDF mediante argumento.
- Línea base Release registrada con ventana receptiva, memoria, persistencia
  local, cero conexiones TCP y cierre limpio.
- Corregidos un ciclo autorreferente en la deduplicación de renders y el ciclo
  de vida del codec de imagen de pdfrx 2.4.7 en Windows.
