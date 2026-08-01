# Próximas tareas de Yamantaka

Hoja de ruta operativa para la evolución de Yamantaka en Windows.

Última reconstrucción completa: 2026-08-01.

Este documento contiene únicamente trabajo futuro. El historial de trabajo
terminado vive en Git y en CHANGELOG.md; no se duplican aquí tareas cerradas ni
se mantienen tickets DONE como relleno.

## 1. Visión del producto

Yamantaka debe convertirse en un lector PDF para Windows rápido, silencioso,
fiable y agradable durante sesiones prolongadas. La aplicación debe sentirse
como una herramienta editorial de escritorio: el documento domina, las
acciones aparecen cuando son necesarias y cada superficie conserva una
jerarquía monocromática clara.

La evolución se evaluará en siete dimensiones inseparables:

- Funcionamiento: cada flujo principal debe ser completo, recuperable y
  coherente entre ratón, teclado y comandos.
- Rendimiento: apertura, render, navegación, búsqueda y biblioteca deben tener
  presupuestos medidos, no impresiones subjetivas.
- Eficiencia: no se harán trabajos repetidos, invisibles u obsoletos; toda
  operación costosa será priorizable, deduplicable y cancelable.
- Limpieza visual: menos chrome, menos ruido, más ritmo y mayor protagonismo del
  PDF y de las portadas.
- Calidad visual: tipografía, iconos, foco, estados, movimiento, contraste y
  layouts responderán a un sistema común.
- Accesibilidad: la aplicación completa será utilizable con teclado, Narrador,
  alto contraste, movimiento reducido y escalas de Windows habituales.
- Fiabilidad: documentos, base de datos, copias, ventanas y recursos nativos
  deben sobrevivir a cierres, errores, archivos extraños y sesiones largas.

## 2. Límites innegociables

- Windows es la única plataforma de esta hoja de ruta.
- El único formato de lectura es PDF.
- Todo dato de biblioteca, progreso, historial, búsqueda, colección, perfil,
  espacio de trabajo o anotación permanece local.
- No habrá cuentas, nube, sincronización remota, telemetría, publicidad,
  analítica, IA ni OCR.
- La aplicación no abrirá conexiones de red durante el uso normal.
- Un PDF nunca se copia, mueve, modifica o elimina sin una acción explícita y
  confirmada por el usuario.
- Las anotaciones y metadatos propios se guardan fuera del PDF.
- La detección automática de portada puede sugerir, pero nunca sustituye una
  portada elegida manualmente.
- El emblema se limita al icono de aplicación, ejecutable, instalador y
  superficies de plataforma. No aparece en biblioteca ni lector.
- No se añadirá una franja visible de pestañas sobre el documento.
- No se introducirán gradientes decorativos, colores saturados, glassmorphism,
  sombras pesadas ni componentes de aspecto genérico.
- Toda dependencia nueva exige justificación, licencia compatible, impacto de
  bundle medido, mantenimiento razonable y una decisión arquitectónica.

## 3. Línea base real — 2026-08-01

- Versión actual: 0.1.0+1.
- Rama de referencia: main, commit 0191676.
- Windows dispone de biblioteca local, importación, portadas, lector PDFium,
  modos Documento y Libro, búsqueda, índice, miniaturas, marcadores,
  anotaciones locales, colecciones, espacios de trabajo, perfiles, historial,
  copias, diagnóstico y safe mode.
- El sistema visual editorial monocromático, el tema oscuro, la densidad
  cómoda/compacta y los breakpoints 680/1100 ya existen.
- El motor mantiene una LRU por bytes, deduplicación de renders, cancelación
  nativa y ownership explícito de imágenes.
- La biblioteca ya adapta portadas a DPR y contempla 16:9, 16:10, Full HD y 2K.
- Existen pruebas unitarias, de widgets, SQLite, PDFium, integración Windows y
  perfiles reproducibles.
- La documentación registra validaciones históricas, pero el último
  endurecimiento no pudo repetir la suite completa por falta de resolución de
  dependencias de Pub. YAM-100 debe restablecer una evidencia limpia antes de
  considerar una versión candidata.
- El parche local de pdfrx sigue siendo una deuda controlada.
- Selección geométrica, teselas, split real, comparación y multi-window no se
  anuncian como funciones disponibles.

Esta línea base no equivale a una certificación de versión. Los gates definidos
a continuación son la única vía para declarar un hito publicable.

## 4. Estados, prioridades, esfuerzo y áreas

### Estados

- READY: definición completa, sin decisión pendiente y preparada para empezar.
- PLANNED: definida, pero espera dependencias o una fase posterior.
- IN_PROGRESS: implementación activa con una persona o agente responsable.
- BLOCKED: no puede avanzar por una dependencia externa demostrada.
- REVIEW: implementación terminada pendiente de evidencia o revisión.
- DONE: criterios de aceptación comprobados y evidencia enlazada.

Una dependencia interna pendiente no convierte una tarea en BLOCKED; permanece
PLANNED. BLOCKED se reserva para impedimentos externos o técnicos que no puedan
resolverse dentro del repositorio.

### Prioridades

- P0: bloquea estabilidad, integridad, accesibilidad básica o publicación.
- P1: mejora sustancial de funcionamiento, rendimiento o experiencia.
- P2: capacidad avanzada que puede esperar a una base estable.
- P3: exploración futura sin compromiso de entrega.

### Esfuerzo

- S: hasta dos días de trabajo acotado.
- M: hasta una semana.
- L: entre una y tres semanas.
- XL: más de tres semanas; debe dividirse antes de pasar a IN_PROGRESS.

### Áreas

- FUNC: funcionamiento y capacidades del producto.
- PERF: rendimiento, memoria, latencia y energía.
- EFF: eficiencia de arquitectura y ejecución.
- VIS: limpieza, calidad y coherencia visual.
- A11Y: teclado, foco, Narrador, contraste y adaptación.
- REL: fiabilidad, recuperación, integridad y seguridad.
- WIN: integración, empaquetado y comportamiento de Windows.

## 5. Gates globales de versión

### G1 — Calidad reproducible

- Una máquina Windows limpia restaura dependencias y ejecuta formato, análisis,
  suite Flutter, pruebas PDFium, integración y build Release con un solo flujo
  documentado.
- El flujo no modifica archivos versionados ni depende de pasos manuales
  ocultos.
- Una ejecución fallida conserva logs suficientes y devuelve un código de error
  inequívoco.

### G2 — Integridad y ciclo de vida

- Cerrar cada documento deja cero handles de PDF, renders activos, listeners
  propios e imágenes retenidas por las caches de Yamantaka.
- Migraciones, backup, restauración y limpieza son transaccionales o
  recuperables.
- Un cierre forzado no provoca bucles de restauración ni pérdida silenciosa de
  datos confirmados.

### G3 — Rendimiento

- Ventana visible en frío: p95 igual o inferior a 1,2 s en el equipo de
  referencia.
- Build de frame: p95 igual o inferior a 8 ms en biblioteca y lector.
- Menos del 1 % de frames supera 16,7 ms en los escenarios de interacción.
- Consultas de biblioteca e índice: p95 inferior a 20 ms con 5.000 documentos.
- Cada escenario nuevo conserva como máximo un 10 % de regresión frente a la
  línea base versionada o documenta y aprueba el intercambio.
- No se fija un umbral RSS universal; se exigen contadores propios a cero tras
  cerrar y una tendencia estable durante pruebas sostenidas.

### G4 — Experiencia visual y adaptable

- No existe overflow, recorte accidental ni solapamiento en 1366×768,
  1600×900, 1920×1080, 1920×1200, 2560×1440 y 3840×2160.
- Se validan DPI 100, 125, 150, 175 y 200 %.
- Biblioteca, lector, panel, paleta, ajustes y diálogos tienen evidencia en
  claro, oscuro y alto contraste cuando corresponda.
- El PDF conserva escala legible y nunca se estira para rellenar espacio
  ultrawide.

### G5 — Accesibilidad

- Todos los flujos principales se completan con teclado.
- Tab y Shift+Tab siguen un orden predecible y Escape devuelve el foco al
  origen correcto.
- Narrador anuncia nombre, rol, estado, valor y cambios relevantes una sola vez.
- Texto normal cumple contraste WCAG AA; foco y componentes no textuales
  alcanzan 3:1 contra superficies adyacentes.
- Movimiento reducido elimina animaciones no esenciales sin romper feedback.

### G6 — Privacidad y seguridad

- Un smoke Release confirma cero conexiones TCP/UDP iniciadas por la
  aplicación.
- Logs y diagnósticos no contienen rutas, títulos, consultas, contraseñas,
  texto PDF ni contenido de anotaciones.
- Enlaces externos requieren confirmación y solo admiten esquemas autorizados.
- Borrar datos explica exactamente qué se eliminará y nunca borra PDF.

### G7 — Distribución Windows

- Instalación limpia, actualización, reparación, desinstalación y paquete
  portable tienen procedimientos verificados.
- Firma, licencias, asociación PDF, argumentos, multi-monitor y rollback se
  comprueban antes de una release candidate.
- No quedan P0 o P1 abiertos para publicar 1.0.

## 6. Roadmap detallado

### Fase 0 — Base de calidad y desbloqueos

#### YAM-100 — Gate reproducible de Windows

- Estado: READY
- Prioridad: P0
- Esfuerzo: L
- Fase: 0
- Áreas: EFF, REL, WIN
- Dependencias: ninguna
- Problema: la validación actual está repartida entre comandos, scripts y
  documentación; además, una cache incompleta de Pub puede impedir incluso
  analizar el proyecto.
- Resultado esperado: cualquier colaborador puede demostrar desde un checkout
  limpio si Yamantaka está preparado para integrarse o publicarse.
- Cambios funcionales y técnicos:
  - Crear un entrypoint PowerShell único con fases independientes y códigos de
    salida acumulados.
  - Verificar SDK, Visual Studio, PDFium, dependencias, espacio de disco y
    permisos antes de compilar.
  - Ejecutar formato en modo comprobación, análisis, suite, PDFium real,
    integración Windows, perfil mínimo, build Release y smoke.
  - Guardar logs y metadatos no privados bajo build/verification.
- Visual y accesibilidad: el gate ejecutará capturas en resoluciones de
  referencia y reportará estados ausentes, no los marcará como aprobados.
- Rendimiento, memoria y privacidad: los informes incluirán tiempos, tamaño del
  bundle y contadores del motor; excluirán rutas y títulos.
- Casos límite y errores: offline con cache válida, cache incompleta, SDK
  distinto, ausencia de toolchain C++, build bloqueado y proceso nativo colgado.
- Criterios de aceptación:
  - [ ] Un comando ejecuta todas las fases desde un checkout limpio.
  - [ ] Cada fase puede repetirse de forma aislada.
  - [ ] Un fallo devuelve código distinto de cero y conserva diagnóstico.
  - [ ] El flujo no altera archivos versionados.
- Validación: ejecución limpia, ejecución offline con cache preparada y tres
  fallos inyectados: dependencia ausente, test fallido y build fallido.
- Riesgos y fuera de alcance: no instalará software ni descargará dependencias
  sin consentimiento; no sustituye CI remota.

#### YAM-101 — Matriz nativa de resoluciones y DPI

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 0
- Áreas: VIS, A11Y, WIN
- Dependencias: YAM-100
- Problema: una captura a un único tamaño no demuestra que la interfaz funcione
  en los monitores y escalas habituales de PC.
- Resultado esperado: biblioteca y lector mantienen jerarquía, legibilidad y
  controles accesibles en ventanas compactas, Full HD, 16:10, 2K y 4K.
- Cambios funcionales y técnicos:
  - Crear una matriz ejecutable para 1366×768, 1600×900, 1920×1080,
    1920×1200, 2560×1440 y 3840×2160.
  - Repetir los casos a 100/125/150/175/200 % de escala de Windows.
  - Registrar tamaño físico, tamaño lógico, DPR, tema, densidad y panel.
  - Cubrir biblioteca vacía/poblada, selección, lector Documento/Libro,
    fullscreen, ajustes, paleta y diálogos críticos.
- Visual y accesibilidad: no habrá recortes, objetivos inaccesibles, texto bajo
  chrome, spreads descentrados ni foco fuera de pantalla.
- Rendimiento, memoria y privacidad: las capturas usarán fixtures generados;
  no se guardarán documentos o títulos reales.
- Casos límite y errores: ventana menor que el mínimo, cambio de DPI en vivo,
  monitor desconectado, taskbar lateral y panel al ancho máximo.
- Criterios de aceptación:
  - [ ] Todos los perfiles tienen resultado y evidencia.
  - [ ] Full HD y 2K conservan densidad editorial sin huecos arbitrarios.
  - [ ] 16:10 aprovecha altura sin agrandar el chrome.
  - [ ] 4K no decodifica imágenes por encima del tamaño físico necesario.
- Validación: checklist nativo, capturas y recorrido de teclado/Narrador en cada
  familia de layout.
- Riesgos y fuera de alcance: no se simulará Narrador mediante widgets; cada
  DPI real exige cambiar la sesión o el monitor de Windows.

#### YAM-102 — Regresión visual automatizada

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 0
- Áreas: VIS, A11Y, REL
- Dependencias: YAM-100, YAM-101
- Problema: las revisiones visuales manuales detectan problemas, pero no
  protegen cada cambio posterior ni garantizan fixtures equivalentes.
- Resultado esperado: los cambios de layout, tokens o estados producen
  diferencias revisables y reproducibles antes de integrarse.
- Cambios funcionales y técnicos:
  - Definir fixtures deterministas sin copyright para todos los estados.
  - Separar capturas de widgets, integración PDFium y ventana Release.
  - Normalizar tamaño, DPR, fuentes, animaciones, reloj y datos.
  - Generar manifiesto con escenario, commit, resolución, tema y tolerancia.
  - Mantener baselines explícitos y requerir aprobación para actualizarlos.
