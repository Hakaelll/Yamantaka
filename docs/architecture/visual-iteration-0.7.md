# Iteración visual 0.7 — diagnóstico y criterios

Fecha: 2026-08-25. Línea base comparada: `0.6.0+7`.

## Diagnóstico priorizado

### P0 — continuidad, geometría y acceso

1. **El chrome activo sigue fragmentado.** Navegación y zoom se presentan como
   dos cápsulas independientes y muy separadas en escritorio. La vista conserva
   el centro del PDF, pero la relación entre la tarea primaria (navegar) y las
   herramientas secundarias (zoom, ajuste, tema y marcador) se aprende por
   posición, no por composición.
2. **La compactación del lector es tardía.** En 640 px las dos cápsulas casi
   ocupan todo el ancho y el porcentaje puede partirse en dos líneas. La
   solución actual no produce overflow, pero reduce legibilidad y hace que el
   chrome parezca una barra móvil.
3. **El resize del panel comunica valor, pero invade el contenido.** La etiqueta
   aparece hacia el interior sin reservar una zona visual y el indicador activo
   no diferencia con suficiente claridad hover, foco y arrastre.
4. **Foco y selección tienen contratos distintos por superficie.** Los estados
   existen, pero algunos controles Material y varios `PopupMenuButton` no usan
   el mismo indicador de forma que los componentes Yama. Esto debilita la
   navegación exclusivamente por teclado.

### P1 — jerarquía y orientación

5. **La cabecera de Biblioteca y sus herramientas no forman una composición.**
   Título/contador quedan en la esquina superior izquierda, mientras acciones y
   filtros forman dos islas a la derecha. El contador tiene poco peso y no
   distingue colección total de resultado filtrado.
6. **Los filtros activos se reconocen, pero no se pueden leer de un vistazo.**
   Un punto y un botón de reset indican que existe contexto, sin explicar
   cuántas condiciones están aplicadas ni separar filtro, agrupación y orden.
7. **El inspector de selección es informativo, pero todavía plano.** En
   expandido concatena autor, progreso y fecha en una sola línea. No presenta
   estado, cifra, línea de progreso y acción como columnas editoriales; además,
   un documento terminado ofrece `Continuar` en vez de `Releer`.
8. **La barra superior del lector conserva dos cápsulas desconectadas.** El
   documento y las herramientas se leen como widgets flotantes equivalentes;
   falta una relación de plano que otorgue al título mayor continuidad y a las
   acciones un orden funcional explícito.
9. **El panel mezcla cabecera, rail y contenido sin un ancla compartida.** El
   título cambia con la sección, pero el indicador activo es el contorno del
   botón circular; la sección no tiene la línea vertical editorial solicitada.
10. **Ajustes es correcto, pero visualmente homogéneo.** Navegación, contenido y
    preview usan superficies casi idénticas; los grupos Preferencias,
    mantenimiento y privacidad no se distinguen, y las acciones destructivas
    dependen solo de su fila individual.
11. **La preview de Ajustes es demasiado pequeña y pasiva.** Demuestra canvas,
    papel y una variante del chrome, pero no permite alternar reposo, activo,
    panel o e-ink. Sus etiquetas ocupan atención sin aportar manipulación.
12. **La paleta usa un icono genérico para todos los comandos.** La selección es
    clara, pero la exploración visual no distingue navegación, documento,
    búsqueda, vista y configuración; tampoco hay preview textual del resultado
    seleccionado en expandido.

### P2 — feedback, estados y consistencia

13. **Los estados semánticos no están completos como vocabulario tipado.** Hay
    colores de superficie para hover/pressed/active y tonos de éxito/alerta/
    error, pero faltan nombres explícitos para disabled, loading, selected y
    focused como conjunto consumible y auditable.
14. **Los estados de portada son parciales.** Ausente, error, favorito,
    terminado y progreso se ven; archivo modificado, análisis pendiente y
    metadatos en revisión no forman todavía una familia común de forma e icono.
15. **El feedback reversible se concentra en snackbars.** Es correcto para
    acciones sin contexto, pero selección, filtros, pin y resize pueden ofrecer
    confirmación directamente en su superficie antes de recurrir a una capa
    global.
16. **Menús y diálogos heredan un tema coherente, pero no un patrón completo.**
    Varias filas de menú mezclan texto solo con icono+texto, y algunos diálogos
    no incluyen explicación breve o foco inicial explícito.

## Dirección aplicada

La iteración usa una **retícula editorial de escritorio**: planos contenidos,
líneas de sección, cifras tabulares y grupos unidos por divisores. Biblioteca
conserva `#080808` y `#B58AFF`; lector conserva el PDF como máximo contraste;
Ajustes adopta una jerarquía master-detail más legible. La personalidad nace de
alineación, ritmo, escala y cambios de plano, no de efectos.

## Soluciones y comprobación

