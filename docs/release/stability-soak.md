# Soak de estabilidad

`tool/soak_windows.ps1` ejecuta ciclos repetidos de apertura/cierre, búsqueda,
panel, zoom, importación, cancelación y limpieza. Cada snapshot conserva solo
contadores: jobs, lifecycle, memoria lógica, handles nativos y RSS. El gate
falla si un contador de recursos propios crece monótonamente o si el p95 cruza
el presupuesto del escenario.

Se requieren dos máquinas Windows, una GPU integrada y una dedicada cuando
estén disponibles, escalas 100/125/150/175/200%, tema claro/oscuro, lector
Documento/Libro y una pasada con Narrador. Los PDF reales deben estar
autorizados por el operador; el corpus sintético es la evidencia reproducible.
