# NEXT_TODOS — hoja de ruta integral de Yamantaka

Actualizado: 2026-08-31.

Este documento es el backlog vivo posterior a la reconstrucción del lector. No
es una colección de ideas sueltas: ordena trabajo de producto, funcionamiento,
calidad, rendimiento, accesibilidad, seguridad, distribución y acabado visual.
Cada tarea debe poder cerrarse con evidencia reproducible.

## Cómo usar esta hoja de ruta

Prioridades:

- **P0 — bloqueo:** impide ejecutar, preservar datos o confiar en una build.
- **P1 — producto:** afecta al flujo principal de biblioteca y lectura.
- **P2 — calidad:** mejora consistencia, accesibilidad, rendimiento o acabado.
- **P3 — exploración:** aporta valor, pero necesita validación antes de entrar
  en el producto estable.

Reglas:

- una casilla solo se marca cuando código, pruebas y documentación coinciden;
- cada cambio visual necesita comparación a 100 %, 125 %, 150 %, 175 % y 200 %
  de escala real, no imágenes reescaladas;
- los gates distinguen una build local completa bajo `build/` de un artefacto
  de distribución; este último exige firma confiable;
- certificados, PFX, contraseñas y credenciales nunca entran en Git;
- una compilación local no equivale a publicar la aplicación;
- no se desactiva Smart App Control, Defender ni otra protección para hacer
  pasar una verificación;
- Yamantaka sigue siendo local-first y sin red durante el uso normal.

## Punto de partida confirmado

- [x] 0.12.0 añade centro de continuidad, espacios completos y cola, modo Foco,
  comparación, revisión global de anotaciones, relaciones/revisiones, reglas
  reversibles y recuperación atómica sobre SQLite v10.
- [x] La matriz 0.12 reúne 172 capturas en 18 hojas, diez comparaciones 0.11/0.12
  y prueba de marca byte-idéntica; 287 pruebas conservan toda la suite anterior.
- [x] Cinco pasadas comparables terminan `pass-with-watch`, sin regresiones
  fiables ni invariantes fallidos, y confirman máximo de 80 documentos.
- [ ] Repetir validación física 0.12 con Narrator, dos monitores, en-US, alto
  contraste real y DPI 125/150/175/200 % cuando el hardware esté disponible.
- [ ] Sustituir el artefacto local sin firma por una distribución Authenticode
  solo cuando exista certificado y autorización expresa de publicación.

- [x] 0.11.0 añade navegador documental `Ctrl+J`, sesiones ricas, cerrados
  recientes, selección anotable real, agrupación plegable e índice reanudable
  sin red, cuentas, telemetría ni modificación del PDF.
- [x] Biblioteca materializa como máximo 80 documentos y agrupa con dos
  consultas de membresía acotadas, sin recorridos del sistema de archivos.
- [x] Portadas fuera del viewport se cancelan y el diagnóstico local mide
  intentos y decodificaciones evitadas sin datos privados.
- [x] El índice acepta prioridades nuevas durante la ejecución y las búsquedas
  solapadas solo publican la generación más reciente.
- [x] La matriz 0.11 incluye 164 capturas revisadas, importación en los ocho
  tamaños base, 130/150/200 % de texto y diez comparaciones 0.10/0.11.
- [ ] Completar la matriz física 0.11 con Narrator, dos monitores y escalas
  reales 100/125/150/175/200; la automatización no sustituye esa evidencia.

- [x] 0.10.0 añade Continuar leyendo, selector avanzado de página, conmutador
  Ctrl+Tab, búsqueda FTS5 filtrable, cola de importación detallada y búsqueda/
  reset por sección en Ajustes sin introducir red, cuentas ni telemetría.
- [x] Dos documentos nativos pueden permanecer calientes; el resto se suspende
  y libera según una política acotada, conservando el snapshot de navegación.
- [x] Arranque, base de datos, primera Biblioteca, primer PDF, consultas y
  búsqueda publican solo métricas locales con nombres constantes.
- [ ] Completar validación física 0.10 con Narrator, varios monitores y escalas
  reales de 100 %, 125 %, 150 %, 175 % y 200 %.
