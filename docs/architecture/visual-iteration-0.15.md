# Pulido de producto 0.15

## Auditoría y dirección — 5 de septiembre de 2026

La referencia es el árbol de trabajo 0.14.0+15, incluyendo sus cambios todavía
sin commit. Las capturas originales permanecen en `build/visual-evidence/v0.14.0`.

Problemas observados y decisiones:

1. La Biblioteca fuerza un negro casi uniforme: adoptar el tema claro/oscuro
   global, con papel cálido y grafito y superficies claramente diferenciadas.
2. Continuidad repite la acción de retomar y muestra cinco contadores vacíos:
   concentrar la lectura activa y mostrar contexto únicamente cuando aporta valor.
3. En compacto el encabezado ocupa casi toda la ventana: limitar bloques de
   contexto y reservar altura para documentos, manteniendo acceso a lo reciente.
4. La búsqueda se descubre por un icono: ofrecer un acceso etiquetado y estable.
5. Colecciones y espacios están enterrados en menús: navegación lateral en ancho
   de escritorio, reutilizando los filtros y callbacks existentes.
6. Portadas sin imagen son rectángulos negros: composición tipográfica legible
   con título y autor, sin generar ni modificar el PDF.
7. La cuadrícula aplica jitter ornamental: alineación estable y selección precisa.
8. El lector y los paneles carecen de separación tonal: pulir chrome, grupos y
   superficies preservando geometría y ownership del visor.
9. Ajustes presenta una miniatura de vista previa poco útil: ampliar su muestra y
   ordenar la sección en un ancho de lectura con jerarquía y controles consistentes.
10. Duplicados expone nombres internos y booleanos: traducir la evidencia a
    explicaciones humanas e identificar claramente los registros y conflictos.
11. Menús y estados secundarios tienen contraste desigual: reforzar tipografía,
    estados disabled y foco sin depender exclusivamente del color.

Paleta: papel cálido `#F7F6F2`, blanco `#FFFFFF`, grafito `#181A20`, superficies
oscuras `#20232B`, violeta `#7055B5` / `#C4AFF4`. El logo se conserva byte a byte.
Las comprobaciones automatizadas y la revisión de PNG se registrarán al finalizar;
las capturas nativas defectuosas de la referencia no cuentan como prueba visual.

## Implementación y revisión transversal — 6 de septiembre de 2026

El checkpoint recuperable temprano, incluidos archivos nuevos, es
`build/checkpoints/pre-polish-20260905-020053.zip`, SHA-256
`fcc42e0e65a6540f0b6fc220edb6d7495d820d31dbb206fe6edbb4434ac4d5ea`.
Declara 0.14.0+15, pero al reconstruirlo el 7 de septiembre se comprobó que ya
contiene parte del nuevo rail y la paleta editorial, además de un orden de
directivas Dart que impide compilar. No se utiliza como fuente limpia de 0.14
ni para recalcular favorablemente la referencia de rendimiento. Se conserva
intacto como checkpoint temprano. La iteración mantiene la secuencia 0.15.0+16.

| Superficie | Defecto y consecuencia | Decisión implementada | Evidencia requerida |
| --- | --- | --- | --- |
| Biblioteca | Negro forzado desconectado de Ajustes | Tema global compartido, papel cálido y grafito | Comparación `library-expanded` y matriz clara/oscura |
| Continuidad | Franjas y acciones repetidas restaban altura al catálogo | Una zona de lectura activa; recientes en menú secundario; ocultar contadores vacíos | `library-compact` a 640×480 y `library-many-1280x720` |
| Organización | Filtros y búsqueda poco descubribles | Rail desde 1100 px y búsqueda etiquetada; conservar menús compactos | Pruebas SQL de navegación y reinicio de todos los ámbitos |
| Documentos | Portadas vacías y retícula desplazada | Sustitución con título/autor; alturas estables sin jitter; títulos de dos líneas | Cuadrícula, lista, ausentes, selección y texto 200 % |
| Lector | Controles ligados al color del PDF | Tema global independiente del lienzo; tinta electrónica conserva su excepción | Prueba de tema global con lienzo opuesto |
| Navegador | Todas las filas decían «Número de página» | Página numerada, tipografía por rol, atajo con nombre de tecla y detalle con scroll | Golden 0.15 y matriz del navegador |
| Anotaciones | Filtros secundarios saturaban la cabecera ampliada | Tipo/color desplegables y cabecera desplazable con altura limitada | 640×480 al 200 % y panel acoplado |
| Ajustes | Miniatura poco útil y separación excesiva en ultrapanorámico | Vista previa ampliada y área de trabajo limitada a 1600 px | Matriz de ocho resoluciones y texto ampliado |
| Duplicados | Booleanos y nombres internos; decisiones compartidas entre candidatos | Evidencias humanas, rutas seleccionables y elecciones por pareja | Comparación de duplicados y prueba de aislamiento |
| Componentes | Foco, marca de selección y controles pequeños | Estados compartidos, check legible y dock accionable de 40 px | Contrastes, estados de botones y pruebas de geometría |

