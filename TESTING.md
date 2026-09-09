# Pruebas

Estado actualizado: 2026-09-09.

## Pase de simplificación de Biblioteca

La revisión actual pasa `flutter analyze --no-pub`, las 329 pruebas de
`flutter test --no-pub` y `flutter build windows --release --no-pub`. El flujo
nativo con PDFium real también pasa: abre un PDF sintético de 12 páginas,
recorre Biblioteca, Lector y Ajustes en claro y oscuro, restaura la página 2 y
confirma que el PDF termina byte-idéntico. No usa datos personales y declara
`physicalDpiValidation: false`.

La evidencia visual regenerada queda en
`build/visual-evidence/library-simplification-after/`; incluye Biblioteca,
Lector, Ajustes, flujo nativo y once comparaciones con 0.15. Se revisaron los
estados claro/oscuro, 640×480, 1280×720, cuadrícula/lista, selección, panel y
texto al 200 % sin detectar overflow ni pérdida de una función esencial.

La mejora posterior de visibilidad se regenera en
`build/visual-evidence/visibility-pass/`. Sus matrices de Biblioteca, Ajustes y
Lector y las once comparaciones `simplified`/`visibility` confirman superficies
oscuras neutras, selección con marca explícita y foco de lista con contorno
completo. Tras ese cambio vuelven a pasar el análisis, las 326 pruebas y la
build Windows Release.

La revisión `cover-first` se regenera en
`build/visual-evidence/cover-first-library/`. Verifica la Biblioteca sin marca
ni rail persistente, portadas de hasta 220 px, continuidad compacta, controles
bajo demanda y los modos `Animaciones completas` / `Movimiento reducido`. La
matriz incluye claro, oscuro negro, 640×480, texto al 200 %, alto contraste,
movimiento reducido, selección y lista. Las pruebas focalizadas y la build
Windows Release pasan. El flujo nativo actualizado con PDFium real también
pasa y deja el PDF sintético byte-idéntico. El gate final aprueba 13 de 14 fases
sin omisiones y falla únicamente al no poder confirmar físicamente 96 DPI/100 %;
la verificación integral se mantiene sujeta a las mismas limitaciones físicas,
de firma y de rendimiento documentadas debajo.

La iteración de cubiertas 3D se conserva en
`build/visual-evidence/three-dimensional-covers/library/`. Sus casos verifican
las seis cubiertas procedurales, perspectiva y volumen, selección, claro/negro,
alto contraste, movimiento reducido y texto al 200 %. La implementación es
vectorial y no incorpora activos externos al paquete. El análisis, las 326
pruebas y la build Release pasan; el paquete de prueba mide 18,77 MiB y supera
el smoke de ocho segundos sin conexiones de red.

El refinamiento posterior se genera en
`build/visual-evidence/procedural-cover-families/library/`. Sustituye la
plantilla repetida de icono central por seis familias visuales, añade un canto
de páginas dibujado, integra las miniaturas reales a sangre y reserva el
paralaje y el brillo para `Animaciones completas`. La matriz visual focalizada
incluye claro, oscuro negro, selección, 640×480, alto contraste, movimiento
reducido y texto al 200 %; el contrato de miniatura comprueba además recorte de
tapa, alineación superior y decodificación limitada al tamaño físico. Tras el
refinamiento pasan análisis, 327 pruebas, Release y el flujo PDFium real con el
PDF byte-idéntico. El ZIP de prueba de 21 entradas mide 18,79 MiB, un 0,078 %
más que la primera versión 3D, y supera el smoke de ocho segundos sin red.

