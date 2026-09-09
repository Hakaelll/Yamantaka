# Yamantaka

Yamantaka es un lector PDF local para Windows orientado a documentos y
sesiones largas de lectura. No requiere cuenta, no sube archivos y conserva
biblioteca, progreso, preferencias, sesiones, marcadores, anotaciones y
metadatos propios en el dispositivo.

## Estado verificado

Versión de Yamantaka 0.15.0+16 revisada el 9 de septiembre de 2026:

- flutter analyze --no-pub: aprobado sin incidencias;
- flutter test --no-pub: 330 pruebas aprobadas;
- flutter build windows --release --no-pub: aprobado, incluido el runner C++;
- el gate actual aprueba 13 fases y falla solo la matriz nativa porque la escala
  física 96 DPI/100 % no pudo confirmarse de forma independiente; sus cuatro
  capturas se conservan como candidatas, no como certificación DPI;
- PDFium y SQLite conservan cobertura automatizada con fixtures sintéticos;
- el bundle Release local permanece receptivo durante un smoke de 8 s con un
  PDF sintético y no abre conexiones TCP/UDP; el harness lo cierra al terminar;
- sus nueve binarios nativos siguen sin firma, por lo que esta evidencia local
  no convierte el bundle en un artefacto apto para distribución.

La Biblioteca actual pone las portadas en primer plano: no muestra nombre, logo
ni navegación lateral persistente, usa una retícula de cubiertas grandes y
reúne vista, selección, facetas y rutas secundarias en menús accesibles. La
franja para retomar lectura ocupa una sola fila. El tema oscuro global vuelve a
negro y grises neutros; el violeta conserva las acciones y estados que requieren
acento. El flujo PDFium real confirma que el PDF de prueba no cambia.

La sesión reciente vive en un menú compacto de la barra. Al seleccionar un
libro aparece un contexto flotante más corto con su miniatura, progreso y una
sola acción principal. En oscuro, la marca se integra como filigrana de muy
baja opacidad y se omite en compacto, alto contraste o texto ampliado.

Las superficies seleccionadas también permanecen neutras: el violeta se limita
a indicadores y acciones. Las tarjetas seleccionadas muestran una marca
explícita y las filas enfocadas por teclado reciben un contorno completo.
En Ajustes se elige entre animaciones completas, cortas y orientadas al estado,
o movimiento reducido sin transiciones decorativas; la opción más restrictiva
del sistema siempre prevalece.

Las carátulas de reserva se representan como libros 3D vectoriales con lomo,
canto de páginas y seis familias gráficas deterministas: órbita, cristal, cubo,
monolito, cinta y estrella. Las carátulas reales se imprimen a sangre sobre el
mismo volumen, con lomo y acabado de tapa, sin añadir activos ni dependencias al
bundle. El puntero inclina y eleva el libro con una respuesta amortiguada en
dos ejes: el centro ofrece resistencia y los bordes producen un gesto más
marcado solo con animaciones completas; movimiento reducido conserva el
volumen estático. La luz mate de la tapa acompaña esa misma respuesta física,
de forma tenue y sin añadir otra animación independiente. En reposo la tapa
queda alineada a píxeles y usa filtrado de alta calidad al inclinarse, evitando
el escalonado de los bordes largos observado en Windows.

El lector divide la barra superior en dos grupos flotantes sin retirar ninguna
acción. La cápsula del documento reserva un mínimo legible, crece con el título
y se detiene antes de invadir las herramientas. Su panel permanece estable al
cambiar entre miniaturas, índice,
marcadores, anotaciones y búsqueda; solo transiciona el contenido. Los avisos
globales son compactos, se sustituyen entre sí y los pasivos desaparecen solos.

El release local versionado se genera en `build/release/0.15.0+16/` e incluye el
ZIP completo, manifiesto con hash por archivo, `SHA256SUMS.txt`, estado de firma,
SBOM CycloneDX 1.5, informe de pruebas, comparación de rendimiento, evidencia
visual con su propio manifiesto y smoke sin red. Debe extraerse entero antes de
probarlo.

No está plenamente aprobado: la comparación de rendimiento actual registra dos
regresiones fiables (p95 de construcción de frames en Biblioteca y lector).
Continuación acotada en `docs/prompts/finish-visual-polish-0.15.md`.

El bundle 0.15 se valida contra su manifiesto después del último cambio. El smoke
sintético comprueba conexiones TCP/UDP y el inventario separa explícitamente los
binarios sin firma; sigue siendo evidencia local, no un artefacto de distribución.

