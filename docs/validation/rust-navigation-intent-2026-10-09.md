# Asynchronous opening and destination changes — 2026-10-09

Search document lookup and native PDF opening now share an explicit presentation
intent in `rust/ui/src/app/open_intent.rs`. Choosing Library, Settings, Search or
the current Reader invalidates a pending lookup. Choosing another document or
starting an import also supersedes it. A queued response is checked again before
it can navigate, even when the actor already delivered that response.

Native opening and presentation have separate lifetimes. When a PDF opening has
already been accepted, changing destination permits the native handle transition
to finish without forcing the visible surface back to Reader. Its selected search
page is retained and saved. A later document opening clears the previous page
target. Existing-reader search results still use canonical navigation.

## Executed checks

- `cargo test --workspace --locked`: **117 tests passed**.
- `cargo test -p yamantaka --locked`: **12 tests passed**, including four new
  presentation-state regression tests. They cover leaving Search before a queued
  lookup is consumed, repeated lookup of different pages of the same document,
  changing surface during opening while retaining the page, and opening another
  book while a lookup is outstanding.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Formatting and scoped `git diff --check`: passed.

These tests exercise the production intent state used by the interface. They do
not simulate operating-system pointer input or prove every reader interaction.
No new screenshots were taken: the search screenshots published with checkpoint
`149e13a` remain evidence of that checkpoint's visual layout, not of this change.
The complete rewrite, remaining acceptance matrix and final release remain open.