- [x] 0.9.0 compacta barra, dock, panel, Biblioteca, búsqueda, paleta y Ajustes
  con medidas verificables, sin alterar PDFium ni la persistencia.
- [x] Cuadrícula/lista conserva selección, filtros y scroll; el cambio usa una
  transición breve que desaparece con movimiento reducido.
- [x] La identidad del visor permanece estable al ocultar chrome, alternar panel
  y cruzar breakpoints; el margen responsive se resuelve dentro del adaptador.
- [ ] Completar validación física 0.9 con Narrator, varios monitores y escalas
  reales de 100 %, 125 %, 150 %, 175 % y 200 %.
- [x] 0.8.0 añade cuadrícula titulada y lista densa con filtros activos
  eliminables, adaptación a texto 150 % e inspector compartido sin alterar los
  datos persistidos.
- [x] El vacío inicial queda reducido a marca, nombre, frase breve y `Abrir PDF`;
  la marca violeta conserva la matriz de píxeles y luminancia de la original.
- [x] Lector, búsqueda global, metadatos, paleta, menús, diálogos y zonas de
  peligro comparten jerarquía y componentes 0.8 sin entrar en la clave del visor.
- [x] La golden `windows-reader-editorial-v08.png`, 54 capturas de widgets y diez
  comparaciones 0.7/0.8 respaldan la revisión; la suite alcanza 252 pruebas.
- [ ] Conservar la validación física histórica 0.8 como referencia; queda
  sustituida para aceptación por la matriz manual pendiente de 0.9.
- [x] 0.7.0 unifica el chrome del lector, eleva el inspector a banda editorial,
  hace legibles los filtros y estados de portada y agrupa Ajustes por tarea.
- [x] La preview de Ajustes alterna cuatro demostraciones sin persistencia ni
  PDFium; la paleta expandida añade iconos y contexto del resultado.
- [x] La golden normativa pasa a `windows-reader-editorial-v07.png` y las
  matrices 0.7 separan Biblioteca, Lector, Ajustes, paleta/menús y diálogos.
- [x] 0.6.0 añade inspector contextual de Biblioteca, dock progresiva, rail del
  panel acoplado y preview viva persistente de Ajustes sin red ni dependencias.
- [x] La matriz widget compara 0.5.0 y 0.6.0 en ocho tamaños y estados de tema,
  contraste, movimiento, panel y chrome en reposo.

- [x] `publish_to: none` impide publicar accidentalmente el paquete Dart en
  pub.dev.
- [x] El repositorio no contiene un flujo de envío a Microsoft Store, winget ni
  otra tienda de aplicaciones.
- [x] El modo Libro enfrenta 1–2 desde el primer pliego y mantiene una política
  de portada aislada solo cuando se solicita de forma explícita.
- [x] La cabecera del lector es opaca, la navegación está centrada y Windows
  dispone de fullscreen Win32 sin caption.
- [x] Existe una referencia golden mantenida del lector en 1600×900.
- [x] El análisis estático, las 204 pruebas actuales y la compilación Release de
  Windows pasan.
- [x] Las herramientas rechazan bundles locales incompletos, marcados como
  descargados o situados fuera de `build/`, sin confundir ausencia de firma con
  un fallo de arranque.
- [x] El empaquetado puede firmar y verificar todos los `.exe` y `.dll`, no solo
  el ejecutable principal, cuando se proporciona un certificado externo.

## M0 — confianza de la build y ejecución local

Objetivo P0: poder distinguir con claridad entre compilar, firmar, probar y
publicar. Yamantaka debe poder permanecer privada y, a la vez, producir una
build verificable.

- [x] Separar los gates que solo compilan de los que ejecutan código nativo.
- [x] Separar el preflight local del gate `-RequireSignature`, que rechaza un
  bundle parcial o totalmente sin firma antes de `Start-Process`.
- [x] Enumerar y comprobar cada PE cargable: ejecutable, motor Flutter, plugins,
  PDFium y SQLite.
- [x] Añadir firma masiva SHA-256, sello de tiempo opcional y verificación
  Authenticode mediante SignTool.
