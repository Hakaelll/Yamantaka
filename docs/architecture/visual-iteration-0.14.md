# Iteración visual 0.14

Fecha de auditoría inicial: 2026-09-03.

## Alcance y restricciones

La iteración parte de `0.13.0+14` y modifica únicamente presentación,
localización, pruebas, herramientas de verificación y documentación. Se
conservan el ciclo documental generacional, las contraseñas solo en memoria,
la recuperación, el backup v9, la consolidación reversible, SQLite v11, el
límite de 80 documentos y el ownership de PDFium. No se añade red, cuenta,
telemetría, OCR, IA ni modificación de PDF.

## Línea base ejecutada

- Git limpio en `main` antes de editar.
- Formato: 241 archivos, 0 cambios.
- Análisis: 0 incidencias en 233,7 s fuera del gate y 194,3 s dentro del gate.
- Suite: 305 pruebas aprobadas.
- Gate Windows: aprobaron higiene, toolchain, dependencias, contratos puros,
  formato, pruebas, corpus, benchmark, build Release, smoke sin red y las dos
  referencias visuales. Fallaron dos fases: el análisis superó el timeout
  global de 180 s pese a terminar sin incidencias y la matriz DPI quedó
  incompleta porque la escala física del host no se confirmó.
- Rendimiento previo: cinco procesos avanzados, cinco de Biblioteca y cinco
  perfiles Windows nativos en `build/performance/0.14.0-baseline/`.

## Inventario inicial

El sistema compartido se concentra en `app_tokens.dart`, `app_theme.dart`,
`app_components.dart`, `app_density.dart` y `yama_components.dart`.

- Breakpoints únicos: compacto `<680`, medio `680–1099`, expandido `>=1100`.
- Espaciado: 12 pasos entre 2 y 64 px.
- Tipografía: 10 roles semánticos, con información esencial a partir de 12 px.
- Forma: 9 radios, 5 niveles de elevación y 12 medidas de sombra.
- Movimiento: feedback, fast, standard, panel, diálogo, inspector, chrome y
  autoocultado, con ruta instantánea para movimiento reducido.
- Colores: tema claro, oscuro y Biblioteca, con aliases semánticos de fondo,
  superficie, texto, icono, borde, foco, selección y estado.
- Componentes comunes: encabezado de sección, vista/etiqueta de estado, pista
  de atajo, contenido de menú, encabezado de diálogo, zona peligrosa, botón de
  icono, grupo/divisor de toolbar, control segmentado y fila de Ajustes.
- Presentación: 31 archivos, 15 diálogos públicos, 31 aperturas de diálogo,
  92 `IconButton` directos, 22 `YamaIconButton`, 28 tooltips explícitos, 14
  widgets animados y 52 regiones `Semantics` explícitas.
- Superficies inventariadas: Biblioteca grid/list, filtros, selección y
  continuidad; lector Documento/Libro/Continua, chrome, panel, navegador,
  búsqueda y comparación; paleta y menús; contraseña, recuperación, archivo
  ausente, duplicados, restore, importación, relaciones, reglas, problemas,
  diagnóstico, ayuda y Ajustes.

## Hallazgos visuales

1. Ajustes usa categorías técnicas (`Controles del lector`, `Atajos`,
   `Historial`, `Datos y privacidad`, `Avanzado`) en vez de las intenciones
   pedidas. Al 200 % aumenta la carga de navegación y los elementos inferiores
   requieren exploración adicional.
2. Los diálogos críticos combinan `AlertDialog.icon`, títulos simples y
   cabeceras manuales. Solo dos consumidores usan `AppDialogHeader`, por lo que
   icono, descripción, separación y botón de cierre varían entre flujos.
3. En contraseña incorrecta, `InputDecoration` sustituye el helper por el
   error; deja de verse la garantía "no se guarda" precisamente en el estado
   de mayor ansiedad.
4. Restore presenta el plan como un bloque de texto y conserva medidas locales
   de 16/6 px. La jerarquía entre resumen, estrategia de conflicto y acción
   destructiva es débil.
5. Duplicados a texto 200 % no desborda, pero comprime navegación, decisiones y
   consolidación en una única fila; rutas y evidencia compiten con la acción
   reversible principal.
6. Las operaciones largas tienen fase o progreso en varios controladores, pero
   su presentación oscila entre línea indeterminada, `ListTile`, estado y
   diálogo específico. Falta un componente común que anuncie fase, progreso y
   cancelación.
7. Quedan medidas visuales locales en flujos críticos y chrome. Las excepciones
   del visor ligadas a PDFium son técnicas; las de diálogos y toolbars pueden
   reemplazarse por tokens.
8. La Biblioteca mantiene una acción primaria reconocible y la matriz de
   grid/list, selección, filtros y texto 130/150/200 % no muestra overflow. El
   centro de continuidad es claro, aunque debe conservar densidad en compacto.