- Visual y accesibilidad: incluir foco visible, texto largo, alto contraste,
  movimiento reducido y estados que no dependan solo del color.
- Rendimiento, memoria y privacidad: ejecutar escenarios de captura fuera del
  benchmark; los fixtures nunca usarán rutas o contenido privado.
- Casos límite y errores: fuente no disponible, banner de debug, ventana host,
  animación incompleta y diferencia por subpíxel.
- Criterios de aceptación:
  - [ ] Una diferencia no aprobada falla el gate.
  - [ ] Cada baseline identifica su escenario y configuración.
  - [ ] Las tolerancias no ocultan cambios de geometría o contraste.
  - [ ] El proceso distingue artefactos del host de la UI de Yamantaka.
- Validación: modificar de forma controlada un token, un texto y una geometría;
  confirmar detección y restaurar baseline.
- Riesgos y fuera de alcance: las capturas no sustituyen Narrador, percepción
  humana ni pruebas de interacción.

#### YAM-103 — Corpus legal de PDF patológicos

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 0
- Áreas: FUNC, PERF, REL
- Dependencias: YAM-100
- Problema: los fixtures pequeños no cubren documentos que fuerzan memoria,
  parser, render, búsqueda o recuperación.
- Resultado esperado: errores y documentos extremos se manejan sin bloqueo,
  fuga, pérdida de estado ni mensajes ambiguos.
- Cambios funcionales y técnicos:
  - Generar PDF vacío, corrupto, truncado, cifrado, 1.000 páginas, página
    raster extrema, vectorial pesada, tamaños mixtos, rotaciones y enlaces.
  - Versionar generadores y hashes, no binarios grandes salvo necesidad.
  - Definir resultado esperado para apertura, render, texto, búsqueda y cierre.
  - Añadir límites de tiempo, memoria y tamaño de imagen por escenario.
- Visual y accesibilidad: cada fallo tendrá título, explicación, acción segura
  y foco inicial/restaurado.
- Rendimiento, memoria y privacidad: corpus sintético, sin copyright; todos los
  recursos propios deben quedar a cero al cerrar.
- Casos límite y errores: contraseña incorrecta, contraseña cancelada, cero
  páginas, página de dimensiones inválidas y operación cancelada a mitad.
- Criterios de aceptación:
  - [ ] Cada familia de PDF tiene generador, hash y expectativa.
  - [ ] Ningún caso bloquea indefinidamente el hilo UI.
  - [ ] Corrupción no registra contenido ni provoca crash del proceso.
  - [ ] Cierre posterior conserva contadores de lifecycle a cero.
- Validación: suite PDFium real, integración Windows y repetición de apertura y
  cierre de cada fixture.
- Riesgos y fuera de alcance: no es fuzzing de seguridad exhaustivo ni valida
  todos los productores PDF existentes.

#### YAM-104 — Auditoría de ownership y ciclo de vida

- Estado: READY
- Prioridad: P0
- Esfuerzo: L
- Fase: 0
- Áreas: EFF, PERF, REL
- Dependencias: ninguna
- Problema: la aplicación combina controladores Flutter, handles PDFium,
  imágenes, caches, listeners, timers y trabajos asíncronos con propietarios
  distintos.
- Resultado esperado: cada recurso tiene un único propietario, un punto de
  cierre y una política clara para cambios de documento, ventana o sesión.
- Cambios funcionales y técnicos:
  - Inventariar creación, transferencia y dispose de recursos.
  - Documentar ownership entre AppServices, controlador de lector, adaptadores,
    caches, paneles y ventanas.
  - Añadir aserciones y contadores de debug para recursos vivos.
  - Eliminar listeners duplicados y callbacks que sobrevivan a su superficie.
- Visual y accesibilidad: cerrar o cambiar documento no dejará overlays,
  anuncios o foco ligados al contexto anterior.
- Rendimiento, memoria y privacidad: el cierre debe cancelar trabajo obsoleto y
  liberar recursos propios; el diagnóstico no incluirá identificadores privados.
- Casos límite y errores: cierre durante render, búsqueda, importación,
  thumbnail, backup, resize o reapertura de sesión.
- Criterios de aceptación:
  - [ ] Todo recurso inventariado tiene propietario y cierre.
  - [ ] No existe dispose doble de una imagen prestada.
  - [ ] Cien ciclos de abrir/cerrar terminan con contadores propios a cero.
  - [ ] Callbacks tardíos no actualizan controladores ya separados.
- Validación: tests de lifecycle, integración repetida y perfil de handles.
- Riesgos y fuera de alcance: el RSS del runtime puede no volver al baseline;
  se medirán recursos propios y tendencia sostenida.

#### YAM-105 — Coordinador único de trabajos asíncronos

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: XL
- Fase: 0
- Áreas: FUNC, EFF, PERF, REL
- Dependencias: YAM-104
- Problema: importación, portadas, renders, búsqueda, indexación, backup y
  recuperación tienen colas y tokens relacionados pero no una política común.
- Resultado esperado: el trabajo visible gana prioridad, el obsoleto se
  cancela y el usuario entiende qué sigue activo.
- Cambios funcionales y técnicos:
  - Diseñar un contrato de trabajo con identidad, categoría, prioridad,
    generación, progreso, cancelación y limpieza.
  - Deduplicar por clave estable y compartir resultados solo entre consumidores
    todavía válidos.
  - Limitar concurrencia por CPU, IO, PDFium y memoria.
  - Integrar primero portadas, miniaturas e índice antes de migrar otros flujos.
- Visual y accesibilidad: progreso y cancelación se anuncian sin banners
  repetidos; los trabajos de fondo no roban foco.
- Rendimiento, memoria y privacidad: no persistir consultas, rutas o contenido
  dentro de la cola; pausar trabajo no esencial bajo presión.
- Casos límite y errores: consumidor cancelado de un trabajo compartido,
  prioridad que cambia, cierre de app y fallo de cleanup.
- Criterios de aceptación:
  - [ ] Ningún resultado obsoleto actualiza la UI.
  - [ ] Trabajos iguales no duplican CPU, IO ni imagen.
  - [ ] Cancelar una vista no invalida consumidores todavía activos.
  - [ ] El diagnóstico muestra solo categoría, tiempos y contadores.
- Validación: pruebas de orden, deduplicación, cancelación, fairness y cierre;
  integración con al menos tres categorías.
- Riesgos y fuera de alcance: debe dividirse antes de IN_PROGRESS; no se
  construirá un framework genérico ajeno a las necesidades de Yamantaka.

#### YAM-106 — Retirada controlada del parche local de pdfrx

- Estado: BLOCKED
- Prioridad: P1
- Esfuerzo: M
- Fase: 0
- Áreas: EFF, REL
- Dependencias: YAM-100, YAM-103
- Problema: mantener un fork local aumenta coste de actualización y riesgo de
  divergencia, pero retirarlo sin evidencia puede reintroducir fallos nativos.
- Resultado esperado: usar una versión upstream corregida sin perder lifecycle,
  render, búsqueda, enlaces o compatibilidad Windows.
- Cambios funcionales y técnicos:
  - Vigilar una release upstream que incluya la corrección equivalente.
  - Comparar diff, licencia, APIs, binarios y requisitos de toolchain.
  - Ejecutar el gate completo con dependencia upstream.
  - Eliminar third_party solo después de una comparación aprobada.
- Visual y accesibilidad: no debe cambiar geometría, foco, texto o render del
  lector.
- Rendimiento, memoria y privacidad: bundle, apertura, render, caches y cierre
  no pueden empeorar fuera del presupuesto.
- Casos límite y errores: API incompatible, regresión del codec, cambio de
  PDFium o falta de binarios Windows.
- Criterios de aceptación:
  - [ ] Existe release upstream verificable con la corrección.
  - [ ] Todos los gates pasan sin el fork.
  - [ ] Licencias y avisos quedan actualizados.
  - [ ] El parche puede restaurarse mediante revert si aparece una regresión.
- Validación: A/B del corpus, suite, integración, perfil y build Release.
- Riesgos y fuera de alcance: permanece BLOCKED hasta existir upstream apto; no
  se actualizará solo para obtener una versión más reciente.

### Fase 1 — Motor PDF y lector

#### YAM-110 — Renderizado adaptativo por teselas

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, PERF, EFF
- Dependencias: YAM-103, YAM-105, YAM-111
- Problema: una página raster extrema puede superar por sí sola el presupuesto
  de la LRU aunque solo una región sea visible.
- Resultado esperado: navegar y ampliar páginas enormes sin picos
  desproporcionados ni pérdida de calidad visible.
- Cambios funcionales y técnicos:
  - Activar teselas solo cuando dimensiones, zoom y bytes estimados superen un
    umbral medido.
  - Clave de cache por documento, página, región, escala, rotación y generación.
  - Componer con solape mínimo para evitar costuras.
  - Priorizar viewport y anillo inmediato; cancelar regiones lejanas.
  - Mantener página completa para escenarios baratos.
- Visual y accesibilidad: no habrá parpadeos persistentes, huecos, costuras o
  cambios de escala al aparecer una tesela.
- Rendimiento, memoria y privacidad: bytes combinados nunca superan el
  presupuesto del gobernador; no se escriben teselas privadas en disco.
- Casos límite y errores: zoom rápido, rotación, resize, cambio de página,
  tesela fallida y documento cerrado durante composición.
- Criterios de aceptación:
  - [ ] Una página extrema no retiene un bitmap completo sobredimensionado.
  - [ ] El viewport alcanza calidad final sin huecos persistentes.
  - [ ] Navegación normal no activa teselas innecesarias.
  - [ ] Cancelación y cierre dejan cero regiones pendientes.
- Validación: fixtures raster/vector extremos, pruebas de planificador,
  inspección visual y perfil comparativo.
- Riesgos y fuera de alcance: se realizará prototipo antes de implementación;
  no se activará si la medición no justifica la complejidad.

#### YAM-111 — Gobernador global de memoria

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 1
- Áreas: PERF, EFF, REL
- Dependencias: YAM-104, YAM-105
- Problema: visor, motor, portadas, miniaturas y futuros lectores tienen
  presupuestos separados que pueden sumar un pico excesivo.
- Resultado esperado: la aplicación adapta calidad, prefetch y retención al
  escenario completo, no a caches aisladas.
- Cambios funcionales y técnicos:
  - Crear un presupuesto agregado configurable por perfil de memoria.
  - Registrar bytes estimados por propietario y prioridad.
  - Reducir prefetch y desalojar elementos recreables antes de afectar lo
    visible.
  - Reservar una cuota mínima para la unidad activa.
  - Exponer contadores redaccionados al diagnóstico.
- Visual y accesibilidad: degradar primero trabajo invisible; la página activa
  no debe desaparecer ni perder foco.
- Rendimiento, memoria y privacidad: objetivo explícito de no exceder el
  presupuesto propio; el RSS solo se usa como señal auxiliar.
- Casos límite y errores: dos lectores, 4K, DPR 200 %, página extrema,
  miniaturas abiertas y limpieza de cache concurrente.
- Criterios de aceptación:
  - [ ] Todos los consumidores registran y liberan bytes.
  - [ ] La suma retenida nunca supera el límite salvo una reserva documentada.
  - [ ] Presión alta reduce prefetch antes de la calidad visible.
  - [ ] Cerrar documento devuelve su cuota al instante lógico.
- Validación: tests de política, perfil con escenarios combinados y contadores
  a cero tras cierre.
- Riesgos y fuera de alcance: no se intentará controlar el asignador del
  runtime o la memoria interna de Windows.

#### YAM-112 — Prefetch predictivo y acotado

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 1
- Áreas: PERF, EFF
- Dependencias: YAM-105, YAM-111
- Problema: precargar siempre el mismo número de páginas desperdicia trabajo
  cuando el usuario cambia de dirección, navega rápido o abre el panel.
- Resultado esperado: la siguiente unidad aparece con menor latencia sin
  competir con la página activa.
- Cambios funcionales y técnicos:
  - Calcular dirección y velocidad con una ventana temporal acotada.
  - Prefetch por unidad de lectura: una página en Documento y spread en Libro.
  - Reducir o pausar bajo presión, ventana inactiva o navegación aleatoria.
  - Invalidar al cambiar zoom, rotación, layout o documento.
- Visual y accesibilidad: no mostrar progreso del prefetch ni emitir anuncios;
  solo mejora latencia percibida.
- Rendimiento, memoria y privacidad: máximo de trabajos y bytes definido por el
  gobernador; cero persistencia.
- Casos límite y errores: salto al final, scrubber, historial atrás/adelante,
  RTL, portada sola y documento de una página.
- Criterios de aceptación:
  - [ ] La unidad probable tiene mejor p95 que sin prefetch.
  - [ ] Cambiar dirección cancela trabajo ya irrelevante.
  - [ ] Ventana inactiva pausa prefetch no esencial.
  - [ ] No aumenta jank ni supera memoria.
- Validación: pruebas de política y perfil A/B con navegación lineal, alterna y
  aleatoria.
- Riesgos y fuera de alcance: no se aprenderán hábitos ni se persistirá
  comportamiento del usuario.

#### YAM-113 — Apertura progresiva de documentos largos

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, PERF, A11Y
- Dependencias: YAM-103, YAM-105, YAM-111
- Problema: obtener índice, texto, miniaturas o metadata completa antes del
  primer uso puede retrasar innecesariamente la lectura.
- Resultado esperado: la primera página utilizable aparece cuanto antes y el
  trabajo secundario continúa de forma cancelable.
