# Prompt maestro — Yamantaka 0.13: flujos, resiliencia y pulido de producto

Trabaja directamente sobre el repositorio de Yamantaka después de completar la
iteración `0.12.0+13`. Implementa una iteración integral `0.13.0+14`: no te
limites a analizar o enumerar ideas. Audita el estado real, diseña sobre los
patrones existentes, modifica código y persistencia cuando corresponda, añade
pruebas, mide el resultado, actualiza la documentación y genera un release local
verificable para Windows.

No publiques, firmes, subas ni envíes archivos o telemetría. Mantén la aplicación
local-first y offline. No abras documentos personales para fabricar fixtures:
usa únicamente PDFs sintéticos dentro de `build/`. Conserva compatibilidad con
los datos creados por versiones anteriores y crea migraciones reversibles cuando
un cambio de esquema sea imprescindible.

## Objetivo de producto

Convierte Yamantaka en un espacio de lectura documental capaz de sostener trabajo
real durante semanas: capturar documentos, decidir qué hacer con ellos, recuperar
contexto, relacionar información y terminar una revisión sin perder confianza en
el estado de la biblioteca.

La iteración debe mejorar de forma comprobable:

1. planificación y finalización de trabajo documental;
2. navegación entre documentos relacionados;
3. operaciones repetitivas con control, previsualización y deshacer;
4. detección de duplicados, revisiones y archivos problemáticos;
5. recuperación ante cierres inesperados o datos dañados;
6. claridad visual, densidad adaptable y coherencia en Windows;
7. accesibilidad completa mediante teclado, Narrator y texto ampliado;
8. rendimiento estable en bibliotecas de 100.000 documentos.

## Principios no negociables

- La interfaz debe seguir centrada en el documento y evitar aspecto de panel de
  métricas o gamificación.
- Toda automatización debe ser local, explícita, reversible y contar con vista
  previa; nunca debe modificar archivos originales silenciosamente.
- Ninguna operación pesada puede bloquear el primer frame, el desplazamiento o
  la interacción del lector.
- No dupliques superficies. Extiende el navegador documental, la paleta, los
  inspectores y los diálogos existentes cuando sean el lugar correcto.
- No mantengas dos implementaciones de una misma regla. Usa una sola fuente de
  verdad para comandos, filtros, estados, etiquetas y acciones masivas.
- Los estados vacío, carga, error, parcial, cancelado y sin permisos forman parte
  del diseño y deben tener cobertura.
- Cada afirmación de rendimiento, accesibilidad o comportamiento nativo debe
  estar respaldada por una prueba reproducible o declararse pendiente.

## Fase 1 — Auditoría y contrato de la iteración

Antes de editar:

1. lee `README.md`, `ARCHITECTURE.md`, `DESIGN.md`, `TESTING.md`, `CHANGELOG.md`,
   `NEXT_TODOS.md` y los informes finales de 0.11 y 0.12;
2. ejecuta estado Git, formato, análisis, pruebas y el gate Windows disponible;
3. inventaría controladores, repositorios, esquemas, comandos, atajos, rutas de
   backup, tareas de fondo y métricas diagnósticas;
4. localiza duplicación de estado, consultas sin límite, trabajo síncrono en UI,
   superficies sin semántica y flujos sin deshacer;
5. registra un baseline de cinco pasadas para arranque, primera biblioteca,
   apertura del lector, cambio de documento, búsqueda global y uso de memoria;
6. escribe en `docs/architecture/visual-iteration-0.13.md` el diagnóstico, las
   decisiones, las métricas iniciales y los riesgos que no se resolverán.

No cambies la arquitectura por preferencia personal. Cada refactor debe habilitar
un requisito medible de esta iteración o eliminar un fallo demostrado.

## Fase 2 — Espacios de trabajo y cola de revisión

Añade espacios de trabajo locales como vistas guardadas, no como copias de los
documentos. Cada espacio puede contener una consulta, orden, agrupación, columnas,
documentos fijados y una cola ordenada de revisión.

Implementa:

- creación, renombrado, duplicado y eliminación con deshacer;
- un espacio predeterminado que no pueda dejar la aplicación inutilizable;
- arrastrar documentos a la cola y reordenarlos con teclado o puntero;
- estados `Pendiente`, `En revisión`, `Bloqueado`, `Revisado` y `Archivado`;
- motivo opcional de bloqueo y siguiente acción, ambos locales y editables;
- acciones `Abrir siguiente`, `Volver al último contexto` y `Marcar y continuar`;
- contador discreto de elementos restantes, sin rachas, confeti ni presión;
- restauración exacta del espacio, elemento activo, pestañas, navegador y
  posición de lectura después de reiniciar.

La biblioteca general y los espacios deben compartir la misma paginación y el
mismo motor de consulta. No materialices más de 80 filas o tarjetas a la vez.

## Fase 3 — Relaciones, referencias y revisiones de un documento

Permite conectar documentos sin crear una base de conocimiento intrusiva:

- relaciones tipadas `Relacionado`, `Reemplaza`, `Es reemplazado por`,
  `Fuente de` y `Adjunto de`;
