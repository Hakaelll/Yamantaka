# Baseline Flutter — 2026-10-08

Verificación de la implementación Flutter existente, sin editar sus fuentes ni actualizar sus dependencias. No se abrió la aplicación Flutter ni se ejecutaron los scripts de empaquetado o los tests de integración con una ventana de aplicación.

## Entorno y preservación

- Ejecutable: `build/tools/flutter/sdk/flutter/bin/flutter.bat`.
- Flutter **3.44.8**, canal stable, framework `058e0af2c2b57e369d905a03ac9748b0ebf543c6`.
- Dart **3.12.2**; DevTools **2.57.0**; engine `0cd610717b`.
- Aplicación: `yamantaka 0.15.0+16`; restricción Dart `^3.12.2` y Flutter del lockfile `>=3.44.0`.
- `pdfrx` permanece como dependencia local `third_party/pdfrx`, versión `2.4.7+yamantaka.2`.
- SHA-256 de `pubspec.lock` antes: `9D7FD2A0DFBE1C3FF56F21EC14B16E873395EF5448C22BD589737627280C60D1`.
- SHA-256 de `pubspec.lock` después: `9D7FD2A0DFBE1C3FF56F21EC14B16E873395EF5448C22BD589737627280C60D1` (**idéntico**).
- El diff inicial de `lib`, `test`, `integration_test`, `third_party`, `pubspec.yaml`, `pubspec.lock` y `windows` estaba vacío.
- El diff final de esas mismas rutas sigue vacío; `git status --short` tampoco muestra cambios en ellas.

Los comandos usan `FLUTTER_SUPPRESS_ANALYTICS=true` y `DART_SUPPRESS_ANALYTICS=true`. `APPDATA` y `LOCALAPPDATA` se dirigen al directorio nuevo `build/validation/flutter-baseline-2026-10-08/tool-state`. Para los tests, `TEMP` y `TMP` también se dirigen a `build/validation/flutter-baseline-2026-10-08/temp`. La caché Pub preexistente se conserva en su ubicación habitual.

## Resultados

| Comprobación | Resultado |
| --- | --- |
| `flutter --version` | Correcto, salida 0 con las versiones anteriores. |
| `flutter pub get --offline --enforce-lockfile` | Las dependencias se resolvieron y se generó `package_config.json`; salida 1 posterior porque Windows no tiene soporte de symlinks habilitado para preparar plugins. |
| `flutter analyze --no-pub lib test integration_test tool third_party/pdfrx/lib` | Correcto, salida 0: `No issues found!`, 17.6 segundos. |
| `flutter test --no-pub --concurrency=1 --reporter=expanded test` | Correcto, salida 0: **330 tests aprobados**, `All tests passed!`, 1 minuto 39 segundos según el runner. |

La suite incluye pruebas reales de PDFium con PDFs generados: apertura, renderizado de bitmap, extracción y búsqueda de texto, errores de PDF corrupto, cancelación y validación de contraseña. También se ejecutaron las pruebas de persistencia, biblioteca, anotaciones, accesibilidad y widgets existentes. No se actualizaron goldens ni se generó una matriz visual adicional mediante los parámetros opcionales de captura.

Todas las carpetas de paquetes hosted fijadas por el lockfile estaban presentes en la caché. La resolución de dependencias de la aplicación se ejecutó con `--offline` y `--enforce-lockfile`; no se pidió actualizar versiones. El mensaje de symlinks es una limitación de preparación del entorno Windows, no una dependencia ausente. No se modificó Developer Mode.

La ejecución directa de Dart dentro del sandbox falló con `_Platform.resolvedExecutable` y con un error de canonicalización del snapshot (error del sistema 5). Las comprobaciones posteriores se ejecutaron mediante el escalado ordinario aprobado. El primer arranque del SDK reconstruyó su herramienta y resolvió las dependencias internas de esa herramienta; esto es distinto de las dependencias de la aplicación, que se verificaron offline.

La invocación de `flutter analyze --no-pub` desde la raíz también examinaba el SDK descargado bajo `build/tools/flutter/sdk/flutter`, incluyendo ejemplos sin dependencias resueltas. Se interrumpió esa invocación y se analizó explícitamente el código de la aplicación, sus tests, scripts Dart y el código de `pdfrx` local. No se editaron las opciones del analizador ni se corrigieron fuentes Dart.

## Evidencia

Los logs nuevos están en `build/validation/flutter-baseline-2026-10-08/`:

- `version.log`: fallo del primer arranque en el sandbox, interrumpido tras sus reintentos.
- `version-escalated.log`: versión confirmada y bootstrap de la herramienta SDK.
- `pub-get.log`: error de canonicalización dentro del sandbox.
- `pub-get-escalated.log`: resolución offline y limitación de symlinks.
- `analyze-verbose.log`: evidencia del alcance excesivo de la invocación raíz, interrumpida.
- `analyze-scoped.log`: análisis correcto del código de aplicación.
- `test.log`: ejecución secuencial de la suite normal `test/`.
- `lock-before.sha256` y `lock-after.sha256`: hashes del lockfile.
- `legacy-before.diff` y `legacy-after.diff`: comprobaciones del código Flutter preservado.

SHA-256 de `test.log`: `9F708CB229BEFC0D28245B7C456CA242969B76DA2A0699C125C8AAC40D3EF5B9`.

SHA-256 de `analyze-scoped.log`: `071281EB6503B30F85E01478206EFBAB90930EEB7533EE364BD13C7BCBD0DA8D`.

La suite normal no equivale a la validación de una compilación release Windows ni a pruebas de integración con la aplicación abierta.