9. El lector mantiene el PDF como protagonista, targets de 40 px, panel
   overlay/acoplado y visor estable. Las capturas widget de Documento, Libro,
   Continua, panel, fullscreen simulado, temas y movimiento reducido no
   presentan saltos ni overflow.
10. La captura nativa WGC no es aprobable: compone rectángulos obsoletos de la
    Biblioteca y devuelve blanco en el lector. UI Automation expone una sola
    jerarquía correcta, de modo que la duplicación no pertenece al árbol de la
    aplicación. La evidencia widget sigue siendo válida; la evidencia visual
    nativa queda pendiente de un capturador fiable.

## Decisiones de diseño

- Reorganizar Ajustes en: Apariencia, Lectura, Accesibilidad, Biblioteca,
  Privacidad, Almacenamiento, Recuperación y Diagnóstico.
- Mantener la identidad editorial existente, la Biblioteca espacial, el logo y
  la geometría del visor; la iteración refina jerarquía y coherencia.
- Añadir un marco de diálogo compartido con encabezado, descripción, límites
  de viewport, scroll interno y acciones adaptativas.
- Añadir un estado de operación compartido con fase, progreso determinado o
  indeterminado, live region y cancelación.
- Migrar primero contraseña, recuperación de sesión, restore, duplicados y
  reglas locales; son los flujos de mayor riesgo o carga cognitiva.
- Conservar el foco inicial del campo de contraseña y el cierre/restauración de
  foco propios de las rutas de diálogo de Flutter.
- Sustituir medidas locales por tokens solo cuando sean presentación. No tocar
  delays ni límites que formen parte del contrato de PDFium.
- Elevar el timeout externo del gate de 180 a 300 s: el análisis limpio ya
  demuestra que 180 s produce un falso negativo reproducible en este host.

## Implementación

- Se añadieron escalas de icono, altura, borde, opacidad y curva a los tokens;
  `YamaIconButton` consume el borde de foco y permite controlar estados en el
  harness visual.
- `AppDialogFrame` concentra cabecera, descripción, scroll, límites y acciones;
  `AppOperationStatus` concentra fase, progreso, cancelación y live region.
- Contraseña, recuperación, restore, duplicados y reglas locales migraron a
  esos componentes. El error de contraseña ya no sustituye la garantía de que
  la clave no se guarda.
- Ajustes usa ocho categorías por intención y un rail que crece hasta 320 px
  con texto grande. No cambia ningún campo persistido ni el esquema v11.
- Cerrar la búsqueda cancela su debounce pendiente; colecciones usan recuento
  indexado y espacios reutilizan snapshots mientras `total_changes()` no
  cambie. El feedback desktop elimina el ripple prolongado y conserva estados
  explícitos de hover, presión y foco.

## Validación ejecutada

- 314 pruebas aprobadas. Los contratos 0.14 cubren tokens, operación,
  hover/foco/selección/deshabilitado, taxonomía, 640×480 con texto 200 %,
  Escape, ES/EN, RTL, alto contraste, movimiento reducido y ocho breakpoints.
- 175 capturas directas más diez comparaciones 0.13/0.14: 185 PNG con SHA-256
  en 20 hojas. Se inspeccionaron las 20 hojas completas; la revisión corrigió
  la única truncación problemática observada en el rail de Ajustes al 200 % y
  la regeneración quedó aprobada.
- Cinco pasadas por familia terminan `pass-with-watch`, cero regresiones fiables
  y nueve invariantes aprobados. Se conservan cinco `watch` ruidosos con tres de
  cinco muestras por encima del 10 %.
- El gate Windows termina sus 14 fases en `passed`, sin omisiones. Compila
  Release, recorre el corpus patológico, ejecuta un smoke de ocho segundos con
  cero conexiones y confirma cuatro casos nativos a 1600×900 sobre los 120
  DPI/125 % reales del monitor disponible.
- Narrator, varios monitores, en-US, alto contraste real y DPI físicos
  100/150/175/200 % no están disponibles. WGC sigue sin ser un capturador fiable
  en este host; esas imágenes no se presentan como evidencia aprobada.

## Release local

`build/release/0.14.0+15/` contiene el bundle, el ZIP, SBOM CycloneDX 1.5,
estado de firma, gate, rendimiento, ambos smokes y la evidencia visual copiada.
El manifiesto visual verifica 217 archivos: 185 PNG originales —incluidas diez
comparaciones— y 20 hojas de contacto. El verificador empareja los 23 archivos
del bundle con el ZIP por ruta, tamaño y SHA-256 y confirma ocho hashes raíz.

El ZIP final mide 19.702.906 bytes y su SHA-256 es
`30b8f03b4f9d4352e8facadf817b8de8d1bb5c3b43c4db020f32763187e5d05e`.
El ejecutable extraído se activa, recibe dos navegaciones, permanece ocho
segundos sin conexiones y se cierra mediante el arnés. Los nueve binarios
nativos están sin firma: son aptos para esta ejecución local desde el bundle
completo, no para presentarlos como distribución firmada.
