# Iteración integral 0.10: continuidad, orientación y trabajo local

Estado: implementación completa y gates automatizados aprobados. La matriz
física con Narrator, varios monitores y DPI reales queda declarada como prueba
manual pendiente.

## Intención

Yamantaka 0.10 convierte la base visual compacta de 0.9 en una experiencia más
continua: retomar una lectura debe ser inmediato, cambiar entre documentos no
debe perder contexto y localizar una página o un resultado debe explicar con
precisión qué ocurrirá. La iteración conserva el motor PDFium/pdfrx, SQLite con
FTS5, el funcionamiento sin cuenta ni red, los seis entornos de lectura, el
movimiento reducido y la identidad visual existente.

## Diagnóstico previo a implementar

La revisión se realizó sobre el commit
`2ed637f feat(ui): completa la iteración visual 0.9 de Yamantaka para Windows`,
con el árbol limpio. `flutter analyze --no-pub` terminó sin incidencias y la
suite base completó 254 tests. Se inspeccionaron las capturas 0.9 del lector en
640×480, 800×600, 1280×720, 1600×900, 1920×1080, 1920×1200, 2560×1440 y
3440×1440.

### Hallazgos de experiencia

1. `LibraryController.continueReading` ya identifica una lectura reciente sin
   terminar, pero la Biblioteca no la presenta. El retorno depende de localizar
   de nuevo el artefacto dentro de la colección.
2. Las pestañas existen como dominio y conservan página, zoom, layout, modo,
   scroll y progreso, pero no hay conmutador visible ni atajos Ctrl+Tab /
   Ctrl+Shift+Tab. Activar una pestaña llama a `openPath`, cierra el documento
   PDFium actual y lo abre otra vez; el estado se recupera, pero se paga trabajo
   nativo evitable y la continuidad perceptiva es débil.
3. El indicador de página abre un diálogo accesible con número y scrubber, pero
   no entiende porcentaje, primera/última página ni expone el historial atrás y
   adelante. Un número inválido puede cerrar el diálogo antes de mostrar el
   error.
4. La búsqueda global ya usa FTS5, fragmentos y vista previa responsive. Sin
   embargo, mezcla todos los tipos sin filtros operables por teclado, refresca
   anotaciones/organización en cada consulta y no distingue búsqueda cancelada,
   índice incompleto u obsoleto y documento ausente.
5. La Biblioteca ya pagina, usa slivers, conserva selección/scroll/filtros y
   ofrece grid/list. La barra todavía separa búsqueda, facetas, filtros y orden
   en varios controles; falta una vista guardada explícita y el inspector no
   diferencia con suficiente claridad Continuar, Releer y Localizar.
6. La importación por carpeta tiene cola persistente, progreso, duplicados,
   errores y cancelación. La interfaz no muestra el estado final por elemento ni
   ofrece acciones útiles para duplicados; cerrar durante la operación está
   bloqueado, pero el resultado operativo queda resumido solo por contadores.
7. Anotaciones ofrecen filtro, orden, edición, borrado, deshacer/rehacer y
   exportación local. Marcadores y anotaciones viven en secciones separadas, por
   lo que revisar todo lo asociado a una página requiere alternar contexto.
8. Ajustes está agrupado y tiene una previsualización viva, pero no ofrece una
   búsqueda de preferencias ni hace siempre explícito qué opción es global y
   cuál pertenece al documento. El restablecimiento total compite con acciones
   de alcance menor.

### Hallazgos visuales

- El contrato 0.9 de barra 56/52 px, dock 48/38 px, objetivos de 40 px, iconos
  de 18–20 px y radios 10–12 px funciona bien y no debe crecer.
- En ventanas compactas no se observan desbordamientos en la evidencia base. En
  pantallas grandes y ultrapanorámicas, el blanco lateral del lector es
  intencional y protege el tamaño del papel; el problema no es llenar el vacío,
  sino aportar orientación discreta sobre lectura, pestañas y progreso.
- La jerarquía pierde fuerza cuando estados de trabajo diferentes reutilizan
  el mismo patrón de lista o contador. Búsqueda, importación e índice necesitan
  estados textuales diferenciados sin añadir tarjetas, gradientes o sombras.
- La marca `yamantaka-logo-purple.png` no requiere cambios. Su hash y dimensiones
  se verificarán contra 0.9.

## Línea base de rendimiento

El benchmark de Biblioteca se ejecutó con paginación de 80 elementos y una base
SQLite temporal:

| Corpus | Inserción | Primera carga | Mediana de consulta | p95 | Materializados | RSS final |
|---:|---:|---:|---:|---:|---:|---:|
| 5 000 | 371,240 ms | 16 349 µs | 2 380 µs | 5 160 µs | 80 | 242 446 336 B |
| 25 000 | 1 632,415 ms | 22 208 µs | 5 647 µs | 28 952 µs | 80 | 262 250 496 B |
| 100 000 | 12 960,634 ms | 169 847 µs | 19 392 µs | 109 575 µs | 80 | 331 087 872 B |