El pulido final se regenera en
`build/visual-evidence/library-polish-final/`. La matriz de Biblioteca verifica
23 casos y la del lector 34: claro/oscuro, selección, 640×480, texto 200 %, alto
contraste, movimiento reducido y panel superpuesto/acoplado. El flujo nativo
PDFium pasa y `native-flow.json` declara `pdfUnchanged: true`; su captura
`library-populated-dark.png` confirma con una miniatura real que la tapa queda
horizontal y sin el escalonado de la antigua rotación permanente. Dos contratos
nuevos fijan reposo/respuesta al ratón y desaparición de avisos pasivos. La barra
del lector conserva targets de 40 px y el cambio de sección no reintroduce el
panel completo.

El gate agregado pasa 13 fases y falla únicamente `native resolution and DPI
matrix`: produjo cuatro capturas nativas a 1600×900 y 96 DPI/100 %, pero el host
no permitió confirmar físicamente esa escala (`scaleConfirmed: false`). No se
usó `-DpiScaleConfirmed` para fabricar esa evidencia. Por tanto, el árbol actual
no es un candidato publicable. El smoke de ocho segundos pasa sin conexiones de
red y el benchmark de catálogo pasa hasta 100 000 documentos, materializando 80
por página y sin datos personales.

Esta pasada no ejecutó nuevos experimentos de rendimiento. Continúan vigentes
las dos regresiones fiables de 0.15: p95 de construcción de frames de Biblioteca
6986 frente a 4672 µs (+49,53 %) y lector 8078 frente a 6760 µs (+19,50 %).
Tampoco acredita Narrator, varios monitores, escalas físicas adicionales ni
firma Authenticode.

Auditoría y decisiones: `docs/architecture/library-simplification-audit.md`.
Informes: `build/verification/verification-report.json`,
`build/verification/native-app-flow.json`,
`build/verification/dpi-matrix/matrix-report.json` y
`build/verification/performance/library-benchmark.json`.

## Iteración editorial 0.15

La suite completa del cierre del 7 de septiembre pasa con 325 pruebas. El gate
integral más reciente, ejecutado entre 11:43:55 y 11:46:45 UTC, pasa sus 14
fases sin omisiones: formato de 242 archivos, análisis limpio, suite, corpus,
benchmark de Biblioteca, build Windows Release, smoke offline, dos goldens y
matriz nativa. El fallo histórico de análisis por una copia bajo build permanece
documentado en el registro de iteración; no se retocó para simular este pase.
Incluye contratos para menús ampliados, textos de reglas locales y sesión
reciente compacta.

La comparación de cinco pasadas actual NO pasa: dos métricas fallidas, una
observación y cero invariantes fallidos. No confundir pruebas funcionales
aprobadas con aceptación de rendimiento o validación integral del paquete.
Dos experimentos exploratorios acotados se revirtieron al no mejorar la apertura
de búsqueda. Sus perfiles se conservan en
`build/performance/0.15.0-experiment-0-before-reader-theme.json`,
`build/performance/0.15.0-experiment-1-reader-inherited-theme.json` y
`build/performance/0.15.0-experiment-2-prewarmed-search.json`; no se ejecutó una
nueva serie final sin una hipótesis prometedora.

Las matrices actuales se generan en `build/visual-evidence/v0.15.0/` mediante
las variables `YAMANTAKA_*_MATRIX_DIR` de los tests visuales. Las ocho familias
de tests producen nueve grupos de superficies; no basta ejecutar la suite sin
esas variables para regenerar la evidencia. Comparaciones y hojas se generan
con `tool/create_visual_comparisons.ps1` y `tool/create_contact_sheets.ps1`.

`tool/run_iteration_performance.ps1` ejecuta cinco pasadas de cada familia,
incluido Flutter profile nativo, y compara con `build/performance/0.14.0-current`.
No ejecutar cargas competidoras ni modificar el código durante una serie final.
`tool/verify_windows_package.ps1` verifica inventario, tamaños, hashes, SBOM,
gate, smoke y evidencia del ZIP local 0.15.0+16.
El candidato regenerado pasa esas comprobaciones hasta el gate de rendimiento y
se rechaza allí de forma intencional porque `performance-report.json` conserva
dos regresiones fiables. No se considera una release aprobada.