Se inspeccionaron PNG a tamaño real de Biblioteca a 1280×720 y 640×480,
Biblioteca al 200 %, lector activo, panel de anotaciones ampliado y oscuro,
navegador y duplicados al 200 %. Las hojas permiten revisar estados vecinos,
pero no sustituyen esas inspecciones individuales.

La revisión encontró y corrigió títulos desalineados, una miniatura «PDF»
recortada en lista, el título partido de la muestra de Ajustes y la saturación
de filtros de anotaciones. También detectó dos problemas del arnés: el antiguo
centro del catálogo no descontaba el rail y las capturas del lector podían
tomarse durante transiciones de tema anidadas. La matriz ahora espera a que
terminen; los colores intermedios no se califican como defectos finales.

Las goldens 0.10/0.11 permanecen intactas como historia. Las nuevas referencias
`windows-reader-editorial-v015.png` y `windows-document-navigator-v015.png`
se aceptaron después de abrir las imágenes candidatas, no mediante actualización
indiscriminada de la suite.

## Límites preservados

Los cambios de UI no alteran PDFium, el contenido de los PDF, SQLite v11,
contraseñas en memoria, backup/rollback ni consolidación reversible. El rail
delega en los filtros del controlador y en consultas paginadas: no filtra una
página parcial en memoria. Se mantiene el máximo de 80 documentos materializados.
No se añaden dependencias, red, cuentas, telemetría ni funciones de IA.

## Estado de la entrega

### Gate completo del 7 de septiembre

`build/verification/verification-report.json`: `passed`, 14 fases aprobadas,
cero omitidas, de 02:13:34 a 02:18:45 UTC. Análisis sin incidencias y suite de
322 pruebas aprobadas. Incluye corpus patológico, benchmark de Biblioteca,
build Windows Release, smoke offline y las dos goldens mantenidas de 0.15.
La matriz nativa registra 96 DPI/100 % y completa cuatro casos; ese valor
reportado por el proceso no acredita por sí solo una comprobación física
independiente del monitor. Narrator y varios monitores siguen pendientes.

### Revisión del flujo nativo (7 de septiembre)

La serie posterior al cierre de notificaciones, aún conservada en
`build/performance/0.15.0-before-search-isolation`, volvió a fallar: Biblioteca p95 de build
8574 µs frente a 4672 µs (+83,52 %, cinco muestras sobre el límite), un `watch`
de render pesado y cero invariantes fallidos. No se acepta como cierre.
La reconstrucción experimental bajo `build/performance-baseline-014` no es una
referencia válida: el checkpoint temprano contiene cambios editoriales y no
compila. No se sustituyen las muestras históricas por ese árbol.