- backlinks visibles en el inspector y en el navegador documental;
- enlace opcional a una página, marcador o anotación concreta;
- búsqueda para enlazar con resultados acotados y navegación de ida y vuelta;
- detección por hash de duplicados exactos;
- candidatos a revisión mediante nombre, tamaño y metadatos, siempre presentados
  como sugerencias y nunca fusionados automáticamente;
- pila de revisiones que conserve historial, lectura y anotaciones por archivo;
- acción segura para designar revisión actual sin borrar versiones anteriores.

Define cómo se comportan las relaciones ante archivo ausente, documento
eliminado, restauración de backup y cambio de ruta. Añade pruebas de ciclos,
relaciones rotas y 10.000 enlaces sintéticos.

## Fase 4 — Flujos locales con vista previa

Crea un motor pequeño de reglas locales para reducir tareas repetitivas. Limita
la primera versión a desencadenantes y acciones comprensibles:

- desencadenantes: importado, carpeta concreta, extensión, etiqueta, autor,
  duplicado exacto o archivo inaccesible;
- condiciones combinables con `todas`/`cualquiera` y negación explícita;
- acciones: aplicar etiqueta, añadir a colección o espacio, cambiar estado,
  fijar prioridad o añadir a la cola;
- editor guiado, resumen en lenguaje natural y validación antes de guardar;
- modo de simulación que enumere los cambios sin escribir;
- confirmación con cantidad y alcance para ejecuciones masivas;
- registro local de las últimas ejecuciones, sin nombres o rutas en diagnósticos;
- deshacer transaccional de la última ejecución cuando los datos no hayan
  cambiado de forma incompatible;
- límites de tiempo, elementos y memoria, cancelación cooperativa y reanudación.

No ejecutes comandos arbitrarios, scripts, red ni operaciones destructivas sobre
los PDFs. Una regla inválida debe desactivarse con explicación y conservarse para
edición, no desaparecer.

## Fase 5 — Recuperación y confianza

Haz visible y comprobable la salud local de la aplicación:

- journal o escritura atómica para estado de sesión y operaciones masivas;
- recuperación de la última sesión tras cierre inesperado con elección entre
  restaurar y empezar limpio;
- comprobación de integridad de SQLite y de referencias a archivos;
- centro de problemas no modal para ausentes, inaccesibles, duplicados y tareas
  interrumpidas;
- acciones individuales y masivas con vista previa;
- backup manual cifrado opcional mediante contraseña, sin guardar la contraseña;
- restauración en modo ensayo que muestre versión, tamaño, conflictos y cambios;
- migraciones idempotentes con rollback seguro y fixture por cada versión
  compatible;
- exportación diagnóstica privada que excluya rutas, títulos, consultas, texto
  del PDF y contenido de anotaciones.

Simula interrupciones en cada límite transaccional importante. Tras reabrir no
debe existir estado medio aplicado, pérdida silenciosa ni tarea zombi.

## Fase 6 — Eficiencia del lector

Profundiza el flujo de revisión sin convertir el lector en un editor complejo:

- modo de revisión que recorra anotaciones, marcadores o resultados pendientes;
- `Siguiente hallazgo` y `Anterior hallazgo` con contexto y retorno al origen;
- panel de anotaciones agrupable por documento, color, tipo, etiqueta y estado;
- estados de anotación `Pendiente`, `Aceptada` y `Resuelta` con acción rápida;
- edición de nota en línea, guardado automático seguro e indicador de error;
- selección múltiple de anotaciones y exportación Markdown/JSON local;
- comparación lado a lado de dos revisiones con desplazamiento vinculado
  opcional, separación clara y consumo de memoria acotado;
- historial de navegación unificado entre páginas, relaciones y hallazgos;
- atajos configurables con detección de conflictos y restablecimiento individual.

Mantén estable la identidad de cada visor al abrir paneles, cambiar densidad,
redimensionar o alternar comparación. Al cerrar una comparación libera handles,
renders y caché no reutilizable.

## Fase 7 — Pulido visual y adaptación

Revisa todas las superficies a 1024×768, 1280×720, 1366×768, 1600×900,
1920×1080, 1920×1200, 2560×1440 y 3440×1440, además de 100/125/150/175/200 %
de escala y 100/130/150/200 % de texto.

Mejora:

- jerarquía tipográfica y contraste entre título, metadatos y acciones;
- densidad cómoda/compacta con alturas, hit targets y truncado coherentes;
- inspector adaptable que pase a hoja o diálogo cuando falte ancho;
- selección, hover, foco, arrastre y destino de drop claramente diferenciados;
- esqueletos estables que no provoquen saltos de layout;
- mensajes de estado con una acción principal y detalles progresivos;
- iconografía consistente y etiquetas visibles para acciones ambiguas;
- menús contextuales, tooltips y barras que no invadan el documento;
- modo oscuro sin negros aplastados y alto contraste sin depender del color;
- movimiento reducido y transiciones breves sin perder orientación.

No rediseñes los assets de marca salvo petición expresa. Verifica por hash que
los recursos protegidos permanezcan intactos.

## Fase 8 — Accesibilidad y Windows real