- Cambios funcionales y técnicos:
  - Separar apertura nativa, primer layout, primer render y enriquecimiento.
  - Priorizar página restaurada o solicitada.
  - Publicar progreso por fases, no porcentajes falsos.
  - Permitir cancelar y volver a biblioteca sin esperar indexación.
- Visual y accesibilidad: estado compacto con mensaje concreto; Narrador
  anuncia fase y finalización sin repetición.
- Rendimiento, memoria y privacidad: primer render no espera miniaturas o
  índice; el trabajo secundario respeta ventana inactiva y presión.
- Casos límite y errores: documento de 1.000 páginas, cifrado, corrupción
  tardía, cierre durante indexación y reapertura inmediata.
- Criterios de aceptación:
  - [ ] Primer contenido visible tiene métrica independiente.
  - [ ] Cancelar devuelve control de UI en menos de 100 ms.
  - [ ] Indexación tardía no reemplaza estado de otro documento.
  - [ ] Errores parciales explican qué función queda disponible.
- Validación: integración con documentos largos, temporización por fase y
  pruebas de carreras.
- Riesgos y fuera de alcance: no se mostrará un porcentaje si PDFium no ofrece
  progreso real.

#### YAM-114 — Restauración precisa del viewport

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, REL, VIS
- Dependencias: YAM-101, YAM-104
- Problema: una fracción de página aproximada puede cambiar de significado al
  rotar, redimensionar, cambiar DPI o alternar Documento/Libro.
- Resultado esperado: volver al mismo punto de lectura perceptible sin saltos,
  páginas adyacentes dominantes ni zoom inesperado.
- Cambios funcionales y técnicos:
  - Definir ancla normalizada por página/unidad y punto focal.
  - Separar zoom solicitado, zoom efectivo y modo de ajuste.
  - Convertir anclas entre Documento, Libro, rotación y RTL.
  - Aplicar resize una vez por frame estable y descartar geometría antigua.
- Visual y accesibilidad: la unidad activa queda centrada o top-aligned según el
  modo; el indicador anuncia la página canónica.
- Rendimiento, memoria y privacidad: persistencia con debounce; no escribir por
  píxel ni guardar texto.
- Casos límite y errores: portada sola, último spread impar, página rotada,
  panel abierto, fullscreen y monitor con otro DPI.
- Criterios de aceptación:
  - [ ] Reapertura conserva página, ancla, zoom y modo dentro de tolerancia.
  - [ ] Resize no alterna entre dos centros.
  - [ ] Ajustar ancho conserva inicio superior.
  - [ ] Historial restaura viewport, no solo página.
- Validación: pruebas puras de transformación, widgets y matriz Windows.
- Riesgos y fuera de alcance: no se promete una coordenada tipográfica exacta
  hasta disponer de selección de texto nativa.

#### YAM-115 — Selección nativa de texto

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, A11Y, REL
- Dependencias: YAM-103, YAM-104
- Problema: el texto extraído permite buscar, pero el adaptador actual no
  expone una selección geométrica fiable para interacción y anotaciones.
- Resultado esperado: seleccionar texto con ratón o teclado, copiarlo de forma
  explícita y conservar geometría correcta.
- Cambios funcionales y técnicos:
  - Prototipar acceso a glifos, índices y rectángulos en pdfrx/PDFium.
  - Normalizar coordenadas para zoom, rotación, crop y spreads.
  - Diseñar handles/selección de escritorio sin overlay pesado.
  - Permitir selección por teclado con inicio, extensión y limpieza.
  - Copiar solo tras comando explícito.
- Visual y accesibilidad: highlight con contraste AA, foco independiente del
  color y anuncio de cantidad seleccionada.
- Rendimiento, memoria y privacidad: no persistir selección ni texto copiado;
  extraer solo páginas necesarias.
- Casos límite y errores: ligaduras, columnas, RTL, texto vertical, página sin
  texto, rotación y selección que cruza páginas.
- Criterios de aceptación:
  - [ ] Índices y rectángulos corresponden al texto copiado.
  - [ ] Zoom/rotación no desplazan el highlight.
  - [ ] Teclado y ratón completan selección y cancelación.
  - [ ] Escaneos sin texto muestran un estado claro, sin sugerir OCR.
- Validación: corpus textual generado, pruebas geométricas e integración nativa.
- Riesgos y fuera de alcance: requiere spike y puede dividirse; no se
  implementará OCR ni edición del PDF.

#### YAM-116 — Anotaciones geométricas locales

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, VIS, REL
- Dependencias: YAM-115
- Problema: las notas por página no pueden señalar un fragmento exacto sin
  geometría de selección estable.
- Resultado esperado: crear highlight, subrayado y tachado local ligados al
  texto seleccionado y revisables si cambia el archivo.
- Cambios funcionales y técnicos:
  - Persistir página, índices de texto, rectángulos normalizados, estilo y
    huella del documento.
  - Reproyectar geometría al zoom y rotación actuales.
  - Integrar undo, edición, eliminación, filtros y exportación.
  - Marcar para revisión si cambia la huella o el texto no coincide.
- Visual y accesibilidad: estilos distinguibles por forma además de color;
  selección y anotación no competirán por foco.
- Rendimiento, memoria y privacidad: overlays virtualizados por páginas
  visibles; texto sensible no se registra.
- Casos límite y errores: selección multipágina, cambio de archivo, rectángulos
  fuera de rango, anotación solapada y backup antiguo.
- Criterios de aceptación:
  - [ ] El PDF nunca se modifica.
  - [ ] Geometría sobrevive a zoom, rotación y reapertura.
  - [ ] Cambio de huella exige revisión, no recolocación silenciosa.
  - [ ] Exportación identifica tipo, página y texto autorizado.
- Validación: migración, repositorio, geometría, widgets, integración y backup.
- Riesgos y fuera de alcance: no habrá dibujo libre, firma digital ni editor de
  PDF.

#### YAM-117 — Búsqueda visible dentro del documento

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, VIS, A11Y, PERF
- Dependencias: YAM-105, YAM-113, YAM-115
- Problema: una lista de coincidencias no basta si el lector no muestra con
  precisión dónde aparece el match activo.
- Resultado esperado: buscar, entender el contexto y recorrer resultados sin
  perder posición ni bloquear la lectura.
- Cambios funcionales y técnicos:
  - Resaltar coincidencias visibles y distinguir la activa.
  - Mostrar snippets, página, recuento y progreso incremental.
  - Navegar anterior/siguiente desde panel, teclado y comandos.
  - Conservar la consulta solo durante la sesión actual.
- Visual y accesibilidad: contraste y contorno distinguibles; live region
  anuncia recuento y cambio activo una sola vez.
- Rendimiento, memoria y privacidad: cancelación inmediata, máximo de snippets
  retenidos y cero persistencia de consultas.
- Casos límite y errores: consulta vacía, documento sin texto, miles de matches,
  búsqueda cancelada y cambio de documento.
- Criterios de aceptación:
  - [ ] Match activo queda dentro del viewport.
  - [ ] Resultados parciales no reordenan de forma confusa.
  - [ ] Escape restaura foco y posición.
  - [ ] Cierre libera token y resultados.
- Validación: pruebas de controlador, integración PDFium, widgets y Narrador.
- Riesgos y fuera de alcance: no hay búsqueda semántica, fuzzy u OCR.

#### YAM-118 — Navegación unificada del lector

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, EFF, A11Y
- Dependencias: YAM-114
- Problema: página, spread, índice, miniatura, scrubber, historial y zonas
  laterales pueden producir semánticas distintas si cada superficie calcula su
  destino.
- Resultado esperado: cualquier entrada ejecuta el mismo comando y termina en
  la misma unidad de lectura.
- Cambios funcionales y técnicos:
  - Unificar destinos en un controlador de navegación.
  - Publicar operaciones por página, unidad, historial y destino PDF.
  - Reservar Page Up/Page Down y resolver conflictos de atajos.
  - Separar navegación estructural de paneo.
- Visual y accesibilidad: indicador único, foco restaurable y estados
  disabled/available anunciados.
- Rendimiento, memoria y privacidad: ningún comando fuerza render innecesario o
  escritura inmediata.
- Casos límite y errores: documento vacío, una página, final impar, RTL,
  selección en campo de texto y tecla mantenida.
- Criterios de aceptación:
  - [ ] Todas las superficies producen el mismo destino canónico.
  - [ ] Campos de texto no reciben comandos del lector.
  - [ ] Historial evita entradas duplicadas.
  - [ ] Límites no generan excepciones ni feedback engañoso.
- Validación: tests de dominio, shortcuts, widgets e integración.
- Riesgos y fuera de alcance: no se crearán bindings arbitrarios que puedan
  interceptar atajos del sistema.

#### YAM-119 — Panel lateral eficiente y accesible

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, PERF, VIS, A11Y
- Dependencias: YAM-101, YAM-105, YAM-118
- Problema: miniaturas, búsqueda, índice, marcadores y anotaciones comparten una
  superficie con cargas y focos distintos.
- Resultado esperado: un panel único, compacto y rápido que conserva sección,
  ancho, scroll y foco por documento.
- Cambios funcionales y técnicos:
  - Virtualizar listas y miniaturas.
  - Cargar sección activa bajo demanda y cancelar al cambiar.
  - Separar estado de resize de contenido.
  - Restaurar el foco al control que abrió el panel.
  - Persistir ancho acotado y sección sin reabrir trabajos antiguos.
- Visual y accesibilidad: divisor con objetivo suficiente, ajuste por teclado y
  valor anunciado; una sola sección visible.
- Rendimiento, memoria y privacidad: miniaturas por DPR, cache acotada y
  consultas no persistidas.
- Casos límite y errores: panel máximo en ventana compacta, resize durante
  búsqueda, documento cerrado y sección vacía.
- Criterios de aceptación:
  - [ ] Abrir/cerrar no cambia el zoom del PDF.
  - [ ] Scroll largo no construye todos los hijos.
  - [ ] Resize funciona con ratón y teclado.
  - [ ] Cerrar cancela renders y restaura foco.
- Validación: widgets, perfil de scroll, integración y Narrador.
- Riesgos y fuera de alcance: no se permitirán varios paneles simultáneos.

#### YAM-120 — Vista dividida con dos lectores

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, PERF, REL, VIS
- Dependencias: YAM-104, YAM-111, YAM-118, YAM-154
- Problema: consultar dos documentos exige cambiar contexto repetidamente y el
  dominio existente no demuestra dos visores reales.
- Resultado esperado: leer dos PDF lado a lado con estado, foco y recursos
  independientes.
- Cambios funcionales y técnicos:
  - Crear dos sesiones lectoras explícitas dentro del workspace.
  - Repartir viewport y memoria con mínimos legibles.
  - Enrutar comandos al lector enfocado.
  - Cerrar cada visor de forma independiente.
  - Guardar split solo con consentimiento.
- Visual y accesibilidad: divisor accesible, lector activo perceptible sin
  acento saturado y colapso automático cuando falta espacio.
- Rendimiento, memoria y privacidad: presupuesto agregado obligatorio; no
  mantener un tercer visor invisible.
- Casos límite y errores: mismo PDF dos veces, uno cifrado, cierre de un lado,
  ventana compacta y presión alta.
- Criterios de aceptación:
  - [ ] Cada lado conserva página, zoom, panel y comandos.
  - [ ] Cerrar un lado libera sus recursos.
  - [ ] Foco determina destino de todos los atajos.
  - [ ] Perfil de dos lectores permanece dentro del gate.
- Validación: dominio, widgets, integración con dos fixtures y perfil nativo.
- Riesgos y fuera de alcance: no se expone hasta superar lifecycle y memoria.

#### YAM-121 — Comparación sincronizada

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: L
- Fase: 1
- Áreas: FUNC, EFF, VIS
- Dependencias: YAM-120
- Problema: dos documentos relacionados requieren avanzar de forma coordinada,
  pero una sincronización rígida puede impedir inspeccionar diferencias.
- Resultado esperado: sincronizar página, zoom o posición de forma opcional y
  desacoplar temporalmente sin perder el vínculo.
- Cambios funcionales y técnicos:
  - Definir modos independiente, página, zoom y posición.
  - Evitar bucles de eventos mediante origen y generación.
  - Permitir offset de páginas entre documentos.
  - Mostrar estado de vínculo y acción de resincronizar.
- Visual y accesibilidad: indicador discreto con texto/semántica, no solo icono
  o color.
- Rendimiento, memoria y privacidad: sincronización no duplica render ni
  persiste contenido.
- Casos límite y errores: distinto número/tamaño de páginas, rotación, fin de un
  documento y usuario manipulando ambos lados.
- Criterios de aceptación:
  - [ ] No existen bucles de navegación.
  - [ ] Desacoplar conserva ambos viewports.
  - [ ] Resincronizar tiene destino predecible.
  - [ ] Estado se anuncia y puede cambiarse con teclado.
- Validación: tests de política, widgets e integración.
- Riesgos y fuera de alcance: no se compararán píxeles ni texto
  automáticamente.

#### YAM-122 — Ventanas de lectura independientes

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, REL, WIN
- Dependencias: YAM-104, YAM-111, YAM-120
- Problema: una sola ventana limita consultas paralelas, pero multi-window puede
  duplicar servicios globales y romper shortcuts o cierre.
- Resultado esperado: abrir hasta dos ventanas lectoras con contexto y recursos
  bien delimitados.
- Cambios funcionales y técnicos:
  - Prototipar ownership por ventana y servicios compartidos seguros.
  - Enrutar comandos, diálogos y notificaciones a la ventana activa.
  - Definir restauración opt-in y límites de ventanas.
  - Coordinar cierre del último proceso y base compartida.
- Visual y accesibilidad: título, taskbar, foco y diálogos pertenecen a su
  ventana; no aparecen controles globales ambiguos.
- Rendimiento, memoria y privacidad: límite de dos lectores y presupuesto
  agregado.
