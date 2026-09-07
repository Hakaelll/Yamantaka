# Yamantaka — rediseño visual y acabado de producto profesional

Trabaja directamente sobre el estado actual de Yamantaka y lleva su interfaz a
un nivel de acabado digno de una aplicación de escritorio consolidada, fiable y
agradable para usuarios exigentes que leen y organizan documentos a diario.

Asume conjuntamente la responsabilidad de diseñador de producto, desarrollador
de interfaz y revisor de calidad. Audita, decide, implementa, abre la aplicación,
comprueba los resultados y corrige lo que siga pareciendo inacabado. La entrega
es la aplicación mejorada, no únicamente un informe o una propuesta.

## 1. Resultado exigido

La mejora debe ser sustancial y reconocible al comparar el antes y el después:
mejor composición, jerarquía, legibilidad, navegación, presentación de los
documentos y coherencia entre pantallas. Cambiar algunos colores, radios o
sombras no satisface por sí solo este encargo.

La primera impresión debe transmitir un producto cuidado. Durante el uso deben
confirmarlo los detalles: controles alineados, estados claros, transiciones
estables, textos comprensibles y flujos completos con ratón y teclado.

Biblioteca, lector, Ajustes, ventanas, diálogos, paneles, menús, tooltips,
formularios y estados transitorios deben parecer diseñados por el mismo equipo.
La calidad debe mantenerse fuera de las escenas ideales y de las capturas de
presentación.

Toma decisiones de diseño concretas y justifícalas brevemente. Puedes cambiar
composición, distribución, componentes y navegación dentro de las funciones
existentes. No tienes que conservar una solución visual solo porque ya exista.

## 2. Punto de partida, permisos y límites

- Lee las instrucciones del repositorio y `README.md`, `DESIGN.md`,
  `ARCHITECTURE.md`, `TESTING.md`, `CHANGELOG.md` y `NEXT_TODOS.md`.
- Comprueba versión, diff, pruebas y comandos actuales. No copies cifras de
  pruebas ni resultados de versiones anteriores como si fueran comprobaciones
  de esta iteración.
- Puedes modificar o sustituir cambios del diff cuando sea necesario para este
  trabajo. Antes conserva una referencia recuperable del estado inicial,
  incluidos los archivos nuevos pertinentes. No uses un reset global ni
  descartes trabajo ajeno al objetivo.
- Mantén Flutter y la arquitectura existente. Reutiliza los componentes,
  controladores y repositorios; refactoriza los que impidan la coherencia.
- Conserva el logotipo y la identidad editorial de Yamantaka.
- Mantén el funcionamiento local-first y offline: sin cuentas, sincronización,
  telemetría, OCR, IA, servicios remotos ni modificación del contenido de los PDF.
- Preserva el ciclo documental seguro frente a carreras, las contraseñas solo
  en memoria, recuperación, backup con ensayo y rollback, consolidación
  reversible de duplicados, SQLite v11 y el límite de 80 documentos materializados.
- No añadas funcionalidades de negocio ajenas al encargo ni dependencias sin
  una necesidad técnica demostrable.
- No hagas commit, tag, push, firma ni publicación. No alteres documentos ni
  datos personales para preparar una demostración: utiliza fixtures aislados.

## 3. Auditoría visual breve y accionable

Abre el ejecutable y comprueba qué versión y qué código representa. Si es
necesario, compila el árbol actual para no revisar una release antigua como si
incluyera los cambios presentes.

Recorre Biblioteca, apertura de PDF, lectura, búsqueda, panel lateral, retorno a
Biblioteca, Ajustes y diálogos importantes. Captura una referencia antes de
editar. Revisa tanto ventanas compactas como un tamaño de escritorio habitual.

Registra los hallazgos en una tabla breve: superficie, defecto observable,
consecuencia para el usuario, cambio propuesto y evidencia para comprobarlo.
Distingue defectos confirmados, hipótesis y limitaciones de las herramientas.

Comprueba especialmente estos problemas presentes en las referencias de 0.14,
sin volver a introducirlos si el árbol actual ya los ha resuelto:

- Biblioteca casi negra, con poca separación tonal y desconectada del tema de
  lector y Ajustes.
- Cabecera de continuidad demasiado extensa, acciones de retomar repetidas y
  contadores a cero que compiten con el catálogo.
- Búsqueda y organización concentradas en iconos o menús poco evidentes.
- Portadas de sustitución que parecen rectángulos vacíos con una inicial.
- Títulos pequeños y truncados, con una cuadrícula de alineación poco estable.
- Vista previa de Ajustes demasiado pequeña para apreciar los cambios y
  descripciones que utilizan términos internos como «canvas» o «chrome».

