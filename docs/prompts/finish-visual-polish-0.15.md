# Yamantaka — terminar 0.15 y preparar las siguientes mejoras

Continúa sobre el árbol actual. El rediseño editorial ya está implementado;
no reinicies el proyecto ni repitas una auditoría general. El objetivo es cerrar
los problemas pendientes con evidencia y entregar un candidato local honesto.

## Contexto que debes comprobar primero

Lee `docs/architecture/visual-iteration-0.15.md`, `NEXT_TODOS.md`, `TESTING.md`
y los informes actuales bajo `build/verification`, `build/performance/0.15.0-current`
y `build/release/0.15.0+16`. Comprueba fechas y correspondencia con el código:
un informe histórico no valida cambios posteriores. Lee las instrucciones del
repositorio y conserva un checkpoint recuperable del diff antes de editar.

El cierre dejó 325 pruebas aprobadas y compilación/smoke correctos. El último
gate conserva 13 fases aprobadas y una fallida por una copia histórica de
diagnóstico dentro de build; esa copia ya se apartó a Temp. El análisis repetido
por separado pasó sin incidencias. Aún hace falta un gate completo verde en
una única ejecución; no modifiques el informe fallido para simularlo.

Ya existen tema editorial compartido claro/oscuro, navegación de Biblioteca,
portadas tipográficas, continuidad compacta, lector y paneles más legibles,
vista previa de Ajustes ampliada y mejoras en menús, reglas y duplicados.
Conserva ese trabajo. No cambies otra vez la dirección artística.

La evidencia reúne 193 PNG, 11 comparaciones y 21 hojas de contacto. El flujo
nativo automatizado usa PDFium real y datos temporales; sus imágenes proceden
del árbol Flutter, no acreditan el compositor, DPI físico o Narrator.

## Prioridad 1: cerrar el rendimiento sin entrar en bucle

Lee `build/performance/0.15.0-current/comparison-0.14-vs-0.15.json` y sus muestras.
La última serie registrada detectó regresiones de construcción de frames p95:
Biblioteca 6986 frente a 4672 microsegundos (+49,53 %) y lector 8078 frente a
6760 (+19,50 %). Son tiempos de construcción, no latencia total ni una prueba
por sí solos de tirones visibles. No los presentes como aprobados.

Ya se intentaron reutilización de portadas, evitar interpolaciones idénticas,
eliminar un Theme redundante y aislar apertura/escritura de búsqueda del catálogo.
Consulta el código y las pruebas antes de repetir esos experimentos.

1. Comprueba comparabilidad de SDK, compilación, fixtures y carga del equipo.
   El checkpoint temprano `pre-polish-20260905-020053.zip` contiene cambios
   parciales y un error de compilación: NO es una base limpia 0.14 ejecutable.
   No reconstruyas una supuesta referencia inicial inventando o retocando datos.
2. Perfila una operación concreta y formula una hipótesis respaldada por trazas.
3. Haz como máximo dos experimentos pequeños y reversibles por sesión. Conserva
   uno solo si mejora la operación y sus contratos funcionales/visuales pasan.
4. Ejecuta la serie completa de cinco pasadas una sola vez al final de cambios
   prometedores, no después de cada ajuste cosmético. Conserva todas las muestras.
5. No rebajes la regla: mediana más de un 10 % peor y al menos cuatro de cinco
   muestras sobre el límite significa regresión fiable. No elimines muestras
   desfavorables ni alteres el benchmark para aprobar.

Si después de esos dos experimentos no hay una mejora demostrable, detén esta
línea y entrega diagnóstico, evidencia y decisión pendiente. No encadenes nuevas
series ni prometas un cierre inmediato que se alarga indefinidamente.

## Prioridad 2: revisión final dirigida

Abre una compilación del árbol actual con fixtures aislados. Recorre Biblioteca
→ PDF → búsqueda/panel → Ajustes → retorno y reanudación. Comprueba contexto,
foco, posición y zoom. No alteres documentos ni preferencias personales.

Busca únicamente defectos reproducibles: desbordamientos, controles inaccesibles,
foco perdido, textos internos, truncado esencial o notificaciones que tapen el
lector. Revisa 1280×720, 640×480 con texto ampliado, claro/oscuro y ES/EN; amplía
la matriz solo donde haya cambios. Contrasta cualquier captura nativa corrupta
con evidencia fiable antes de atribuirla a la aplicación.

Las siguientes mejoras visuales son candidatas, no defectos ya confirmados:

- Equilibrar catálogo y detalle con uno o pocos documentos si la vista real
  muestra demasiado vacío o una selección que tapa contenido.
- Mejorar descubrimiento de filtros y acciones secundarias si el recorrido
  demuestra que siguen ocultos; conservar navegación compacta y teclado.
- Afinar resultados de búsqueda, paneles y diálogos densos con títulos largos
  y texto al 200 %, sin reducir fuentes para encajar.

Implementa como máximo tres correcciones pequeñas confirmadas. Las propuestas
restantes van al backlog, con problema, beneficio y criterio de aceptación.
No conviertas el cierre en otro rediseño completo.

## Prioridad 3: una entrega verificable

Tras el último cambio funcional ejecuta formato, análisis, suite completa y
`tool/verify_windows.ps1 -AllowDirty -DpiScaleConfirmed` sin saltar fases.
La opción DPI no demuestra por sí sola una prueba física: comprueba qué midió
el informe y declara la limitación. Ejecuta el recorrido nativo si sus
superficies cambiaron. Inspecciona las imágenes afectadas antes de aceptar goldens.

Genera el paquete mediante `tool/package_windows.ps1` y compruébalo con
`tool/verify_windows_package.ps1`. Mantén sus validaciones estrictas. Verifica
el ejecutable exacto extraído del ZIP, manifiesto, SHA-256, SBOM y smoke offline.
Si el rendimiento sigue fallando, entrega únicamente un candidato no aprobado;
un gate funcional verde no elimina ese fallo.

Actualiza documentación con resultados reales, número exacto de pruebas,
fechas, rutas y limitaciones. Narrator, varios monitores y escalas físicas no
disponibles quedan pendientes, nunca aprobados mediante simulación.

## Límites y forma de trabajar

- Mantén Flutter, arquitectura, SQLite v11, máximo 80 documentos materializados,
  ciclo documental seguro, contraseñas solo en memoria y operaciones reversibles.
- Conserva identidad y logotipo, funcionamiento local-first/offline y PDF intacto.
- No añadas cuentas, red, telemetría, OCR, IA ni nuevas funciones de negocio.
- No hagas reset global, commit, tag, push, firma ni publicación.
- No actualices versión automáticamente si solo estás cerrando 0.15.0+16.
- Comunica avances breves: resultado conseguido, problema pendiente y siguiente
  paso. No repitas auditorías o mediciones sin una hipótesis nueva verificable.
- Cuando se alcance el límite de experimentos, cierra la sesión con lo probado.
  Solicita una decisión concreta si hace falta ampliar el alcance.

Entrega un resumen breve con cambios perceptibles, comprobaciones, enlaces a
comparaciones, ejecutable y ZIP, tamaño y SHA-256, y un máximo de cinco pendientes
priorizados. Distingue claramente terminado, fallido y no verificado. Incluye
un mensaje de commit convencional listo para copiar, sin ejecutarlo.
