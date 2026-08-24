# Rendimiento

## Render PDF 0.4.0

La ruta caliente instrumenta localmente `pdf_open`, preview, refinamiento,
región/tesela, aciertos/fallos, cancelaciones, resultados obsoletos, presión de
memoria y tiempos build/raster/total de frame. Cada serie conserva como máximo
256 duraciones y exporta conteo, p50, p95 y máximo; no contiene ruta, huella,
título, número de página, consulta ni texto.

Presupuestos actuales:

| Recurso | Presupuesto normal | Comportamiento bajo presión |
| --- | ---: | --- |
| Páginas completas | 72 MiB | LRU reduce límite y libera `ui.Image` |
| Regiones/teselas | 24 MiB | LRU independiente; prioridad visible |
| Texto plano/estructurado | 8 MiB | LRU independiente |
| Bitmap individual | 16.384 px/eje, 192 MiB | reduce escala antes de PDFium |
| Preview | 110 DPI | se conserva hasta que llega refinamiento |

La vista Continua localiza páginas próximas con búsqueda binaria y no recorre
las 5.000 geometrías en cada paint. La interacción activa pausa índice y
análisis de biblioteca; 350 ms después de terminar puede reanudarse el índice,
mientras el análisis permanece pausado hasta cerrar el lector. Las generaciones
impiden que zoom, rotación, resize o navegación rápidos publiquen resultados
antiguos.

La comparación de referencia usa como “antes” el perfil reproducible 0.3.0 de
la misma secuencia de 100 páginas documentado más abajo (apertura pesada p95
15,718 ms; render pesado p95 6,986 ms; build lector p95 5,570 ms). El informe
“después” se genera bajo
`build/performance/performance-profile.json`; solo se actualizarán cifras aquí
tras ejecutar Profile en el host Release y conservar el JSON junto al gate.
`tool/compare_pdf_profile.ps1` compara automáticamente esas métricas con un 20 %
de tolerancia; build de frame usa 35 % por la varianza del pequeño conjunto de
frames Profile y conserva además el presupuesto estricto de 16,7 ms en el test.
El script escribe un resultado agregado sin datos privados.

## Resultado Profile 0.4.0 — 2026-08-23

Host: Windows 11, Flutter Profile, 1440×900 físicos / 1152×720 lógicos y DPR
1,25. Cinco secuencias de interfaz, 15 ciclos PDFium tras warm-up, documento
comparable de 100 páginas y documento virtualizado de 5.000 páginas.

| Métrica | 0.3.0 p50 / p95 | 0.4.0 p50 / p95 |
| --- | ---: | ---: |
| Apertura vectorial pesada | 14,729 / 15,718 ms | 15,927 / 16,997 ms |
| Render pesado deduplicado/cacheado | 5,354 / 6,986 ms | 5,113 / 7,349 ms |
| Interacción de lector (100 páginas) | 83,997 / 87,589 ms | 74,253 / 81,270 ms |
| Build de frame del lector | 2,684 / 5,570 ms | 3,059 / 7,237 ms |

La comparación automática aprueba las cuatro métricas dentro de su tolerancia.
El build p95 contiene un outlier, pero se observaron cero frames por encima de
16,7 ms. El render p50 mejora; apertura p50 empeora ~1,2 ms por la identidad de
contenido y controles adicionales. No se ocultan esas compensaciones.

Mediciones nuevas sin equivalente 0.3.0:

- primera página visible: 86,044 ms (100 páginas) y 187,049 ms (5.000);
- cambio de disposición en 5.000 páginas: p50 82,833 ms, p95 83,434 ms;
- navegación continua distante: p50 15,977 ms, p95 17,785 ms;
- scroll continuo: 17,122 ms; resize p95: 17,852 ms;
- 45 solicitudes directas: 15 hits, 30 misses y 15 deduplicaciones;
- 26 previews, 6 cancelaciones y, tras cerrar, cero documentos, trabajos,
  imágenes o bytes de caché vivos.

RSS observado: 145,66 MiB de base, 274,34 MiB de pico y 264,28 MiB mínimo tras
volver. En 15 ciclos pesados el RSS tras cierre creció 3,57 MiB, por debajo del
umbral automatizado de 16 MiB. El allocator puede retener memoria, por lo que
se informa el dato sin presentarlo como prueba absoluta de ausencia de fugas.

## Instrumentación implementada

En builds de desarrollo, sin registrar rutas ni contenido privado:

- tiempo de bootstrap;
- tiempo de apertura y número de páginas;
- tiempo de render de miniatura/región;
- aciertos y fallos de la LRU;
- bytes estimados y páginas retenidas;
- tiempo, resultados y cancelación de búsqueda;
- cierre explícito de cada documento.

Las portadas se recortan por antigüedad al máximo configurable de 64–512 MiB.
Solicitudes idénticas se deduplican sin mezclar la cancelación de consumidores.
Al cerrar un documento, los renders activos se cancelan y esperan antes de
disponer el handle nativo. Limpiar caché libera memoria propia, archivos y
referencias SQLite.

