# Backup y recuperación

## Backup manual

Desde Ajustes, crea un backup y elige una ruta local. El JSON versionado incluye
Biblioteca, progreso, preferencias, espacios/cola, anotaciones, relaciones,
revisiones, reglas, journal y el historial de consolidaciones; no incluye PDF,
portadas, índice FTS ni cachés. El manifiesto registra versión de esquema,
cantidad y SHA-256 de cada categoría.

El formato no implementa criptografía propia. Si la copia necesita
confidencialidad, guárdala en un volumen cifrado por el sistema operativo o en
un archivo cifrado por una herramienta auditada. El límite físico de entrada es
128 MiB y cada tabla admite como máximo 100.000 filas.

## Restaurar

Selecciona el backup y ejecuta primero el modo ensayo. Yamantaka valida formato,
versión, esquema y hashes sin escribir. La vista previa informa tamaño,
entidades, altas, reemplazos, eliminaciones y conflictos de identidad. Confirma
solo después. La aplicación conserva una copia previa y aplica la sustitución
en una transacción; un fallo hace rollback y mantiene el pre-backup para
recuperación manual.

La estrategia no combina campos por fecha ni usa "última escritura gana". El
archivo representa un snapshot coherente: las identidades existentes se
reemplazan, las nuevas se añaden y las ausentes se retiran como una sola unidad
transaccional. Un conflicto de identidad se muestra en el ensayo y exige una
confirmación explícita antes de instalar ese snapshot.

## Cierre inesperado

La sesión se guarda atómicamente con espacio, documento, huella, página, total,
fase de apertura y posible documento en cuarentena. Si el cierre no fue limpio,
el siguiente inicio ofrece `Restaurar sesión` o `Empezar limpio`. Restaurar
verifica primero archivo, huella y total; nunca reabre automáticamente el PDF
que causó el fallo. Elegir limpio elimina el snapshot recuperable, no documentos
ni anotaciones.

Las operaciones masivas que quedaron `running` pasan a `interrupted`. Cada
journal declara fase, motivo, hash de compatibilidad y capacidad `retry`,
`resume`, `rollback` o `none`. La pantalla Problemas solo ofrece reintentar o
deshacer cuando el journal lo permite; también deja descartar el aviso o
ejecutar `PRAGMA integrity_check`. Un rollback incompleto queda como fallo
explícito y nunca como tarea activa.

Un archivo ausente se marca inaccesible, pero su registro y contexto se conservan
para relocalización exacta. Yamantaka nunca busca por todo el disco durante el
arranque ni elimina relaciones o revisiones por su cuenta.

## Duplicados

`Problemas y recuperación` abre la revisión de duplicados. Cada candidato
muestra si la huella coincide o si solo es probable, las evidencias, el número
de registros afectados y cada conflicto de metadatos. `Mantener ambos` no
escribe nada. Consolidar exige una copia previa, mueve únicamente registros
locales dentro de una transacción, conserva la procedencia de anotaciones y
ofrece deshacer restaurando la copia. Los dos PDF permanecen intactos en disco.