El recorrido de aplicación completa con PDFium real se ejecuta con:

    flutter drive --driver=test_driver/professional_polish_driver.dart --target=integration_test/professional_polish_flow_test.dart -d windows --profile --no-pub --dart-define=YAMANTAKA_NATIVE_FLOW_DIR=build/visual-evidence/v0.15.0/native-flow

En ejecución nativa, utiliza una ruta absoluta para el directorio de evidencia
si el directorio de trabajo del runner no es la raíz del repositorio. El informe
del driver queda en `build/verification/native-app-flow.json`. Comprueba un flujo
con PDF de 12 páginas, restauración de página, tema global, Ctrl+F y hash del PDF
sin cambios. No utiliza preferencias ni documentos personales. Las capturas son
del árbol renderizado de integración, con geometría lógica fijada a 1280×720;
no certifican DPI físico, Narrator ni el compositor de Windows.

El registro de revisión y las limitaciones físicas están en
`docs/architecture/visual-iteration-0.15.md`. Las secciones de iteraciones
anteriores que siguen debajo son históricas y no certifican esta versión.

Este documento describe únicamente evidencia reproducible del árbol actual. Se
han retirado las tablas y capturas históricas que correspondían a lectores
anteriores.

## Gate local

Desde la raíz del repositorio:

    flutter analyze --no-pub
    flutter test --no-pub
    flutter build windows --release --no-pub

El flujo agregado de Windows sigue disponible:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/verify_windows.ps1 -AllowDirty

El gate no descarga dependencias. Cada fase fallida o saltada queda explícita
en build/verification/verification-report.json. La ejecución local solo acepta
un bundle completo bajo `build/` y sin Mark-of-the-Web. Compilar no publica la
aplicación ni sustituye la firma exigida para distribución.

## Cobertura del lector

La suite automatizada comprueba:

- Una página, Dos páginas, Continua y restauración de preferencias;
- matriz de paridad de acciones comunes entre disposiciones en 640 y
  1280 px;
- pliegos 1–2, 3–4 y final impar, además de paridad explícita y RTL;
- ajuste de pliego, aislamiento de unidades, centrado y límites de paneo;
- navegación por botones, flechas, Page Up/Down, Home/End y espacio;
- exclusión de atajos mientras un campo de texto tiene foco;
- F11, Escape y separación entre fullscreen, chrome e inmersivo;
- superficie superior opaca y ausencia de degradado transparente;
- disposiciones accesibles desde `Ctrl+K`, sin selector permanente, y
  navegación inferior centrada respecto a la ventana;
- composición compacta, media y expandida sin overflow;
- alto contraste, movimiento reducido y perfil de tinta electrónica;
- panel lateral, búsqueda, marcadores, temas de Documento y persistencia;
- panel overlay en 680–1099 px y panel restado del visor en ≥1100 px, también
  durante resize y sin reconstruir el adaptador PDF;
- dock de reposo accionable sin hover, revelado por teclado y rail vertical del
  panel acoplado con ancho semántico visible;
- barra 56/52 px, dock 48/38 px y targets de 40 px medidos por widget;
- identidad estable del adaptador al abrir/cerrar panel y cruzar 680/1100 px.
- navegador `Ctrl+J` con 100 destinos, filtro, teclado, lista + detalle,
  retorno `Alt+←` e identidad estable del adaptador;
- anotación desde selección real, fallback multipágina honesto y persistencia
  de texto, contexto y coordenadas normalizadas.

La biblioteca cubre inspector contextual, selección conservada y explicación al
filtrar, además de metadatos PDF, cuatro páginas iniciales, título multilínea,
autor/editor explícito o dudoso, fallback de nombre legible, ausencia de texto,
temas ES/EN, límite de muestra, reanálisis y preservación manual, facetas
completas, migración 7→8 y backup v7 con ajustes de pliego.
La iteración 0.11 añade máximo materializado de 80, agrupación por autor,
colección, etiqueta y estado, plegado estable, límite de tres pestañas fijadas,
sesiones ricas, cerrados recientes, metas locales y cursor FTS reanudable.

