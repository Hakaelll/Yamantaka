use super::*;

const SETTLE: Duration = Duration::from_millis(120);
const PREVIEW_SIDE: u32 = 768;

#[derive(Default)]
pub(super) struct RenderPlan {
    target: Option<(RenderKey, Instant)>,
}

pub(super) struct RenderDecision {
    pub request: Option<RenderKey>,
    pub wake_after: Option<Duration>,
}

impl RenderPlan {
    pub(super) fn ready(&self, image: Option<&RenderKey>) -> bool {
        self.target
            .as_ref()
            .is_some_and(|(key, _)| image == Some(key))
    }

    pub(super) fn next(
        &mut self,
        target: &RenderKey,
        image: Option<&RenderKey>,
        requested: Option<&RenderKey>,
        now: Instant,
    ) -> RenderDecision {
        if self.target.as_ref().is_none_or(|(key, _)| key != target) {
            self.target = Some((target.clone(), now));
        }
        let mut decision = RenderDecision {
            request: None,
            wake_after: None,
        };
        if image == Some(target) || requested == Some(target) {
            return decision;
        }
        if image.is_none() {
            // Finish the first small raster rather than restarting it on resize.
            if requested.is_none() {
                let mut preview = target.clone();
                let longest = preview.physical_width.max(preview.physical_height);
                if longest > PREVIEW_SIDE {
                    preview.physical_width = ((u64::from(preview.physical_width)
                        * u64::from(PREVIEW_SIDE))
                        / u64::from(longest))
                    .max(1) as u32;
                    preview.physical_height = ((u64::from(preview.physical_height)
                        * u64::from(PREVIEW_SIDE))
                        / u64::from(longest))
                    .max(1) as u32;
                }
                decision.request = Some(preview);
            }
            return decision;
        }
        let due = self.target.as_ref().unwrap().1 + SETTLE;
        if now < due {
            decision.wake_after = Some(due - now);
        } else {
            decision.request = Some(target.clone());
        }
        decision
    }
}

pub(super) fn page_mesh(
    texture: egui::TextureId,
    paper: egui::Rect,
    image_rotation: u16,
    rotation: yamantaka_domain::reader_preferences::PageRotation,
) -> egui::Mesh {
    use yamantaka_domain::reader_preferences::PageRotation;
    let image_rotation = PageRotation::from_degrees(i64::from(image_rotation))
        .expect("Rendered reader pages use quarter-turn rotations");
    let mut mesh = egui::Mesh::with_texture(texture);
    for [x, y] in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        let [u, v] = image_rotation.rotate_point(rotation.unrotate_point([x, y]));
        mesh.vertices.push(egui::epaint::Vertex {
            pos: paper.min + egui::vec2(x as f32 * paper.width(), y as f32 * paper.height()),
            uv: egui::pos2(u as f32, v as f32),
            color: Color32::WHITE,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    mesh
}
