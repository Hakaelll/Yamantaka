# ADR-0002: impresión delegada al documento original

- Estado: preparado para implementación nativa
- Decisión: Windows debe mostrar el diálogo de impresión del documento original
  y no generar un PDF temporal silencioso.

La primera integración usará una API/dependencia mantenida solo después de
confirmar soporte de rango, orientación, cancelación y errores. Si Windows
exige un temporal, el nombre será aleatorio dentro de la carpeta temporal del
usuario, se borrará en éxito, cancelación y excepción, y nunca se incluirá en
logs. La función no se habilita por defecto hasta que `verify_windows.ps1`
registre el binario y el smoke de cancelación.
