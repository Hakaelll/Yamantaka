# Rendimiento

## Instrumentación implementada

En builds de desarrollo, sin registrar rutas ni contenido privado:

- tiempo de bootstrap;
- tiempo de apertura y número de páginas;
- tiempo de render de miniatura/región;
- aciertos y fallos de la LRU;
- bytes estimados y páginas retenidas;
- tiempo, resultados y cancelación de búsqueda;
- cierre explícito de cada documento.

La caché de imágenes propias está limitada a 96 MiB por bytes estimados. El
visor virtualizado conserva solo páginas visibles/próximas con otro límite de
96 MiB. Las portadas se recortan por antigüedad al máximo configurable de
64–512 MiB. Solicitudes idénticas se deduplican. Al cerrar un documento, los
renders activos se cancelan mediante tokens y se esperan antes de disponer el
handle nativo. Limpiar caché libera memoria propia, archivos y referencias
SQLite.

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

El primer hito renderiza páginas completas virtualizadas. El contrato acepta
regiones para poder añadir teselas si las mediciones de documentos extremos lo
justifican; todavía no se implementan ni se anuncian como disponibles.