El motor cubre PDFium real, contraseña, corrupción, cancelación previa,
generaciones obsoletas, cierre sin pendientes, reducción dinámica de LRU y
diagnóstico de rendimiento sin campos privados. El perfil Windows añade
documentos sintéticos de 1, 50, 500 y 5.000 páginas, vector pesado y raster
extremo; los resultados bajo `build/` no contienen rutas ni títulos.

El perfil nativo reproducible se ejecuta en modo Profile y después se compara:

    flutter drive --driver=test_driver/performance_driver.dart --target=integration_test/performance_profile_test.dart -d windows --profile --no-pub
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/compare_pdf_profile.ps1

La referencia PDF aún corresponde a `0.3.0+4`; es histórica, no una línea base
0.9 comparable. Un fallo frente a ella se conserva y documenta, y no autoriza a
sobrescribir la referencia para obtener un resultado verde.

## Referencia visual mantenida

La única captura normativa es:

    docs/screenshots/windows-reader-editorial-v010.png

La genera y compara esta prueba:

    flutter test --no-pub test/reader_widgets_test.dart --plain-name "book reader matches the maintained editorial v0.10 baseline"

Para aceptar deliberadamente un cambio visual:

    flutter test --no-pub --update-goldens test/reader_widgets_test.dart --plain-name "book reader matches the maintained editorial v0.10 baseline"

Es una referencia de widgets ejecutados a 1600×900, no una captura nativa ni un
mockup manual. Valida la composición fullscreen del contenido: barra opaca,
pliego enfrentado, canal central, controles laterales, navegación inferior
centrada y acciones de vista a la derecha sin un selector permanente.

Las matrices widget de Biblioteca, Lector y Ajustes cubren 640×480, 800×600,
1280×720, 1600×900, 1920×1080, 1920×1200, 2560×1440 y 3440×1440, además de
tema oscuro, alto contraste, movimiento reducido, chrome en reposo y panel
abierto. La evidencia previa se conserva en `build/visual-evidence/v0.5.0/` y
la de 0.6 se conserva en `build/visual-evidence/v0.6.0/`, la de 0.7 en
`build/visual-evidence/v0.7.0/` y la de 0.8 en
`build/visual-evidence/v0.8.0/`. La actual se escribe en
`build/visual-evidence/v0.13.0/`; ninguna se versiona.

La matriz 0.11 conserva la prueba geométrica del chrome y añade el navegador
documental en los ocho tamaños, cerrados recientes, agrupación y selección
anotable. Mantiene Continuar leyendo, filtros FTS5, búsqueda de Ajustes y texto
al 130/150/200 %. El diálogo real de importación se captura con documentos
nuevos, duplicados, inaccesibles e inválidos. Conserva los casos de Biblioteca,
Lector, panel, Ajustes, búsqueda, paleta, menús y diálogos en claro, oscuro, alto
contraste y movimiento reducido. Las comparaciones usan 0.10 como fuente
anterior y registran el SHA-256 de ambos PNG.

## Validación nativa de Windows

El build Release compila también el canal C++ que elimina el frame de la
ventana. La comprobación manual debe confirmar:

1. abrir un PDF y elegir Libro;
2. comprobar que se muestran 1–2 enfrentadas;
3. pulsar F11 y verificar que desaparecen caption y barra de tareas;
4. pulsar Escape y comprobar que se restaura exactamente la ventana anterior;
5. repetir en cada monitor y escala real disponible.

Antes de cualquier `Start-Process`, las herramientas comprueban la estructura
del bundle, que la build local permanezca dentro del repositorio y que sus
binarios no tengan Mark-of-the-Web. `-RequireSignature` añade la comprobación de
todos los `.exe` y `.dll`; si una firma falta o no es válida, la fase se omite
antes de intentar el arranque.

