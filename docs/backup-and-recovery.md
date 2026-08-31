# Backup y recuperación

## Backup manual

Desde Ajustes, crea un backup y elige una ruta local. El JSON versionado incluye
Biblioteca, progreso, preferencias, espacios/cola, anotaciones, relaciones,
revisiones, reglas y journal; no incluye PDF, portadas, índice FTS ni cachés.

## Restaurar

Selecciona el backup y ejecuta primero el modo ensayo. Yamantaka valida formato,
versión, hashes, referencias y conflictos sin escribir. Revisa la vista previa y
confirma solo después. La aplicación conserva una copia de seguridad previa y
aplica la restauración en una transacción; un fallo hace rollback.

## Cierre inesperado

La sesión se guarda atómicamente con workspace y documento activo. Si el cierre
no fue limpio, el siguiente inicio ofrece `Restaurar sesión` o `Empezar limpio`.
Elegir limpio elimina el snapshot recuperable, no documentos ni anotaciones.

Las operaciones masivas que quedaron `running` pasan a `interrupted`. La pantalla
Problemas permite simularlas de nuevo y reanudarlas como ejecución reversible,
descartar el aviso o ejecutar `PRAGMA integrity_check`. Cancelar una operación
intenta deshacer en orden inverso; un rollback incompleto queda como fallo
explícito y nunca como tarea activa.

Un archivo ausente se marca inaccesible, pero su registro y contexto se conservan
para relocalización exacta. Yamantaka nunca busca por todo el disco durante el
arranque ni elimina relaciones o revisiones por su cuenta.