- [x] Registrar recuento y sujetos de firma en el manifiesto del paquete.
- [ ] Elegir una vía de confianza sin publicación en Store:
  Azure Artifact Signing/Trusted Signing, certificado OV de una CA del Trusted
  Root Program o SignPath Foundation si el proyecto cumple sus requisitos.
- [ ] Obtener un certificado RSA de firma de código y documentar propietario,
  renovación, revocación y recuperación; ECC no es compatible actualmente con
  Smart App Control.
- [ ] Conectar la firma a CI mediante identidad federada o almacén seguro, sin
  exportar una clave privada al repositorio ni a artefactos de build.
- [ ] Firmar todos los PE después de la última modificación y antes de comprimir
  el bundle; cualquier cambio posterior debe invalidar el gate.
- [ ] Aplicar sello RFC 3161 SHA-256 y verificar el binario tanto conectado como
  sin conexión una vez exista una cadena y un timestamp válidos.
- [ ] Validar una build firmada en una instalación limpia de Windows 11 con
  Smart App Control en enforcement.
- [ ] Confirmar que abrir, buscar, imprimir y cerrar un PDF no carga ninguna DLL
  sin firma que no aparezca en el manifiesto.
- [ ] Añadir un inventario SBOM de dependencias nativas con versión, licencia,
  hash y procedencia.
- [ ] Definir una política de caducidad: una Release no puede declararse lista si
  el certificado expira antes del horizonte mínimo acordado.
- [ ] Añadir un procedimiento de emergencia para revocar una build comprometida
  sin afectar documentos ni datos locales.

Criterio de salida: una build privada, sin entrada en ninguna tienda, se ejecuta
en un host limpio con Smart App Control activo y cada binario cargado presenta
una firma válida y trazable.

## M1 — apertura, cierre y recuperación de documentos

Objetivo P0/P1: ningún PDF válido debe dejar la aplicación atrapada y ningún PDF
inválido debe comprometer la sesión.

- [ ] Unificar en una máquina de estados explícita: vacío, abriendo, listo,
  recuperable, bloqueado por contraseña, dañado y cerrado.
- [ ] Cancelar una apertura anterior cuando llega otra sin publicar estados
  tardíos del documento cancelado.
- [ ] Diferenciar en la interfaz archivo ausente, acceso denegado, contraseña
  incorrecta, PDF dañado y fallo interno del motor.
- [ ] Implementar contraseña de PDF con reintento acotado y sin persistirla.
- [ ] Recuperar el último documento y posición solo después de verificar ruta,
  huella y número de páginas.
- [ ] Probar cierres durante carga, render, búsqueda, indexado e impresión.
- [ ] Garantizar que todos los handles de PDF, isolates, timers y streams se
  liberan al cerrar una pestaña o la ventana.
- [ ] Añadir recuperación tras cierre inesperado sin reabrir automáticamente un
  archivo que causó el fallo anterior.
- [ ] Permitir volver a localizar un archivo movido conservando metadatos,
  progreso, marcadores y anotaciones cuando la huella coincide.
- [ ] Añadir un modo seguro de solo lectura cuando la base de datos no pueda
  migrarse, con exportación de diagnóstico y copia de seguridad.
- [ ] Probar rutas largas, Unicode, unidades externas, carpetas de red
  desconectadas y cambios de permisos durante una sesión.

Criterio de salida: cada error termina en una acción comprensible —reintentar,
localizar, introducir contraseña, abrir diagnóstico o volver a biblioteca— y no
quedan recursos nativos retenidos.

## M2 — modelo de lectura y navegación

Objetivo P1: Documento y Libro deben comportarse como dos modelos coherentes,
no como variaciones visuales de un mismo scroll.

- [ ] Formalizar unidades de lectura para página simple, pliego, portada aislada
  y página final sin pareja.
- [ ] Exponer la política de portada por documento con vista previa inmediata y
  opción para restablecer el valor recomendado.
- [ ] Resolver paridad y orden de lectura para LTR y RTL en toda la navegación,
  miniaturas, teclado, historial y progreso.
- [ ] Mantener el ancla visual al rotar páginas o cambiar entre Documento y
  Libro, evitando saltos a una unidad distinta.