Una vez disponible un certificado RSA confiable, puede firmarse el bundle sin
publicarlo:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/sign_windows_bundle.ps1 -BundlePath build/windows/x64/runner/Release -CertificateThumbprint <THUMBPRINT>

El gate también puede firmar después de compilar y exigir esa firma antes de los
smokes nativos:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/verify_windows.ps1 -AllowDirty -CertificateThumbprint <THUMBPRINT> -RequireSignature

La matriz automatizable usa solo entradas públicas de la aplicación. Para una
matriz destinada a distribución se añade `-RequireSignature`:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/run_visual_matrix.ps1 -HostScalePercent 125 -ScaleConfirmed -RequireSignature

Debe ejecutarse una vez por cada escala real del host. El script ya no genera
cinco registros idénticos ni pasa argumentos visuales inexistentes. El
capturador admite PreserveWindowBounds y SendKeys para conservar una ventana
fullscreen y enviar F11.

El 8 de agosto de 2026 se lanzó el bundle Release local en este host: el proceso
permaneció activo, presentó una ventana que aceptó cierre normal y no generó un
evento de error de Yamantaka. Por tanto, `NotSigned` no se usa como sinónimo de
“no arrancable”. Los nueve binarios siguen sin firma y no constituyen evidencia
de distribución; `-RequireSignature` conserva ese gate separado.

El 24 de agosto de 2026 el bundle local `0.5.0+6` superó además un smoke de 5 s
con un PDF sintético: permaneció activo hasta el cierre del harness y no abrió
conexiones TCP/UDP. El informe está junto al paquete versionado bajo `build/`.

La iteración `0.6.0+7` repitió formato, `git diff --check`, análisis, las 244
pruebas, golden y build Release. El paquete de 21 archivos (41,11 MiB; ZIP de
17,83 MiB) superó otro smoke de 5 s sin conexiones TCP/UDP. Manifiesto, hashes y
reporte están en `build/release/0.6.0+7/`. Los nueve binarios nativos continúan
sin firma; es evidencia local de prueba, no de distribución.

La iteración `0.7.0+8` aprueba formato de 212 archivos, `git diff --check`,
análisis sin incidencias, 247 pruebas y la golden 0.7. Las matrices generan 28
capturas de Biblioteca, 16 del Lector, 16 de Ajustes, 6 de paleta/menú y 5 de
diálogos; `tool/create_visual_comparisons.ps1` produce diez pares 0.6/0.7 con
hash de cada fuente. El Release contiene 21 archivos (43.138.872 bytes), su ZIP
18.705.581 bytes y un smoke sintético de 8 s registra cero conexiones. Los nueve
binarios siguen sin firma.

La iteración `0.8.0+9` aprueba análisis sin incidencias, 252 pruebas y la golden
0.8. Sus matrices generan 11 capturas de Biblioteca, 16 del Lector, 16 de
Ajustes, 6 de paleta/menú y 5 de diálogos; diez pares 0.7/0.8 registran hash de
cada fuente. `tool/recolor_brand_assets.py` recupera la marca desde `HEAD`,
recolorea cada coordenada sin recorte ni deformación y escribe
`build/visual-evidence/v0.8.0/branding/pixel-proof.json`: dimensiones y alfa son
idénticas, la correlación de luminancia del PNG 512 es 0,999998978 y la
desviación máxima es inferior a 0,461. Narrator, varios monitores y DPI físico
siguen siendo validaciones manuales pendientes; una matriz de widgets no se
presenta como sustituto.

El gate Windows 0.8 aprueba además corpus sintético, benchmark con 5.000,
25.000 y 100.000 documentos materializados en ventanas de 80, build Release,
smoke de 8 s con cero conexiones y baseline visual. La matriz DPI nativa se
omitió explícitamente. El paquete contiene 22 archivos (43.523.377 bytes) y su
ZIP ocupa 19.418.873 bytes; los nueve PE permanecen `NotSigned`.

