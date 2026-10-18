# Folder import through the existing PDF pipeline

Tools now includes **Import folder…**, backed by the native folder picker. The
selected folder and its ordinary subfolders are scanned on a dedicated worker.
PDF extensions are matched without case sensitivity. Other files are ignored.
The existing Tools **Cancel import** action stops remaining traversal and native
work. Committed documents remain available throughout the import.

## Ownership and bounds

The scanner receives one demand and returns one path at a time through channels
with capacity one. It does not collect a directory tree or complete file list in
memory. Traversal pauses while the import pipeline opens, analyzes, closes and
commits the current PDF. The next path is requested only after that pipeline
advances. Reader admission and the existing native close barrier are preserved.

Directory iteration and canonicalization run outside the UI and repository actor.
The actor registers channel readiness rather than polling an idle scanner.
Dropping owned work closes the demand channel, cancels the worker's own stop token
and joins its thread. That internal stop token is separate from the import token,
so closing the scanner does not hide an error or cancel successful completion.

Folder symlinks and Windows directory reparse points, including junctions, are
excluded. Discovered canonical paths must remain under the authorized root.
Traversal is depth-first in filesystem order, with at most 256 open directory
levels. Exceeding that limit or encountering an unreadable entry reports an error
and stops the batch. Previously committed documents remain saved. A protected or
invalid PDF uses the existing import error path and likewise stops the batch;
there is no silent skip of failed PDFs.

This is an explicit import. It does not register a watcher or automatically import
future files. Repeating the selected folder is safe for already catalogued paths;
the existing repository preserves their identifiers and manual decisions.

## Verification

One new focused test exercises the scanner and real repository/PDFium runtime:

- Unicode paths and nested subfolders, including a directory named `looks-like.pdf`.
- Uppercase PDF extensions, ignored non-PDF files and an unrelated PDF outside the root.
- One-path demand, cancellation and joined disposal of scanner work.
- Three imported documents and unchanged identifiers after a repeated folder import.
- Empty folders, cancelled admission and a missing root.
- Original PDF hashes and release of tracked native memory after runtime shutdown.

The retained reader-preemption import regression also passed. Workspace formatting,
Clippy with all targets and warnings denied, and the locked native build passed.
The minimal suite now contains 53 tests. Only the new folder test and the affected
reader-preemption test ran for this checkpoint; no full-suite pass is claimed.

Three [native captures](../screenshots/rust-folder-import) show a synthetic folder
import, the Spanish light Tools menu and the menu with 200% text in a small window.
For the enlarged-text capture, the synthetic harness scrolls Tools to its last
Settings item; ordinary menus open at their initial position. Each successful
import adds three PDFs to the ten-document demo catalogue. The
control records verify 13 documents. Manifests bind captures to source and build
hashes; no personal files are published.

## Remaining acceptance

Native folder-picker interaction, physical DPI, Narrator, junction behavior on
real external volumes, changing directory trees, very large trees and measured
I/O latency remain unverified. Durable restart recovery and a per-file error review
queue remain pending. There is no claim of full file-management parity or a Rust
replacement release.
