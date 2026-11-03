# Abstract open-folio mark

The owner requested a more abstract, original black logo. Two angular leaves now surround an open central space, suggesting an open folio in perspective. The symmetrical silhouette replaces the offset page folds. The paper tile and existing interface colors remain unchanged.

The canonical SVG generates all 28 existing PNG and ICO assets. The publication guard's exact binary allowlist was updated from the reviewed generated manifest. No external artwork, fonts or dependencies were added.

Executed checks:

- All generated assets match the SVG and manifest.
- The native offline debug build passes.
- Resource inspection confirms all ten Windows executable icon sizes match the ICO.
- Native before, dark and light library captures complete at the same actual 1920 by 1200 dimensions. Their header regions were compared at original resolution.
- The size sheet was inspected at 320, 192, 96, 48, 32, 24 and 16 pixels.
- The five existing publication-guard checks pass. No standalone tests were added.

These checks do not verify cached Windows shortcuts or physical display scaling. The final rewrite Release remains pending its broader acceptance gates.