- Casos límite y errores: cerrar principal antes que secundaria, mismo PDF,
  monitor desconectado y actualización concurrente de progreso.
- Criterios de aceptación:
  - [ ] Cada ventana tiene estado y comandos propios.
  - [ ] Base compartida no produce carreras.
  - [ ] Cerrar todas deja cero handles.
  - [ ] Restauración nunca abre rutas ausentes en bucle.
- Validación: spike nativo, ADR, integración multi-window y perfil.
- Riesgos y fuera de alcance: no se añade plugin hasta aprobar licencia,
  mantenimiento y bundle.

#### YAM-123 — Presentación y pantalla completa endurecidas

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: M
- Fase: 1
- Áreas: FUNC, VIS, A11Y, WIN
- Dependencias: YAM-101, YAM-114, YAM-118
- Problema: una sesión de presentación necesita chrome mínimo, salida clara y
  control de suspensión sin convertir el lector en una interfaz distinta.
- Resultado esperado: mostrar el PDF de forma inmersiva y salir siempre con
  Escape o un control revelable.
- Cambios funcionales y técnicos:
  - Añadir keep-awake local y opt-in durante presentación.
  - Ocultar panel, barras y zonas decorativas sin perder navegación.
  - Restaurar geometría y chrome al salir.
  - Manejar pérdida de foco, monitor y cierre.
- Visual y accesibilidad: indicador temporal de entrada/salida, control
  accesible y movimiento reducido.
- Rendimiento, memoria y privacidad: no precargar todo el documento ni registrar
  duración.
- Casos límite y errores: Escape capturado por diálogo, taskbar, doble monitor,
  cambio DPI y app minimizada.
- Criterios de aceptación:
  - [ ] Siempre existe salida por teclado.
  - [ ] Keep-awake termina al salir o cerrar.
  - [ ] Viewport se restaura sin salto.
  - [ ] No queda chrome interactivo invisible.
- Validación: widgets, integración Windows y recorrido manual en dos monitores.
- Riesgos y fuera de alcance: no incluye control remoto ni transmisión.

#### YAM-124 — Impresión local explícita

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: XL
- Fase: 1
- Áreas: FUNC, REL, WIN
- Dependencias: YAM-100, YAM-103, YAM-160
- Problema: imprimir exige integración Windows segura y puede crear copias
  temporales o exponer rutas si se delega sin contrato.
- Resultado esperado: imprimir el PDF o un rango mediante diálogo nativo y
  acción inequívoca.
- Cambios funcionales y técnicos:
  - Evaluar API nativa o dependencia mantenida mediante ADR.
  - Delegar documento original sin copiarlo silenciosamente cuando sea posible.
  - Soportar rango, orientación y cancelación proporcionados por Windows.
  - Limpiar temporales si la plataforma exige generarlos.
- Visual y accesibilidad: comando visible en ficha/paleta, diálogo nativo y
  errores claros con foco restaurado.
- Rendimiento, memoria y privacidad: no registrar impresora, rango o ruta; no
  mantener documento después de cancelar.
- Casos límite y errores: PDF cifrado, impresora ausente, spooler detenido,
  rango inválido y cancelación.
- Criterios de aceptación:
  - [ ] Imprimir requiere acción explícita.
  - [ ] Cancelar no deja temporales propios.
  - [ ] Errores no cierran el lector.
  - [ ] Dependencia/licencia/bundle quedan aprobados.
- Validación: pruebas de contrato, smoke con impresora virtual y cancelación.
- Riesgos y fuera de alcance: no incluye edición, firma ni creación de un PDF
  alternativo.

### Fase 2 — Biblioteca y organización

#### YAM-130 — Biblioteca escalable a 5.000 documentos

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 2
- Áreas: FUNC, PERF, EFF
- Dependencias: YAM-153, YAM-154
- Problema: mantener todos los documentos y cálculos visuales residentes no es
  sostenible al crecer desde decenas a miles.
- Resultado esperado: buscar, filtrar, ordenar y recorrer 5.000 documentos con
  respuesta inmediata y memoria acotada.
- Cambios funcionales y técnicos:
  - Introducir consultas paginadas y ventanas de datos.
  - Separar conteos/filtros del materializado de tarjetas.
  - Mantener selección por identidad aunque salga del viewport.
  - Virtualizar layouts y medir coste de jitter, portadas y semántica.
- Visual y accesibilidad: posición estable, skeleton discreto y anuncio de
  recuento sin crear miles de nodos semánticos.
- Rendimiento, memoria y privacidad: consultas p95 inferiores a 20 ms; no cargar
  blobs ni texto PDF en la biblioteca.
- Casos límite y errores: cero resultados, filtro cambia durante paginación,
  elemento retirado y orden con valores nulos.
- Criterios de aceptación:
  - [ ] 5.000 documentos no construyen 5.000 widgets.
  - [ ] Scroll, selección y filtros permanecen estables.
  - [ ] Memoria no crece linealmente con páginas ya descartadas.
  - [ ] Consultas cumplen gate.
- Validación: benchmark SQLite, widgets con dataset sintético e integración.
- Riesgos y fuera de alcance: no se convierte Yamantaka en gestor documental
  empresarial ni se escanea el disco completo.

#### YAM-131 — Cola de importación recuperable

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, EFF, REL, A11Y
- Dependencias: YAM-105, YAM-130
- Problema: una importación grande necesita progreso, cancelación y errores por
  archivo sin bloquear registro o portada.
- Resultado esperado: revisar una carpeta, confirmar, importar y continuar
  usando la biblioteca mientras la cola termina.
- Cambios funcionales y técnicos:
  - Separar descubrimiento, inspección, registro, portada e indexación.
  - Persistir solo trabajos confirmados que puedan reanudarse de forma segura.
  - Aislar fallos por archivo y permitir reintento.
  - Deduplicar rutas y huellas antes de trabajo costoso.
- Visual y accesibilidad: resumen compacto, progreso por fases, lista de errores
  navegable y cancelación anunciada.
- Rendimiento, memoria y privacidad: concurrencia acotada, sin lectura completa
  innecesaria ni rutas en logs.
- Casos límite y errores: carpeta desconectada, permisos, symlinks, miles de
  archivos, cancelación y cierre.
- Criterios de aceptación:
  - [ ] Biblioteca sigue receptiva.
  - [ ] Cancelar detiene trabajo futuro y conserva registros válidos.
  - [ ] Reabrir no duplica documentos.
  - [ ] Error individual no aborta el lote.
- Validación: tests de cola, filesystem temporal, widgets e integración.
- Riesgos y fuera de alcance: no se monitorizan carpetas hasta YAM-133.

#### YAM-132 — Duplicados y fusión segura de metadatos

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, REL
- Dependencias: YAM-130, YAM-131
- Problema: copias o rutas movidas pueden fragmentar progreso, etiquetas,
  anotaciones y portadas.
- Resultado esperado: identificar duplicados probables y resolverlos sin borrar
  archivos ni perder información.
- Cambios funcionales y técnicos:
  - Clasificar coincidencia exacta, probable y solo-ruta.
  - Mostrar diferencias de metadata antes de fusionar.
  - Definir precedencia explícita por campo y conservar backup transaccional.
  - Permitir mantener ambos documentos.
- Visual y accesibilidad: comparación textual sobria, foco claro y confirmación
  que explica que los PDF no se borran.
- Rendimiento, memoria y privacidad: hashes progresivos y reutilizables; no
  enviar muestras ni nombres fuera del dispositivo.
- Casos límite y errores: archivos iguales modificados, anotaciones en conflicto,
  ruta inaccesible y fusión interrumpida.
- Criterios de aceptación:
  - [ ] Nunca se fusiona automáticamente un caso probable.
  - [ ] La operación es transaccional y reversible mediante prebackup.
  - [ ] Progreso, historial y anotaciones tienen regla documentada.
  - [ ] Mantener ambos sigue disponible.
- Validación: repositorio, migración, conflictos y diálogo accesible.
- Riesgos y fuera de alcance: no se borran duplicados físicos ni se compara
  contenido visual página a página.

#### YAM-133 — Recuperación y carpetas autorizadas

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 2
- Áreas: FUNC, EFF, REL, WIN
- Dependencias: YAM-105, YAM-130, YAM-132
- Problema: documentos movidos quedan ausentes y una monitorización indiscriminada
  sería costosa e invasiva.
- Resultado esperado: relocalizar archivos de forma asistida y observar solo
  carpetas elegidas expresamente.
- Cambios funcionales y técnicos:
  - Buscar por nombre, tamaño, fecha y huella dentro de una raíz confirmada.
  - Añadir watchers con debounce, límites y pausa.
  - Convertir eventos en revalidaciones, no en importaciones automáticas.
  - Mostrar propuesta antes de cambiar una ruta.
- Visual y accesibilidad: estado Ausente conserva geometría, acción Localizar y
  explicación del alcance observado.
- Rendimiento, memoria y privacidad: no escanear unidades completas; watchers
  pausados cuando la carpeta no existe o la app está inactiva.
- Casos límite y errores: unidad externa, rename masivo, tormenta de eventos,
  symlink, permisos y dos candidatos.
- Criterios de aceptación:
  - [ ] Solo se observan raíces autorizadas.
  - [ ] Ningún evento borra o importa automáticamente.
  - [ ] Relocalización requiere confirmación si no es exacta.
  - [ ] Pausa y retirada de permiso son inmediatas.
- Validación: filesystem temporal, eventos simulados e integración Windows.
- Riesgos y fuera de alcance: no hay indexador de todo el sistema.

#### YAM-134 — Organización y vistas inteligentes

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, VIS, EFF
- Dependencias: YAM-130, YAM-138, YAM-153
- Problema: colecciones, etiquetas, favoritos, estado y búsqueda pueden saturar
  la interfaz si se presentan como controles independientes permanentes.
- Resultado esperado: guardar combinaciones útiles como vistas y recuperarlas
  desde una superficie compacta.
- Cambios funcionales y técnicos:
  - Definir reglas tipadas sobre metadata local.
  - Permitir colecciones manuales y vistas calculadas claramente diferenciadas.
  - Guardar filtro, orden y densidad sin guardar consultas sensibles por defecto.
  - Evaluar reglas en una consulta o pasada medible.
- Visual y accesibilidad: selector editorial, recuento, estado vacío y acción de
  editar; sin nube de chips.
- Rendimiento, memoria y privacidad: reglas locales, paginadas e indexadas; las
  consultas textuales requieren opt-in para guardarse.
- Casos límite y errores: regla vacía, colección borrada, nombre duplicado y
  regla incompatible tras migración.
- Criterios de aceptación:
  - [ ] Reglas son comprensibles y reversibles.
  - [ ] Combinar filtros no multiplica consultas.
  - [ ] Borrar vista no afecta documentos.
  - [ ] Teclado completa crear, activar y eliminar.
- Validación: dominio, repositorio, consultas, widgets y backup.
- Riesgos y fuera de alcance: no se incluye lenguaje de consultas arbitrario.

#### YAM-135 — Acciones masivas accesibles

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, VIS, A11Y, REL
- Dependencias: YAM-130, YAM-134
- Problema: repetir la misma acción documento a documento es ineficiente, pero
  una selección masiva puede ocultar riesgos.
- Resultado esperado: seleccionar varios artefactos y ejecutar acciones locales
  seguras con resumen previo.
- Cambios funcionales y técnicos:
  - Añadir selección múltiple por Ctrl/Shift y comandos.
  - Implementar favorito, etiquetas, colección, archivar, retirar y regenerar
    portada.
  - Separar operaciones reversibles de las que requieren confirmación.
  - Procesar lotes con progreso y errores por elemento.
- Visual y accesibilidad: barra contextual temporal, recuento anunciado y foco
  visible en cada seleccionado.
- Rendimiento, memoria y privacidad: operaciones agrupadas y transaccionales
  cuando proceda; no cargar PDF para cambios de metadata.
- Casos límite y errores: selección filtrada, elemento desaparece, fallo parcial
  y Escape.
- Criterios de aceptación:
  - [ ] Selección no depende solo del color.
  - [ ] Retirar nunca borra el PDF.
  - [ ] Fallo parcial muestra resultado por elemento.
  - [ ] Escape limpia selección y restaura foco.
- Validación: dominio de selección, widgets, repositorio y Narrador.
- Riesgos y fuera de alcance: no habrá mover/copiar/borrar archivos en lote.

#### YAM-136 — Pipeline de portadas mejorado

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, PERF, VIS, REL
- Dependencias: YAM-105, YAM-111, YAM-130
- Problema: una cola serial segura puede tardar con miles de documentos y no
  explica si una portada está pendiente, cancelada o fallida.
- Resultado esperado: portadas nítidas y estables aparecen según prioridad
  visible sin bloquear importación.
- Cambios funcionales y técnicos:
  - Priorizar documentos visibles, seleccionados y recién abiertos.
  - Mantener escalas por DPR y regenerar al cambiar algoritmo.
  - Añadir reintento acotado y estado de error persistible.
  - Conservar selección manual como autoridad.
  - Medir coste de scoring y codificación.
- Visual y accesibilidad: placeholder mantiene proporción; pendiente/error se
  distinguen por icono, texto y semántica.
- Rendimiento, memoria y privacidad: máximo de trabajos, bytes y reintentos;
  análisis local sin OCR.
- Casos límite y errores: PDF cifrado, seis páginas vacías, cancelación,
  documento retirado y cache sin espacio.
- Criterios de aceptación:
  - [ ] Visible gana prioridad sin starvation.
  - [ ] Manual nunca es sobrescrita.
  - [ ] Archivos parciales se eliminan.
  - [ ] Cientos de portadas cumplen gate de memoria.