El informe está en `build/verification-010-baseline/library-benchmark.json` y
registra integridad `ok` sin rutas privadas. El benchmark avanzado no llegó a
ejecutarse en esta línea base: el compilador Dart 3.12 falló durante la
transformación FFI con `InvalidType is not a subtype of FunctionType`. La causa
era una dependencia invertida del repositorio de espacios de trabajo hacia
tokens visuales de Flutter. 0.10 mueve el límite persistente al dominio y el
benchmark Dart puro vuelve a ejecutarse.

## Decisiones de implementación

- Añadir Continuar leyendo como una franja editorial compacta, condicional y
  con una sola acción primaria, sin convertirla en otra cuadrícula de tarjetas.
- Añadir un conmutador de pestañas compacto con orden estable y control por teclado;
  mantener recursos nativos solo para el documento activo y un conjunto caliente
  acotado, liberando los suspendidos de forma explícita.
- Convertir el selector de página en un editor validado que acepte página,
  porcentaje, primera/última y enlaces al historial, manteniendo Escape y el
  retorno de foco.
- Aplicar filtros de búsqueda en SQL, debounce y generaciones cancelables; no
  reconstruir filas de índice recreables durante cada pulsación.
- Reutilizar componentes de estado y tokens 0.9. El PDF sigue siendo la superficie
  dominante y el chrome no entra en la clave estable del visor.
- No añadir cuentas, telemetría, llamadas de red, movimiento o copia implícita de
  PDFs, ni datos privados a diagnósticos, manifiestos o benchmarks.
- Mantener las restricciones persistentes de los espacios de trabajo en el
  dominio para que repositorios y benchmarks no dependan del runtime visual.

## Contratos que no cambian

- Barra superior 56 px en escritorio y 52 px en compacto.
- Dock 48 px activo y 38 px en reposo.
- Objetivos de interacción ≥40 px, foco visible de 2 px y cifras tabulares.
- Título real con una línea, ellipsis y tooltip.
- Panel lateral redimensionable y persistente que no participa en la identidad
  del visor.
- Orden visual `focus > pressed > selected > hover > rest`.
- Seis entornos de lectura, alto contraste y movimiento reducido.
- Datos, búsqueda, anotaciones, exportaciones y diagnósticos exclusivamente
  locales.

## Cambios de experiencia entregados

- Biblioteca incorpora una franja condicional de Continuar leyendo, conserva
  selección/filtros/scroll/foco y acepta uno o varios PDF por drop sin duplicar
  rutas.
- Las pestañas permiten Ctrl+Tab/Ctrl+Shift+Tab y apertura nueva con
  Ctrl+Enter. Cada instancia conserva página, zoom, layout, modo, scroll, tema
  y panel; un contador monotónico evita colisiones al duplicar el mismo PDF.
- Ir a página interpreta número, porcentaje e inicio/fin, mantiene validación
  inline y devuelve el foco al visor al cerrar.
- La búsqueda FTS5 filtra seis fuentes, ordena por relevancia con recencia solo
  como desempate, cancela generaciones obsoletas y permite recorrer resultados
  y abrirlos con Enter/Ctrl+Enter sin ratón.
- La importación expone estado por archivo. Cada duplicado permite abrir el
  existente, comparar huella/tamaño/fecha o ignorarlo durante la revisión.
- El panel de Anotaciones combina notas y marcadores, filtra por tipo y ordena
  por página. El salto anuncia y dibuja un foco temporal de 2 px fuera del PDF.
- Ajustes añade búsqueda, alcance explícito y reset por sección. Paleta,
  previews y superficies auxiliares crecen o hacen scroll a texto 150/200 %.

## Riesgos y límites de verificación

- Retener documentos PDF abiertos consume memoria nativa; la política caliente
  debe tener límite, liberar de manera determinista y medirse con PDFs grandes.
- FTS5 permite filtrar con precisión, pero refrescar fuentes derivadas dentro del
  gesto de búsqueda introduciría bloqueos; esa actualización debe salir del
  camino interactivo.
- Narrator, varios monitores y DPI físico 100/125/150/175/200 % requieren prueba
  manual real. Las matrices widget y capturas no sustituyen esa validación y se
  informarán como pendientes si no se ejecutan.
- Firma Authenticode y publicación permanecen fuera de alcance. El release local
  incluirá inventario de firma y hashes, sin afirmar firma inexistente.

## Evidencia final automatizada