Se elimina un LayoutBuilder que no utilizaba restricciones en las portadas de
sustitución y se reutiliza su subárbol mientras título, autor, estado, colores,
tipografía, escala e idioma efectivo no cambien. El almacenamiento queda ligado
a cada portada montada, sin caché global ni nuevos documentos materializados.
La interpolación de una paleta consigo misma conserva su identidad. Las 25
pruebas específicas pasan, incluido que el cambio de densidad reutilice el texto
y que tema oscuro y metadatos nuevos sigan actualizando la portada.
La primera medición exploratoria con esa optimización registra Biblioteca p95
5698 µs (`build/performance/0.15.0-placeholder-reuse-probe.json`): mejora el
coste observado, pero todavía no demuestra el cierre del umbral ni sustituye
una serie de cinco. Se elimina además la reconstrucción del catálogo por cada
pulsación durante el debounce y el Theme heredado redundante de Biblioteca.
El contrato de Ctrl+F comprueba identidad de la portada antes de que llegue
el resultado SQL y confirma que después se filtra y se recupera correctamente.
El segundo perfil exploratorio (`0.15.0-query-rebuild-probe.json`) no mejoró
el pico: 9279 µs. El desglose mostró que el primer frame de apertura de búsqueda
dominaba el p95. Se aisló su visibilidad en un ValueNotifier: abrir el panel
reconstruye solo el overlay. El contrato compara también la identidad de la
tarjeta al abrir Ctrl+F. Las 23 imágenes regeneradas de Biblioteca conservan
exactamente sus SHA-256, incluidos oscuro, RTL, texto ampliado y compactos.

La notificación accionable de Ajustes permaneció visible en el arnés nativo
y ocultó controles inferiores al cambiar de pantalla. Se añadió un cierre
explícito común, sin ejecutar «Deshacer» ni suprimir la confirmación del cambio.
El contrato de cierre pasa; el recorrido nativo utiliza ese mismo control antes
de capturar el estado estable. Este cambio de tema compartido es posterior al
gate de 02:18 UTC y exige un gate final actualizado antes del empaquetado.
El recorrido final pasó con cinco capturas inspeccionadas: lector claro en
página 2, Biblioteca clara, Ajustes, Biblioteca oscura y lector oscuro retomado
en página 2. Ctrl+F abre el campo de búsqueda real; cerrar la notificación no
deshace el tema. El informe confirma el SHA-256 del PDF sin cambios.

La captura del recorrido con PDFium real mostró una franja de continuidad
todavía demasiado alta y el texto «1 pendientes». Se sustituyó por «Sesión
reciente», plegada inicialmente y con acciones y contadores dentro de un área
desplazable acotada. Los contadores usan etiqueta y valor, sin una concordancia
incorrecta. Se conserva la acción principal de retomar fuera de ese panel.
Las 12 pruebas específicas de revisión y contratos 0.12 pasan, incluida la
apertura del panel y el acceso a reanudar con texto al 200 %. Un contrato nuevo
limita a 56 px la franja plegada con texto normal.

La prueba de integración utiliza PDF y base de datos temporales, preferencias
en memoria y PDFium real. Sus PNG se obtienen del árbol renderizado de Flutter,
no de la captura del compositor de Windows; no acreditan DPI físico ni Narrator.
Se corrigió el simulador de teclado del arnés especificando las teclas físicas
de Ctrl+F y Escape para su ejecución en perfil.

### Segunda pasada de revisión

Se abrieron las 20 hojas de contacto y PNG individuales de Ajustes, duplicados,
anotaciones al 200 %, menús ampliados y reglas oscuras. La revisión encontró
un menú de fixture que no utilizaba el componente compartido y partía palabras;
el componente ahora coloca el atajo debajo al ampliar texto y el fixture usa
esa misma implementación. Dos nuevos contratos cubren ese comportamiento y las
reglas en ES/EN, claro/oscuro y 640×480 al 200 %.

El gestor de reglas exponía enums y su cabecera se desbordaba con texto al
200 %. Ahora reutiliza etiquetas traducidas y redistribuye la cabecera con
scroll limitado. El texto negro de la captura oscura era una transición de
tema sin terminar: se verificó esperando a que finalizase, sin cambiar colores
para ocultar el artefacto. Las matrices de workflows fijan también el locale.