- Validación: política de cola, fixtures, filesystem y perfil de biblioteca.
- Riesgos y fuera de alcance: no se reconocerá semántica editorial mediante IA.

#### YAM-137 — Editor local de metadatos

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: M
- Fase: 2
- Áreas: FUNC, VIS, REL
- Dependencias: YAM-130
- Problema: títulos o autores extraídos pueden ser poco útiles y hoy no existe
  una corrección local clara.
- Resultado esperado: editar título, autor y notas de biblioteca sin tocar el
  archivo PDF.
- Cambios funcionales y técnicos:
  - Separar metadata original, detectada y override local.
  - Validar longitud, espacios y campos vacíos.
  - Incluir overrides en backup, búsqueda e importación.
  - Permitir restaurar valor original.
- Visual y accesibilidad: formulario compacto con etiquetas persistentes,
  errores asociados y foco predecible.
- Rendimiento, memoria y privacidad: escritura SQLite pequeña y con debounce
  solo cuando sea necesario.
- Casos límite y errores: título vacío, caracteres Unicode, documento ausente,
  conflicto de backup y edición concurrente.
- Criterios de aceptación:
  - [ ] PDF no cambia.
  - [ ] Override domina en UI e índice.
  - [ ] Restaurar elimina override.
  - [ ] Backup conserva origen y override sin ambigüedad.
- Validación: repositorio, backup, búsqueda y widgets.
- Riesgos y fuera de alcance: no se editarán XMP, bookmarks o contenido interno.

#### YAM-138 — Índice local incremental

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 2
- Áreas: FUNC, PERF, EFF, REL
- Dependencias: YAM-105, YAM-153
- Problema: indexar todo de nuevo es costoso y una interrupción puede dejar
  estados difíciles de explicar.
- Resultado esperado: índice reconstruible, incremental, pausable y visible en
  diagnóstico.
- Cambios funcionales y técnicos:
  - Versionar algoritmo y estado por documento/página.
  - Priorizar documento activo y cambios de huella.
  - Procesar lotes transaccionales.
  - Exponer pausa, reanudar, reconstruir y borrar.
  - Validar integridad entre índice y documentos.
- Visual y accesibilidad: progreso en ajustes/privacidad, no banner permanente;
  controles anunciados.
- Rendimiento, memoria y privacidad: texto permanece SQLite local; consultas no
  se registran ni persisten.
- Casos límite y errores: cierre durante lote, disco lleno, documento cambia,
  FTS no disponible y reconstrucción cancelada.
- Criterios de aceptación:
  - [ ] Reapertura continúa desde estado estable.
  - [ ] Índice corrupto se reconstruye sin afectar metadata.
  - [ ] Documento modificado invalida solo sus entradas.
  - [ ] Pausa devuelve CPU/IO rápidamente.
- Validación: migración, fallos inyectados, benchmark y búsqueda global.
- Riesgos y fuera de alcance: no se indexan imágenes ni se hace OCR.

#### YAM-139 — Espacios de trabajo y modo estudio

- Estado: PLANNED
- Prioridad: P2
- Esfuerzo: XL
- Fase: 2
- Áreas: FUNC, VIS, A11Y
- Dependencias: YAM-118, YAM-134, YAM-138
- Problema: los contextos actuales pueden restaurar documentos, pero falta una
  superficie coherente para sesiones de estudio sin añadir pestañas visibles.
- Resultado esperado: fijar pocos documentos, notas y destinos dentro de un
  workspace recuperable y discreto.
- Cambios funcionales y técnicos:
  - Definir inbox, documentos fijados, historial y nota de sesión.
  - Abrir workspace desde paleta o biblioteca.
  - Integrar split solo cuando YAM-120 esté disponible.
  - Guardar contexto con límites y recuperación segura.
- Visual y accesibilidad: overlay o diálogo dedicado, nunca franja persistente;
  orden navegable y acciones con texto.
- Rendimiento, memoria y privacidad: suspender documentos no visibles y guardar
  solo IDs/posiciones locales.
- Casos límite y errores: archivo ausente, workspace antiguo, más documentos que
  límite y cierre inesperado.
- Criterios de aceptación:
  - [ ] No aparecen tabs sobre el lector.
  - [ ] Reapertura falla de forma parcial y explicada.
  - [ ] Documentos suspendidos no retienen PDFium.
  - [ ] Teclado completa crear, abrir, ordenar y eliminar.
- Validación: dominio, repositorio, recuperación, widgets e integración.
- Riesgos y fuera de alcance: no hay colaboración, sincronización o canvas
  infinito.

### Fase 3 — Limpieza y evolución visual

#### YAM-140 — Auditoría completa del sistema visual

- Estado: READY
- Prioridad: P0
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, EFF, A11Y
- Dependencias: ninguna
- Problema: aunque existen tokens, valores directos de tamaño, borde, sombra,
  elevación o duración pueden producir divergencias.
- Resultado esperado: todas las superficies comparten un lenguaje visual
  pequeño, legible y fácil de ajustar.
- Cambios funcionales y técnicos:
  - Inventariar valores visuales y clasificarlos como token o excepción.
  - Centralizar escalas tipográficas, espacios, tamaños, radios, bordes,
    elevaciones, sombras y movimiento.
  - Eliminar tokens duplicados o sin uso.
  - Documentar cuándo una excepción está permitida.
- Visual y accesibilidad: preservar identidad monocromática, foco fuerte y
  contraste; reducir sombras y chrome.
- Rendimiento, memoria y privacidad: tokens constantes; cero paquete visual
  nuevo.
- Casos límite y errores: tema oscuro, alto contraste, densidad compacta y
  movimiento reducido.
- Criterios de aceptación:
  - [ ] No quedan valores mágicos repetidos sin justificar.
  - [ ] Claro/oscuro comparten semántica.
  - [ ] Cambiar un token no rompe geometría fuera de su categoría.
  - [ ] Documentación y tests reflejan la escala final.
- Validación: búsqueda estática, tests de tokens y revisión de capturas.
- Riesgos y fuera de alcance: no se rediseñará branding ni se añadirá una nueva
  librería de componentes.

#### YAM-141 — Refinamiento visual de la biblioteca

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, FUNC, A11Y, PERF
- Dependencias: YAM-101, YAM-102, YAM-130, YAM-140
- Problema: una colección espacial debe seguir siendo eficiente y predecible
  cuando cambia cantidad, ventana, selección o metadata.
- Resultado esperado: portadas con presencia editorial, información gradual y
  navegación clara sin parecer una cuadrícula genérica.
- Cambios funcionales y técnicos:
  - Ajustar escala, jitter, ritmo, columnas y reserva de información.
  - Reducir chrome de filtros y acciones permanentes.
  - Unificar selección, foco, hover, ausente y error.
  - Mantener slivers y caché física por DPR.
- Visual y accesibilidad: selección no depende del color; título largo tiene
  tooltip y semántica completa.
- Rendimiento, memoria y privacidad: no animar listas completas ni decodificar
  por encima del tamaño visible.
- Casos límite y errores: 0, 1, 2, 6, 7, 100 y 5.000 elementos; portada extrema,
  texto largo y ventana compacta.
- Criterios de aceptación:
  - [ ] No hay overlap o overflow en matriz.
  - [ ] Foco visual sigue el orden de navegación.
  - [ ] Información aparece sin salto de layout.
  - [ ] 16:9/16:10/2K conservan densidad prevista.
- Validación: geometría pura, widgets, capturas y perfil.
- Riesgos y fuera de alcance: no se usa el logo ni se convierte en estantería
  fotorrealista.

#### YAM-142 — Simplificación del chrome del lector

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, FUNC, A11Y
- Dependencias: YAM-101, YAM-102, YAM-114, YAM-140
- Problema: el lector acumula barra superior, controles inferiores, panel,
  zonas laterales e indicadores que pueden competir con el PDF.
- Resultado esperado: el documento domina y las acciones frecuentes siguen
  disponibles sin menús profundos.
- Cambios funcionales y técnicos:
  - Auditar frecuencia de cada control.
  - Mantener navegación, página, zoom, modo y panel en la primera capa.
  - Mover acciones infrecuentes a menú/paleta.
  - Unificar auto-hide global y preferencias por documento.
  - Eliminar zonas invisibles en fullscreen.
- Visual y accesibilidad: chrome compacto, foco visible, tooltips y etiquetas
  semánticas completas.
- Rendimiento, memoria y privacidad: ocultación no recrea el visor ni cambia
  zoom; animaciones de 100–180 ms o cero.
- Casos límite y errores: ratón quieto, teclado, campo de texto, fullscreen,
  panel abierto y movimiento reducido.
- Criterios de aceptación:
  - [ ] PDF conserva viewport al mostrar/ocultar chrome.
  - [ ] Acciones principales requieren máximo una interacción.
  - [ ] Fullscreen no deja controles invisibles activos.
  - [ ] Teclado revela controles y restaura foco.
- Validación: widgets, capturas e integración.
- Riesgos y fuera de alcance: no se ocultan funciones sin ruta alternativa.

#### YAM-143 — Reorganización integral de Ajustes

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 3
- Áreas: FUNC, VIS, A11Y
- Dependencias: YAM-102, YAM-140
- Problema: muchas capacidades locales pueden convertir Ajustes en una lista
  extensa, difícil de buscar y con acciones de riesgo mezcladas.
- Resultado esperado: encontrar una preferencia por intención, entender su
  efecto y aplicarla sin alterar el contexto del lector.
- Cambios funcionales y técnicos:
  - Reagrupar en Apariencia, Lectura, Biblioteca, Atajos, Datos, Privacidad y
    Avanzado.
  - Añadir búsqueda local de etiquetas y descripciones.
  - Separar preferencias inmediatas de operaciones.
  - Diseñar preview ligero sin montar un PDF real.
  - Detectar cambios externos y conflictos.
- Visual y accesibilidad: navegación lateral en escritorio, fallback compacto,
  encabezados persistentes y controles con labels.
- Rendimiento, memoria y privacidad: búsqueda sobre catálogo estático; abrir
  Ajustes no inicia indexación ni visor.
- Casos límite y errores: ventana compacta, texto largo, reset parcial,
  directorio no disponible y operación en progreso.
- Criterios de aceptación:
  - [ ] Cada ajuste tiene propietario, valor por defecto y efecto inmediato o
    explícito.
  - [ ] Cambiar tema/densidad no cambia zoom PDF.
  - [ ] Reset explica alcance y conserva datos fuera de selección.
  - [ ] Teclado y Narrador recorren todas las secciones.
- Validación: catálogo, repositorio, widgets, capturas y Narrador.
- Riesgos y fuera de alcance: no se añade cuenta, nube o telemetría.

#### YAM-144 — Primitivas compartidas para overlays

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, EFF, A11Y
- Dependencias: YAM-102, YAM-140
- Problema: diálogos, menús, paleta, tooltips y overlays pueden divergir en
  tamaño, foco, elevación, cierre y mensajes.
- Resultado esperado: superficies temporales coherentes y predecibles.
- Cambios funcionales y técnicos:
  - Crear primitivas para encabezado, acciones, ancho, scroll, loading y error.
  - Unificar foco inicial, trampa de foco, Escape y restauración.
  - Centralizar confirmaciones de riesgo.
  - Definir límites por resolución y contenido.
- Visual y accesibilidad: borde/elevación discretos, contraste AA y nombre/rol
  correcto.
- Rendimiento, memoria y privacidad: overlays cerrados liberan controladores y
  no conservan entradas sensibles.
- Casos límite y errores: overlay sobre fullscreen, diálogo anidado, texto largo,
  teclado IME y operación asíncrona.
- Criterios de aceptación:
  - [ ] Escape cierra solo la capa superior.
  - [ ] Foco vuelve al invocador.
  - [ ] Ninguna superficie desborda ventana.
  - [ ] Riesgo usa texto y no solo color.
- Validación: tests de primitivas, auditoría de usos y capturas.
- Riesgos y fuera de alcance: no se construirá un framework de UI separado de
  Flutter/Material.

#### YAM-145 — Estados vacíos, carga y error coherentes

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 3
- Áreas: FUNC, VIS, A11Y, REL
- Dependencias: YAM-103, YAM-140, YAM-144
- Problema: ausencia, contraseña, corrupción, cancelación, fallo parcial y
  recuperación requieren mensajes distintos y acciones seguras.
- Resultado esperado: cada estado explica qué ocurrió, qué se conserva y cuál
  es la siguiente acción recomendable.
- Cambios funcionales y técnicos:
  - Catalogar estados por feature y FailureKind.
  - Asignar título, explicación, acción principal, secundaria y diagnóstico.
  - Evitar spinners indefinidos mediante timeout y transición.
  - Conservar geometría para reducir saltos.
- Visual y accesibilidad: una acción principal, iconografía neutra, live region
  para transiciones y foco inicial útil.
- Rendimiento, memoria y privacidad: mensajes no incluyen ruta completa ni
  debug privado.
- Casos límite y errores: error durante error, reintento cancelado, sin espacio,
  password errónea y archivo que desaparece.
- Criterios de aceptación:
  - [ ] Todos los fallos tipados tienen representación.
  - [ ] Cancelación no se muestra como error crítico.
  - [ ] Reintento no duplica trabajo.
  - [ ] Narrador anuncia cambio una vez.
- Validación: catálogo exhaustivo, widgets y corpus.
- Riesgos y fuera de alcance: no se ocultan detalles necesarios; diagnóstico
  ampliado queda en YAM-161.

#### YAM-146 — Tipografía e iconografía

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: M
- Fase: 3
- Áreas: VIS, A11Y
- Dependencias: YAM-140
- Problema: escalas, truncado e iconos inconsistentes reducen claridad incluso
  cuando el layout cabe.
- Resultado esperado: lectura inmediata de jerarquía, acciones reconocibles y
  texto legible en DPI alto.