La iteración `0.9.0+10` añade aserciones geométricas para las barras del lector,
la identidad estable del visor al redimensionar, la conservación de posición al
cambiar entre lista y cuadrícula y una matriz visual propia de la búsqueda
global. Las matrices producen 59 capturas de superficies y diez comparaciones
0.8/0.9. Tras empaquetar, este comando contrasta inventario, tamaños y SHA-256
del manifiesto con el contenido real del ZIP y rechaza rutas inseguras o PDF:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/verify_windows_package.ps1

La iteración `0.10.0+11` añade pruebas específicas para destinos de página,
cancelación y filtros FTS5, marcadores en el índice y orden del conmutador de
pestañas, apertura nueva con estado independiente, drag & drop múltiple,
teclado en búsqueda y filtro combinado de notas. Las matrices de Lector,
Biblioteca, Búsqueda, Ajustes, paleta, menús y diálogos usan los ocho tamaños
principales y añaden 130/150/200 % de texto. La salida actual contiene 128 PNG
de superficies y diez comparaciones 0.9/0.10. La evidencia manual de Narrator,
varios monitores y DPI físico sigue separada y nunca se deduce de estas
capturas.

La iteración `0.11.0+12` aprueba 277 pruebas y añade contratos para 100 destinos
sin reconstruir el visor, retorno al origen, selección geométrica y fallback,
sesiones ricas, máximo de tres pestañas fijadas, cursor FTS reanudable,
prioridad cambiada mientras el índice está activo, consultas globales
solapadas y decodificaciones de portada evitadas al salir del viewport. Las
matrices producen 164 PNG de origen: 21 Biblioteca, 43 Lector, 16 Búsqueda, 18
Ajustes, 28 paleta/menús, 28 diálogos —incluida importación— y diez
comparaciones 0.10/0.11. `tool/create_contact_sheets.ps1` genera 17 hojas con
hashes para revisar el conjunto completo.

Los cinco perfiles comparables están bajo
`build/performance/0.10.0-baseline/` y
`build/performance/0.11.0-current/`. El comparador exige una mediana superior
al 10 % y al menos cuatro de cinco muestras actuales sobre el límite para
declarar una regresión fiable. El resultado 0.11 es `pass-with-watch`: no hay
regresiones ni invariantes fallidos. Dos medianas crudas quedan en observación
—interacción de Biblioteca +17,08 % y scroll continuo +18,98 %— porque solo
tres de cinco muestras cruzan el límite y los rangos se solapan.

La validación física disponible corresponde a un único monitor 1920×1200 a DPI
96/escala 100 %, locale es-ES y alto contraste del sistema desactivado. El gate
nativo cubre sus cuatro casos a esa escala. No se extrapola a 125/150/175/200
%, dos monitores, Narrator, en-US ni alto contraste real: esas combinaciones
permanecen pendientes hasta disponer del hardware o estado del sistema.

El smoke físico del ZIP se ejecutó contra el `yamantaka.exe` exacto del bundle:
arrancó en español, permitió mover la selección de Biblioteca con teclado,
abrir el PDF local con Enter, avanzar de la página 1/2 a la 2/2 y cerrar de
forma normal. La inyección de teclado de la automatización de Windows no logró
reproducir `Ctrl+J`, por lo que ese atajo conserva cobertura de widget pero no
se presenta como validado físicamente. La captura WGC mostró rectángulos
obsoletos del frame anterior; se usó el árbol UIA para comprobar el estado y
esas imágenes no forman parte de la evidencia visual aprobada.

## Corpus y privacidad

El PDF mínimo local se genera así:

    dart run tool/create_test_pdf.dart build/verification/corpus/normal.pdf