| Superficie | Fricción | Solución | Evidencia de mejora |
| --- | --- | --- | --- |
| Sistema global | Estados dispersos | Completar tokens de interacción, escala tipográfica editorial y componentes de indicador/divisor | tests de tokens, contraste y targets |
| Biblioteca | Cabecera y filtros separados | Cabecera editorial con kicker, contador fuerte, acción primaria y contexto de filtros visible | matrices 640–3440, ES/EN y filtros activos |
| Inspector | Metadatos concatenados | Banda por columnas con estado icónico, progreso numérico+línea y acción contextual Abrir/Continuar/Releer/Localizar | pruebas por progreso, ausente y breakpoints |
| Lector | Dos islas inferiores | Una dock activa unificada: navegación primaria, divisor y herramientas secundarias compactables | geometría del visor y buildCount permanecen estables |
| Barra superior | Cápsulas equivalentes | Superficie continua con región documental flexible, divisor y herramientas ordenadas | acciones no duplicadas por breakpoint |
| Temas | Muestras poco contextuales | Muestra de canvas+papel+chrome, nombre, selección, foco y explicación de que no modifica el PDF | teclado y semántica para seis temas |
| Panel | Activo poco expresivo | Rail plano con línea vertical activa; cabecera y contenido comparten plano; resize con estados distintos | pruebas de foco, drag, teclado, valor y una sección montada |
| Ajustes | Tres columnas homogéneas | Navegación agrupada, encabezado de sección editorial y preview con plano hundido | matrices, texto aumentado y scroll independiente |
| Preview | Demostración pasiva | Selector local Reposo/Activo/Panel/e-ink que nunca persiste ni abre PDFium | prueba de cambio local y ausencia de segundo visor |
| Paleta | Comandos indistinguibles | Iconos por familia, metadatos, atajos alineados y preview textual expandida | Enter/Escape, retorno de foco y ES/EN |
| Menús/diálogos | Mezcla de patrones | Altura/padding/iconos comunes y estructura explicativa con acción segura | tests de foco, Escape, Enter seguro y texto largo |

## Riesgos y límites

- **Overflow:** toda composición cambia en `<680`, `680–1099` y `>=1100`; se
  prueba además con texto 1,3× y títulos largos.
- **Reconstrucción del visor:** ningún estado visual nuevo forma parte de
  `_viewerCacheKey`. Chrome, preview, panel, hover y selección quedan fuera del
  adaptador PDF; el test de `buildCount` debe permanecer en uno.
- **Cambio de geometría:** dock y barra continúan como overlays. El panel solo
  resta ancho en expandido, según el contrato existente.
- **Foco:** overlays y diálogos restauran el invocador; Escape cierra primero la
  superficie superior. La selección nunca sustituye al foco.
- **Movimiento:** pressed 80 ms, selección 160 ms, inspector 160 ms, chrome
  160 ms, panel 200 ms y diálogo 200 ms; todos pasan por reducción de movimiento.
- **Privacidad/rendimiento:** sin red, paquetes, tablas ni persistencia nueva.
  La preview usa widgets ligeros y no abre PDFium.

## Alternativas descartadas

- Una cabecera de Biblioteca a pantalla completa: reduce superficie útil y
  compite con las portadas.
- Más tarjetas o fondos por sección: fragmentan la retícula y aumentan niveles
  de elevación sin mejorar orientación.
- Una dock inferior con todas las acciones siempre visibles: empeora compacto
  y niega la divulgación progresiva.
- Gradientes, blur, textura y sombras mayores: añaden presencia al chrome y no
  resuelven jerarquía.
- Una miniatura de lector basada en PDFium: duplica motor, memoria y lifecycle
  para una demostración que puede componerse con widgets reales de chrome.

## Criterios de terminado de la iteración visual

- La cabecera de Biblioteca explica colección, resultado y filtros sin abrir
  un menú; el reset visible conserva la selección si vuelve al resultado.
- El inspector ofrece estado, progreso y acción correcta en los tres
  breakpoints y no tapa ni desplaza la constelación.
- Dock activa y reposo comparten una sola familia visual; alternarlas no cambia
  tamaño, centro ni clave del visor.
- Barra, dock y panel muestran selección, foco, hover, pressed y disabled sin
  depender solo del color.
- La preview alterna cuatro demostraciones sin cambiar preferencias reales ni
  crear otro lector.
- Paleta, menús y diálogos tienen jerarquía, teclado, Escape y retorno de foco
  predecibles.
- No existe overflow en la matriz requerida; análisis, pruebas, golden,
  matrices, build, smoke, hashes e inventario de firma quedan registrados.

## Resultado verificado

- Biblioteca presenta colección y resultado como información separada, hace
  visible el número de filtros y dispone `Abrir/Continuar/Releer/Localizar` sin
  abrir un menú. Estados de archivo y análisis combinan forma, icono y texto.
- Lector usa una sola superficie superior y una dock activa única. La prueba
  geométrica mantiene la navegación primaria centrada y las regresiones de
  `buildCount` prueban que chrome, panel y resize no reconstruyen el visor.
- El rail tiene indicador vertical y el divisor diferencia hover, foco y drag.
  Ajustes agrupa navegación, separa destrucción y alterna cuatro demos locales.
- Paleta usa iconos por comando, metadata y preview expandida; Enter, Escape y
  retorno de foco están cubiertos.
- `dart format`, `git diff --check`, análisis, 247 pruebas, golden, matrices y
  build Windows Release están aprobados.
- Evidencia: 71 capturas de widgets y diez composiciones 0.6/0.7 generadas por
  script, todas bajo `build/visual-evidence/v0.7.0/`.
- Release local: 21 archivos, 43.138.872 bytes; ZIP 18.705.581 bytes. El smoke
  de 8 s con PDF sintético registra cero conexiones TCP/UDP.
- Firma: 0 de 9 binarios nativos tienen Authenticode válido. No se firmó ni se
  publicó ningún artefacto.

## Limitaciones pendientes

- Narrador, varios monitores, táctil real y escalas físicas 125/150/175/200 %
  requieren una pasada manual en hardware; las matrices de widget no la
  sustituyen.
- El selector de tema demuestra canvas/papel/chrome con widgets ligeros; el PDF
  real sigue sin filtros, pero el contraste de cada PDF concreto depende del
  contenido original.
- La estructura global normaliza tema y ejemplos principales de menú/diálogo;
  futuros diálogos de dominio deben migrar gradualmente a una cabecera
  explicativa común cuando se modifique su flujo.