- Cambios funcionales y técnicos:
  - Revisar escala tipográfica, alturas de línea y pesos.
  - Normalizar iconos, tamaños ópticos y labels.
  - Definir truncado, wrap y tooltip por tipo de contenido.
  - Evitar caracteres Unicode usados como iconos de acción.
- Visual y accesibilidad: texto ampliado no se corta; iconos tienen tooltip y
  nombre semántico.
- Rendimiento, memoria y privacidad: reutilizar fuentes del sistema; no añadir
  paquetes o assets pesados.
- Casos límite y errores: títulos CJK/RTL, rutas largas, zoom de texto y fuente
  fallback.
- Criterios de aceptación:
  - [ ] Cada estilo tiene función documentada.
  - [ ] No hay texto esencial truncado sin alternativa.
  - [ ] Iconos iguales significan la misma acción.
  - [ ] Matriz DPI conserva alineación.
- Validación: capturas, auditoría semántica y prueba de strings extremos.
- Riesgos y fuera de alcance: no se diseña una tipografía propia.

#### YAM-147 — Layout adaptativo de escritorio

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, FUNC, WIN
- Dependencias: YAM-101, YAM-140
- Problema: breakpoints generales no cubren por sí solos proporción, altura,
  panel, DPI, ultrawide o cambios de monitor.
- Resultado esperado: una interfaz estable para 16:9, 16:10, Full HD, 2K, 4K,
  compacta y ultrawide.
- Cambios funcionales y técnicos:
  - Definir perfiles por espacio lógico disponible, no resolución física sola.
  - Restar panel/chrome antes de calcular PDF y grid.
  - Acotar ancho de lectura y distribuir espacio sobrante.
  - Mantener mínimos de ventana y fallback de una columna.
- Visual y accesibilidad: el PDF no se estira; controles no quedan lejos sin
  razón y el orden visual coincide con foco.
- Rendimiento, memoria y privacidad: DPR solo afecta decodificación física;
  resize se debounced por frame.
- Casos límite y errores: ultrawide, 4K a 200 %, ventana muy baja, taskbar y
  cambio en vivo.
- Criterios de aceptación:
  - [ ] Todos los perfiles de G4 pasan.
  - [ ] Full HD usa hasta ocho columnas y 2K hasta diez cuando procede.
  - [ ] 16:10 aprovecha altura.
  - [ ] Ultrawide conserva centro y escala legible.
- Validación: métricas puras, widgets y matriz nativa.
- Riesgos y fuera de alcance: no se optimiza para mosaicos de más de dos
  ventanas simultáneas.

#### YAM-148 — Accesibilidad completa con Narrador

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: XL
- Fase: 3
- Áreas: A11Y, FUNC, WIN
- Dependencias: YAM-101, YAM-140, YAM-147
- Problema: semántica de widgets no demuestra comportamiento con Narrador,
  teclado y DPI reales.
- Resultado esperado: completar biblioteca, lector, panel, paleta, ajustes y
  diálogos sin ratón.
- Cambios funcionales y técnicos:
  - Inventariar orden de foco y shortcuts por superficie.
  - Añadir names, roles, states, values y live regions.
  - Corregir anuncios duplicados y foco perdido.
  - Documentar patrones de retorno tras overlay.
  - Ejecutar matriz Narrador real.
- Visual y accesibilidad: foco visible 3:1, objetivos de escritorio al menos
  40×40 y estados no dependientes del color.
- Rendimiento, memoria y privacidad: semántica virtualizada; no anunciar cientos
  de elementos fuera del viewport.
- Casos límite y errores: modal, fullscreen, resize, resultados dinámicos,
  selección múltiple y movimiento reducido.
- Criterios de aceptación:
  - [ ] Flujos principales completados sin ratón.
  - [ ] Narrador no duplica cambios.
  - [ ] Foco nunca queda atrapado o perdido.
  - [ ] Evidencia existe para cada superficie y DPI.
- Validación: tests de semantics, checklist manual y grabación textual de
  esperado/observado.
- Riesgos y fuera de alcance: no se declarará DONE solo con tests Flutter.

#### YAM-149 — Alto contraste, movimiento reducido y tinta electrónica

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 3
- Áreas: VIS, A11Y, PERF, WIN
- Dependencias: YAM-101, YAM-140, YAM-148
- Problema: oscuro no equivale a alto contraste y un perfil e-ink requiere más
  que desaturar colores.
- Resultado esperado: superficies legibles con alto contraste de Windows,
  animaciones eliminables y refresco visual contenido.
- Cambios funcionales y técnicos:
  - Detectar alto contraste del sistema y mapear tokens semánticos.
  - Hacer que reduced motion elimine desplazamientos y fades no esenciales.
  - Revisar perfil e-ink: blanco/negro, bordes claros, sin sombras y menos
    repaints.
  - Mantener feedback mediante forma, borde y texto.
- Visual y accesibilidad: contraste AA/3:1 y foco siempre visible.
- Rendimiento, memoria y privacidad: e-ink reduce animación y repaints; no cambia
  calidad del PDF salvo opción explícita.
- Casos límite y errores: cambio de tema en vivo, canvas oscuro, selección,
  error y fullscreen.
- Criterios de aceptación:
  - [ ] Alto contraste no depende de temas personalizados.
  - [ ] Reduced motion produce duración cero donde corresponde.
  - [ ] E-ink elimina sombras/animación no esencial.
  - [ ] Ningún estado desaparece por pérdida de color.
- Validación: tests de tokens, capturas y revisión en dispositivo si está
  disponible.
- Riesgos y fuera de alcance: no se promete rendimiento de hardware e-ink sin
  dispositivo real.

### Fase 4 — Rendimiento, eficiencia y fiabilidad

#### YAM-150 — Arranque perezoso

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: PERF, EFF, REL
- Dependencias: YAM-104
- Problema: inicializar todos los servicios antes de mostrar biblioteca aumenta
  latencia y acopla fallos secundarios al arranque.
- Resultado esperado: ventana útil rápidamente y servicios secundarios
  inicializados después o bajo demanda.
- Cambios funcionales y técnicos:
  - Medir bootstrap por servicio.
  - Clasificar crítico, diferible y bajo demanda.
  - Iniciar diagnóstico, índice, portadas y mantenimiento después del primer
    frame estable.
  - Aislar fallos secundarios y exponer reintento.
- Visual y accesibilidad: biblioteca muestra contenido o estado inmediato, no
  splash prolongado.
- Rendimiento, memoria y privacidad: p95 de ventana visible ≤1,2 s; no iniciar
  escaneos ocultos en primer frame.
- Casos límite y errores: DB migrando, safe mode, restore opt-in, ruta explícita
  y servicio diferido fallido.
- Criterios de aceptación:
  - [ ] Timeline identifica cada fase.
  - [ ] Solo críticos bloquean primera superficie.
  - [ ] Fallo diferido no cierra app.
  - [ ] Gate de arranque se cumple.
- Validación: instrumentación, procesos fríos e integración.
- Riesgos y fuera de alcance: no se ocultan migraciones necesarias para
  integridad.

#### YAM-151 — Estado granular y reducción de rebuilds

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 4
- Áreas: PERF, EFF
- Dependencias: YAM-104, YAM-140
- Problema: ChangeNotifier amplios pueden reconstruir superficies que no
  consumen el valor modificado.
- Resultado esperado: cambios de hover, progreso, zoom o búsqueda actualizan
  únicamente widgets relevantes.
- Cambios funcionales y técnicos:
  - Perfilar rebuilds de biblioteca, lector y panel.
  - Separar estado persistente, efímero y derivado.
  - Introducir selectores/listenables pequeños donde la medición lo justifique.
  - Encapsular repaints de portadas y páginas.
- Visual y accesibilidad: optimización no retrasa foco, semántica o feedback.
- Rendimiento, memoria y privacidad: reducir builds p95 y asignaciones sin
  mantener copias de estado sensibles.
- Casos límite y errores: selección rápida, zoom continuo, búsqueda incremental
  y tema global.
- Criterios de aceptación:
  - [ ] Contadores muestran reducción en escenarios objetivo.
  - [ ] No aparecen estados desincronizados.
  - [ ] p95 de frame cumple G3.
  - [ ] Tests existentes conservan comportamiento.
- Validación: DevTools, contadores de debug, widgets y perfil.
- Riesgos y fuera de alcance: no se migrará a otro framework de estado sin ADR.

#### YAM-152 — Trabajos CPU/IO en segundo plano

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 4
- Áreas: PERF, EFF, REL
- Dependencias: YAM-105
- Problema: hashing, indexación, análisis de portada y exportación pueden ocupar
  el isolate UI.
- Resultado esperado: operaciones pesadas avanzan sin jank y pueden cancelarse
  o cerrarse de forma segura.
- Cambios funcionales y técnicos:
  - Medir primero qué trabajos superan el presupuesto.
  - Definir mensajes serializables y resultados acotados.
  - Mover hashing/indexación/scoring gradualmente.
  - Mantener PDFium en el contexto seguro exigido por su binding.
  - Integrar cancelación por generación.
- Visual y accesibilidad: progreso no bloqueante y acciones cancelables.
- Rendimiento, memoria y privacidad: evitar copiar bitmaps grandes entre
  isolates; no serializar texto innecesario.
- Casos límite y errores: isolate termina, cierre de app, trabajo compartido y
  disco lleno.
- Criterios de aceptación:
  - [ ] Solo se migra trabajo medido como costoso.
  - [ ] UI mantiene gate de frames.
  - [ ] Cancelar no publica resultados.
  - [ ] Fallo se recupera o informa por tarea.
- Validación: benchmarks A/B, tests de mensajes e integración.
- Riesgos y fuera de alcance: debe dividirse antes de IN_PROGRESS; no se moverá
  PDFium si el binding no lo admite.

#### YAM-153 — Optimización y mantenimiento SQLite

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: PERF, EFF, REL
- Dependencias: YAM-100, YAM-104
- Problema: 5.000 documentos, índice y organización requieren consultas y
  mantenimiento medidos.
- Resultado esperado: biblioteca e índice responden dentro del gate y la base
  puede repararse o compactarse de forma segura.
- Cambios funcionales y técnicos:
  - Capturar EXPLAIN QUERY PLAN para consultas críticas.
  - Añadir solo índices con mejora demostrada.
  - Paginar y agrupar transacciones.
  - Programar optimize/checkpoint/compactación cuando la app esté inactiva y
    haya espacio.
  - Verificar integridad antes/después.
- Visual y accesibilidad: mantenimiento manual muestra alcance y progreso;
  automático no bloquea ni roba foco.
- Rendimiento, memoria y privacidad: p95 <20 ms en dataset objetivo; DB nunca
  sale del dispositivo.
- Casos límite y errores: disco lleno, bloqueo, interrupción, índice corrupto y
  migración antigua.
- Criterios de aceptación:
  - [ ] Cada índice tiene consulta y benchmark asociado.
  - [ ] Mantenimiento es recuperable.
  - [ ] Consultas cumplen gate.
  - [ ] Integridad permanece ok.
- Validación: benchmarks multiproceso, fallos inyectados y migraciones.
- Riesgos y fuera de alcance: no se añade servidor de base de datos.

#### YAM-154 — Gate de rendimiento

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: PERF, REL, WIN
- Dependencias: YAM-100, YAM-103
- Problema: una única máquina y cinco pasadas internas no permiten convertir
  observaciones en umbrales fiables.
- Resultado esperado: detectar regresiones de arranque, frames, consultas,
  apertura, render, búsqueda, scroll y cierre.
- Cambios funcionales y técnicos:
  - Versionar escenarios y baseline por hardware.
  - Ejecutar procesos fríos y pasadas calientes.
  - Registrar mediana, p95, máximos, bundle y contadores.
  - Comparar con tolerancia de 10 % y exigir justificación.
  - Añadir segundo equipo/GPU antes de fijar gate universal.
- Visual y accesibilidad: escenarios incluyen tema, densidad, panel y DPI; el
  benchmark no sustituye revisión visual.
- Rendimiento, memoria y privacidad: informes redaccionados bajo build/; no
  rutas, títulos o texto.
- Casos límite y errores: ruido de antivirus, thermal throttling, GPU distinta y
  corrida incompleta.
- Criterios de aceptación:
  - [ ] Baseline declara hardware, build y corpus.
  - [ ] Regresión fuera de tolerancia falla.
  - [ ] Cierre comprueba contadores propios.
  - [ ] Dos equipos validan umbrales finales.
- Validación: tres procesos fríos y siete pasadas por escenario.
- Riesgos y fuera de alcance: no se publican cifras universales sin suficiente
  muestra.

#### YAM-155 — Pruebas sostenidas de recursos

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: PERF, REL
- Dependencias: YAM-104, YAM-105, YAM-111, YAM-154
- Problema: una apertura limpia no revela fugas por listeners, handles, timers
  o caches durante horas.
- Resultado esperado: sesiones prolongadas mantienen contadores propios y
  tendencia de memoria estables.
- Cambios funcionales y técnicos:
  - Crear soak de abrir/cerrar, buscar, panel, zoom, importar y limpiar.
  - Medir handles, documentos, renders, bytes, jobs y timers.
  - Tomar snapshots periódicos y detectar crecimiento monotónico.
  - Repetir tras errores y cancelaciones.
- Visual y accesibilidad: el soak verificará que la UI sigue receptiva y no
  acumula overlays/anuncios.
- Rendimiento, memoria y privacidad: corpus sintético; RSS como tendencia, no
  único criterio.
- Casos límite y errores: 500 ciclos, sleep/wake, minimización, cache clear y
  cierre forzado.