- [ ] Definir comportamiento exacto de Inicio, Fin, RePág, AvPág, flechas,
  espacio, Shift+espacio y rueda.
- [ ] Añadir salto a página/pliego con validación en línea y tolerancia a números
  escritos durante render activo.
- [ ] Mostrar progreso por unidad y por página sin porcentajes contradictorios.
- [ ] Mantener un historial atrás/adelante que recuerde zoom, rotación, modo,
  página y ancla relativa.
- [ ] Añadir marcadores temporales A/B para comparar dos zonas del documento sin
  abrir una segunda ventana.
- [ ] Probar documentos de 1, 2, 3, 999 y más de 10 000 páginas.
- [ ] Probar tamaños mezclados, páginas apaisadas, crop boxes desiguales,
  rotaciones internas y páginas con dimensiones extremas.
- [ ] Evitar que una unidad futura se haga visible por los bordes al ajustar un
  pliego en monitores ultrapanorámicos.

Criterio de salida: la misma acción de navegación produce una unidad predecible
en ratón, teclado, menú, command palette y accesibilidad.

## M3 — geometría y acabado del modo Libro

Objetivo P1/P2: conseguir un pliego creíble, estable y legible sin disfrazar el
PDF con adornos que interfieran con su contenido.

- [ ] Calcular escala sobre ambas páginas y mantener sus bordes superiores
  alineados cuando las proporciones difieren.
- [ ] Definir una anchura máxima de pliego que preserve contexto alrededor sin
  reducir innecesariamente el texto.
- [ ] Hacer que el lomo responda al color real del borde de página y no oscurezca
  contenido impreso junto al medianil.
- [ ] Ajustar la sombra interior según zoom y DPI con límites perceptuales, no
  con píxeles fijos sin escalar.
- [ ] Diseñar el tratamiento de página única final: lado correcto, equilibrio
  del lienzo y ausencia de una página fantasma.
- [ ] Diseñar portada aislada opcional con transición clara hacia el primer
  pliego y sin cambiar la numeración real.
- [ ] Añadir un modo de separación mínima configurable para escaneos cuyo borde
  oscuro impide distinguir el medianil.
- [ ] Evitar seams, líneas subpíxel y parpadeos al hacer zoom fraccional.
- [ ] Validar colores y sombras en SDR, HDR, alto contraste y perfiles ICC
  habituales.
- [ ] Probar lomo, bordes y selección de texto con páginas transparentes,
  fondos oscuros y PDFs escaneados.
- [ ] Añadir comparación perceptual del golden con umbral pequeño y una máscara
  solo para contenido PDF no determinista.

Criterio de salida: a simple vista se reconocen dos páginas físicas enfrentadas,
pero texto, selección, enlaces y anotaciones siguen siendo el contenido dominante.

## M4 — fullscreen, ventanas, monitores y DPI

Objetivo P1: la ventana debe obedecer las expectativas de Windows en todas las
transiciones, no solo en el monitor principal.

- [ ] Ejecutar pruebas manuales firmadas de F11 en cada monitor disponible.
- [ ] Restaurar exactamente estilo, posición, tamaño, estado maximizado y monitor
  después de salir de fullscreen.
- [ ] Cubrir taskbar normal, auto-oculta, situada en cada borde y monitores con
  áreas de trabajo diferentes.
- [ ] Manejar desconexión del monitor activo mientras la aplicación está en
  fullscreen.
- [ ] Responder a cambios de DPI durante el arrastre entre monitores sin rehacer
  el PDF a una escala borrosa.
- [ ] Probar escalas reales de 100 %, 125 %, 150 %, 175 % y 200 %.
- [ ] Mantener objetivos táctiles y texto legible en pantallas de alta densidad.
- [ ] Evitar guardar las dimensiones del monitor como bounds de ventana normal.
- [ ] Definir interacción con Snap Layouts, maximizar, minimizar y restaurar.
- [ ] Impedir que paneles, diálogos o tooltips queden detrás del fullscreen.
- [ ] Probar Escape cuando hay menús, búsqueda, diálogo, panel o selección activa
  antes de abandonar fullscreen.
