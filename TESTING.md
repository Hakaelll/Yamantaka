# Pruebas

## Estado verificado — 2026-07-29

- `flutter analyze`: aprobado sin incidencias.
- `flutter test --reporter expanded`: 34 pruebas aprobadas.
- `flutter test integration_test -d windows`: aprobado.
- `flutter build windows --release`: aprobado; genera
  `build\windows\x64\runner\Release\yamantaka.exe`.
- `flutter doctor -v`: toolchain Windows aprobado con Build Tools 2022 17.14.37
  y Windows SDK 10.0.26100.0.
- Smoke Release: ventana visible y receptiva, `stderr` vacío, cero conexiones
  TCP, persistencia SQLite local y cierre limpio.

No hay pruebas omitidas marcadas como aprobadas.

## Comandos

```powershell
dart format --output=none --set-exit-if-changed lib test integration_test tool
flutter analyze
flutter test --reporter expanded
flutter test integration_test -d windows
flutter build windows --release
```

La captura reproducible se regenera con:

```powershell
flutter test integration_test -d windows `
  --dart-define=YAMANTAKA_CAPTURE_PATH=build/manual/integration-reader.png
```

## Cobertura actual

- Dominio: progreso, límites, pares enfrentados, portada, impar y RTL.
- Plataforma: normalización de rutas y argumento `.pdf`.
- Errores: extensión no compatible, archivo ausente, PDF vacío, PDF corrupto,
  contraseña requerida y contraseña errónea se convierten en fallos tipados.
- Persistencia: esquema SQLite estricto, migración inicial, recientes sin
  duplicados, preferencias, marcadores, sesiones, huella/relocalización,
  invalidación de portada y ajustes globales.
- Caché: expulsión LRU por presupuesto de bytes y recorte de portadas por
  límite global.
- Búsqueda: resultados, navegación, documento sin texto y cancelación inmediata.
- Comandos: registro, atajos y filtrado/ejecución de la paleta.
- Widgets: biblioteca vacía, ausente, lector, barra, navegación, progreso,
  panel, seis temas, márgenes, actividad de teclado y error recuperable.
- Motor real: PDF legal generado durante el test, apertura PDFium, bitmap con
  contenido visible, extracción, búsqueda en dos páginas y flujo AES-128 con
  contraseña ausente, errónea y correcta.
- Integración Windows: `ui.Image` con lectura de píxeles, pintura del visor en
  `RepaintBoundary`, portada individual, zoom, avance/retroceso, búsqueda,
  marcador, cierre, reapertura y restauración.

La prueba estándar valida el bitmap crudo de PDFium. La conversión a `ui.Image`
necesita un rasterizador nativo vivo y por eso pertenece a la integración
Windows.

## Matriz Windows

Completado entre pruebas automáticas y smoke del hito:

- apertura por argumento con un PDF textual real generado;
- render nativo y pintura de la portada;
- vista enfrentada con portada sola, zoom y navegación;
- extracción, búsqueda, marcador, cierre, reapertura y restauración;
- errores de PDF corrupto/vacío y los tres casos de contraseña;
- build Release, proceso visible y ventana receptiva, persistencia local,
  ausencia de conexiones TCP y cierre limpio.

Pendiente para endurecimiento posterior:

1. Abrir mediante selector y drop.
2. Probar PDF escaneado, grande, Unicode, ruta larga y variantes patológicas
   adicionales.
3. Verificar continuo, único, enfrentado, portada, impar, LTR/RTL, zoom,
   rotación, F11, ocultación y clic lateral.
4. Buscar texto largo, cancelar y probar documento sin texto.
5. Crear/eliminar/etiquetar marcador; cerrar y reabrir.
6. Mover el PDF, observar ausente y relocalizarlo.
7. Probar teclado completo, tabulación y lector de pantalla básico.
8. Probar escalado de Windows 100 %, 125 %, 200 % y ventanas de 640–1600 px.
9. Limpiar caché/historial/datos y comprobar que el PDF original permanece.
10. Cerrar durante búsqueda/render y revisar liberación de memoria.

Los resultados restantes se registrarán con fecha, build, equipo y PDF de
prueba. Ningún caso pendiente se presenta como aprobado.