Las capturas nativas disponibles han mostrado regiones antiguas o duplicadas.
Si ocurre de nuevo, contrasta la ventana y el estado real con otra evidencia
fiable. No presentes ese artefacto como un defecto confirmado de la aplicación
ni como una captura válida del resultado.

La auditoría debe desembocar pronto en implementación y revisión visual.

## 4. Dirección artística: editorial, contemporánea y de escritorio

Adopta una dirección coherente de papel cálido, superficies claras, grafito en
oscuro y un acento violeta contenido relacionado con la identidad existente.
El documento aporta el contenido visual principal; el acento sirve para
selección, foco y acciones prioritarias.

Como punto de partida, evalúa fondos claros próximos a `#F7F6F2`, superficies
blancas, fondo oscuro próximo a `#181A20` y superficies oscuras diferenciadas.
Ajusta los valores mediante contraste y revisión real; no los trates como una
paleta aprobada sin comprobar.

Centraliza el sistema visual:

- Tipografía de interfaz apropiada para Windows. Texto principal normalmente
  de 14–15 px lógicos, metadatos de 12–13 cuando sean legibles y títulos de
  pantalla de 24–28. Define roles y una escala corta; respeta el escalado del sistema.
- Espaciado consistente sobre una base de 4/8 px, con ritmos claros entre
  contenido, grupos y secciones. Densidad cómoda y compacta bien resueltas.
- Controles con objetivos de interacción de al menos 40 px lógicos, iconos
  ópticamente equilibrados y alineación entre texto, iconos y campos.
- Superficies diferenciadas por función, bordes finos, radios moderados y
  sombras discretas donde expliquen elevación o superposición.
- Tokens semánticos compartidos para colores, medidas, estados y movimiento.
  Evita mantener sistemas visuales paralelos para Biblioteca y el resto.

Evita degradados decorativos, cristal excesivo, brillos, sombras pesadas,
tarjetas anidadas sin función, insignias para cualquier dato y espacios vacíos
que solo hagan parecer más grande la interfaz. Conserva las convenciones útiles
de Windows: barra de título, menús, foco, atajos, scroll y redimensionamiento.

## 5. Biblioteca: rediseñar su composición

Implementa una organización reconocible:

1. Cabecera contenida con identidad, título y una acción principal «Abrir PDF».
2. Navegación lateral en ancho suficiente para Biblioteca, En lectura,
   Favoritos, Terminados y colecciones; reutiliza los filtros existentes.
3. Búsqueda visible y etiquetada, con acceso por Ctrl+F. Diferencia claramente
   la búsqueda del catálogo y la búsqueda global cuando tengan alcances distintos.
4. Orden, filtros y vista agrupados junto al catálogo, con filtros activos
   comprensibles y una forma evidente de limpiarlos.
5. Una única zona de «Continuar leyendo», con documento, progreso y acción.
6. Catálogo como superficie dominante, tanto en cuadrícula como en lista.

La barra lateral es una propuesta concreta de partida: valida su utilidad y
adáptala a la anchura disponible. En ventanas estrechas conviértela en navegación
compacta accesible; no suprimas colecciones ni acciones por falta de espacio.

Reduce la continuidad a información que ayude a actuar. Los contadores sin
contenido no necesitan ocupar una franja permanente. Mantén acceso a cerrados
recientemente y recuperación mediante una presentación secundaria y predecible.
En 1280×720 con texto al 100 %, la primera fila de documentos debe quedar visible
con comodidad; la cabecera no debe consumir casi media ventana.

Cuida cada documento como una pieza editorial:

- Miniatura real nítida cuando exista, sin deformar ni recortar información
  relevante de la página.
- Sustitución tipográfica deliberada cuando no haya portada: título, autor si
  existe y una identificación discreta del formato. Debe parecer terminada.
- Retícula estable, sin rotaciones o desplazamientos aleatorios que dificulten
  comparar, seleccionar y recorrer documentos.
- Título legible, hasta dos líneas cuando convenga, metadatos secundarios y
  progreso discretos; acceso al título completo sin depender solo del hover.
- Selección, favorito, terminado y archivo ausente reconocibles sin ruido ni
  dependencia exclusiva del color.
- Lista con columnas alineadas y prioridades de contenido claras en compacto.
- Sin tarjetas que cambien de tamaño o desplacen a sus vecinas al seleccionarlas.

Preserva selección, filtros, foco y scroll al abrir y cerrar documentos. Las
acciones masivas deben aparecer al entrar en selección y dejar de competir con
la lectura cuando no se están utilizando.