Las pruebas PDFium, SQLite, backup, biblioteca y seguridad siguen formando
parte de flutter test. Los fixtures no contienen documentos personales y los
informes permanecen bajo build/, que está ignorado por Git.

## Regla de mantenimiento

- Una modificación visual del lector actualiza código, pruebas y la única
  referencia visual en el mismo cambio.
- Una captura manual no sustituye una aserción geométrica.
- Una aserción de widget no se presenta como prueba de caption, barra de tareas,
  DPI físico, Narrador o varios monitores.
- Un gate saltado nunca se describe como aprobado.
- Un bundle descargado, incompleto o fuera de la raíz `build/` nunca se abre
  automáticamente. La build local se distingue de una distribución firmada y
  nunca se desactiva Smart App Control para hacer pasar un gate.

## Verificación 0.12

La suite alcanza 287 pruebas. `iteration_012_test.dart` cubre migración v10,
anotaciones acotadas y masivas, vista/cola de espacio, backlinks/revisiones,
sesión atómica, journal interrumpido, reglas simuladas/aplicadas/canceladas/
deshechas, límite de 100.000, texto 200 % y liberación de comparación.
El mismo caso fuerza el límite temporal a cero y verifica truncado seguro antes
de procesar el primer documento; producción usa 30 segundos.
`iteration_012_visual_test.dart` ejecuta las ocho superficies nuevas y escribe
un manifiesto SHA-256.

La evidencia actual está en `build/visual-evidence/v0.12.0/`: 172 PNG (incluye
10 comparaciones 0.11/0.12) y 18 hojas bajo `build/visual-review-012/`. No se
versionan. Marca, alto contraste simulado, movimiento reducido, claro/oscuro,
texto 130/150/200 % y los ocho tamaños base forman parte del arnés.

Las cinco pasadas comparables están en `build/performance/0.12.0-baseline/` y
`build/performance/0.12.0-current/`. El resultado es `pass-with-watch`: cero
regresiones fiables y cero invariantes fallidos. Permanecen cinco observaciones
ruidosas porque solo tres de cinco muestras cruzan el umbral: p95 de búsqueda y
colecciones, consulta de 25.000, primera carga de 100.000 y memoria mínima tras
volver. La búsqueda caliente mediana baja de 592 a 28 µs; 100.000 documentos
materializan 80 y no recorren archivos en la primera carga.

La validación física declarada sigue limitada al monitor disponible 1920×1200,
96 DPI/100 %, es-ES y alto contraste desactivado. Narrator, varios monitores y
DPI 125/150/175/200 % no se infieren de widgets y quedan pendientes.

## Verificación 0.13

La suite completa alcanza 305 pruebas. `reader_widgets_test.dart` cubre aperturas
solapadas y cierre durante la apertura nativa sin publicar estado ni retener el
handle tardío. `document_service_error_test.dart` cubre rutas Unicode y rutas
locales largas dentro del límite, pérdida de permisos tras descubrir el archivo
y unidad desconectada sin filtrar la ruta. Las pruebas PDFium reales distinguen
contraseña requerida, incorrecta y válida.

`iteration_013_test.dart` construye fixtures con la secuencia real de cada
esquema v1–v11, migra dos veces al actual y verifica identidad e integridad.
También cubre snapshot sin rutas ni contraseñas, cuarentena tras fallo, estados
tipados, journal con fase/capacidad/hash y consolidación con procedencia, aristas
deduplicadas, ciclos eliminados y undo. `local_backup_service_test.dart` valida
el formato v9, manifiesto, ensayo, instalación interrumpida y rollback
transaccional ante una fila inválida.

`iteration_013_visual_test.dart` comprueba semántica, acciones, seguridad del
campo de contraseña y ausencia de overflow del diálogo de duplicados a texto
200 %. La matriz completa contiene 179 PNG en 19 hojas, diez comparaciones
0.12/0.13 y las superficies de contraseña, archivo ausente, restore, problemas
y recuperación. Su evidencia reproducible se conserva en
`build/visual-evidence/v0.13.0/` y `build/visual-review-013/`.

