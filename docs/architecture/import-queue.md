# Cola de importación local

`FolderImportController` mantiene la cola de trabajo en SQLite mediante
`ImportQueueRepository`, dentro de `app_metadata`. Solo se guardan identidad
de archivo, ruta normalizada, huella y estado; no se copia el PDF ni se guarda
su contenido.

El flujo tiene cuatro garantías:

1. Un escaneo solo produce una previsualización. La importación requiere la
   acción explícita del usuario.
2. Cada elemento se reclama y persiste antes de abrir el PDF. Si el proceso
   termina, los elementos `running` vuelven a `pending` en la siguiente sesión.
3. Antes de registrar un elemento reanudado se vuelve a calcular su huella. Un
   archivo modificado falla de forma individual y no aborta el lote.
4. El registro de biblioteca se deduplica por ruta normalizada o huella; el
   cierre del PDF nativo ocurre en `finally` tras cada elemento.

La cola tiene un límite de 5.000 elementos y los errores se expresan como
códigos sin rutas ni títulos en diagnósticos.
