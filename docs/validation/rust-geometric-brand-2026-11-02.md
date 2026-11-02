# Geometric application identity

The owner requested a corporate symbol connected to the application and rejected the initial monogram. The canonical SVG now depicts an open volume: two geometric page planes, a clear binding gap and a separate lower page layer. It retains the existing violet and white palette.

The generator updates all 28 existing PNG and ICO variants. Native Windows resources embed ten icon sizes, from 16 to 256 pixels, together with application version information. Historical mobile and web assets use the same symbol without enabling those targets.

Executed checks:

- Deterministic generation check: all 28 assets match the canonical source.
- Publication guard tests: five passed, including rejection of unapproved replacement icon bytes.
- Native Windows debug build: passed.
- Executable resource inspection: all ten embedded icon frames match the generated ICO.
- Native synthetic library captures in light and dark themes: reviewed. The header preserves the icon silhouette.
- Raster size sheet reviewed at 192, 64, 32, 24 and 16 pixels.

[Dark library](../screenshots/rust-geometric-brand/dark.png) · [Light library](../screenshots/rust-geometric-brand/light.png)

GitHub's short repository description was set to: “A private PDF library for reading, study, and annotation on Windows.”

The executable inspected was built from the working tree, which also contains ongoing duplicate-consolidation changes. These branding checks do not establish acceptance of that separate feature or completion of the rewrite. Existing pinned Windows shortcuts and physical DPI configurations were not manually inspected.