Diseña también biblioteca vacía, carga, ausencia de resultados, títulos largos,
metadatos incompletos, bibliotecas grandes y documentos inaccesibles. Cada estado
debe explicar qué ocurre y ofrecer una siguiente acción útil.

## 6. Lector: calma visual y controles precisos

El lector debe ser la superficie más cuidada. Conserva el protagonismo, la
nitidez y la estabilidad del PDF, con separación sutil entre página y lienzo.

- Reorganiza la barra superior en grupos claros: regreso y documento;
  navegación entre documentos cuando proceda; búsqueda, panel y herramientas.
- Trata títulos largos y pestañas múltiples sin aplastar los controles ni
  introducir saltos de anchura. Evita mostrar pestañas redundantes sin necesidad.
- Compón la navegación inferior en grupos legibles: página, progreso, zoom y
  modo de lectura. La página actual debe poder localizarse y modificarse con
  facilidad; los iconos solos no deben ocultar funciones importantes.
- Distingue claramente modo Documento, Libro, continuo, fullscreen, inmersivo y
  Foco donde existan, con acceso comprensible y estados seleccionados visibles.
- En reposo, reduce la presencia de controles. Al recibir foco o interacción,
  muéstralos inmediatamente. No ocultes el elemento que tiene el foco ni hagas
  que el usuario persiga una barra que desaparece mientras la usa.
- Rediseña paneles de miniaturas, índice, búsqueda, marcadores y anotaciones
  con cabecera, selección, resultados y scroll consistentes. Mantén el
  redimensionamiento y no reconstruyas el visor al abrirlos o cerrarlos.
- Haz legibles resultados, fragmentos, contadores y acciones secundarias. Evita
  filas saturadas de chips, botones o textos cortados.
- Cuida apertura, contraseña, contraseña incorrecta, carga, cancelación,
  documento ausente y error. Conserva el contexto y una recuperación clara.

Comprueba la transición Biblioteca → lector → Biblioteca y la conservación de
posición, zoom y contexto. No introduzcas escalados visuales artificiales del
PDF ni ajustes que comprometan geometría, cachés o ciclo de vida del renderizado.

## 7. Ajustes y superficies secundarias

Mantén la organización por intención: Apariencia, Lectura, Accesibilidad,
Biblioteca, Privacidad, Almacenamiento, Recuperación y Diagnóstico.

Mejora la jerarquía de las secciones, el ancho de lectura y la alineación de
controles. Agrupa opciones relacionadas sin convertir cada fila en una tarjeta.
Al reducir el ancho, mueve el control debajo de su etiqueta cuando sea necesario.

La vista previa debe ser suficientemente grande para apreciar tema, página y
controles. Debe reflejar las preferencias reales. En compacto puede integrarse
en el flujo o contraerse con un acceso claro. Evita una columna grande dedicada
a una miniatura diminuta.

Haz coherente el tema de toda la aplicación y diferencia la apariencia de la
interfaz del color del lienzo de lectura. Explica con lenguaje sencillo cuándo
una preferencia es global y cuándo pertenece a un documento.

Aplica los mismos criterios a contraseña, recuperación, duplicados, backup y
restauración, comparación, relaciones, automatizaciones, importación, problemas
y diagnóstico. Para cada diálogo o panel:

- Título inequívoco, explicación breve y contenido bien agrupado.
- Acción primaria reconocible, secundarias discretas y destructivas explícitas.
- Ancho adecuado al contenido, scroll interno y acciones siempre alcanzables.
- Foco inicial lógico, orden de teclado y regreso del foco al cerrar.
- Escape cuando no implique perder trabajo sin protección; confirmación o
  deshacer según el riesgo real.
- Errores junto al origen y mensajes que indiquen cómo continuar.

Revisa todos los textos visibles en español e inglés: etiquetas, tooltips,
atajos, mensajes y plurales. Sustituye nombres internos, booleanos y claves
técnicas por explicaciones humanas. Los detalles técnicos pueden conservarse
en una sección expandible de diagnóstico cuando sean útiles.

## 8. Interacción, accesibilidad y adaptación

Define y aplica de manera común normal, hover, pulsado, foco, seleccionado,
desactivado, carga y error. Los estados desactivados deben seguir siendo
legibles y no confundirse con controles activos.

Usa movimiento breve únicamente para explicar cambios; respeta movimiento
reducido. No animes grandes superficies durante lectura o redimensionamiento.
Las operaciones largas deben informar de su fase o progreso real y ofrecer
cancelación cuando el motor la soporte; no inventes porcentajes.

Verifica teclado, semántica, contraste AA, alto contraste, ausencia de trampas
de foco y estados comprensibles sin color. Evita reducir la tipografía para
resolver desbordamientos: redistribuye, envuelve o habilita scroll.

