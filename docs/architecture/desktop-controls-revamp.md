# Revisión de controles de escritorio

## Cambios

- Inicio en pantalla completa real mediante el servicio de ventana existente.
  Conserva la geometría normal y la ventana maximizada para restaurar al salir.
  F11 (o su asignación personalizada) funciona también fuera del lector; la
  Biblioteca ofrece además un botón para entrar y salir con el ratón.
- Barra de título nativa oscura con texto blanco. Windows 11 permite fijar
  el fondo negro; Windows 10 conserva la alternativa de chrome oscuro del
  sistema. Se mantienen minimizar, restaurar, cerrar y el comportamiento nativo.
- Biblioteca: búsqueda y filtros con nombre, alternancia directa entre
  carátulas y lista, selección múltiple visible y contador de documentos.
  Las variantes compactas conservan los menús y todos sus comandos.
- Abrir PDF y Retomar tienen texto en escritorio; Retomar usa una flecha y
  colores de botón primario con contraste correcto. En compacto conservan
  etiquetas accesibles e información emergente.
- Radios pequeños y consistentes en botones, campos, menús, selectores y
  grupos del lector. Estados hover/pulsado neutros, selección definida por
  superficie y borde, y deslizadores con un indicador más fino.

La retícula, los volúmenes 3D, el renderizado de carátulas, sus resortes y
preferencias de movimiento no cambian. Los callbacks de lectura, búsqueda,
filtrado, selección, importación y organización siguen siendo los existentes.

## Verificación

- Suite completa: 330 pruebas aprobadas, incluidas biblioteca, física,
  navegación del lector, persistencia, accesibilidad y referencias visuales.
- Referencias del lector y navegador revisadas y actualizadas en
  `docs/screenshots/windows-*-v015.png`.
- Matriz de Biblioteca regenerada bajo `build/visual-review-desktop/library/`:
  claro/oscuro, alto contraste, movimiento reducido, texto ampliado, RTL,
  estados de catálogo y selección. Inspección visual de escritorio oscuro y
  compacto con texto al 200 % sin desbordamientos.
- Informes locales: `build/visual-review-desktop/{tests,analyze,build}.log`.

## Mensaje de commit

```text
feat(ui): renovar controles de escritorio y abrir en pantalla completa

- Aplicar barra de título negra y conservar los controles nativos de Windows
- Iniciar en pantalla completa con salida desde Biblioteca y atajo configurable
- Mostrar búsqueda, filtros, vistas y selección directa en la biblioteca
- Mejorar las acciones Abrir PDF y Retomar y unificar los controles del lector
- Conservar carátulas 3D, física, accesibilidad y funciones existentes
- Actualizar pruebas y referencias visuales
```
