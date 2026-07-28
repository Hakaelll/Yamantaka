# Contribuir

1. Mantén los PDF fuera del repositorio salvo fixtures pequeños y generados.
2. Explica cualquier dependencia nueva en `DECISIONS.md`.
3. Actualiza `NEXT_TODOS.md` y criterios de aceptación.
4. Ejecuta formato, análisis y pruebas antes de proponer cambios.
5. No registres datos de documentos ni añadas red, telemetría o cuentas.
6. Conserva los límites entre UI, dominio, repositorios y motor PDF.
7. Si se toca `third_party/pdfrx`, documenta el cambio en `PATCHES.md` y en un
   ADR; no mezcles cambios upstream no relacionados.

Formato:

```powershell
dart format lib test integration_test tool
flutter analyze
flutter test
```
