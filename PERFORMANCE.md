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

## Metodología de endurecimiento pendiente

En un equipo Windows medio:

1. Build `profile` y `release`.
2. Tres ejecuciones en frío y cinco en caliente.
3. PDF controlados de 10, 100 y 1000 páginas, de texto e imagen.
4. Registrar mediana y p95 de ventana, apertura, primera página y navegación.
5. Recorrer 100 páginas y medir jank, pico de memoria y memoria tras cerrar.
6. Medir búsquedas completas/canceladas y aciertos de caché.
7. Repetir a 100 %, 125 % y 200 % de escala.

DevTools valida memoria, CPU y frames. Ninguna cifra se publica sin fecha,
hardware, modo de build y documento.

## Estado real

La prueba automatizada confirma que PDFium puede abrir, renderizar un bitmap y
una `ui.Image`, pintar la primera página, extraer y buscar el fixture. El build
Release y la línea base anterior están verificados. Eso no sustituye métricas
de producto: mediana/p95, fluidez, pico de memoria y cierre con un PDF largo
siguen pendientes en YAM-011/YAM-015.

## Limitación conocida

El primer hito renderiza páginas completas virtualizadas. El contrato acepta
regiones para poder añadir teselas si las mediciones de documentos extremos lo
justifican; todavía no se implementan ni se anuncian como disponibles.