- Criterios de aceptación:
  - [ ] Recursos propios vuelven a cero tras cada cierre lógico.
  - [ ] No existe tendencia monotónica no explicada.
  - [ ] P95 no degrada con el tiempo.
  - [ ] Soak produce informe reproducible.
- Validación: Profile/Release con snapshots y análisis manual.
- Riesgos y fuera de alcance: el asignador puede retener RSS; se documentará sin
  confundirlo con fuga.

#### YAM-156 — Eficiencia energética e inactividad

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: M
- Fase: 4
- Áreas: PERF, EFF, WIN
- Dependencias: YAM-105, YAM-112, YAM-150, YAM-154
- Problema: prefetch, indexación, animaciones o timers pueden seguir consumiendo
  recursos con la ventana minimizada o inactiva.
- Resultado esperado: Yamantaka reduce actividad de fondo sin perder trabajo
  confirmado.
- Cambios funcionales y técnicos:
  - Observar foco, visibilidad, minimización y energía cuando Windows lo permita.
  - Pausar prefetch, portadas e índice no prioritario.
  - Reducir timers periódicos y reanudar con backpressure.
  - Mantener guardado crítico y cierre seguro.
- Visual y accesibilidad: reanudar no muestra ráfagas de feedback ni roba foco.
- Rendimiento, memoria y privacidad: medir CPU en idle y número de wakeups.
- Casos límite y errores: presentación keep-awake, operación confirmada,
  suspensión del sistema y reanudación con carpeta desconectada.
- Criterios de aceptación:
  - [ ] Idle/minimizada no mantiene trabajos no esenciales.
  - [ ] Reanudar respeta prioridades.
  - [ ] Datos confirmados se guardan.
  - [ ] Presentación es excepción explícita.
- Validación: perfil de CPU/energía, integración de lifecycle Windows.
- Riesgos y fuera de alcance: no se prometerán métricas de batería en equipos
  sin batería.

#### YAM-157 — Backup y restauración de nueva generación

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: XL
- Fase: 4
- Áreas: FUNC, REL
- Dependencias: YAM-103, YAM-153
- Problema: al crecer metadata, índice y anotaciones aumenta el coste de una
  restauración parcial o corrupta.
- Resultado esperado: previsualizar, validar, restaurar y revertir datos locales
  sin copiar PDF.
- Cambios funcionales y técnicos:
  - Versionar manifest, categorías y compatibilidad.
  - Validar schema, hashes, tamaños y referencias antes de escribir.
  - Crear prebackup y transacción de restauración.
  - Añadir cifrado autenticado opcional solo tras ADR y auditoría.
  - Excluir índice reconstruible cuando proceda.
- Visual y accesibilidad: preview por categorías, riesgos antes de confirmar y
  resultado detallado.
- Rendimiento, memoria y privacidad: streaming y límites de tamaño; contraseña
  nunca persistida ni registrada.
- Casos límite y errores: contraseña errónea, espacio insuficiente, versión
  futura, archivo truncado y cancelación.
- Criterios de aceptación:
  - [ ] Fallo deja estado previo intacto.
  - [ ] Prebackup existe y se valida.
  - [ ] PDF nunca entra en backup.
  - [ ] Cifrado, si se adopta, es autenticado y versionado.
- Validación: round-trip, versiones, corrupción, falta de espacio y rollback.
- Riesgos y fuera de alcance: cifrado requiere dependencia/cripto revisada; no
  hay recuperación de contraseña.

#### YAM-158 — Recuperación y safe mode endurecidos

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: REL, FUNC
- Dependencias: YAM-103, YAM-104, YAM-153, YAM-157
- Problema: cierres inesperados, migraciones fallidas, falta de espacio o base
  corrupta necesitan rutas de recuperación distintas.
- Resultado esperado: volver a una biblioteca utilizable sin borrar datos ni
  repetir el fallo.
- Cambios funcionales y técnicos:
  - Clasificar causa probable y último checkpoint seguro.
  - Omitir restauración/documento/feature problemática en safe mode.
  - Ofrecer integrity check, backup, reconstrucción de índice y reintento.
  - Registrar solo contadores y códigos redaccionados.
- Visual y accesibilidad: banner sobrio con acción clara; modo oscuro se
  conserva; foco llega a la recuperación.
- Rendimiento, memoria y privacidad: safe mode desactiva trabajo secundario y no
  expone rutas.
- Casos límite y errores: tres cierres, crash durante recuperación, DB ilegible,
  backup inválido y disco lleno.
- Criterios de aceptación:
  - [ ] No hay bucle de arranque.
  - [ ] Ninguna acción destructiva es automática.
  - [ ] Cierre limpio reinicia contador.
  - [ ] Usuario puede salir o reintentar explícitamente.
- Validación: arranques simulados, DB corrupta, migración y widgets.
- Riesgos y fuera de alcance: no se diagnostica automáticamente la causa exacta
  de un kill del sistema.

#### YAM-159 — Centro de privacidad y almacenamiento

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 4
- Áreas: FUNC, VIS, REL
- Dependencias: YAM-137, YAM-138, YAM-157
- Problema: biblioteca, portadas, índice, historial, workspaces y anotaciones
  ocupan categorías distintas que el usuario debe entender y controlar.
- Resultado esperado: ver tamaño, propósito y operaciones disponibles para cada
  categoría local.
- Cambios funcionales y técnicos:
  - Calcular tamaños de DB, portadas, logs, diagnósticos y backups.
  - Ofrecer limpiar cache, índice, historial, sesiones o todos los datos.
  - Separar datos reconstruibles de datos personales.
  - Integrar exportación previa y confirmaciones.
- Visual y accesibilidad: tabla/lista clara, texto “los PDF no se borran” y
  acciones con alcance.
- Rendimiento, memoria y privacidad: cálculo incremental; no abrir cada PDF para
  medir almacenamiento.
- Casos límite y errores: archivo bloqueado, limpieza parcial, cálculo cancelado
  y backup en otra ruta.
- Criterios de aceptación:
  - [ ] Cada categoría explica contenido y efecto.
  - [ ] Limpiar no borra PDF.
  - [ ] Fallo parcial se reporta.
  - [ ] Reabrir refleja tamaño actualizado.
- Validación: filesystem, repositorios, widgets y Narrador.
- Riesgos y fuera de alcance: no gestiona espacio de otros programas.

#### YAM-160 — Endurecimiento de seguridad local

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 4
- Áreas: REL, FUNC, WIN
- Dependencias: YAM-103, YAM-104
- Problema: PDF protegidos, enlaces, rutas y errores nativos atraviesan límites
  sensibles.
- Resultado esperado: entradas no confiables no ejecutan acciones, filtran datos
  o bloquean el proceso.
- Cambios funcionales y técnicos:
  - Revisar límites de tamaño, path normalization y validación de esquema.
  - Aislar contraseña en memoria y borrarla al cerrar.
  - Confirmar http/https/mailto y bloquear esquemas no autorizados.
  - Limitar mensajes nativos y redacción de excepciones.
  - Añadir smoke de cero red.
- Visual y accesibilidad: confirmaciones explican destino y riesgo; password
  tiene show/hide accesible y error no destructivo.
- Rendimiento, memoria y privacidad: contraseña nunca se persiste; logs sin
  rutas/texto.
- Casos límite y errores: URI malformada, file URI, UNC, ruta larga, password
  vacía y PDF hostil.
- Criterios de aceptación:
  - [ ] Esquemas no autorizados se bloquean.
  - [ ] Contraseña no llega a repositorio/log.
  - [ ] Smoke mantiene cero conexiones.
  - [ ] Errores nativos se transforman en FailureKind seguro.
- Validación: unitarias de entrada, corpus, integración y revisión de logs.
- Riesgos y fuera de alcance: no sustituye auditoría de seguridad de PDFium.

#### YAM-161 — Diagnóstico local redactado

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: M
- Fase: 4
- Áreas: REL, EFF
- Dependencias: YAM-159, YAM-160
- Problema: soporte necesita evidencia, pero rutas, títulos, consultas y notas
  son información privada.
- Resultado esperado: exportar voluntariamente un paquete útil y verificable
  sin contenido personal.
- Cambios funcionales y técnicos:
  - Definir allowlist de versión, plataforma, esquema, conteos, integridad,
    caches, tiempos y códigos de error.
  - Redactar identificadores y excluir texto libre.
  - Incluir manifest que declara campos.
  - Mostrar preview antes de guardar.
- Visual y accesibilidad: diálogo explica qué incluye/no incluye y permite
  cancelar.
- Rendimiento, memoria y privacidad: generación streaming, límite de logs y
  verificador que falla si detecta patrones prohibidos.
- Casos límite y errores: log malformado, ruta elegida no escribible, paquete
  parcial y safe mode.
- Criterios de aceptación:
  - [ ] Preview coincide con manifest.
  - [ ] Detector no encuentra rutas/títulos/consultas/notas.
  - [ ] Exportación requiere acción explícita.
  - [ ] Fallo elimina archivo parcial.
- Validación: fixtures con secretos sembrados y escáner de salida.
- Riesgos y fuera de alcance: no envía automáticamente el paquete.

### Fase 5 — Integración y distribución Windows

#### YAM-170 — Instalador firmado y reproducible

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: XL
- Fase: 5
- Áreas: WIN, REL
- Dependencias: YAM-100, YAM-154, YAM-155, YAM-157, YAM-158, YAM-160
- Problema: un build Release suelto no cubre firma, instalación, reparación,
  desinstalación ni confianza de Windows.
- Resultado esperado: paquete instalable, firmado, reproducible y documentado.
- Cambios funcionales y técnicos:
  - Comparar MSIX/MSI por asociaciones, firma, actualización manual y datos.
  - Crear ADR y pipeline de empaquetado.
  - Incluir licencias y metadatos correctos.
  - Separar binarios de datos locales.
  - Documentar renovación de certificado.
- Visual y accesibilidad: nombre, icono, editor y diálogos del instalador
  coherentes con la aplicación.
- Rendimiento, memoria y privacidad: medir tamaño; instalador no añade servicios,
  telemetría o red.
- Casos límite y errores: actualización fallida, versión anterior, arquitectura
  incorrecta y firma expirada.
- Criterios de aceptación:
  - [ ] Instalación limpia y reparación pasan.
  - [ ] Firma es válida.
  - [ ] Datos sobreviven actualización.
  - [ ] Rollback tiene procedimiento probado.
- Validación: VM Windows limpia, actualización y desinstalación.
- Riesgos y fuera de alcance: no habrá auto-updater de red.

#### YAM-171 — Asociación PDF e instancia única

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 5
- Áreas: FUNC, WIN, REL
- Dependencias: YAM-170
- Problema: abrir varios PDF desde Explorer puede crear procesos competidores o
  perder argumentos.
- Resultado esperado: “Abrir con Yamantaka” entrega rutas válidas a una
  instancia existente y muestra errores por archivo.
- Cambios funcionales y técnicos:
  - Registrar asociación opcional mediante instalador.
  - Implementar canal local autenticado de activación.
  - Deduplicar y conservar orden de argumentos.
  - Llevar ventana existente al frente sin robar foco indebidamente.
- Visual y accesibilidad: feedback por documento y acción para revisar fallos.
- Rendimiento, memoria y privacidad: canal solo local, mensajes acotados y sin
  persistencia.
- Casos límite y errores: 20 archivos, ruta Unicode, proceso arrancando, canal
  ocupado y archivo retirado.
- Criterios de aceptación:
  - [ ] No se pierde ningún argumento válido.
  - [ ] Duplicados se ignoran de forma predecible.
  - [ ] Fallo de un archivo no bloquea los demás.
  - [ ] Desinstalar elimina asociación propia.
- Validación: Explorer, línea de comandos, drag/drop e integración multi-launch.
- Riesgos y fuera de alcance: no se registra como visor predeterminado sin
  elección del usuario.

#### YAM-172 — Ventana, monitores y cambios de DPI

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 5
- Áreas: WIN, VIS, REL
- Dependencias: YAM-101, YAM-147
- Problema: coordenadas persistidas pueden quedar fuera de pantalla o cambiar de
  escala al desconectar un monitor.
- Resultado esperado: Yamantaka reaparece visible, con tamaño útil y layout
  correcto en cualquier monitor disponible.
- Cambios funcionales y técnicos:
  - Persistir rect relativo, monitor y estado maximizado.
  - Validar contra work areas actuales.
  - Reubicar con margen seguro si el monitor falta.
  - Recalcular viewport una sola vez tras cambio DPI.
- Visual y accesibilidad: ningún diálogo o control queda fuera de pantalla;
  foco se conserva.
- Rendimiento, memoria y privacidad: persistir geometría con debounce, sin
  nombres de documentos.
- Casos límite y errores: monitor izquierdo con coordenadas negativas, taskbar,
  4K→1080p, RDP y fullscreen.
- Criterios de aceptación:
  - [ ] Ventana siempre reaparece visible.
  - [ ] Maximizado se restaura correctamente.
  - [ ] Cambio DPI no altera punto de lectura.
  - [ ] Overlays se recolocan.
- Validación: matriz manual multi-monitor y tests de normalización.
- Riesgos y fuera de alcance: comportamiento depende de APIs Windows y
  window_manager.

#### YAM-173 — Paquete portable y desinstalación segura

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 5
- Áreas: WIN, REL
- Dependencias: YAM-157, YAM-170
- Problema: usuarios pueden preferir instalación o paquete portable y necesitan
  saber dónde viven sus datos.
- Resultado esperado: ambas distribuciones tienen límites y limpieza
  comprensibles.
- Cambios funcionales y técnicos:
  - Definir modo instalado y portable mediante marcador explícito.
  - Elegir rutas de datos coherentes y migración opt-in.
  - Permitir conservar o eliminar datos al desinstalar.
  - Evitar mezclar bases entre modos accidentalmente.
