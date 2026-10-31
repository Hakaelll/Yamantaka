# Alternate-compositor PDF covers checkpoint

Date: 2026-10-31. Scope: governed extracted PDF covers, manual selection and automatic restoration in the Glow compositor.

## Connected behavior

The library already retained a bounded set of visible cover slots and used governed native wgpu uploads. Glow previously kept procedural jackets and rejected cover previews because its governed upload path was unavailable. Both compositors now use the same image-upload dispatch for reader pages, thumbnails, library covers, picker previews and restored cover images.

The alternate route reserves GPU payload and its temporary image copy through the existing managed presenter. Library cover admission pressure retains the current image and one governed CPU result per visible slot. A capacity-epoch change retries deferred uploads through the selected compositor. Cancellation, document identity and delivery generation checks remain in the owning cover consumer. At most 16 visible cover slots are retained. Leaving their visible set cancels requests and releases their image owners.

Book geometry, materials, cover mapping and spring physics are unchanged. The library now displays its extracted PDF textures through that existing 3D composition in Glow. Manual previews and committed selections use the same path, as does restoration of automatic selection. Shutdown explicitly cancels and clears cover slots and picker images before presenter teardown.

## Executed evidence

Offline formatting, Clippy with warnings denied and the native application build passed. Five native synthetic profiles exercise:

- A light Spanish Glow library with extracted covers and a tilted book.
- A saved manual cover in Glow, installed from source page index 1.
- Automatic-cover restoration in Glow, installed from its verified source page index 1.
- Three-cover regeneration under an artificially full GPU budget. The existing controller verifies old texture identities remain visible while uploads are denied, restores capacity and verifies new images from automatic page index 0. Three retired managed textures were observed after GPU fences completed.
- The existing light wgpu library as a control for the shared dispatch.

All five profiles retained ten ready covers within the 16-slot limit, with no pending or deferred cover upload at capture. Glow temporary-copy usage returned to zero and its managed presenter settled without an error. The ordinary ten-cover fixture charged 7,864,320 GPU payload bytes; this is a governed payload count, not total VRAM or a performance result. Artifact, source and executable hashes are in [the capture manifest](../screenshots/rust-alternate-covers/capture-manifest.json). All PDFs and library records in these captures are synthetic.

## Remaining acceptance

UI/font textures, portions of live text and opaque driver allocations remain outside the memory scope. Rapid long catalogue traversal, repeated pressure/close cycles, physical graphics failure and process-wide resource/performance measurements still need acceptance. The broader historical migration, PDF corpus, accessibility and Windows Release gates remain open. No personal library or PDF was published.
