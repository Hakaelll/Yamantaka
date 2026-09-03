# Auditoría 0.13 — resiliencia documental y confianza local

Estado inicial: `0.12.0+13`, rama `main` limpia, 287 pruebas y gate Windows de
14 fases aprobado sin omisiones el 31 de agosto de 2026. La evidencia base se
conserva en `build/verification/0.13.0-baseline/` y las cinco pasadas comparables
en `build/performance/0.13.0-baseline/`.

## Hallazgos

- El lector distinguía errores tipados, pero su estado público se reducía a
  `idle/loading/ready/failure`. La contraseña se solicitaba una sola vez y una
  apertura obsoleta podía llegar a registrar metadatos antes de comprobar su
  generación.
- La sesión atómica solo retenía espacio y documento. No guardaba huella, fase
  de apertura ni documento causante del fallo, por lo que la restauración no
  podía demostrar compatibilidad antes de reabrir.
- El backup 0.12 tenía manifiesto, hashes, pre-backup y transacción, pero su
  preview solo contaba cuatro entidades y no explicaba conflictos ni el plan de
  conservación/reemplazo.
- La consolidación de duplicados era transaccional y nunca tocaba los PDF, pero
  no conservaba procedencia explícita de anotaciones, no trasladaba todas las
  relaciones/revisiones/colas y no ofrecía un contrato de undo basado en su
  pre-backup.
- El journal registraba estado, cantidad y undo, pero no fase, causa de
  interrupción, hash de compatibilidad ni capacidad real de reanudación.
- La validación de archivos ya era incremental, acotada y cancelable; debe seguir
  siendo la única ruta de mantenimiento y nunca convertirse en un recorrido del
  sistema de archivos durante una consulta.

## Decisiones

1. Extender SQLite a v11 de forma aditiva. La migración no eliminará columnas ni
   reescribirá PDFs y conservará copia previa para bases en disco.
2. Añadir un modelo de ciclo de vida inmutable con generación y fallo tipado,
   manteniendo `ReaderStatus` como adaptador temporal para no duplicar lógica de
   presentación.
3. Mantener contraseñas únicamente en variables locales del diálogo/controlador,
   con tres intentos máximos y sin incorporarlas a logs, sesiones o diagnósticos.
4. Hacer que el ensayo de backup sea una comparación por identidad estable y
   produzca cantidades de conservar, añadir, reemplazar y conflicto antes de
   permitir una importación destructiva.
5. Usar el pre-backup completo como unidad de undo de una consolidación. No se
   implementará criptografía propia: el cifrado opcional queda aplazado hasta
   disponer de una primitiva auditada compatible con el toolchain offline.
6. Enriquecer journals sin datos privados. Fase, razón y hash solo contienen
   identificadores/códigos agregados; nunca rutas, títulos, consultas o texto.

## Baseline y presupuestos

La comparación 0.11/0.12 terminó `pass-with-watch`, sin regresiones fiables ni
invariantes fallidos. Permanecen bajo observación búsqueda global p95, consulta
de colecciones p95, consulta de 25.000, primera carga de 100.000 y RSS mínimo al
volver. La regla 0.13 bloquea solo una mediana peor en más del 10 % cuando cuatro
de cinco muestras superan el límite.

Se mantienen estos invariantes: máximo 80 documentos materializados, cero
recorridos iniciales del sistema de archivos, cero documentos/renders/páginas
abiertos al cerrar y ausencia de datos privados en evidencia.

## Riesgos y límites físicos

- Una restauración completa reemplaza datos locales y seguirá exigiendo resumen,
  confirmación y pre-backup; el ensayo nunca escribe.
- Cerrar durante una apertura nativa no puede interrumpir todas las llamadas de
  terceros, pero su generación queda invalidada y cualquier handle tardío se
  libera antes de publicar estado.
- Narrator, varios monitores y DPI físicos 125/150/175/200 % solo se marcarán
  como aprobados si el hardware real está disponible. Las pruebas widget y las
  capturas simuladas se documentarán por separado.
- El release seguirá sin firma mientras no exista certificado y autorización.
  No se alterarán Smart App Control, Defender ni las protecciones del host.

## Resultado implementado

- El lector expone acciones tipadas para reintentar, localizar, introducir
  contraseña o volver a Biblioteca. El diálogo de contraseña es seguro y sus
  tres intentos permanecen exclusivamente en memoria.
- El ensayo de restore nombra explícitamente la estrategia de snapshot completo
  y que no usa última escritura. La revisión de duplicados conserva evidencia,
  impacto, conflictos, copia previa y undo.
- Problemas muestra journals interrumpidos e inaccesibilidad documental sin
  bloquear el arranque; recuperación de sesión exige elegir entre contexto
  verificado y estado limpio.
- La matriz ejecutada contiene 179 PNG y 19 hojas de contacto. Incluye las seis
  superficies de resiliencia, consolidación a texto 200 % y diez comparaciones
  0.12/0.13 con hashes de ambas fuentes.

## Evidencia revisada

- Fuentes: `build/visual-evidence/v0.13.0/`.
- Resiliencia: `build/visual-evidence/v0.13.0/resilience/resilience-matrix.json`.
- Comparaciones: `build/visual-evidence/v0.13.0/comparisons/comparisons.json`.
- Revisión completa: `build/visual-review-013/contact-sheets.json`.

Se inspeccionaron las 19 hojas. No hay overflow, excepciones de layout, acciones
fuera del viewport ni texto esencial truncado. La contraseña incorrecta y la
estrategia de restore permanecen legibles a texto 200 %. La evidencia es de
widgets ejecutados: no se presenta como validación física de Narrator, varios
monitores o DPI 125/150/175/200 %.