Garantiza orden de foco lógico, foco restaurado tras cada capa temporal, acciones
con nombres accesibles, estados anunciados sin ruido y navegación completa sin
puntero. Prueba español e inglés, RTL estructural aunque no se distribuya un
locale RTL, texto ampliado, contraste alto y movimiento reducido.

Valida físicamente cuando el entorno lo permita:

- Narrator en Biblioteca, cola, lector, relaciones, reglas y restauración;
- maximizar, restaurar, fullscreen y cambio de monitor;
- DPI reales distintos por monitor;
- teclado, ratón, rueda, touchpad, drag & drop y menús contextuales;
- rutas largas, caracteres Unicode, archivos bloqueados y carpetas de red
  desconectadas sin realizar tráfico iniciado por Yamantaka;
- Smart App Control/Gatekeeper equivalente de Windows sin desactivar defensas.

Separa claramente prueba widget, captura simulada y validación física. No marques
como aprobado un caso que el hardware no permitió ejecutar.

## Fase 9 — Rendimiento, observabilidad y límites

Ejecuta cinco pasadas comparables con 500, 5.000, 25.000 y 100.000 documentos,
10.000 relaciones, 1.000 elementos de cola, 500 anotaciones y 50 reglas.

Registra al menos:

- tiempo hasta primer frame y primera biblioteca útil;
- latencia p50/p95 de consulta, cambio de espacio y apertura del siguiente;
- apertura fría/caliente del navegador y del inspector;
- coste de simulación y aplicación de reglas;
- restauración de sesión y ensayo de backup;
- frames lentos durante scroll, resize y comparación;
- RSS pico y después de cerrar todos los documentos;
- documentos, renders, decodes y tareas pendientes tras cancelación/cierre.

Una regresión es bloqueante si la mediana empeora más del 10 % y al menos cuatro
de cinco muestras superan el límite. Conserva movimientos ruidosos como `watch`
con muestras y rangos completos. La observabilidad debe usar identificadores
agregados y nunca contenido o metadatos privados.

## Fase 10 — Pruebas y evidencia

Añade pruebas unitarias, de repositorio, widget, golden y de integración para:

- espacios, cola, estados, reordenado y restauración;
- relaciones, backlinks, revisiones, ciclos y enlaces rotos;
- reglas válidas, inválidas, simuladas, canceladas, reanudadas y deshechas;
- recuperación en cada punto de interrupción transaccional;
- conflictos de backup, migraciones repetidas e integridad dañada;
- navegación de hallazgos, comparación y liberación completa de recursos;
- teclado, semántica, foco, contraste y escalado de texto;
- consultas solapadas donde solo la generación más reciente pueda publicar;
- biblioteca patológica y todos los límites de materialización.

Genera matrices visuales para Biblioteca, espacios, cola, lector, comparación,
relaciones, reglas, problemas, backup/restauración, búsqueda, ajustes, menús y
diálogos. Incluye vacío, carga, datos, error, parcial, cancelado y sin permisos.
Crea hojas de contacto con hashes y revisa visualmente todas, no solo las goldens.

## Fase 11 — Documentación y release

Actualiza como mínimo:

- `README.md` con los flujos visibles y sus atajos;
- `CHANGELOG.md` para `0.13.0+14`;
- `ARCHITECTURE.md` con espacios, relaciones, reglas, journal y privacidad;
- `DESIGN.md` con los patrones visuales nuevos;
- `TESTING.md` con comandos, matrices, corpus y resultados reales;
- `NEXT_TODOS.md` separando completado, observado y pendiente físico;
- `pubspec.yaml` a `0.13.0+14`.

Ejecuta el gate Windows sin fases omitidas. Después genera un bundle limpio bajo
`build/release/0.13.0+14/`, ZIP, manifiesto por archivo y `SHA256SUMS.txt`.
Comprueba inventario, tamaños y SHA-256 contra el contenido real del ZIP, rechaza
rutas inseguras y confirma que no contiene PDFs. Ejecuta el binario exacto del
ZIP durante al menos ocho segundos y demuestra cero conexiones iniciadas.

Si los binarios no están firmados, decláralo y presenta el resultado como artefacto
local de prueba, no como distribución para terceros. No hagas commit, tag, push ni
publicación salvo autorización explícita.

## Entrega final obligatoria

Entrega un resumen verificable que incluya:

1. problemas encontrados y decisiones tomadas;
2. mejoras funcionales, visuales y de accesibilidad implementadas;
3. archivos y áreas principales modificados;
4. pruebas, matrices y recuentos exactos;
5. comparación de rendimiento con `pass`, `watch` o `fail`;
6. rutas del gate, evidencia visual y release;
7. tamaño y SHA-256 del ZIP;
8. estado de firma y smoke sin red;
9. validaciones físicas realizadas y pendientes;
10. limitaciones reales y siguientes pasos concretos;
11. un mensaje de commit convencional, detallado y listo para copiar.

La iteración solo se considera terminada cuando código, datos, interfaz, pruebas,
documentación y paquete cuentan la misma historia y no queda una fase automatizada
omitida o un resultado supuesto presentado como evidencia.