La compilación de prueba más reciente está en
`build/tryouts/yamantaka-0.15.0+16-library-physics-final-windows-x64.zip`:
19.720.813 bytes, 20 archivos y SHA-256
`2ECF1EBE46AE269BA26F93525696A22B427AF6DCEEA1AB23653AC46CE064848F`.
El smoke se ejecutó sobre ese ZIP después de extraerlo, con dos avances de
página y cero conexiones observadas.

La hoja de ruta integral está en [NEXT_TODOS.md](NEXT_TODOS.md) y la estrategia
vigente de pruebas en [TESTING.md](TESTING.md).

## Acabado editorial 0.15

- Biblioteca adopta el tema global claro/oscuro, navegación lateral adaptativa,
  búsqueda etiquetada y portadas tipográficas con retícula estable.
- Continuar leyendo conserva una acción principal; la sesión reciente queda
  en un menú discreto para dejar espacio al catálogo sin perder recuperación
  ni cola.
- El lector separa el tema de sus controles del lienzo PDF y mejora navegación,
  nombres de página, paneles y filtros de anotaciones con texto ampliado.
- Ajustes ofrece una vista previa mayor y un ancho de trabajo contenido.
- Menús redistribuyen atajos al 200 %; reglas y duplicados usan textos humanos.

La simplificación posterior se documenta en
[auditoría de Biblioteca](docs/architecture/library-simplification-audit.md).

Decisiones y pruebas: [registro de la iteración](docs/architecture/visual-iteration-0.15.md).

## Base de coherencia visual y UX 0.14

- Iconos, targets, bordes, opacidades y curvas consumen tokens compartidos; los
  breakpoints 680/1100 y el viewport de PDFium no cambian.
- Contraseña, recuperación, restore, duplicados y operaciones largas comparten
  cabecera, contenido desplazable, acciones adaptativas, fase, progreso y
  cancelación accesible.
- La explicación de contraseña en memoria permanece visible durante el error y
  las acciones siguen alcanzables a 640×480 con texto al 200 %.
- Ajustes se ordena por intención: Apariencia, Lectura, Accesibilidad,
  Biblioteca, Privacidad, Almacenamiento, Recuperación y Diagnóstico.
- La matriz ejecutada cubre estados de control, ES/EN, RTL, oscuro, alto
  contraste, movimiento reducido, texto 130/150/200 % y ocho tamaños base.

## Resiliencia y confianza local 0.13

- La apertura del documento tiene un ciclo de vida explícito y generacional;
  una apertura obsoleta no publica estado y libera cualquier handle tardío.
- Los PDF protegidos admiten tres intentos. La contraseña solo vive en memoria
  durante cada intento y no entra en SQLite, snapshots, logs ni diagnósticos.
- La recuperación valida identidad y páginas y pone en cuarentena el documento
  que causó un fallo, evitando reabrirlo automáticamente en el siguiente inicio.
- El backup v9 muestra un plan de altas, reemplazos, eliminaciones y conflictos
  antes de escribir, conserva hashes y esquema, crea pre-backup y restaura en
  transacción. No incluye PDF ni implementa criptografía propia.
- La revisión de duplicados separa coincidencias exactas y probables, muestra
  evidencias e impacto, resuelve cada conflicto, conserva procedencia de
  anotaciones y permite deshacer mediante la copia previa. Nunca borra un PDF.
- Los journals declaran fase, causa, compatibilidad y si una operación permite
  reintentar, reanudar o deshacer; la pantalla Problemas solo ofrece acciones
  respaldadas por datos suficientes.
- SQLite v11 conserva migraciones idempotentes verificadas desde cada esquema
  v1–v11. La evidencia visual cubre contraseña, archivo ausente, consolidación,
  restore, problemas y recuperación, incluida la consolidación a texto 200 %.

## Continuidad y revisión 0.12

- El inicio reúne lectura activa, pestañas, fijados, pendientes, espacios y
  problemas sin recorrer archivos ni abrir PDF durante el primer frame.
- Los espacios guardan consulta, agrupación, orden, densidad, columnas, pestañas,
  documento activo, modo Foco y una cola reordenable con estados y siguiente
  acción; crear, renombrar, duplicar y eliminar con deshacer son operaciones
  locales.
- El lector ofrece modo Foco por espacio y comparación lado a lado con página y
  zoom sincronizables, intercambio de lados y diferencias marcadas manualmente.
- La revisión global de anotaciones filtra y agrupa, edita notas con guardado
  automático, aplica estado/etiqueta en lote y exporta Markdown o JSON.
- Relaciones direccionales, backlinks y grupos de revisiones conservan siempre
  cada archivo; nunca fusionan ni eliminan PDF automáticamente.
