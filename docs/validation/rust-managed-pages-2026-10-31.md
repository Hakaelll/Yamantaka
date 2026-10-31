# Alternate-compositor PDF memory checkpoint

Date: 2026-10-31. Scope: egui-managed PDF page and thumbnail uploads in the Glow compositor. This does not cover all application or driver memory.

## Connected behavior

The common page-delivery path now admits two costs before constructing a managed image: RGBA GPU payload and the temporary `ColorImage` copy. Both use the page or tile category of the existing global governor. Pixel length, nonzero dimensions and the current device limit are validated before allocation. Admission failures preserve their typed memory-budget error and do not create texture metadata.

The temporary-copy lease follows a weak reference to egui's actual image allocation. Visible frames reclaim it only after all strong image references disappear. The managed texture owns its GPU lease. Dropping that owner first queues egui's texture deletion, then queues its charge for retirement. A later painted-frame boundary inserts an OpenGL fence after earlier painting and texture frees. Zero-timeout polling retains the charge until the fence signals. Pending work requests another frame; settled accounting does not request continuous repaint.

A fence failure retains charges conservatively and reports the error. Shutdown deletes outstanding sync objects and retains their charges until eframe destroys its painter and drops the application. This follows the pinned eframe 0.36.2 teardown order. Native wgpu uploads keep their existing payload/staging admission and fence ownership.

## Executed evidence

- Formatting, offline UI Clippy with warnings denied and the offline native application build passed.
- The retained `page_presenter::tests::atomic_upload_admission_includes_live_cpu_and_row_padding` test was extended and passed. A real managed upload with an eight-byte CPU raster and a 23-byte total budget rejects the additional eight-byte GPU payload and eight-byte copy before texture creation. Texture metadata stays unchanged; GPU/transient charges remain zero and dropping the source restores total usage to zero. No standalone test was added.
- Four native profiles exercise high-zoom Glow reading, Glow thumbnails, a runtime device-fault restart into Glow and ordinary wgpu reading. All settled without pending image copies or retirement fences. Glow cases observed completed texture retirement; their temporary-copy usage returned to zero.
- The high-zoom single-page case charged exactly 14,678,832 GPU bytes, matching its installed raster dimensions. The thumbnail case retained three visible thumbnail slots and charged 3,516,328 GPU bytes: 3,255,208 for the main image plus 261,120 for thumbnails. These are governed payload counters, not total VRAM or a performance measurement.

Artifacts, source and executable hashes are recorded in [the capture manifest](../screenshots/rust-managed-pages/capture-manifest.json). Captures use generated PDFs and contain no personal documents.

## Remaining acceptance

GPU cover textures, UI/font atlases, portions of live text and opaque driver allocations remain outside this scope. Hidden-window reclamation is conservative until a visible frame; no hidden-frame GL work is introduced. Physical driver/fence failure, repeated pressure/close cycles, process-wide resource measurements and paired performance acceptance remain pending. This checkpoint does not establish a p95 frame budget or complete memory accounting for the whole application.
