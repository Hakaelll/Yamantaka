# Graphics recovery

Yamantaka normally starts with its DirectX 12 / wgpu compositor. If that compositor cannot initialize, the application releases the failed attempt and starts a fresh process with the alternate Glow / OpenGL compositor. It tries that route once.

The retry preserves command-line arguments as Windows strings, including file paths with spaces and Unicode characters. It does not rewrite your graphics preferences or disable wgpu permanently. A later normal launch tries the default compositor again. Startup history and safe-mode decisions continue to use their existing policy; a failed start is not marked as a clean application shutdown.

To choose the alternate compositor yourself, close the running app and launch:

```powershell
.\yamantaka-rust.exe --opengl
```

`--software` is an alias for this alternate compositor; it does not promise CPU-only rasterization or a software OpenGL driver. PDFium still renders the PDF, and OpenGL composes the page and interface. The alternate route keeps whole-page rendering and CPU preview preparation; optional GPU crop refinement and nearby GPU preview residency use the default compositor.

The alternate reader also supports comparison. Open a PDF, choose **Compare pages** from the reader options, then keep another page or choose a reference document. Page, zoom and position linking use the same controls in both compositors. [Alternate comparison validation](../validation/rust-alternate-comparison-2026-10-31.md) records the exercised flows.

The alternate compositor accounts for PDF pages, thumbnails and extracted cover textures, plus their temporary upload copies, against the PDF memory budget. Replaced textures retain their charge until GPU completion. The 3D library, manual cover picker and automatic-cover restoration use that same governed path. [Managed page validation](../validation/rust-managed-pages-2026-10-31.md) and [alternate-cover validation](../validation/rust-alternate-covers-2026-10-31.md) record the scope. UI/font textures and opaque driver allocations remain outside this accounting.

A storage error, an error creating the application, or a failure of the alternate compositor is reported normally. It does not start a restart loop.

If wgpu reports a device loss or an uncaptured rendering error during use, Yamantaka requests an orderly close. It confirms queued application commands, saves reading continuity and sessions, then releases the instance before trying the alternate compositor. A failed persistence checkpoint stops the automatic restart and reports an error.

This recovery attempts to restore the saved reading context even when normal session restoration is disabled, without changing that preference. Safe mode continues to suppress automatic restoration. Recovery of unsaved editor drafts and every in-flight tool workflow still needs further validation.

See the [startup checkpoint](../validation/rust-graphics-fallback-2026-10-31.md) and [runtime recovery checkpoint](../validation/rust-runtime-graphics-2026-10-31.md) for the exercised cases. Physical driver-loss acceptance remains pending.
