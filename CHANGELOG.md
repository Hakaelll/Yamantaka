# Changelog

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