La evidencia focal puede regenerarse con:

    flutter test --no-pub --dart-define=YAMANTAKA_013_VISUAL_DIR=build/verification/visual/0.13 test/iteration_013_visual_test.dart

Las capturas son de widgets, no validan Narrator, DPI físico ni varios
monitores. Esas comprobaciones siguen pendientes.

Rendimiento usa cinco procesos por benchmark avanzado, Biblioteca y perfil
nativo para cada versión. El `HEAD` limpio 0.12 se reconstruyó y midió junto a
0.13 tras detectar deriva de host; la serie histórica no se sobrescribe. El
resultado canónico emparejado es `pass`: cero fallos, cero `watch`, cero
invariantes rotos, máximo 80 documentos y cero handles/renders/páginas tras el
cierre. Detalles y la comparación histórica fallida están en
`docs/performance/0.13-watches.md`.

El empaquetado genera `SBOM.cdx.json` sin red a partir de `pubspec.lock`, el
package config y los binarios Release. `verify_windows_package.ps1` exige
CycloneDX 1.5 y que cada `.exe`/`.dll` del manifiesto aparezca en el SBOM.

## Verificación 0.14

La suite completa alcanza 314 pruebas. `iteration_014_test.dart` fija tokens,
los ocho breakpoints, el orden y localización ES/EN de Ajustes, diálogo a
640×480 con texto al 200 %, Escape, progreso determinado, cancelación,
semántica, normal/hover/foco/seleccionado/deshabilitado, RTL, alto contraste y
movimiento reducido. Los 305 contratos previos permanecen sin cambios.

`iteration_014_visual_test.dart` añade estados focales de controles y operación,
contraseña normal/incorrecta, recuperación y restore. Junto a Biblioteca,
lector, Ajustes, búsqueda, paleta, diálogos, workflows y resiliencia produce 175
capturas directas; las diez comparaciones 0.13/0.14 elevan el conjunto a 185 PNG
en 20 hojas de contacto. Manifiestos y capturas incluyen SHA-256 y permanecen
bajo `build/visual-evidence/v0.14.0/` y `build/visual-review-014/`.
Las 20 hojas finales se inspeccionaron completas; no se aceptó la primera
generación de Ajustes hasta corregir la truncación de su rail al 200 %.

La evidencia focal se regenera con:

    flutter test --no-pub --dart-define=YAMANTAKA_014_VISUAL_DIR=build/verification/visual/0.14 test/iteration_014_visual_test.dart

Rendimiento conserva la regla de cinco procesos por familia y un umbral del
10 % que solo falla cuando mediana y cuatro de cinco muestras coinciden. El
timeout de comandos externos del gate pasa de 180 a 300 s; no cambia ninguna
aserción, solo evita que un análisis limpio sea declarado fallido por el host.
La serie final es `pass-with-watch`, sin regresiones fiables ni invariantes
rotos; conserva cinco `watch` de Biblioteca con solo tres de cinco muestras por
encima del límite. El diagnóstico y los valores están en
`docs/performance/0.14-watches.md`.

El gate Windows termina sus 14 fases en `passed`, sin omisiones. La matriz
nativa detecta y confirma el monitor real del host a 120 DPI/125 % y ejecuta
Biblioteca, Documento, Libro y Libro a pantalla completa a 1600×900. El smoke
sin red dura ocho segundos, navega dos veces, cierra el proceso y observa cero
conexiones.

La captura WGC nativa de este host devuelve frames obsoletos en Biblioteca y
blanco en lector, aunque UI Automation expone una única jerarquía correcta. Esas
imágenes no se aprueban. Narrator, varios monitores, en-US, alto contraste real
y DPI físicos 100/150/175/200 % siguen pendientes y no se infieren de widgets.