- Las reglas locales tienen editor guiado, simulación, confirmación, lotes de 80,
  cancelación, journal y deshacer. No ejecutan scripts, red ni cambios en el PDF.
- La sesión se persiste atómicamente. Tras una interrupción puede restaurarse o
  descartarse y el centro no modal de problemas permite comprobar SQLite,
  reanudar o descartar operaciones.

Atajos y recuperación se documentan en [docs/SHORTCUTS.md](docs/SHORTCUTS.md) y
[docs/backup-and-recovery.md](docs/backup-and-recovery.md).

## Lector

- `Ctrl+K` permite elegir Una página, Dos páginas y Continua desde la paleta de
  comandos, sin ocupar permanentemente la barra; conserva la página activa y
  la disposición por PDF.
- Dos páginas navega por pliegos completos y permite ajustar por documento si
  la portada queda sola y si la primera página comienza a izquierda o derecha;
  el emparejado respeta LTR/RTL, páginas pares/impares y la última sin pareja.
- Las páginas del pliego son contiguas y comparten un canal central ligado a
  sus bordes, por lo que el lomo acompaña zoom y paneo.
- Los tres modos comparten los seis entornos de lectura; el tema se aplica
  al canvas y al chrome sin alterar los píxeles del PDF.
- La barra superior mide 56 px (52 px en compacto), deja 8 px superiores y usa
  targets de 40 px con iconos de 18 px; título y herramientas comparten una sola
  superficie sin reservar una franja en el visor.
- En ventanas expandidas el panel lateral ocupa una columna propia y el visor
  se recentra en el ancho restante y usa navegación interna vertical; en
  compacto/medio es overlay con scrim y navegación superior.
- La dock mide 48 px activa y 38 px en reposo; clic, toque, foco o teclado revelan
  navegación y zoom sin alterar el centro del PDF.
- Los laterales ofrecen zonas claras de paso de página, con estado deshabilitado
  y soporte semántico.
- Flechas, Page Up/Down, Home/End, espacio, F11 y Escape comparten la misma
  navegación canónica.
- F11 usa una transición Win32 sin frame, conserva la geometría anterior y no
  guarda como normales las dimensiones fullscreen.
- Movimiento reducido, alto contraste y perfil de tinta electrónica tienen
  rutas específicas.

## Otras funciones

- Apertura mediante selector nativo, arrastrar y soltar uno o varios PDF, o
  argumentos múltiples; las rutas repetidas se descartan sin perder el orden.
- “Continuar leyendo” aparece solo cuando existe una lectura reciente sin
  terminar y resume con título, página, total, porcentaje y última apertura.
- `Ctrl+J` abre un navegador documental único para índice, miniaturas, búsqueda,
  marcadores, anotaciones, historial y posiciones A/B; `Ctrl+Mayús+J` mantiene
  el acceso directo al panel. Los saltos no reconstruyen el visor ni cambian el
  zoom y `Alt+←` vuelve al origen.
- Los documentos cerrados recientemente restauran página, zoom, layout, modo,
  tema, scroll y panel. Continuar, Releer y Terminado son acciones explícitas;
  una meta opcional por páginas o minutos permanece solo en el dispositivo. El
  menú del lector expone además `Cerrar sesión y conservar contexto`.
- `Ctrl+Tab` y `Ctrl+Shift+Tab` abren un conmutador compacto de documentos; dos
  documentos PDFium permanecen calientes como máximo y los demás se suspenden
  conservando página, zoom, layout, modo, scroll, tema y panel.
- Biblioteca local con portadas, filtros combinables por autor, tema, estado y
  favorito, agrupación plegable por autor, colección, etiqueta o estado,
  colecciones, etiquetas, vistas inteligentes,
  selección múltiple y operaciones por lote.
- La virtualización de Biblioteca eleva la prioridad de portadas visibles y
  cancela decodificaciones recreables al salir del viewport; el diagnóstico
  local expone intentos, cancelaciones y decodificaciones evitadas sin guardar
  rutas, títulos ni identificadores.
- La biblioteca alterna una cuadrícula con títulos y una lista densa para
  comparar documento, estado, progreso y última apertura. A 150 % de texto la
  lista reduce columnas y conserva estado, páginas y progreso en cada fila.
- El estado inicial muestra únicamente el símbolo de Yamantaka, el nombre, una
  frase breve y `Abrir PDF`. El símbolo conserva exactamente el dibujo original
  y sustituye el azul por el violeta `#B58AFF` mediante recoloración 1:1.
- Título, autor y hasta tres temas se proponen mediante metadatos y texto PDF
  procesados localmente; los resultados dudosos se revisan y las ediciones
  manuales prevalecen siempre.