Comprueba los tamaños 640×480, 800×600, 1280×720, 1600×900, 1920×1080,
1920×1200, 2560×1440 y 3440×1440. Revisa texto al 100, 130, 150 y 200 %, temas
claro y oscuro, español e inglés, dirección RTL y movimiento reducido. Combina
los casos de mayor riesgo; no hace falta generar un producto cartesiano sin valor.

En ventanas grandes evita estirar textos y separaciones hasta perder relación
visual. En pequeñas conserva contenido y acciones principales. Comprueba menús
junto a los bordes, paneles abiertos y diálogos con texto ampliado.

Distingue pruebas simuladas de DPI físico, Narrator y varios monitores. Realiza
las disponibles y declara las demás pendientes, sin presentarlas como aprobadas.

## 9. Implementación por hitos y revisión obligatoria

Trabaja en hitos verificables: sistema visual y Biblioteca; lector y paneles;
Ajustes y diálogos; revisión transversal. Al terminar cada hito, ejecuta las
comprobaciones pertinentes y observa la interfaz antes de extender el cambio.

Después de implementar realiza una pasada de revisión separada, con criterio
de usuario nuevo, no una autoevaluación basada únicamente en el diff:

1. Recorre los flujos completos con ratón y teclado.
2. Inspecciona las pantallas a resolución real y los casos de estrés.
3. Compara antes y después con el mismo contenido, tamaño, tema y escala.
4. Registra lo que aún transmite improvisación y corrígelo.
5. Vuelve a comprobar las superficies afectadas y sus estados vecinos.

Busca especialmente márgenes desiguales, separadores duplicados, iconos
descentrados, controles con alturas distintas, metadatos demasiado tenues,
scroll poco evidente, saltos de layout, tooltips incorrectos, textos internos y
espacios sin propósito. No des por revisada una pantalla porque su captura se
haya generado: ábrela y examínala.

## 10. Evidencia, funcionamiento y entrega

Conserva PNG originales del antes y del después, comparaciones lado a lado,
hojas de contacto, manifiesto de escenarios y hashes. Identifica claramente
capturas nativas, de widgets y simuladas. Los mocks pueden orientar decisiones,
pero no cuentan como prueba de una interfaz implementada.

Añade o adapta pruebas unitarias, de widgets, golden e integración sobre los
cambios reales. Una golden nueva solo es aceptable después de revisar su imagen;
no actualices referencias indiscriminadamente para ocultar regresiones.

Ejecuta formato, análisis, pruebas y el gate Windows completo del proyecto sin
omitir fases. Mide antes y después las operaciones afectadas: apertura,
primera página, navegación, paneles, resize, scroll y memoria al cerrar.
Usa cinco pasadas comparables y conserva resultados ruidosos como observaciones.
Considera regresión fiable cuando la mediana empeore más del 10 % y al menos
cuatro de cinco muestras superen el límite. No inventes una medición inicial
si no existe una referencia ejecutable comparable.

Actualiza la documentación con decisiones y comprobaciones reales. Si esta
iteración continúa 0.14.0+15, prepara 0.15.0+16; si el repositorio ya ha avanzado,
respeta su secuencia y documenta el punto de partida.

Genera una release local con el flujo existente: bundle, ZIP, manifiesto,
SHA-256, informe del gate, rendimiento, evidencia visual, SBOM y estado de firma.
Verifica el ejecutable exacto extraído del ZIP mediante el smoke offline. Esto
no autoriza firma, publicación ni cambios remotos.

La entrega debe incluir:

- Resumen breve de los cambios perceptibles y de los flujos mejorados.
- Enlaces a las comparaciones de Biblioteca, lector, Ajustes y diálogos.
- Pruebas ejecutadas, número exacto, análisis, gate y rendimiento reales.
- Archivos principales modificados y componentes compartidos consolidados.
- Ruta del ejecutable y del ZIP, tamaño y SHA-256; resultado del smoke.
- Limitaciones y verificaciones físicas pendientes, distinguidas de fallos.
- Mensaje de commit convencional listo para copiar, sin realizar el commit.

No declares acabado profesional únicamente porque las pruebas pasan. La
aceptación estética exige una mejora visible y una revisión explícita de
composición, legibilidad, consistencia, interacción y estados no ideales.

Continúa mientras queden defectos corregibles dentro del alcance. Si una
limitación real impide verificar o terminar algo, descríbela con precisión y
entrega lo comprobado sin presentar el conjunto como plenamente validado.

El resultado debe ser convincente al abrirlo, cómodo durante una lectura larga
y consistente al explorar sus funciones: una aplicación que parezca terminada
también cuando el usuario sale de la pantalla principal.
