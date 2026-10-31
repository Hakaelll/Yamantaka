# Alternate-compositor comparison checkpoint

Date: 2026-10-31. Scope: comparison availability and governed rendering in the Glow compositor.

## Change

The comparison entry point and its reader-options control no longer require a native wgpu presenter. A loaded reader context remains required. Comparison already uses the shared page-delivery path, which now admits Glow GPU payloads and temporary image copies through the memory governor. No parallel comparison renderer, PDF worker or alternate navigation state was introduced.

Both panes retain independent reader surfaces, navigation and cancellation. Shared-document comparison keeps its separate positions without opening another native document. A selected reference uses the existing two-hot-document admission and password flow. Whole-page previews and refinement remain available in Glow; optional region refinement and nearby GPU residency retain their existing wgpu policy.

## Executed evidence

Offline formatting, UI Clippy with warnings denied and the native application build passed. Four native synthetic capture profiles exercise:

- Independent pages 0 and 1 of the same PDF.
- A distinct reference document with its thumbnail navigator.
- The existing synchronization controller: linked page/zoom, normalized position following, independent reference zoom, reverse synchronization and marked-difference navigation. Its existing assertions and final confirmation flags passed.
- A protected reference: one incorrect password followed by the fixture credential, without replacing the primary reading. The final reference is open and both native page surfaces are presented.

Capture assertions require images in both panes, settled managed uploads, no transient image-copy charge, positive GPU accounting and the default 72 MiB page-category bound. The final governed page-category counts ranged from 15,796,408 to 25,290,144 bytes in these fixtures; they are payload-accounting snapshots, not process memory, peak VRAM or performance measurements. The synchronization case completed 25 managed texture retirements.

[The capture manifest](../screenshots/rust-alternate-comparison/capture-manifest.json) records source, executable and artifact hashes. Only synthetic PDF fixtures were used. Password values and personal documents are absent from the published controls and screenshots. No standalone test was added.

## Remaining acceptance

This checkpoint does not certify every comparison tool in Glow. Workspace/session handoff of an active comparison, all annotation and keyboard flows, repeated pressure/close cycles and physical graphics failure remain to be exercised. UI/font textures, live-text portions and opaque driver allocations are outside current payload accounting. Performance, physical DPI/Narrator/multi-monitor and broader migration/Release gates remain pending.