## Línea base Release — 2026-07-29

Smoke único, no presentado como benchmark:

- equipo: Windows 11 Home 10.0.26200, Intel Core i7-1165G7 y 15,8 GiB de RAM;
- build: `flutter build windows --release`, paquete de 18 archivos y 39,0 MiB;
- entrada: PDF textual controlado de dos páginas y 864 bytes;
- ventana principal visible: 449 ms desde `Start-Process`;
- a los dos segundos: 148,6 MiB de working set y 135,5 MiB privados;
- proceso receptivo, título `Yamantaka`, cero conexiones TCP, `stderr` vacío y
  cierre limpio mediante la ventana.

La base SQLite local se actualizó al cerrar y no se copió el PDF original.
Estas cifras solo demuestran un arranque funcional medido en este equipo; no
son mediana, p95 ni evidencia suficiente para documentos largos.

## Smoke tras el rediseño visual — 2026-07-29

Una ejecución Release adicional, en el mismo equipo y sin presentarla como
benchmark:

- bundle: 18 archivos y 39,03 MiB, sin nuevas dependencias;
- ventana principal visible: 442 ms desde `Start-Process`;
- a los dos segundos: 114,1 MiB de working set y 114,2 MiB privados;
- proceso receptivo y cierre limpio mediante `CloseMainWindow`.

La medición no muestra una regresión frente al smoke inicial, pero una sola
ejecución y un fixture mínimo no permiten atribuir la diferencia de memoria al
rediseño ni sustituir la metodología de endurecimiento.

## Presupuesto de interfaz ligera — 2026-07-29

La tanda YAM-025 a YAM-029 se validó con estos límites estructurales:

- filtro y tres órdenes sobre los 40 recientes ya residentes, sin consultas por
  tecla, índice secundario ni persistencia del texto;
- portadas decodificadas al tamaño físico de su caja (`ancho/alto × DPR`) con
  `FilterQuality.low` y aislamiento de repintado;
- listas conservadas como slivers y densidad mediante tokens constantes;
- movimiento reducido usa salto inmediato en la paleta y duración cero en
  cambios de tema;
- `pubspec.yaml` y `pubspec.lock` no cambian: cero dependencias nuevas;
- bundle Release final: 19 archivos, 41.095.311 bytes (39,19 MiB).

Formato, análisis, 66 pruebas, integración Windows, perfil y build Release
aprobaron. Las capturas nativas ancha/compacta no muestran overflow.

## Perfil reproducible de interfaz — 2026-07-29

Procedimiento:

- Windows 11 Home `10.0.26200`, Intel64 Family 6 Model 140, 8 procesadores
  lógicos; Flutter 3.44.8 y Dart 3.12.2;
- build Profile; ventana 1440×900 física y 1152×720 lógica a escala nativa
  125 %;
- cinco pasadas dentro del mismo proceso por escenario;
- biblioteca con 40 PDF reales y portadas pequeñas controladas;
- documento textual de 100 páginas;
- documento vectorial pesado de 8 páginas y 1200 formas por página;
- filtro rápido, estado, densidad, panel, resize, zoom, navegación, cierre y
  retorno a biblioteca.

| Secuencia medida | Mediana | p95 |
| --- | ---: | ---: |
| Biblioteca: texto + estado + densidad | 123,420 ms | 135,266 ms |
| Lector: panel + resize + zoom + página | 83,997 ms | 87,589 ms |
| Build de frame en biblioteca | 3,294 ms | 6,360 ms |
| Build de frame en lector | 2,684 ms | 5,570 ms |
| Apertura del PDF vectorial pesado | 14,729 ms | 15,718 ms |
| Render pesado con deduplicación/caché | 5,354 ms | 6,986 ms |

Las diez pasadas acumularon cero frames fuera del presupuesto de build y cero
fuera del presupuesto de raster. El raster reporta tiempos cercanos a cero en
este runner, por lo que no se interpreta como una medida de GPU independiente.

Diagnóstico del motor durante cinco aperturas pesadas:

- 15 solicitudes de render;
- 5 misses, 5 solicitudes simultáneas deduplicadas y 5 hits;
- 0 renders cancelados en este escenario concreto;
- tras cerrar: 0 páginas/bytes pendientes en la caché, 0 renders pendientes y
  0 documentos abiertos.

Una prueba PDFium separada cancela el token antes de renderizar y confirma un
contador de cancelación, cero trabajos pendientes y cierre limpio.

RSS del proceso:

- línea base con biblioteca: 132,7 MiB;
- pico observado: 283,3 MiB;
- muestras tras cerrar/limpiar/volver: 280,2, 280,2, 280,2, 280,2 y 280,3 MiB;
- mínimo tras retorno: 280,2 MiB, 3,1 MiB por debajo del pico.

El cierre libera recursos propios y handles; el RSS no vuelve a la línea base
porque el runtime y el asignador de Windows pueden retener páginas. Esto no se
presenta como prueba de ausencia de fugas a largo plazo. El informe íntegro,
sin rutas ni títulos, queda en
`build/performance/performance-profile.json` y se regenera con:

