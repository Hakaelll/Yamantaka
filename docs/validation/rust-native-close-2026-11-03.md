# Serial native window-close checks

Three synthetic launches of the frozen debug runtime were closed one at a time through the native window-close request. Each process was awaited before the next launch, with cleanup of that owned child on any failure. All three exited normally with code 0, without forced cleanup; their stderr logs were empty.

The launch configurations request the PDF-cover gallery, the reader and a protected document. An actual Yamantaka window was observed before closing. This exercises early window closure; its title alone does not prove that a page or password modal had finished opening.

The [result artifact](artifacts/native-close-lifecycle.json) binds the executions to the same executable hash as the selection-recovery checkpoint. All 42 generated source PDF hashes remain unchanged. The processes were confirmed terminal. Raw logs stay local under `build/verification/close-lifecycle`.

The previous accumulation of interactive test previews is handled by `tool/open_rust_preview.ps1`, which replaces managed synthetic previews rather than leaving them open. Regular captures already wait for their process and clean up their own child on timeout.

No application-code change was needed for these successful close cases. This does not certify every shutdown race, completed-reader or modal state, prolonged resource retention, driver cleanup, power-loss durability or Release performance. The broader acceptance matrix remains open.