- [ ] Añadir telemetría local de transición de ventana al diagnóstico, sin datos
  de documentos ni envío de red.

Criterio de salida: una matriz firmada de monitor × DPI × estado de ventana pasa
sin barra nativa residual, taskbar visible involuntariamente ni bounds corruptos.

## M5 — controles, jerarquía y experiencia de lectura

Objetivo P1/P2: cada control debe tener una ubicación predecible, una función
única y una alternativa de teclado.

- [ ] Auditar frecuencia y criticidad para separar navegación, vista, edición y
  acciones del documento.
- [ ] Mantener Documento/Libro en el centro geométrico incluso cuando cambian los
  controles laterales.
- [ ] Mantener navegación primaria centrada y zoom/vista en una zona secundaria
  estable.
- [ ] Sustituir iconos ambiguos por icono + texto cuando haya espacio y conservar
  tooltips específicos en compacto.
- [ ] Mostrar claramente estado activo, hover, foco, pulsado y deshabilitado sin
  depender solo del color.
- [ ] Unificar menús de desbordamiento para que una acción no aparezca duplicada
  en dos lugares de la misma anchura.
- [ ] Diseñar autoocultado del chrome con pausa al usar teclado, lector de
  pantalla, menú o panel.
- [ ] Añadir un bloqueo opcional de controles visibles para presentaciones y
  lectura asistida.
- [ ] Mostrar una ayuda de atajos contextual y buscable desde el propio lector.
- [ ] Permitir personalizar solo atajos no reservados por Windows y explicar los
  conflictos antes de guardar.
- [ ] Añadir feedback breve al alcanzar primera/última página sin animaciones
  intrusivas.
- [ ] Probar todos los breakpoints sin overflow a 640×480, 800×600, 1280×720,
  1600×900, 1920×1080, 2560×1440 y ultrawide.

Criterio de salida: un usuario nuevo puede abrir, cambiar modo, navegar, ajustar,
buscar y salir de fullscreen sin explorar menús al azar.

## M6 — biblioteca y organización

Objetivo P1: encontrar y recuperar documentos debe ser tan sólido como leerlos.

- [ ] Unificar búsqueda, filtros, colecciones y estados de lectura en un único
  modelo de consulta serializable.
- [ ] Mantener selección, scroll y foco al filtrar, ordenar o volver del lector.
- [ ] Añadir filtros combinables por favorito, progreso, etiqueta, colección,
  fecha, tamaño y disponibilidad del archivo.
- [ ] Diseñar un estado claro para archivo ausente con localizar, retirar de la
  biblioteca y ver información; nunca ofrecer abrir o revelar una ruta inválida.
- [ ] Detectar duplicados por huella sin fusionar documentos distintos que solo
  comparten nombre.
- [ ] Permitir combinar duplicados de forma transaccional conservando el dato
  más reciente de progreso, anotaciones y portada manual.
- [ ] Regenerar portadas dañadas o antiguas de forma limitada y cancelable.
- [ ] Añadir selección por teclado completa, menú contextual con Shift+F10 y
  orden espacial determinista.
- [ ] Diseñar vistas vacía, cargando, sin resultados, error y recuperación con
  una acción principal inequívoca.
- [ ] Probar 5 000, 25 000 y 100 000 registros con materialización acotada.
- [ ] Medir tiempo hasta primer contenido y estabilidad de layout durante la
  carga de portadas.
- [ ] Añadir importación incremental de carpetas con resumen por archivo y
  reanudación segura.

Criterio de salida: la biblioteca conserva contexto y responde con fluidez aun
cuando portadas, rutas o unidades externas no estén disponibles.

## M7 — búsqueda, texto, enlaces y anotaciones

Objetivo P1/P2: las capas interactivas deben compartir coordenadas y sobrevivir
a zoom, rotación, recorte y cambio de modo.

- [ ] Verificar extracción de texto por página con cancelación y límites de
  memoria.
- [ ] Resaltar todos los resultados visibles sin bloquear render ni selección.
- [ ] Navegar resultados por documento, respetando orden LTR/RTL y pliegos.
- [ ] Añadir estados para PDF sin capa de texto y ofrecer OCR solo como futura
  capacidad explícita, local y descargable por separado.
