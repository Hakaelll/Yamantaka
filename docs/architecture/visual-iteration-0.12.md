# Iteración integral 0.12

## Diagnóstico de partida

La base `0.11.0+12` se auditó antes de modificar código. El árbol Git estaba
limpio; formato, análisis, 277 pruebas, build Windows, smoke sin red y matriz
nativa de la escala física disponible pasaron. La evidencia inicial queda en
`build/verification/` y las cinco pasadas nuevas en
`build/performance/0.12.0-baseline/`.

Los contratos de render, caché, búsqueda cancelable y paginación de 80 filas
son sólidos. Los principales riesgos encontrados fueron:

- `app_shell.dart` y varias vistas concentran demasiada coordinación;
- `WorkspaceRepository.all()` ejecutaba una consulta de pestañas y otra de
  documento por cada espacio, un patrón N+1;
- espacios, comparación y sesiones tenían modelos útiles, pero no el flujo de
  producto completo ni estados de cola;
- las anotaciones distinguían asociación técnica, pero no el estado editorial
  Pendiente/Aceptada/Resuelta ni operaciones masivas;
- relaciones, revisiones, reglas y journal no tenían persistencia propia;
- el inicio ofrecía Continuar y cerrados recientes, pero no componía el contexto
  de trabajo en un centro único;
- la evidencia a texto 200 % mostró un menú que partía palabras y la captura
  nativa fullscreen quedó vacía, por lo que no se acepta como evidencia visual;
- la validación física solo cubre el monitor y DPI disponibles. Narrator, varios
  monitores y el resto de escalas siguen siendo comprobaciones físicas, no se
  infieren de pruebas de widget.

## Decisiones

### Una sola fuente de estado

`DocumentRepository`, `WorkspaceRepository`, `AnnotationRepository` y los
controladores actuales siguen siendo las fuentes canónicas. El centro de
continuidad solo proyecta datos ya persistidos. No recorre archivos ni abre
PDFium durante el primer frame.

### Persistencia v10

La migración v10 añade columnas compatibles a espacios y anotaciones, y tablas
independientes para relaciones, grupos de revisión, reglas y journal. La
migración es transaccional e idempotente; la apertura conserva el backup previo
y termina con `PRAGMA integrity_check`.

Los espacios guardan consulta serializada, agrupación, orden, densidad,
columnas, cola, documento activo, pestañas y modo de concentración. La cola usa
las mismas pestañas/documentos ya paginados y la UI no materializa más de 80.

### Operaciones reversibles

Las reglas separan simulación y aplicación. Toda ejecución masiva crea una
entrada de journal, avanza por lotes con cancelación cooperativa y registra solo
tipo, cantidades, estado y códigos seguros. El journal permite detectar trabajo
interrumpido y evita presentar una aplicación parcial como completa.

Eliminar un espacio conserva un snapshot de undo en memoria hasta la siguiente
mutación. Relaciones y revisiones nunca eliminan ni fusionan PDFs.

### Comparación acotada

La comparación abre como máximo un segundo handle, renderiza únicamente la
página visible de cada lado y permite sincronía opcional. Al cerrar cancela y
libera el segundo documento y sus cachés. Las diferencias son marcas manuales;
no se promete comparación automática.

### Accesibilidad y adaptación

Toda acción con puntero tiene alternativa de teclado. Los grupos nuevos usan
orden de foco explícito, semántica de estado, targets mínimos de 44 px y layouts
que pasan a una columna en ventanas estrechas. Movimiento reducido elimina las
transiciones espaciales.

## Presupuestos 0.12

- primer frame: sin I/O de sistema de archivos ni PDF;
- consultas de Biblioteca/espacios: `LIMIT <= 80`;
- handles PDF calientes normales: 2; comparación: un segundo handle temporal;
- reglas: máximo 100.000 candidatos, lotes de 80 y cancelación entre lotes;
- navigator: publicación por generación y lista virtualizada;
- relaciones: consultas por documento con índices, sin N+1;
- regresión fiable: mediana >10 % y al menos 4 de 5 muestras sobre el límite;
- cachés PDF: se conservan los presupuestos separados de 72/24/8 MiB.

## Privacidad

No se añaden red, telemetría, scripts ni modificación de PDFs. Diagnóstico y
journal excluyen rutas, títulos, consultas, texto PDF y contenido de notas. Las
exportaciones Markdown/JSON y los backups solo se crean por acción explícita.

## Evidencia y límites físicos

Las matrices de widget prueban composición, foco, semántica, texto ampliado,
temas, alto contraste simulado y movimiento reducido. Solo una prueba nativa
puede validar caption, taskbar, DPI real, Narrator o varios monitores. Cualquier
combinación no ejecutada se declara pendiente y nunca se convierte en `pass`.

## Resultado medido

La implementación final elimina los N+1 de espacios, etiquetas de anotación y
grupos de revisiones; añade LRU de ocho búsquedas con TTL de un segundo e
invalidación por mutación. Las cinco pasadas 0.11/0.12 terminan
`pass-with-watch`, con cero regresiones fiables y cero invariantes fallidos. La
mediana caliente de búsqueda baja de 592 a 28 µs, restaurar un espacio baja de
669 a 182 µs y abrir la primera página del PDF sintético de 5.000 páginas queda
en 188.862 µs (-0,53 %). Cinco métricas ruidosas se conservan como `watch` con
sus muestras completas; ninguna se oculta ni se reclasifica manualmente.

La evidencia visual final contiene 172 PNG y 18 hojas de contacto. La revisión
detectó un desbordamiento de 12 px en los controles de cola y un arnés sin las
fuentes reales; ambos se corrigieron antes de aceptar las capturas. La marca
supera la prueba de hash byte a byte.