- El menú de cada documento y el menú general permiten volver a analizar
  metadatos y temas sin perder ninguna corrección manual.
- El lector muestra una previsualización rápida y la sustituye por la región
  nítida al estabilizar zoom/desplazamiento; en Continua solo agenda páginas
  visibles o próximas, incluso en documentos de miles de páginas.
- Biblioteca comparte el tema global: papel cálido en claro y negro con grises
  neutros en oscuro; mantiene selección violeta, foco independiente del color y
  controles superiores discretos.
- La selección abre un inspector contextual con estado icónico, progreso y
  `Abrir`, `Continuar`, `Releer` o `Localizar`; al filtrar conserva el documento
  o explica su exclusión.
- “Abrir PDF” es la acción primaria y los filtros activos ofrecen un
  restablecimiento visible sin volver a abrir el menú.
- Búsqueda PDF cancelable, índice, miniaturas y panel lateral redimensionable.
- Marcadores y anotaciones comparten filtro y orden por página; saltar a uno
  anuncia y enfoca temporalmente el destino. Se conservan notas por página,
  historial local opcional y navegación atrás/adelante.
- La selección real del texto permite resaltar, subrayar o tachar con geometría
  normalizada. Si el motor no entrega geometría fiable se crea y anuncia una
  nota de página, sin inventar coordenadas.
- Paleta de comandos con iconos específicos, metadatos, preview expandida,
  documentos recientes y atajos personalizables con detección de conflictos.
- Recuperación opt-in de documento, página, zoom, disposición, panel, chrome y
  geometría de ventana.
- Copia de seguridad JSON versionada, validada y transaccional; nunca incluye
  los PDF.
- Búsqueda global FTS5 opt-in y reanudable con filtros de documento, página, anotación,
  marcador, colección y etiqueta, relevancia con recencia solo como desempate,
  cancelación, diagnósticos redactados y safe mode conservador.
- La búsqueda global usa lista y preview en ancho expandido; selección,
  indexación y carga tienen texto de estado y no dependen de un spinner aislado.
- El editor de metadatos separa valores detectados de overrides locales,
  identifica cambios sin guardar y permite restaurar cada campo sin tocar el
  PDF original.
- Confirmación de enlaces externos y bloqueo de esquemas no seguros.
- Sistema visual monocromático con Segoe UI Variable, temas claro/oscuro,
  densidad cómoda/compacta y breakpoints en 680/1100 px.
- Ajustes usa master-detail agrupado, búsqueda de preferencias, reset por
  sección y una previsualización viva persistente del canvas, la página y el
  chrome, alternable entre reposo, activo, panel y e-ink.
- La ventana se abre maximizada y conserva su geometría normal para restaurarla.

## Referencia visual vigente

![Modo Libro editorial 0.15 mantenido por prueba](docs/screenshots/windows-reader-editorial-v015.png)

![Navegador documental 0.15 mantenido por prueba](docs/screenshots/windows-document-navigator-v015.png)

Esta imagen se genera desde widgets ejecutados a 1600×900 y se compara en la
suite. No es una captura nativa ni reutiliza estados de versiones anteriores.
La matriz actual 0.15 reúne 193 PNG en 21 hojas de contacto. Cubre Biblioteca,
Lector, navegador, panel, Ajustes, paleta, búsqueda, menús, bienvenida e
importación y añade contraseña normal/incorrecta, archivo ausente, consolidación
a texto 200 %, ensayo de restore, problemas y recuperación de sesión. Las once
comparaciones 0.14/0.15 registran el hash de ambas fuentes. La base se conserva
en `build/visual-evidence/v0.14.0/`, la actual en
`build/visual-evidence/v0.15.0/` y la revisión completa en
`build/visual-review-015/`. Cinco PNG proceden del recorrido nativo de integración
con PDFium real y datos aislados; tampoco certifican DPI físico ni Narrator.
La identidad del logo violeta se comprueba por hash;
no se regenera ni se altera su geometría.

## Requisitos

- Windows 10/11 de 64 bits.
- Flutter estable 3.44.8 o compatible y Dart 3.12.
- Visual Studio 2022 con Desktop development with C++.
- Modo de desarrollador de Windows para los enlaces simbólicos de plugins.

El SDK local .flutter-sdk está ignorado por Git y no forma parte del producto.

## Preparación y compilación local

    flutter doctor -v
    flutter pub get
    flutter build windows --release --no-pub

Este comando solo escribe artefactos bajo `build/`, que está ignorado por Git:
no publica Yamantaka en pub.dev, Microsoft Store ni ningún otro canal.