- [ ] Abrir enlaces externos únicamente tras una acción clara y mostrar el
  dominio antes de abandonar la aplicación.
- [ ] Validar enlaces internos, destinos con zoom y referencias a páginas
  rotadas.
- [ ] Consolidar anotaciones, marcadores y resultados sobre un sistema común de
  coordenadas normalizadas.
- [ ] Añadir deshacer/rehacer persistente dentro de la sesión de anotación.
- [ ] Exportar Markdown/JSON con versión de esquema y sin copiar el PDF.
- [ ] Detectar anotaciones cuya página cambió y ofrecer revisión asistida en vez
  de recolocarlas silenciosamente.
- [ ] Añadir navegación accesible por lista para anotaciones que no puedan
  manipularse con precisión mediante puntero.

Criterio de salida: toda superposición apunta al mismo contenido antes y después
de rotar, hacer zoom, cambiar de monitor o restaurar una sesión.

## M8 — accesibilidad e internacionalización

Objetivo P1/P2: todas las funciones esenciales deben ser utilizables sin ratón,
sin color y con tecnologías de asistencia.

- [ ] Ejecutar auditoría completa de orden de foco y retorno de foco tras cerrar
  cada menú, panel y diálogo.
- [ ] Exponer nombre, función, estado y atajo de todos los controles mediante UI
  Automation.
- [ ] Evitar zonas clicables sin un equivalente semántico y de teclado.
- [ ] Mantener contraste WCAG AA en temas, selección, foco y estados
  deshabilitados.
- [ ] Probar alto contraste de Windows sin sustituir colores del sistema por
  valores decorativos fijos.
- [ ] Respetar movimiento reducido en transiciones, autoocultado y navegación.
- [ ] Probar escalado de texto del sistema separado del DPI.
- [ ] Sustituir cadenas incrustadas restantes por recursos localizables.
- [ ] Completar español e inglés con pruebas de paridad de claves y textos largos.
- [ ] Preparar pluralización, fechas, porcentajes y orden RTL sin concatenaciones
  dependientes del idioma.
- [ ] Añadir una pasada manual con Narrator para biblioteca, lector, búsqueda,
  ajustes y recuperación de errores.

Criterio de salida: todo flujo P0/P1 se completa mediante teclado y Narrator, con
foco visible y sin pérdida de contexto.

## M9 — rendimiento, memoria y consumo

Objetivo P1/P2: mantener interacción fluida sin convertir la precarga en presión
de memoria oculta.

- [ ] Definir presupuestos medibles para arranque, apertura, primer render,
  cambio de página, búsqueda y cierre.
- [ ] Medir percentiles p50/p95/p99 y conservar hardware, DPI y corpus junto al
  resultado.
- [ ] Separar caché de miniaturas, páginas visibles, prefetch y texto extraído con
  límites independientes.
- [ ] Adaptar prefetch a dirección, velocidad, modo Libro, memoria disponible y
  actividad en segundo plano.
- [ ] Cancelar trabajos fuera de viewport antes de empezar una generación nueva.
- [ ] Comprobar que zoom continuo no acumula texturas o tiles obsoletos.
- [ ] Añadir soak firmado de apertura/cierre y navegación con tendencia de RSS,
  handles, GDI y GPU.
- [ ] Probar PDFs vectoriales complejos, escaneos de alta resolución, páginas
  transparentes y documentos patológicos sintéticos.
- [ ] Pausar indexado y mantenimiento al minimizar, con reanudación justa.
- [ ] Evitar escrituras de progreso por evento; agrupar y forzar flush seguro al
  cerrar.
- [ ] Establecer umbrales que fallen CI solo ante regresiones estadísticamente
  relevantes, no por una única muestra ruidosa.

Criterio de salida: los presupuestos quedan documentados y ninguna prueba de
larga duración presenta crecimiento monótono de memoria o handles.

## M10 — sistema visual y pulido

Objetivo P2: construir una identidad sobria y consistente sin reducir densidad
útil ni competir con el documento.

- [x] Consolidar tokens semánticos para lienzo, superficies, bordes, foco,
  estados, elevación y texto; eliminar valores locales equivalentes.
