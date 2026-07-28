# Changelog

## 0.1.0-dev — 2026-07-28

- Inicialización Flutter para Windows, Android e iOS.
- Arquitectura, diseño, privacidad, decisiones, roadmap y plan de pruebas.
- Motor `pdfrx`/PDFium aislado para apertura, render, texto, búsqueda, índice,
  enlaces y contraseña.
- Biblioteca SQLite, recientes, ausentes, progreso, sesiones, preferencias,
  portadas e invalidación por huella.
- Visor continuo/único/enfrentado, modo libro, LTR/RTL, seis temas, navegación,
  zoom, rotación, pantalla completa y restauración de posición.
- Búsqueda cancelable, panel lateral, marcadores, paleta, atajos y ajustes.
- Márgenes por documento, restauración de geometría de ventana y cancelación
  activa de render al cerrar.
- LRU por bytes, límite de portadas y logging de rendimiento sin datos privados.
- 34 pruebas estándar aprobadas, incluidas regresiones PDFium, errores tipados
  y contraseña AES-128 con archivos legales generados.
- Integración Windows aprobada con render visible, zoom, avance/retroceso,
  búsqueda, marcador y restauración.
- Build Release Windows aprobado y ejecutado con un PDF mediante argumento.
- Línea base Release registrada con ventana receptiva, memoria, persistencia
  local, cero conexiones TCP y cierre limpio.
- Corregidos un ciclo autorreferente en la deduplicación de renders y el ciclo
  de vida del codec de imagen de pdfrx 2.4.7 en Windows.
