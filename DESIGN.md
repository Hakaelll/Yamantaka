# Diseño de Yamantaka

## Principios

El PDF es el protagonista. La interfaz es silenciosa, sobria, precisa en modo
documento y cálida en modo libro. La jerarquía se obtiene con espacio,
tipografía y contraste; no con tarjetas, sombras o adornos.

## Sistema base

- Unidad de espacio: 4 px. Escala: 4, 8, 12, 16, 24, 32 y 48.
- Tipografía: familia del sistema; 12 px auxiliar, 14 px controles, 16 px cuerpo,
  20 px título y 28 px llamada principal.
- Radios: 4 px en controles compactos y 8 px en diálogos. Sin píldoras
  decorativas.
- Bordes: 1 px, bajo contraste. Elevación solo para paneles flotantes.
- Objetivos interactivos: mínimo 40 × 40 px en escritorio y preparados para
  48 × 48 px en táctil.
- Foco: contorno de 2 px con el acento, siempre visible al usar teclado.

## Paleta de aplicación

- Superficie: `#F4F3EF`.
- Superficie elevada: `#FCFBF8`.
- Texto principal: `#252522`.
- Texto secundario: `#66645E`.
- Borde: `#D8D5CE`.
- Acento: `#675E4A`.
- Error: `#A33A32`.

La variante oscura usa `#1D1D1B`, `#262624`, `#ECEAE4`, `#AAA79F` y
`#4A4944`. El color nunca es el único portador de estado.

## Entornos de lectura

Los temas solo cambian el lienzo alrededor del PDF:

| Tema | Fondo |
| --- | --- |
| Claro | `#F5F5F3` |
| Marfil | `#ECE7D8` |
| Sepia | `#D8CBB4` |
| Gris | `#B9BAB7` |
| Oscuro | `#242422` |
| Negro | `#000000` |

No se modifica ni se filtra destructivamente la página.

## Biblioteca

La acción dominante es “Continuar leyendo”, seguida de “Abrir PDF”. Los
recientes forman una lista densa con miniatura, nombre, progreso, última
lectura y estado de existencia. El estado vacío explica una sola acción.

## Lector

La barra superior tiene 48 px: volver, nombre, búsqueda, modo y menú. El
contenido ocupa el resto. Una barra inferior de 3 px expresa progreso sin
competir con la página. El panel lateral, de 304 px, muestra una sola sección y
se superpone cuando el ancho es compacto.

En modo libro la página se centra, los márgenes son configurables y los
controles se ocultan tras una pausa. Una zona lateral de 64 px avanza o retrocede.
En vista enfrentada la portada queda sola; después se forman pares. Si cada
página quedaría por debajo de un ancho legible, se usa una página.

## Tamaños adaptables

- Compacto: menos de 600 px; una página, panel superpuesto, controles inferiores.
- Medio: 600–999 px; una página y panel opcional.
- Expandido: desde 900 px; panel y dos páginas cuando sea legible.

La acción de lectura es independiente de teclado, ratón o gesto.

## Movimiento y tinta electrónica

Transiciones de 120 ms para aparición y 160 ms para paneles. No hay desenfoque
ni transparencias imprescindibles. El perfil de movimiento reducido usa
duración cero; el contrato visual admite alto contraste, blanco y negro,
controles grandes y navegación sin desplazamiento.

El perfil `electronicInk` ya es un valor tipado y seleccionable. En este hito
activa página única, fondo claro y ausencia de animación; alto contraste
forzado, refresco completo y botones físicos pertenecen a la fase específica.

## Accesibilidad

Orden de tabulación acorde a la lectura, etiquetas semánticas en iconos,
mensajes textuales, contraste WCAG AA en controles y compatibilidad con el
escalado del sistema. Todo flujo principal tiene alternativa de teclado.
