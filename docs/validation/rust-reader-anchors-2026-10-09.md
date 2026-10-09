# Two-dimensional canonical reader anchors

ReaderPosition now stores a normalized horizontal coordinate alongside its page
and vertical coordinate. Both refer to the displayed PDF page before the user's
additional rotation. Existing vertical positions default to the horizontal page
center. The transformed horizontal point aligns with the viewport center, and
the transformed vertical point aligns with its top edge. Destinations using a
left-edge PDF coordinate will therefore need viewport-to-center conversion when
they are applied.

Scroll restoration and feedback use clockwise quarter-turn transforms and their
inverse. Painting, restoration and feedback now share a single page-rectangle
calculation. It handles continuous page offsets and the selected physical page
in LTR or RTL spreads. Continuous visible-page lookup remains logarithmic;
rectangle calculation performs constant-time indexed page access.

Navigation history and A/B marks retain the complete point. Applying reader
preferences preserves those points. Explicit page controls choose the displayed
page's top center under the current rotation. Loading document preferences also
initializes the existing opening anchor at that displayed start, replacing it
without adding a history entry. Preference initialization moved from the main
reply dispatcher into the existing reader-options module.

Four new domain tests check literal clockwise coordinates, inverse transforms,
horizontal pan and finite-canvas clamping, selected-page LTR/RTL spread pan, and
history/marks/rotated page starts. The existing zoom/rotation/resize test now checks
both coordinates of a noncentral anchor. Its viewports deliberately fit within
the scrollable page because a finite canvas cannot preserve an off-canvas point.
A separate test checks a viewport larger than the page and explicit edge clamping.

Validation on 2026-10-09:

```text
cargo test --workspace --locked                              150 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all -- --check                                   passed
```

The strengthened noncentral anchor test passed again after its final fixture
change. No file crossed 1,000 lines. No new native screenshots or physical
accessibility certification are claimed. Interactive pan/rotation acceptance and
persisted per-tab scroll contexts remain pending. The outline tab, destination
zoom/fit application and conversion of PDF left-edge targets to viewport-center
anchors are still required before outline navigation is complete.