El gate reproducible de Windows terminó sin fallos. Aprobó higiene, toolchain,
resolución offline de dependencias, contratos puros, formato, análisis estático,
266 pruebas unitarias/widget, corpus PDF sintético, benchmark de Biblioteca,
build Release, smoke nativo sin red y golden mantenida. La matriz física de DPI
se omitió de forma explícita, por lo que el reporte global usa estado `partial`
sin ocultar ese único pendiente manual. El informe íntegro está en
`build/verification-010-final-gate/verification-report.json`.

### Rendimiento posterior

Una ejecución repetida y estable, conservada junto a las demás mediciones en
`build/verification-010-final/`, produjo:

| Corpus | Inserción | Primera carga | Mediana | p95 | Δ inserción | Δ primera | Δ mediana | Δ p95 | Δ RSS |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 5 000 | 394,620 ms | 14 354 µs | 2 290 µs | 5 644 µs | +6,3 % | −12,2 % | −3,8 % | +9,4 % | +0,53 % |
| 25 000 | 1 612,370 ms | 23 183 µs | 4 225 µs | 12 852 µs | −1,2 % | +4,4 % | −25,2 % | −55,6 % | +0,49 % |
| 100 000 | 6 672,120 ms | 82 888 µs | 9 746 µs | 46 001 µs | −48,5 % | −51,2 % | −49,7 % | −58,0 % | +1,72 % |

Todas las bases terminaron con integridad `ok`, 80 documentos materializados y
cero recorridos del sistema de archivos en primera carga. La pasada de
estabilidad deja el p95 de 5 000 en +9,4 %, dentro del límite del 10 %, y todas
las variaciones de RSS por debajo del 2 %. Se conservan también las ejecuciones
anteriores más ruidosas; no se descarta evidencia por silencio.

El benchmark avanzado completó 500 documentos, 100 anotaciones, diez
colecciones y cinco pestañas con integridad `ok`: búsqueda global mediana
700 µs/p95 5 249 µs, consulta de colección 323/620 µs y restauración de espacio
620/1 219 µs, sin crecimiento de RSS sobre la muestra inicial. El informe no
contiene rutas ni títulos.

El perfil nativo Profile se ejecutó dos veces. La segunda pasada registró
apertura pesada p95 19 008 µs, render pesado p95 6 577 µs, frame del lector p95
6 840 µs, interacción p95 77 276 µs, primera página de 100/5 000 páginas en
85 622/185 441 µs y crecimiento de RSS tras 15 ciclos pesados de 5 357 568
bytes. Al cerrar quedaron cero documentos, renders pendientes y bytes de caché.

El comparador histórico queda en `fail` porque su referencia es 0.3.0+4, no
0.9: apertura pesada marca +20,9 % y frame p95 +22,8 %, mientras render mejora
5,9 % e interacción mejora 11,8 %. Entre las dos pasadas actuales la apertura
varió −6,9 %, render −9,4 %, frame −2,5 % e interacción −24,2 %, señal de que la
última métrica no es estable. No se recalibra la referencia: para aceptar una
comparación de release debe repetirse 0.9 y 0.10 con el mismo harness, build y
host limpio.

### Evidencia visual, privacidad y binarios

- 128 PNG ejecutados de superficies y diez comparaciones, descritos por siete
  manifiestos, cubren Biblioteca, Lector, notas, búsqueda, Ajustes, paleta,
  menús y diálogos; incluyen los ocho tamaños de 640×480 a 3440×1440,
  claro/oscuro, alto contraste, movimiento reducido y texto 150/200 %.
- Diez comparaciones laterales 0.9/0.10 muestran el cambio real. La revisión
  manual mediante hojas de contacto no encontró desbordamientos; los detectados
  en la preview de Ajustes, en filas/preview de la paleta y en los filtros del
  panel de notas a 200 % se corrigieron antes de regenerarlas. El detalle
  compacto de búsqueda se revisó también a 200 % y su manifiesto registra que
  la acción de cierre permanece dentro del viewport.
- La golden 0.10 tiene el mismo hash SHA-256 que la 0.9 porque el contrato del
  lector no debía variar: `42fac2f97817ee07741b5c9b0bccbb714fc0cdcc2805689dd3fb19437cbcb007`.
- La marca permanece en 512×512, 293 272 bytes y SHA-256
  `fd8216f02844c51b621d40f01c6a5020ef12fcd90590a9340ace659de88dd8a3`;
  su blob de Git coincide con `HEAD`.
- El corpus incluye once PDF sintéticos, sin datos personales ni contenido con
  copyright. El smoke de ocho segundos abrió `normal.pdf` en la build Release y
  observó cero conexiones TCP/UDP.
- Los nueve binarios nativos tienen estado `NotSigned`. Son aptos para prueba
  local dentro del repositorio, no se presentan como distribución firmada.

La prueba manual pendiente es Narrator y la matriz física 100/125/150/175/200 %
en varios monitores. La firma Authenticode y la publicación siguen fuera de
alcance.
