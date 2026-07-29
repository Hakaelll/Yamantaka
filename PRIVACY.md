# Privacidad

Yamantaka es local-first:

- Los documentos no se suben a Internet ni se copian a la biblioteca interna.
- No se requiere cuenta.
- Biblioteca, marcadores, progreso, sesiones y preferencias son locales.
- No existe telemetría por defecto, publicidad ni venta o intercambio de datos.
- Los logs no incluyen texto, selección, consultas, títulos, nombres, rutas
  locales, marcadores, contraseñas ni contenido PDF.
- La aplicación no hace comprobaciones remotas silenciosas.
- El usuario puede limpiar caché e historial y eliminar todos los datos internos.

El filtro textual de biblioteca y la consulta de la paleta existen solo en
memoria mientras se usan. Se pueden persistir el filtro de estado, atajos y
contexto de recuperación, pero no los términos buscados.

## Historial y recuperación

Las sesiones de lectura contienen documento local, inicio/fin, páginas y
segundos activos. No se derivan hábitos, rachas, perfiles ni métricas
personales. El historial puede desactivarse, limitarse, borrar una entrada o
limpiarse entero.

La recuperación opt-in guarda la última ruta y estado de interfaz. “Iniciar
siempre en la biblioteca” lo desactiva de forma efectiva. Si la ruta falta o no
abre, se elimina para impedir intentos repetidos. Ningún PDF permanece abierto
entre procesos.

## Copias locales

La exportación JSON es manual, versionada y no usa red. Incluye, cuando existen:

- rutas y metadatos de biblioteca;
- progreso y preferencias;
- marcadores e historial;
- ajustes internos.

Declara explícitamente que no contiene archivos PDF ni sus bytes. Aun así,
rutas, títulos e historial pueden ser sensibles: el archivo exportado debe
protegerse como los datos originales. Antes de importar se valida y muestra un
resumen; la aplicación crea una copia local previa en su directorio de datos.
Una importación incompatible no sustituye silenciosamente nada.

Abrir ubicación y copiar ruta requieren una acción explícita. Retirar un
documento elimina sus datos locales y declara que el PDF original no se borra.
Relocalizar exige que la huella coincida antes de asociar una ruta nueva.

Un PDF puede contener enlaces. Los destinos internos no salen del documento.
Para `http`, `https` o `mailto`, Yamantaka muestra la dirección y requiere una
confirmación; solo después entrega el enlace a la aplicación predeterminada del
sistema. Esa acción explícita puede usar red fuera de Yamantaka. Otros esquemas
se bloquean.

No existe sincronización. Cualquier propuesta futura requerirá una nueva
decisión explícita; no forma parte de las copias locales actuales.

## Anotaciones, organización e índice

Anotaciones, etiquetas, colecciones, workspaces y perfiles se guardan solo en
SQLite local. Pueden revelar hábitos, proyectos y fragmentos del documento:
limpiar “historial” no los elimina; “borrar todos los datos” sí. El backup JSON
v2 y las exportaciones manuales de anotaciones contienen datos sensibles y
deben protegerse como el PDF, aunque nunca contienen sus bytes.

El índice FTS es opcional y reconstruible. Solo se activa desde la búsqueda
global de la sesión, procesa un archivo por vez, no hace OCR y puede
pausarse/cancelarse/borrarse. No se incluye en backup. Las consultas no se
persisten, registran ni envían; los snippets solo aparecen en pantalla.

Importar una carpeta es siempre una acción explícita con preview y límite; no
instala watcher ni escanea en segundo plano. Recuperar documentos solo
autoasocia una huella exacta. Candidatos ambiguos o de baja confianza no mutan
datos.

## Diagnóstico y modo seguro

El informe diagnóstico usa campos permitidos: versión, sistema, arquitectura,
esquema, integridad, tamaños/conteos, caché y contadores del motor. Enmascara la
ruta de datos y excluye nombres, títulos, rutas de documentos, consultas y
texto de anotaciones. Copiar/exportar y abrir la carpeta requieren acción
explícita.

Safe mode registra únicamente un contador de arranques no limpios. Al tercer
caso evita restauración y preferencias personalizadas; no transmite ni borra
datos. La aplicación no crea conexiones TCP en los flujos medidos.
