# Privacidad

Yamantaka es local-first:

- Los documentos no se suben a Internet ni se copian a la biblioteca interna.
- No se requiere cuenta.
- Biblioteca, marcadores, progreso, sesiones y preferencias son locales.
- No existe telemetría por defecto, publicidad ni venta o intercambio de datos.
- Los logs no incluyen texto del PDF, selección, contraseñas ni contenido.
- La aplicación no hace comprobaciones remotas silenciosas.
- El usuario puede limpiar caché e historial y eliminar todos los datos internos.

Un PDF puede contener enlaces. Los destinos internos no salen del documento.
Para `http`, `https` o `mailto`, Yamantaka muestra la dirección y requiere una
confirmación; solo después entrega el enlace a la aplicación predeterminada del
sistema. Esa acción explícita puede usar red fuera de Yamantaka. Otros esquemas
se bloquean.

Una futura exportación/restauración será explícita y local. Una futura
sincronización deberá ser opcional, privada y objeto de una nueva decisión.
