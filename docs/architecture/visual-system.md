# Sistema visual

La interfaz usa un vocabulario semántico único. Las superficies no deben
introducir colores, radios, sombras o duraciones ad hoc cuando existe un token
equivalente.

## Fuentes de verdad

| Categoría | Fuente | Regla |
| --- | --- | --- |
| Color | `lib/app/theme/app_tokens.dart` | Usar `context.appColors`; claro, oscuro y alto contraste conservan la misma semántica. |
| Espaciado, tamaños y radios | `AppSpacing`, `AppSizes`, `AppRadii` | Elegir el token más cercano; una excepción requiere motivo geométrico local. |
| Elevación y sombras | `AppElevation`, `AppShadow` | Superficies planas por defecto; sombras solo para overlays, páginas y portadas. |
| Movimiento | `AppMotion` | Pasar siempre por `whenAllowed`; reduced motion produce duración cero. |
| Densidad | `AppDensityTokens` | La densidad compacta reduce ritmo, no objetivos de interacción. |
| Layout | `AppBreakpoints`, `AppWindowClass` | Clasificar por espacio lógico disponible, no por resolución física aislada. |
| Tipografía | `AppTheme` | Segoe UI Variable/Segoe UI con fallback; los estilos tienen función semántica. |

## Excepciones permitidas

- El fondo y la sombra de la página PDF pueden variar con el tema para mantener
  legibilidad del papel sin cambiar el contenido.
- Una portada puede reservar un tamaño físico distinto al de una fila cuando la
  composición lo necesita; el tamaño sigue saliendo de `AppSizes` o de los
  tokens de densidad.
- Los colores `danger`, `warning` y `success` son estados semánticos y siempre
  deben acompañarse de texto, icono o borde; nunca son el único indicador.

## Contrato de accesibilidad

- El foco usa `focusRing` y un borde de 2 px en los controles globales.
- Los objetivos de escritorio conservan al menos 40×40 logical px; el tamaño
  táctil disponible es 48×48.
- El texto normal y los componentes no textuales se validan en
  `test/accessibility_audit_test.dart`.
- La matriz de color alto contraste reemplaza los tokens por blanco/negro y
  foco fuerte sin depender de un tema personalizado.

Los cambios visuales deben actualizar este documento o justificar la excepción
en el mismo cambio. Las capturas y la matriz nativa siguen siendo evidencia
externa: este contrato no las sustituye.
