# Abstract monochrome application identity

The owner requested a more abstract black symbol after reviewing the violet open-book mark. Two offset page folds now form the canonical mark in `assets/branding/yamantaka-logo.svg`. A warm paper tile keeps the black geometry visible on both light and dark Windows surfaces. The interface's functional violet remains unchanged.

`tool/generate_brand_assets.py` generates the existing 28 PNG and ICO variants from that SVG. The Windows ICO contains ten sizes from 16 to 256 pixels. The historical `yamantaka-logo-purple.png` filename remains because both Flutter and Rust reference it; its generated pixels are now monochrome. Historical mobile and web assets remain consistent without activating those platforms.

## Executed branding checks

- All 28 generated assets match the canonical SVG and manifest.
- The five existing publication-guard checks pass, including rejection of unreviewed replacement icon bytes.
- The native debug build passes with the existing lightweight development profile.
- Resource inspection of that executable confirms exact agreement with all ten ICO frames.
- Native synthetic-library captures exit successfully in light and dark modes. The header mark is visible against both themes. The earlier and current dark headers were compared at the same 1800 by 1200 capture dimensions; their language labels differ.
- The raster size sheet was reviewed at 320, 192, 96, 48, 32, 24 and 16 pixels.

[Dark library](../screenshots/rust-abstract-brand/dark.png) · [Light library](../screenshots/rust-abstract-brand/light.png)

The executable was built from the working tree, which also includes ongoing Release-packaging changes. This branding check does not establish acceptance of that separate work, physical DPI coverage, cached Windows shortcuts or completion of the rewrite. No new standalone tests were added.

## Flutter cleanup

`tool/prune_flutter_cache.ps1` previews a fixed list of generated Dart/Flutter paths. `-Apply` removes them, and `-IncludeSdk` also selects the downloaded workspace-only Flutter SDK and archive. The script refuses tracked files, running Dart/Flutter tools, linked directories and paths outside the workspace. It preserves the Dart source, lockfile at the repository root, migration baseline, native Rust output and application libraries.

The executed cleanup removed seven entries totalling 5,158,188,795 file bytes, approximately 5.16 GB. A second preview found no remaining candidates. These are logical file sizes rather than a measurement of physical NTFS allocation. Running the historical Flutter baseline again requires downloading its SDK and restoring generated dependencies.

The existing native checkpoint pruner also removed two old executables totalling 130,543,616 file bytes. It retained the newest checkpoints and skipped a locked executable.

To preview or repeat the cleanup:

```powershell
./tool/prune_flutter_cache.ps1
./tool/prune_flutter_cache.ps1 -IncludeSdk -Apply
```

This cleanup does not retire the Flutter source. The rewrite's parity and migration acceptance gates remain open.