La primera serie completa está archivada en
`build/performance/0.15.0-before-final-review`: 1 regresión fiable
(`native.frame_build.library_p95`, 5983 µs frente a 4672 µs, +28,061 %),
8 observaciones y 9 invariantes sanos. No se acepta como rendimiento final.
Se eliminó la construcción redundante de ThemeData en Biblioteca y capas
constantes de opacidad/saturación y animación de duración cero en las portadas.
Las verificaciones siguientes deben demostrar la corrección sin rebajar el umbral.
El informe archivado conservaba etiquetas de versión antiguas (0.13/0.14),
aunque sus rutas de series y muestras corresponden a 0.14/0.15. Se corrigió
esa metainformación en el comparador para las siguientes ejecuciones, sin
alterar los datos archivados ni la regla de aceptación.

Las imágenes de discrepancias históricas de golden se conservaron en
`build/checkpoints/visual-review-failures-015`, fuera de `test/`.

### Cierre acotado solicitado por el usuario

Se detienen las optimizaciones y los cambios de diseño. La última serie completa
`build/performance/0.15.0-current/comparison-0.14-vs-0.15.json` termina en `fail`:
dos métricas fallidas, una observación y ningún invariante fallido. Biblioteca
p95 de construcción de frames: 6986 frente a 4672 µs (+49,53 %); lector:
8078 frente a 6760 µs (+19,50 %). No se rebaja el umbral ni se presenta el
rendimiento como aprobado. Estos tiempos no equivalen a latencia total de frame.

La entrega se considera candidata local, no plenamente validada. La serie
interrumpida permanece en `build/performance/0.15.0-interrupted-1` y no cuenta
como perfil completo. El checkpoint temprano no permite reconstruir una base
limpia 0.14 compilable; no se inventa una medición inicial alternativa.

El prompt `docs/prompts/finish-visual-polish-0.15.md` permite continuar sin repetir
la auditoría: máximo dos experimentos de rendimiento, hasta tres correcciones
visuales confirmadas y una validación final, con parada explícita si no mejora.

Comprobación final del árbol actual: formato de 242 archivos sin cambios,
325 pruebas aprobadas, compilación Release correcta, smoke offline correcto,
dos goldens mantenidas y matriz nativa aprobados. El gate conservado registra
13 fases aprobadas y una fallida: el analizador encontró la copia histórica
defectuosa en `build/performance-baseline-014`. Se movió sin borrarla a
`C:/Users/yagol/AppData/Local/Temp/yamantaka-baseline-diagnostic-435da28c7a3e4427a50b1462388d2379`.
La repetición independiente de `flutter analyze --no-pub` pasó sin incidencias
en 164,1 s. No se altera el informe original ni se declara el gate integral verde.

Paquete candidato generado en `build/release/0.15.0+16/`, sin firma:
`yamantaka-0.15.0+16-windows-x64.zip`, 19.309.158 bytes, SHA-256
`12610e7b768e2dfabf3f14ce9025e76859ff6d5828f0f5176478b5a7d69370ca`.
Ejecutable: `yamantaka/yamantaka.exe` dentro de ese directorio. El smoke del
ejecutable exacto extraído del ZIP pasó durante 8 segundos, dos teclas enviadas
y ninguna conexión, con cierre por el arnés. Incluye manifiesto, hashes, SBOM,
informes originales y evidencia visual. El verificador integral termina con
error porque el gate empaquetado no es completamente verde; además permanece
la comparación de rendimiento fallida. No es una release plenamente aprobada.

Mensaje de commit sugerido (no ejecutado):
`feat(ui): unify editorial desktop experience and document release candidate`

## Cierre final acotado — 7 de septiembre de 2026

Antes de editar se guardaron los 67 archivos modificados o nuevos y un parche
binario recuperable en `build/checkpoints/pre-close-20260907-131729.zip`,
SHA-256 `5dd5f8b3b3039aab371132b723903c737561c61360895526f80148d2fe9fa2f3`.
El directorio expandido se movió a
`C:/Users/yagol/AppData/Local/Temp/yamantaka-pre-close-20260907-131729` para que
el analizador no tratase sus copias Dart como fuentes; el ZIP permanece bajo
`build/checkpoints` y no se eliminó ningún dato.

