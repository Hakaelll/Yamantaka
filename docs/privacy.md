# Privacidad local

Yamantaka 0.12 no requiere cuenta ni red y no incorpora telemetría remota. Los
PDF originales nunca se suben, copian al backup, reescriben ni modifican por una
regla. Las automatizaciones no aceptan scripts, comandos ni URLs.

SQLite guarda rutas y metadatos necesarios para volver a abrir documentos. Esa
información permanece en el dispositivo. Los informes exportables y el journal
seguro excluyen rutas, títulos, consultas, texto del PDF, texto seleccionado y
contenido de anotaciones. Solo incluyen nombres de métricas, cantidades, estados
y códigos constantes.

Markdown/JSON de anotaciones y backups se crean únicamente mediante una acción
explícita y en la ruta elegida por el usuario. Borrar diagnósticos o datos desde
Ajustes no borra PDF. Las conexiones TCP/UDP se comprueban durante el smoke de
ocho segundos del bundle local.
