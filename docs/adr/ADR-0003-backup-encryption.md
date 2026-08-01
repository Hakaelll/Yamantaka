# ADR-0003: backup autenticado opcional

- Estado: diferido
- Decisión: la versión actual exporta JSON local sin PDF y sin cifrado propio.

No se inventa criptografía ni se presenta una contraseña como cifrado. Si se
adopta cifrado, debe usar una primitive autenticada mantenida, versionar el
algoritmo, validar el tag antes de parsear tablas y conservar el backup previo
si falla cualquier paso. Hasta esa auditoría el manifest declara
`containsPdfFiles: false` y el usuario puede inspeccionar las categorías.
