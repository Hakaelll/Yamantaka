# Renderizado PDF 0.4

## Flujo

```text
viewport/zoom/rotación
        |
        v
generación + prioridad ---- cancelación por consumidor
        |
        v
JobCoordinator (deduplica la misma clave)
        |
        v
PDFium: preview 110 DPI -> región nítida tras 75 ms
        |                         |
        v                         v
LRU página 72 MiB          LRU región 24 MiB
        \_________________________/
                    |
                    v
          paint + métricas agregadas
```

Texto plano y estructurado usa otra LRU de 8 MiB. Miniaturas persistidas viven
en la caché de portadas y no comparten ownership con bitmaps del lector.

## Invariantes de ciclo de vida

1. Una clave incluye identidad de contenido, página, rotación, escala física,
   DPI, región y generación; una ruta no identifica el contenido.
2. Cada consumidor recibe su propio `JobHandle`. PDFium solo se cancela cuando
   ya no queda ningún consumidor de la clave deduplicada.
3. Una generación menor que la activa falla antes del render. Si cambia durante
   PDFium, la imagen se dispone y nunca entra en caché.
4. La LRU es la única propietaria de una imagen cacheada. Desalojo, limpieza y
   cierre la disponen exactamente una vez.
5. Cerrar un documento cancela tokens y handles, espera su finalización, vacía
   las cuatro cachés asociadas y solo entonces dispone el documento nativo.
   El motor serializa los renders de cada documento; documentos distintos sí
   pueden avanzar en paralelo. Ningún puntero `FPDF_DOCUMENT` cruza isolates ni
   se comparte explícitamente entre hilos de Yamantaka.
6. Escala y DPR se validan antes de calcular bitmap; ningún eje supera 16.384 px
   ni una solicitud supera 192 MiB estimados.
7. El diagnóstico contiene nombres de métrica y agregados, nunca rutas, títulos,
   páginas, búsquedas o contenido.

## Scheduling y vista continua

Prioridad estricta: visible, próxima, prefetch y miniatura. En Continua las
geometrías están ordenadas verticalmente; dos búsquedas binarias delimitan el
anillo que cruza viewport más cache extent. Solo ese rango participa en paint y
render, y el trabajo activo exterior se cancela. El bitmap previo permanece
visible mientras se refina para evitar blanco/parpadeo.

El prefetch se reduce ante presión de memoria y la interacción del lector pausa
índice/enriquecimiento. Cambiar de zoom, rotación, disposición, viewport o
documento debe iniciar una generación nueva en cualquier consumidor directo del
motor.

## Validación

- unitarias: LRU/evicción, deduplicación, cancelación, generaciones, límites y
  redacción;
- PDFium real: abrir, renderizar, buscar, contraseña, corrupción y cierre;
- widgets: layouts, resize, zoom, navegación, movimiento reducido y golden;
- Profile Windows: corpus 1/50/500/5.000 páginas, vector/raster extremos,
  p50/p95, frames >16,7 ms, RSS pico y memoria tras ciclos abrir/cerrar;
- Release Windows: analyze, test, build, smoke local sin red, manifest y SHA-256.
