# Checklist de release candidate

Una RC solo se puede marcar como lista cuando cada casilla tiene un artefacto
fechado. El comando principal es `tool/verify_windows.ps1` desde un checkout
limpio.

- [ ] G1: formato, dependencias, análisis y tests.
- [ ] G2: corpus sintético con hashes y cero documentos vivos tras cierre.
- [ ] G3: matriz de 1366×768 a 3840×2160 y escalas 100–200%.
- [ ] G4: regresión visual con manifiesto, tolerancia y revisión aprobada.
- [ ] G5: rendimiento frío/caliente, p95 y segundo equipo.
- [ ] G6: backup/restore, integrity check, safe mode y smoke de cero red.
- [ ] G7: build firmado, asociación PDF, instancia única, instalación,
  actualización, reparación, portable y desinstalación.

Los artefactos viven en `build/verification` y no se deben copiar al
repositorio. Una baseline visual solo se actualiza con
`-ApproveBaselineUpdate`, revisión humana y una explicación en el changelog.
