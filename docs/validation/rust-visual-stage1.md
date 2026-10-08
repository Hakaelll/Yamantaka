# Initial Rust section: executed visual evidence

Date: 2026-10-08. Windows native execution, eframe 0.36.2, DX12 by default and
the independently captured OpenGL path. The collection is explicitly synthetic;
it reads a real PDFium-rendered fixture, and the tilted cover uses a capture-only
synthetic pointer through the real spring and rigid projection.

The seven captures in `docs/screenshots/rust-stage1` are copied directly from the
application's native viewport screenshots. Logical 1440×960 corresponds to
1800×1200 physical pixels on this host. The compact capture is 640×480 logical,
800×600 physical. None is an image-generated mockup.

Review sequence: the first full independent review returned **rebuild**. A second
full review accepted the connected cover geometry and projected lettering and
returned **fix** for six material findings. One batch resolved four; the verdict
identified two partial findings (compact caption gap and light slider rail).
The second batch resolved both, confirmed by the same reviewer on renewed native
captures. Final disposition: **ship at the scored findings' scope**. No approval
of the whole surface, whole rewrite or physical accessibility is claimed.

The section now shows its selected destinations, violet primary actions, bounded
search, cover-only physical motion, reduced-motion static volumes, and small
window reflow. UI/data tests cover the spring, late delivery cancellation, and
the actual Windows secondary process. Full import, reader, study, organisation,
trust flows and physical DPI/Narrator checks are tracked separately in the parity
matrix. These screenshots cannot be used as evidence for unimplemented features.

The reviewer initially inherited the build transcript; later reviews and verdicts
used the supplied image files. The final documenter received a fresh context.
This limitation is recorded instead of claiming a wholly blind initial review.