El ejecutable de Flutter no es autónomo: debe permanecer junto a `data/`,
`flutter_windows.dll` y las DLL de plugins. Si se usa el ZIP, hay que extraer el
bundle completo antes de abrirlo; copiar o ejecutar solo `yamantaka.exe` puede
fallar por dependencias ausentes.

Para iniciar la build local con preflight de estructura, ruta y
Mark-of-the-Web:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/run_windows.ps1

También puede abrir un PDF directamente:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/run_windows.ps1 -PdfPath "C:\libros\ejemplo.pdf"

Las builds Flutter de Windows no incluyen por defecto una firma Authenticode.
El launcher acepta una build local completa situada bajo `build/`, pero nunca la
presenta como artefacto firmado. Smart App Control puede bloquear binarios
descargados o desconocidos; no se recomienda desactivar protecciones de Windows
para evitarlo.

Cuando exista un certificado RSA de firma de código de un proveedor confiable,
se firma el bundle completo —ejecutable y DLL— así:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/sign_windows_bundle.ps1 -BundlePath build/windows/x64/runner/Release -CertificateThumbprint <THUMBPRINT>

Una build firmada o la build local comprobada puede abrirse también como proceso
normal:

    & build/windows/x64/runner/Release/yamantaka.exe "C:\libros\ejemplo.pdf"

Firmar no publica la aplicación. El certificado y su clave se administran fuera
del repositorio. No se recomienda desactivar Smart App Control como solución de
desarrollo.

## Compilación y pruebas

    dart format --output=none --set-exit-if-changed lib test integration_test tool test_driver
    flutter analyze --no-pub
    flutter test --no-pub
    flutter build windows --release --no-pub

Gate reproducible:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/verify_windows.ps1

Los informes quedan bajo build/verification. El gate local puede ejecutar el
bundle que acaba de compilar cuando está completo, permanece bajo `build/` y no
tiene Mark-of-the-Web. La firma sigue siendo obligatoria para un gate de
distribución; con un certificado disponible:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/verify_windows.ps1 -CertificateThumbprint <THUMBPRINT> -RequireSignature

Sin certificado, `-RequireSignature` omite las fases que abrirían el bundle y
explica qué binarios carecen de firma. No desactiva ni modifica Smart App
Control.

PDF sintético para smoke:

    dart run tool/create_test_pdf.dart build/verification/corpus/normal.pdf

Matriz visual nativa sobre una escala real del host; use `-RequireSignature`
para evidencia de distribución:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/run_visual_matrix.ps1 -HostScalePercent 125 -ScaleConfirmed

Captura de una ventana ya abierta, conservando sus bounds y enviando F11:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tool/capture_windows_window.ps1 -TargetProcessId <PID> -OutputPath build/verification/book-fullscreen.png -PreserveWindowBounds -SendKeys "{F11}"

## Estructura

- lib/app: arranque, composición, tema, comandos y atajos.
- lib/core: errores, plataforma y logging privado.
- lib/features: biblioteca, lector, búsqueda, marcadores y ajustes.
- lib/pdf_engine: contrato del motor, modelos, visor y caché.
- lib/storage: SQLite, rutas, repositorios y backup local.
- test: dominio, widgets, SQLite, PDFium y referencia visual.
- integration_test: flujos nativos con PDF generado.
- tool: utilidades reproducibles de desarrollo y validación.
- third_party/pdfrx: parche local, aislado y licenciado de pdfrx 2.4.7.

## Limitaciones

- No hay OCR, IA, clasificación remota ni edición o firma del PDF. Los PDF
  escaneados sin capa de texto conservan el nombre de archivo hasta una edición
  manual.
- Highlight, subrayado y tachado usan la geometría normalizada que entrega el
  adaptador nativo. Selecciones multipágina o sin geometría fiable degradan de
  forma anunciada a nota de página.
- Split/comparación conservan modelos internos, pero no se anuncian sin dos
  visores reales y medidos.
- Android e iOS están inicializados, pero esta iteración solo valida Windows.
- El build local actual no está firmado. Puede usarse para desarrollo desde el
  bundle completo bajo `build/`, pero distribuirlo sin advertencias requiere un
  certificado RSA confiable o un canal que firme el artefacto.
- El gate actual registra 96 DPI/100 % comunicados al proceso, pero falla la
  aceptación de la matriz porque esa escala no tiene confirmación física
  independiente. Narrator, varios monitores, en-US nativo, alto contraste real
  y las escalas físicas no verificadas siguen pendientes.

No se incluyen nube, cuentas, telemetría, IA, EPUB ni formatos distintos de
PDF.
