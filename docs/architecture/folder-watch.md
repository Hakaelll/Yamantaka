# Observación de carpetas autorizadas

`FolderWatchController` adapta `Directory.watch` a
`AuthorizedFolderWatch`. Solo acepta raíces concedidas explícitamente, filtra
eventos PDF, agrupa cambios durante el debounce y limita el número de eventos
pendientes.

Un evento nunca importa, mueve, borra ni reubica un PDF por sí solo. El
controlador entrega una lista de rutas y la composición de la aplicación
recarga la biblioteca para que la superficie pueda ofrecer revalidación o
importación explícita. La casilla del diálogo de importación es el único punto
que autoriza una raíz; no se infiere permiso por haber elegido una carpeta.
Pausar la aplicación vacía la cola de eventos y retirar una raíz cancela
inmediatamente su suscripción y descarta sus eventos.