La comparación de rendimiento es parcialmente comparable: ambas series usan
Windows Profile, 1440×900 físicos, 1152×720 lógicos, escala 125 %, 40 documentos
y los mismos fixtures y repeticiones. El informe conserva Windows y ocho
procesadores, pero no registra la revisión exacta de Flutter/Dart ni la carga
histórica del equipo. El toolchain actual es Flutter 3.44.8, revisión
`058e0af2c2`, motor `0cd610717b` y Dart 3.12.2. No se reconstruyó una referencia
0.14 a partir del checkpoint temprano defectuoso.

Las muestras de frame identifican una operación concreta: abrir búsqueda es el
primer frame medido y domina ambos recorridos. En 25 ejecuciones conservadas,
su mediana es 6460 µs en Biblioteca y 7536 µs en el lector; redimensionar,
cerrar, hacer zoom y avanzar quedan por debajo. Se agotó el límite de dos
experimentos y ambos se revirtieron byte a byte:

1. Reutilizar el tema heredado del lector no mejoró la apertura en la pareja
   exploratoria: 9894→10005 µs (+1,12 %). Perfiles:
   `build/performance/0.15.0-experiment-0-before-reader-theme.json` y
   `build/performance/0.15.0-experiment-1-reader-inherited-theme.json`.
2. Mantener las superficies de búsqueda montadas fuera de escena dejó el lector
   en 7537 µs (+0,01 %) y empeoró Biblioteca a 11177 µs (+73,02 %). Perfil:
   `build/performance/0.15.0-experiment-2-prewarmed-search.json`.

No se ejecutó otra serie de cinco sin una hipótesis prometedora. Sigue vigente
el resultado `fail` de `build/performance/0.15.0-current/`: Biblioteca 6986
frente a 4672 µs (+49,53 %) y lector 8078 frente a 6760 µs (+19,50 %), con una
observación, cero invariantes fallidos y ningún frame de esas series por encima
de 16,7 ms. Son tiempos de construcción, no latencia total ni evidencia por sí
solos de tirones visibles.

El recorrido nativo se repitió a las 11:38:41 UTC con PDFium real, PDF y base de
datos temporales: pasó, mantuvo el SHA-256 del PDF, volvió a la página 2 y
regeneró cinco PNG claro/oscuro en
`build/visual-evidence/v0.15.0/native-flow/`. La revisión individual y los casos
estrechos/al 200 % no confirmaron un defecto nuevo que justificase cambiar la
UI. La posible superposición del inspector con una portada única permanece como
propuesta medible de backlog, no como fallo aceptado sin criterio.

El gate completo nuevo de `build/verification/verification-report.json` pasó
sus 14 fases, sin omisiones, entre 11:43:55 y 11:46:45 UTC: formato de 242
archivos, análisis limpio, 325 pruebas, corpus patológico, benchmark de
Biblioteca, build Windows Release, smoke offline, dos goldens y matriz nativa.
La matriz solo confirma cuatro casos 1600×900 en el monitor disponible a 96
DPI/100 %. La opción `-DpiScaleConfirmed` no acredita Narrator, otros monitores
ni escalas físicas 125/150/175/200 %.

El paquete se regeneró a las 11:48 UTC en `build/release/0.15.0+16/`:
`yamantaka-0.15.0+16-windows-x64.zip`, 19.710.752 bytes, SHA-256
`609a82ac79f8eab0227c762f6f2ac4657c0f69896f0f518a9dae047ba35e9fcd`.
El ejecutable exacto es `yamantaka/yamantaka.exe`, 226.816 bytes, SHA-256
`8a6b7abd46e9820c95d0aa48df1f95cef16be3870484a24e031831d80e758ed4`.
El smoke del ejecutable extraído pasó ocho segundos, envió dos de dos teclas,
cerró mediante el arnés y registró cero conexiones. Inventario, hashes, SBOM
CycloneDX 1.5 con 119 componentes y evidencia visual 193/11/21 son coherentes.
Los nueve binarios siguen sin firma.

`tool/verify_windows_package.ps1` rechaza correctamente el candidato en su gate
de rendimiento: el informe empaquetado no demuestra cinco series aprobadas.
La entrega queda como candidato local funcional y verificable, pero no como
release plenamente aprobada. Ampliar la optimización requiere una decisión
explícita para instrumentar builds por widget o aceptar el coste editorial;
esta sesión no encadena más experimentos.
