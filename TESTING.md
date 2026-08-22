# Pruebas

Estado actualizado: 2026-08-02.

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

- Documento, Libro y restauración de preferencias;
- matriz de paridad de acciones comunes entre Documento y Libro en 640 y
  1280 px;
- pliegos 1–2, 3–4 y final impar, además de paridad explícita y RTL;
- ajuste de pliego, aislamiento de unidades, centrado y límites de paneo;
- navegación por botones, flechas, Page Up/Down, Home/End y espacio;
- exclusión de atajos mientras un campo de texto tiene foco;
- F11, Escape y separación entre fullscreen, chrome e inmersivo;
- superficie superior opaca y ausencia de degradado transparente;
- selector de modo y navegación inferior centrados respecto a la ventana;
- composición compacta, media y expandida sin overflow;
- alto contraste, movimiento reducido y perfil de tinta electrónica;
- panel lateral, búsqueda, marcadores, temas de Documento y persistencia.

## Referencia visual mantenida

La única captura normativa es:

    docs/screenshots/windows-reader-book-fullscreen.png

La genera y compara esta prueba:

    flutter test --no-pub test/reader_widgets_test.dart --plain-name "book reader matches the maintained fullscreen baseline"

Para aceptar deliberadamente un cambio visual:

    flutter test --no-pub --update-goldens test/reader_widgets_test.dart --plain-name "book reader matches the maintained fullscreen baseline"

Es una referencia de widgets ejecutados a 1600×900, no una captura nativa ni un
mockup manual. Valida la composición fullscreen del contenido: barra opaca,
pliego enfrentado, canal central, controles laterales, selector superior
centrado, navegación inferior centrada y acciones de vista a la derecha.

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
