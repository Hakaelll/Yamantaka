# Pruebas

Estado actualizado: 2026-08-24.

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
  panel acoplado con ancho semántico visible.

La biblioteca cubre inspector contextual, selección conservada y explicación al
filtrar, además de metadatos PDF, cuatro páginas iniciales, título multilínea,
autor/editor explícito o dudoso, fallback de nombre legible, ausencia de texto,
temas ES/EN, límite de muestra, reanálisis y preservación manual, facetas
completas, migración 7→8 y backup v7 con ajustes de pliego.

El motor cubre PDFium real, contraseña, corrupción, cancelación previa,
generaciones obsoletas, cierre sin pendientes, reducción dinámica de LRU y
diagnóstico de rendimiento sin campos privados. El perfil Windows añade
documentos sintéticos de 1, 50, 500 y 5.000 páginas, vector pesado y raster
extremo; los resultados bajo `build/` no contienen rutas ni títulos.

## Referencia visual mantenida

La única captura normativa es:

    docs/screenshots/windows-reader-editorial-v08.png

La genera y compara esta prueba:

    flutter test --no-pub test/reader_widgets_test.dart --plain-name "book reader matches the maintained editorial v0.8 baseline"

Para aceptar deliberadamente un cambio visual:

    flutter test --no-pub --update-goldens test/reader_widgets_test.dart --plain-name "book reader matches the maintained editorial v0.8 baseline"

Es una referencia de widgets ejecutados a 1600×900, no una captura nativa ni un
mockup manual. Valida la composición fullscreen del contenido: barra opaca,
pliego enfrentado, canal central, controles laterales, navegación inferior
centrada y acciones de vista a la derecha sin un selector permanente.

Las matrices widget de Biblioteca, Lector y Ajustes cubren 640×480, 800×600,
1280×720, 1600×900, 1920×1080, 1920×1200, 2560×1440 y 3440×1440, además de
tema oscuro, alto contraste, movimiento reducido, chrome en reposo y panel
abierto. La evidencia previa se conserva en `build/visual-evidence/v0.5.0/` y
la de 0.6 se conserva en `build/visual-evidence/v0.6.0/`, la de 0.7 en
`build/visual-evidence/v0.7.0/` y la actual se escribe en
`build/visual-evidence/v0.8.0/`; ninguna se versiona.

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
