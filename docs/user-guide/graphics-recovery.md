# Graphics startup fallback

Yamantaka normally starts with its DirectX 12 / wgpu compositor. If that compositor cannot initialize, the application releases the failed attempt and starts a fresh process with the alternate Glow / OpenGL compositor. It tries that route once.

The retry preserves command-line arguments as Windows strings, including file paths with spaces and Unicode characters. It does not rewrite your graphics preferences or disable wgpu permanently. A later normal launch tries the default compositor again. Startup history and safe-mode decisions continue to use their existing policy; a failed start is not marked as a clean application shutdown.

To choose the alternate compositor yourself, close the running app and launch:

```powershell
.\yamantaka-rust.exe --opengl
```

`--software` is an alias for this alternate compositor; it does not promise CPU-only rasterization or a software OpenGL driver. PDFium still renders the PDF, and OpenGL composes the page and interface. The alternate route keeps whole-page rendering and CPU preview preparation; optional GPU crop refinement and nearby GPU preview residency use the default compositor.

A storage error, an error creating the application, or a failure of the alternate compositor is reported normally. It does not start a restart loop. This startup fallback does not yet recover a graphics device lost during an active reading session.

See the [validation checkpoint](../validation/rust-graphics-fallback-2026-10-31.md) for the exercised native cases and remaining recovery work.
