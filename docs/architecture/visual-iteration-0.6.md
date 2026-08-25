# Iteración visual 0.6 — diagnóstico y criterios

Fecha: 2026-08-24. Línea base comparada: `0.5.0+6`.

## Diagnóstico priorizado

1. **Biblioteca sin consecuencia contextual.** La portada seleccionada cambiaba
   borde, escala y halo, pero título, estado y siguiente acción vivían en una
   placa pasiva. Para continuar era necesario recordar el doble clic o abrir el
   menú. Al filtrar fuera de la selección, no se explicaba por qué desaparecía.
2. **Chrome del lector binario.** El autoocultado retiraba a la vez título,
   navegación, zoom y posición. El PDF dominaba, pero el estado de lectura y la
   forma de recuperar controles quedaban implícitos. Los dos grupos inferiores
   parecían herramientas separadas.
3. **Ajustes describía en lugar de demostrar.** La vista previa era una placa
   con una página y texto; no mostraba la relación entre canvas, papel, chrome,
   densidad, progreso, movimiento o e-ink. Además desaparecía al cambiar de
   sección.
4. **Panel acoplado desaprovechaba altura.** Cinco iconos horizontales consumían
   una fila completa. El divisor comunicaba resize solo por cursor y semántica,
   sin feedback visual del ancho durante foco o arrastre.

## Fricción y pasos eliminados

- Seleccionar un documento ofrece ahora `Abrir`/`Continuar`/`Localizar` en el
  mismo contexto; se elimina el paso de abrir el menú o descubrir doble clic.
- Favorito e información quedan como las dos únicas acciones secundarias del
  inspector. Escape, cerrar y gesto descendente compacto retiran la superficie.
- Si un filtro excluye la selección, una explicación permite restablecer los
  filtros sin perder el documento seleccionado.
- La dock en reposo mantiene posición y progreso y se expande con clic, toque,
  foco o teclado. No depende de hover y no reserva espacio al PDF.
- Ajustes mantiene una muestra viva en expandido y usa el mismo
  `YamaToolbarGroup`/`YamaIconButton` del lector, no un segundo chrome.

## Riesgos y mitigaciones

- **Overflow:** inspector, dock y Ajustes cambian composición en `<680`,
  `680–1099` y `>=1100`; las matrices cubren ocho tamaños de 640×480 a
  3440×1440.
- **Accesibilidad:** foco blanco y selección violeta siguen independientes; las
  acciones tienen nombre semántico, 40 px mínimos y alternativa sin hover.
- **Movimiento:** entrada, salida y expansión pasan por `AppMotion.whenAllowed`;
  `disableAnimations` produce cambio inmediato.
- **Visor:** ningún estado nuevo entra en la clave de `_cachedViewer`. Chrome,
  dock, panel, rail y resize continúan en overlays o en la columna externa.
- **Privacidad/rendimiento:** no hay dependencias, red, persistencia nueva ni
  acceso adicional a PDFium/SQLite. La previsualización de Ajustes es geometría
  liviana y no abre documentos.

## Criterios verificables

- La acción primaria del documento seleccionado es visible sin hover.
- El documento seleccionado se conserva durante una consulta y se explica si
  deja de formar parte del resultado.
- Tras el autoocultado existe una dock accionable; activarla revela navegación
  y zoom sin cambiar el tamaño del visor.
- El panel acoplado usa rail vertical, el overlay conserva barra superior y solo
  una sección está montada.
- La previsualización persistente reacciona a tema, densidad, canvas, chrome,
  progreso, movimiento y perfil e-ink.
- Análisis, suite, golden normativa, matrices, build Release, manifiesto, hashes
  y smoke local deben quedar registrados antes de cerrar 0.6.0.
