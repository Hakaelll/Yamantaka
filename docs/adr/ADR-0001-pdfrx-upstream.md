# ADR-0001: dependencia local de pdfrx hasta una release upstream equivalente

- Estado: aceptado temporalmente
- Alcance: Windows y lifecycle de PDFium

Yamantaka conserva `third_party/pdfrx` porque la corrección de ownership del
codec nativo todavía no tiene una release upstream verificada en el gate local.
El parche está descrito en `third_party/pdfrx/PATCHES.md` y mantiene su licencia.

La retirada solo se autoriza cuando una release upstream:

- contiene la corrección equivalente y no cambia la geometría del lector;
- pasa el corpus patológico, suite real de PDFium, matriz DPI, búsqueda,
  enlaces, cierre y comparación visual;
- tiene la misma licencia y binarios compatibles con el toolchain de Windows.

Hasta entonces YAM-106 permanece bloqueado por una dependencia externa. No se
debe eliminar el directorio local para “limpiar” el repositorio sin esa
evidencia.