- [ ] Definir una escala tipográfica de escritorio con métricas estables para
  español e inglés.
- [ ] Revisar iconografía para grosor, caja óptica, alineación y significado
  consistente.
- [x] Unificar radios, alturas, separación y sombras de menús, paneles, diálogos
  y barras flotantes.
- [x] Mantener el PDF como superficie de mayor contraste y reducir chrome visual
  cuando no se usa.
- [ ] Diseñar temas Documento claro, sepia, oscuro y alto contraste sin aplicar
  transformaciones que falseen color de ilustraciones.
- [ ] Definir estados skeleton solo donde estabilicen layout; evitar shimmer con
  movimiento reducido.
- [ ] Revisar empty states y errores para que tengan una acción principal, una
  secundaria como máximo y texto específico.
- [x] Añadir microinteracciones de 80–200 ms únicamente cuando expliquen causa y
  efecto.
- [x] Ajustar hit targets a un mínimo de 40 px y 48 px en perfiles táctiles.
- [ ] Crear una galería ejecutable de componentes y estados críticos para QA
  visual, sin convertirla en una segunda implementación.
- [ ] Revisar la aplicación en pantallas OLED, LCD de bajo contraste y tinta
  electrónica.

Criterio de salida: componentes equivalentes se ven y se comportan igual en
biblioteca, lector, ajustes y diálogos, con una sola fuente de tokens.

## M11 — datos, privacidad y seguridad

Objetivo P0/P1: proteger la biblioteca local y hacer verificable la promesa de no
enviar documentos ni actividad.

- [ ] Documentar exactamente qué se guarda, dónde, durante cuánto tiempo y cómo
  se elimina.
- [ ] Verificar que el uso normal abre cero conexiones TCP/UDP y no resuelve DNS.
- [ ] Repetir las pruebas de red locales sobre una build firmada para convertir
  la evidencia de desarrollo en evidencia de distribución.
- [ ] Sanitizar logs: sin rutas completas salvo exportación explícita de
  diagnóstico y sin texto extraído de PDFs.
- [ ] Añadir exportación de diagnóstico con vista previa de todos los datos que
  se incluirán.
- [ ] Validar backup y restore con hashes, versión de esquema y restauración
  transaccional.
- [ ] Añadir copia de seguridad automática antes de cada migración destructiva.
- [ ] Probar rollback ante falta de espacio, interrupción y corrupción parcial.
- [ ] Mantener documentos fuera del backup salvo selección explícita del usuario.
- [ ] Auditar dependencias y binarios nativos por CVE, licencia y procedencia.
- [ ] Establecer actualización de dependencias con ventana de observación y plan
  de reversión.
- [ ] Modelar amenazas para enlaces externos, PDFs hostiles, rutas manipuladas,
  archivos temporales y carga de DLL.

Criterio de salida: una auditoría puede demostrar con artefactos locales que el
lector no transmite documentos y que una migración fallida no pierde datos.

## M12 — impresión, exportación y portabilidad

Objetivo P2: completar flujos secundarios sin introducir una plataforma de
publicación implícita.

- [ ] Probar impresión por rango, páginas impares/pares, orientación, escala y
  pliegos sin confundir índice interno con número visible.
- [ ] Mostrar vista previa con el mismo crop, rotación y orden que recibirá el
  sistema de impresión.
- [ ] Exportar anotaciones y metadatos sin modificar el PDF original.
- [ ] Mantener un paquete portable con marcador explícito y datos junto al
  ejecutable solo cuando el usuario lo elija.
- [ ] Impedir que paquetes y manifiestos incluyan PDFs personales, bases de datos
  o rutas del equipo de desarrollo.
- [x] Hacer que `-RequireSignature` falle si una sola DLL queda sin firma.
- [ ] Añadir hashes reproducibles y comparación del contenido del ZIP con el
  manifiesto.
- [ ] Definir actualización manual offline antes de plantear cualquier sistema
  automático de red.
- [ ] Evaluar MSIX, instalador tradicional y ZIP portable como opciones técnicas
  separadas; ninguna implica publicar en Store.