- Visual y accesibilidad: información de ubicación y acciones en Privacidad;
  confirmación de eliminación clara.
- Rendimiento, memoria y privacidad: portable no escribe fuera de su ámbito
  salvo selección del usuario.
- Casos límite y errores: carpeta read-only, USB retirado, dos modos abiertos y
  actualización manual.
- Criterios de aceptación:
  - [ ] Modos no comparten DB por accidente.
  - [ ] Desinstalar no borra datos sin confirmación.
  - [ ] Portable funciona sin privilegios de admin.
  - [ ] Backup permite migrar entre modos.
- Validación: VM, carpeta read-only, USB simulado y desinstalación.
- Riesgos y fuera de alcance: no se garantiza ejecución desde medios que
  bloqueen SQLite.

#### YAM-174 — Localización castellano/inglés

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: XL
- Fase: 5
- Áreas: FUNC, VIS, A11Y
- Dependencias: YAM-140, YAM-143, YAM-144, YAM-146
- Problema: textos incrustados dificultan consistencia, pluralización, testing y
  adaptación de layout.
- Resultado esperado: seleccionar castellano o inglés y mantener todas las
  superficies completas.
- Cambios funcionales y técnicos:
  - Extraer catálogo tipado.
  - Definir plurales, parámetros y fallbacks.
  - Persistir idioma y aplicar sin reinicio si es viable.
  - Añadir test que detecte strings visibles no localizados.
- Visual y accesibilidad: revisar expansión de texto, labels, tooltips y
  anuncios de Narrador.
- Rendimiento, memoria y privacidad: catálogos locales; cero servicio externo.
- Casos límite y errores: clave ausente, plural cero/uno/muchos, fecha/número y
  texto técnico.
- Criterios de aceptación:
  - [ ] Ninguna superficie principal mezcla idiomas.
  - [ ] Fallback es determinista.
  - [ ] Matriz visual pasa en ambos idiomas.
  - [ ] Errores conservan significado.
- Validación: tests de catálogo, screenshots y Narrador.
- Riesgos y fuera de alcance: no se añaden otros idiomas hasta estabilizar la
  infraestructura.

#### YAM-175 — Ayuda integrada y primer arranque

- Estado: PLANNED
- Prioridad: P1
- Esfuerzo: L
- Fase: 5
- Áreas: FUNC, VIS, A11Y
- Dependencias: YAM-143, YAM-145, YAM-159, YAM-174
- Problema: funciones locales avanzadas necesitan contexto, pero una landing
  inicial rompería el carácter directo de la biblioteca.
- Resultado esperado: primer arranque breve y ayuda contextual disponible sin
  interrumpir la lectura.
- Cambios funcionales y técnicos:
  - Mostrar una única introducción opcional con abrir/importar, privacidad y
    atajos.
  - Añadir ayuda contextual a backups, índice, recuperación y diagnósticos.
  - Mantener un centro de ayuda local buscable.
  - Permitir reiniciar consejos.
- Visual y accesibilidad: tipografía editorial, una acción principal, teclado y
  Narrador; sin carrusel.
- Rendimiento, memoria y privacidad: contenido estático local; no recopila
  progreso de onboarding.
- Casos límite y errores: usuario omite, safe mode, ventana compacta e idioma
  cambia.
- Criterios de aceptación:
  - [ ] Biblioteca sigue siendo primera superficie útil.
  - [ ] Ayuda explica acciones de riesgo antes de ejecutarlas.
  - [ ] Se puede cerrar y volver a abrir.
  - [ ] No aparece branding dentro de lector/biblioteca.
- Validación: widgets, capturas, teclado y prueba de comprensión.
- Riesgos y fuera de alcance: no hay tours animados, vídeos o contenido web.

#### YAM-176 — Checklist de release candidate

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: L
- Fase: 5
- Áreas: REL, WIN
- Dependencias: YAM-100, YAM-101, YAM-102, YAM-103, YAM-154, YAM-155,
  YAM-160, YAM-170, YAM-171, YAM-172, YAM-173, YAM-174, YAM-175
- Problema: publicar exige coordinar calidad, licencias, firma, migraciones,
  instalación y privacidad.
- Resultado esperado: una release candidate solo existe si cada gate tiene
  evidencia fechada.
- Cambios funcionales y técnicos:
  - Crear checklist versionado con responsables y artefactos.
  - Verificar versión, changelog, licencias, firma y símbolos.
  - Probar instalación limpia, actualización, reparación y rollback.
  - Ejecutar smoke de red, corpus, backup y asociación PDF.
- Visual y accesibilidad: incluir matriz final de temas, DPI, Narrador y
  resoluciones.
- Rendimiento, memoria y privacidad: adjuntar perfiles redaccionados y tamaño de
  bundle.
- Casos límite y errores: gate sin evidencia, certificado expirado, baseline
  actualizado sin aprobación y migración irreversible.
- Criterios de aceptación:
  - [ ] Todos los gates G1–G7 están aprobados.
  - [ ] No hay P0/P1 abiertos.
  - [ ] Artefactos se reproducen desde commit etiquetado.
  - [ ] Rollback está ensayado.
- Validación: ensayo completo en VM y equipo real.
- Riesgos y fuera de alcance: checklist no puede dispensar un gate; cualquier
  excepción requiere retirar la RC.

#### YAM-177 — Periodo de estabilidad previo a 1.0

- Estado: PLANNED
- Prioridad: P0
- Esfuerzo: XL
- Fase: 5
- Áreas: FUNC, PERF, A11Y, REL, WIN
- Dependencias: YAM-176
- Problema: una RC que pasa una vez no demuestra estabilidad en sesiones largas
  o hardware distinto.
- Resultado esperado: convertir la RC en 1.0 solo después de uso prolongado y
  cierre de regresiones.
- Cambios funcionales y técnicos:
  - Congelar funciones y aceptar solo correcciones.
  - Ejecutar sesiones de varias horas con corpus real autorizado y sintético.
  - Repetir en al menos dos equipos/GPU y múltiples DPI.
  - Clasificar hallazgos y reiniciar el periodo ante P0/P1.
  - Verificar instalación, actualización y desinstalación final.
- Visual y accesibilidad: repetir Narrador, temas y layouts después de cada
  corrección visual.
- Rendimiento, memoria y privacidad: soak, perfiles y smoke de cero red en cada
  candidata.
- Casos límite y errores: suspensión, RDP, impresora, unidades externas,
  contraseña, falta de espacio y cierre forzado.
- Criterios de aceptación:
  - [ ] Periodo termina sin P0/P1.
  - [ ] Tendencia de recursos es estable.
  - [ ] Gates siguen verdes en commit final.
  - [ ] Documentación y binarios coinciden.
- Validación: informe final de estabilidad y aprobación explícita.
- Riesgos y fuera de alcance: no se añadirán funciones durante la congelación.

## 7. Orden recomendado de ejecución

### Ola 1 — Preparar una base verificable

1. YAM-100 — Gate reproducible.
2. YAM-104 — Ownership y lifecycle.
3. YAM-140 — Auditoría visual.

Estas tres tareas pueden empezar en paralelo porque no dependen entre sí. Son
las únicas marcadas READY al reconstruir la hoja de ruta.

### Ola 2 — Fijar contratos y medidas

- YAM-101, YAM-102 y YAM-103.
- YAM-105 y YAM-111.
- YAM-114 y YAM-147.
- YAM-150, YAM-153 y YAM-154.

No se implementará una optimización sin baseline, ni una nueva superficie sin
matriz visual.

### Ola 3 — Mejorar el núcleo

- Motor: YAM-110, YAM-112, YAM-113 y YAM-117.
- Lector: YAM-118, YAM-119, YAM-123 y YAM-124.
- Biblioteca: YAM-130 a YAM-138.
- Visual: YAM-141 a YAM-149.
- Fiabilidad: YAM-151 a YAM-161.

### Ola 4 — Funciones avanzadas

- YAM-115 y YAM-116 — selección/anotación geométrica.
- YAM-120 y YAM-121 — split y comparación.
- YAM-122 — multi-window.
- YAM-139 — modo estudio.

Estas funciones no pasan a interfaz estable sin demostrar memoria, cierre,
teclado y recuperación.

### Ola 5 — Distribución

- YAM-106 cuando upstream lo permita.
- YAM-170 a YAM-175.
- YAM-176 y YAM-177 como puertas finales.

## 8. Tabla compacta de dependencias

| Bloque | Fundamentos obligatorios | Desbloquea |
| --- | --- | --- |
| Calidad | YAM-100, YAM-101, YAM-102, YAM-103 | Evidencia de todas las fases |
| Lifecycle | YAM-104, YAM-105, YAM-111 | Teselas, jobs, split, multi-window |
| Viewport | YAM-114, YAM-118 | Panel, presentación, split, estudio |
| Datos | YAM-153, YAM-157, YAM-158 | Escala, backup, instalador, RC |
| Visual | YAM-140, YAM-147, YAM-148 | Biblioteca, lector, ajustes y distribución |
| Rendimiento | YAM-150, YAM-154, YAM-155 | Funciones avanzadas y release |
| Windows | YAM-160, YAM-170, YAM-171, YAM-172 | Release candidate |

## 9. Riesgos y prototipos obligatorios

### Selección geométrica

La selección depende de capacidades nativas de pdfrx/PDFium. YAM-115 empezará
con un spike. Si la geometría no puede mantenerse correctamente con rotación,
zoom y texto complejo, no se expondrán anotaciones geométricas.

### Teselas

YAM-110 solo avanzará tras medir páginas extremas. La página completa seguirá
siendo el camino por defecto para no aumentar renders, costuras y complejidad.

### Split y multi-window

Dos visores y varias ventanas multiplican memoria y superficies de lifecycle.
YAM-120/YAM-122 necesitan prototipo, perfil y ADR antes de UI estable.

### Impresión

YAM-124 puede requerir API nativa o dependencia. No se añadirá ninguna sin
licencia, mantenimiento, bundle y política de temporales aprobados.

### Cifrado de backup

YAM-157 no diseñará criptografía propia. El cifrado opcional exige formato
autenticado, dependencia revisada y advertencia de contraseña irrecuperable.

### Watchers de carpetas

YAM-133 observará únicamente raíces elegidas y convertirá eventos en
revalidaciones. Una tormenta de eventos no puede importar, borrar ni bloquear.

### Firma y empaquetado

YAM-170 depende de certificado y decisiones de distribución. La ausencia de
certificado puede bloquear firma, pero no autoriza publicar sin identificar el
riesgo.

## 10. Reglas de mantenimiento del backlog

- Solo puede haber una tarea IN_PROGRESS por responsable.
- No habrá más de tres tareas READY simultáneas salvo que pertenezcan a
  responsables y subsistemas independientes.
- Una tarea XL debe dividirse en entregas verticales antes de IN_PROGRESS.
- Cada cambio de estado incluirá fecha, evidencia y resultado.
- Una tarea pasa a REVIEW solo cuando todos sus checks de implementación están
  completos.
- Una tarea pasa a DONE solo cuando sus criterios de aceptación y validación
  están comprobados.
- Las tareas visuales requieren capturas revisadas.
- Las tareas de rendimiento requieren baseline, mediana, p95, hardware y modo
  de build.
- Las tareas Windows requieren ejecución nativa; widgets no sustituyen la
  plataforma.
- Las tareas de accesibilidad requieren Narrador real cuando así se indique.
- Un fallo de privacidad o integridad eleva la tarea a P0.
- No se cambia prioridad para ocultar un gate fallido.
- El trabajo terminado se resume en CHANGELOG.md y se elimina de este archivo
  en la siguiente reconstrucción de release.
- Nuevas ideas deben indicar problema real, usuario afectado, aceptación y
  coste; una lista de deseos sin validación no entra en el roadmap.
- Toda dependencia nueva exige ADR antes de modificar pubspec.

## 11. Equivalencias de referencias históricas

Otros documentos todavía mencionan algunos identificadores del backlog
anterior. Esta tabla mantiene la navegación hasta que esos documentos se
actualicen; no restaura el contenido histórico ni convierte los aliases en
tareas activas.

| Referencia antigua | Nueva referencia |
| --- | --- |
| YAM-011 / YAM-034 / YAM-048 / YAM-073 | YAM-154 y YAM-155 |
| YAM-014 | YAM-106 |
| YAM-015 | YAM-101, YAM-154 y YAM-172 |
| YAM-022 / YAM-023 / YAM-024 | YAM-101, YAM-102, YAM-147 y YAM-149 |
| YAM-033 / YAM-041 | YAM-148 |
| YAM-050 | YAM-115 y YAM-116 |
| YAM-062 | YAM-120 |
| YAM-063 | YAM-121 |
| YAM-025 a YAM-029 | Línea base ya implementada; sin sucesor activo |

## 12. Definición de 1.0

Yamantaka 1.0 para Windows podrá proponerse únicamente cuando:

- YAM-176 esté DONE y YAM-177 haya completado su periodo de estabilidad.
- Gates G1–G7 estén aprobados en el mismo commit candidato.
- No existan P0 o P1 abiertos.
- Instalador, portable, asociación PDF y rollback estén verificados.
- La matriz nativa incluya Full HD, 16:10, 2K, 4K y DPI 100–200 %.
- Biblioteca y lector funcionen con teclado y Narrador.
- El corpus patológico no provoque bloqueos o fugas propias.
- Cierre deje cero documentos, renders, jobs y bytes propios retenidos.
- Backup/restauración y safe mode estén probados ante fallos.
- El smoke Release confirme cero conexiones de red.
- La identidad visual siga siendo editorial, monocromática y centrada en el
  documento.

Las funciones P2 pueden quedar fuera de 1.0 si sus dependencias no están listas,
siempre que no se anuncien ni dejen controles inactivos visibles.