```powershell
dart run tool/create_performance_fixtures.dart build/performance/fixtures
flutter drive --driver=test_driver/performance_driver.dart `
  --target=integration_test/performance_profile_test.dart `
  -d windows --profile
```

## Metodología de endurecimiento restante

En un equipo Windows medio:

1. Build `profile` y `release`.
2. Tres procesos en frío adicionales, no solo cinco ciclos internos.
3. PDF controlados de 1000 páginas y páginas raster extremas.
4. Perfil de búsqueda completa/cancelada y navegación sostenida.
5. Repetir a 100 %, 150 % y 200 % de escala nativa.
6. Repetir en otra GPU/equipo antes de fijar presupuestos de regresión.

DevTools valida memoria, CPU y frames. Ninguna cifra se publica sin fecha,
hardware, modo de build y documento.

## Estado real

PDFium abre, renderiza, extrae, busca y cierra fixtures reales; YAM-011/YAM-034
disponen ya de una distribución Profile reproducible. YAM-015 conserva la
validación manual y multi-equipo/DPI, no cubierta por una única máquina.

## Limitación conocida

Las regiones visibles ya tienen caché y prioridad separadas, pero el tamaño
real de memoria de PDFium/Skia no equivale exactamente a `width × height × 4`.
Por eso el límite preventivo, el governor, el diagnóstico y el soak se mantienen
como defensas complementarias. El RSS retenido por el runtime tras cerrar no se
presenta por sí solo como fuga ni como prueba de ausencia de ella.

## Perfil de tercera iteración — 2026-07-29

Se repitió el escenario Profile de cinco pasadas después de integrar
anotaciones, contextos documentales suspendibles, organización, búsqueda y
diagnóstico:

| Secuencia | Mediana | p95 |
| --- | ---: | ---: |
| Interacción de biblioteca | 127,831 ms | 137,071 ms |
| Interacción de lector | 80,934 ms | 88,532 ms |
| Build de frame en biblioteca | 3,214 ms | 6,643 ms |
| Build de frame en lector | 2,651 ms | 4,646 ms |
| Apertura vectorial pesada | 14,690 ms | 15,511 ms |
| Render pesado deduplicado/cacheado | 5,528 ms | 6,912 ms |

Hubo cero frames de build/raster fuera de presupuesto. El motor terminó con
0 documentos abiertos, 0 renders pendientes y 0 bytes/páginas cacheados; las
15 solicitudes produjeron 5 misses, 5 deduplicaciones y 5 hits. RSS pasó de
136,6 MiB de base a 280,8 MiB de pico y liberó 41,2 MiB al volver, con mínimo
de 239,6 MiB. El runtime de Windows puede retener memoria; esto no prueba
ausencia de fugas sostenidas.

Un benchmark SQLite/Dart independiente ejecutó tres procesos, cada uno con 500
documentos, 100 anotaciones, 10 colecciones, 5 tabs y 7 pasadas:

| Métrica | Medianas observadas | p95 máximo |
| --- | --- | ---: |
| FTS global | 0,600 / 0,818 / 0,798 ms | 6,009 ms |
| Consulta de colección | 0,507 / 0,432 / 0,425 ms | 1,250 ms |
| Restauración de workspace | 0,778 / 0,870 / 0,892 ms | 1,969 ms |

Las tres bases midieron 659.456 bytes, terminaron con integridad `ok` y RSS
final inferior al baseline observado. Los informes viven bajo `build/performance`
y declaran `contains_paths_or_titles: false`. Son evidencia local reproducible,
no un umbral universal; falta otro hardware/GPU y corpus real grande.

El gate Release final produjo 19 archivos y 41.504.911 bytes (39,58 MiB), sin
cambios en dependencias. Un smoke oculto creó la ventana en 588 ms, midió
117,0 MiB de working set y 118,5 MiB privados, mantuvo cero conexiones TCP y
cerró limpiamente mediante `CloseMainWindow`. Es una observación única, no una
distribución.

## Biblioteca de artefactos — 2026-07-29

- El registro de importación termina antes del análisis de portada; una cola
  secuencial limita PDFium a un trabajo adicional y permite cancelar al retirar.
- Solo se analizan seis páginas a escala 0,16; el resultado final guarda escalas
  0,24 y 0,56. No se carga el PDF completo ni se hace OCR.
- La UI usa `SliverGrid.builder` desde siete objetos, `cacheWidth` por DPR y
  calidad baja salvo selección. Jitter/layout son cálculo puro O(n) sin IO.
- Búsqueda, estado, favorito y orden se resuelven en una pasada sobre los
  metadatos ya residentes. Hover queda encapsulado en cada artefacto.
- La prueba estructural simula 100 documentos, valida separación y navegación;
  falta un perfil Windows sostenido con cientos de PDF/portadas reales antes de
  fijar un umbral universal.