- [ ] No activar distribución pública hasta que exista una decisión explícita de
  producto, soporte, firma y política de actualizaciones.

Criterio de salida: el artefacto se puede inspeccionar, verificar y mover sin
datos personales, y su canal de distribución siempre es una decisión explícita.

## M13 — automatización, pruebas y mantenimiento

Objetivo P1/P2: convertir los invariantes importantes en gates pequeños,
comprensibles y difíciles de falsear.

- [ ] Mantener una pirámide de pruebas: dominio rápido, widgets focalizados,
  golden reducido e integración nativa firmada.
- [x] Añadir pruebas PowerShell del preflight local y de `-RequireSignature`
  usando bundles sintéticos sin ejecutar sus binarios.
- [x] Verificar que una build local completa puede probarse sin presentarse como
  firmada y que `-RequireSignature` rechaza el mismo bundle con una explicación.
- [ ] Añadir una fixture firmada de prueba solo en infraestructura segura, nunca
  una clave privada en el repositorio.
- [ ] Reducir flakiness de golden cargando fuentes e iconos deterministas.
- [ ] Registrar dimensiones reales, DPI real, commit, estado dirty y hashes en
  cada evidencia visual.
- [ ] Separar fallos del producto, toolchain, política del host y falta de
  credenciales en el reporte de verificación.
- [ ] Ejecutar `git diff --check`, formato, análisis y pruebas antes del build
  nativo para fallar pronto.
- [ ] Añadir cobertura de migraciones desde cada versión de esquema soportada.
- [ ] Detectar imports, tokens, archivos, capturas y documentación sin referencias
  antes de cada release.
- [ ] Mantener una única captura vigente por escenario estable; el resto debe ser
  evidencia generada o historial de Git.
- [ ] Revisar trimestralmente tareas P3 y eliminar las que ya no respondan a un
  problema observado.

Criterio de salida: cada gate informa qué probó, qué omitió y por qué; no fabrica
evidencia ni convierte una limitación ambiental en un fallo del producto.

## M14 — exploraciones posteriores

Objetivo P3: validar utilidad antes de aumentar superficie y mantenimiento.

- [ ] Prototipar vista dividida para comparar páginas distantes con un único
  documento y presupuesto de memoria estricto.
- [ ] Evaluar modo presentación con controles bloqueables y puntero temporal.
- [ ] Explorar reflow solo para PDFs con estructura etiquetada suficiente; nunca
  prometerlo para escaneos o maquetación compleja.
- [ ] Evaluar OCR completamente local como módulo opcional con tamaño, licencia y
  consumo visibles antes de instalarlo.
- [ ] Prototipar lectura en voz alta usando APIs locales del sistema y resaltado
  sincronizado accesible.
- [ ] Explorar sincronización únicamente como plugin opcional cifrado y nunca
  como dependencia del funcionamiento base.
- [ ] Evaluar lápiz y táctil después de cerrar navegación, accesibilidad y
  geometría de escritorio.
- [ ] Probar perfiles por tipo de documento que sugieran ajustes pero nunca los
  cambien silenciosamente.

Criterio de entrada a P2: prototipo medido, problema real documentado, coste de
mantenimiento estimado y decisión explícita de privacidad.

## Orden recomendado de ejecución

1. M0 confianza de build y firma externa.
2. M1 recuperación y M11 integridad de datos.
3. M2 navegación y M4 ventanas/DPI.
4. M3 modo Libro y M5 controles.
5. M6 biblioteca y M7 capas interactivas.
6. M8 accesibilidad y M9 rendimiento.
7. M10 pulido visual, M12 portabilidad y M13 automatización.
8. M14 solo después de cerrar los criterios P0/P1.

## Definición global de terminado

Una tarea solo está terminada cuando:

- existe una prueba o evidencia manual reproducible proporcional al riesgo;
- funciona en los breakpoints, DPI, temas y estados afectados;
- contempla teclado, foco, semántica y movimiento reducido;
- no añade red, publicación, persistencia o permisos de forma implícita;
- actualiza documentación y elimina la alternativa obsoleta;
- no deja warnings, código muerto ni una excepción ambiental presentada como
  éxito del producto.
