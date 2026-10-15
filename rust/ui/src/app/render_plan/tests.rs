use super::*;
use yamantaka_domain::reader_preferences::PageRotation;

fn key(width: u32, height: u32, generation: u64) -> RenderKey {
    RenderKey {
        document_id: "reader".into(),
        page: 3,
        region: None,
        physical_width: width,
        physical_height: height,
        rotation: 0,
        generation,
    }
}

#[test]
fn first_visible_raster_is_small_then_refines_without_an_idle_poll() {
    let now = Instant::now();
    let target = key(2400, 3200, 4);
    let mut plan = RenderPlan::default();
    let first = plan.next(&target, None, None, now);
    let preview = first.request.unwrap();
    assert_eq!(
        [preview.physical_width, preview.physical_height],
        [576, 768]
    );
    assert_eq!(preview.document_id, "reader");
    assert_eq!(preview.page, 3);
    assert_eq!(preview.generation, 4);
    assert!(first.wake_after.is_none());
    let waiting = plan.next(
        &target,
        None,
        Some(&preview),
        now + Duration::from_millis(20),
    );
    assert!(waiting.request.is_none());
    assert!(waiting.wake_after.is_none());
    let visible = plan.next(
        &target,
        Some(&preview),
        Some(&preview),
        now + Duration::from_millis(30),
    );
    assert!(visible.request.is_none());
    assert_eq!(visible.wake_after, Some(Duration::from_millis(90)));
    let refined = plan.next(&target, Some(&preview), Some(&preview), now + SETTLE);
    assert_eq!(refined.request, Some(target.clone()));
    let pending = plan.next(&target, Some(&preview), Some(&target), now + SETTLE);
    assert!(pending.request.is_none());
    assert!(pending.wake_after.is_none());
    assert!(!plan.ready(Some(&preview)));
    assert!(plan.ready(Some(&target)));
    let idle = plan.next(
        &target,
        Some(&target),
        Some(&target),
        now + Duration::from_secs(1),
    );
    assert!(idle.request.is_none());
    assert!(idle.wake_after.is_none());
}

#[test]
fn rapid_zoom_changes_keep_the_image_and_request_only_the_final_refinement() {
    let now = Instant::now();
    let image = key(600, 800, 1);
    let mut plan = RenderPlan::default();
    let mut final_target = image.clone();
    for step in 1..=30 {
        final_target = key(600 + step * 30, 800 + step * 40, u64::from(step + 1));
        let decision = plan.next(
            &final_target,
            Some(&image),
            None,
            now + Duration::from_millis(u64::from(step) * 8),
        );
        assert!(decision.request.is_none());
        assert_eq!(decision.wake_after, Some(SETTLE));
    }
    let decision = plan.next(
        &final_target,
        Some(&image),
        None,
        now + Duration::from_millis(360),
    );
    assert_eq!(decision.request, Some(final_target));
}

#[test]
fn small_pages_need_only_one_raster_and_resizing_does_not_starve_the_first_preview() {
    let now = Instant::now();
    let mut plan = RenderPlan::default();
    let small = key(300, 400, 1);
    assert_eq!(
        plan.next(&small, None, None, now).request,
        Some(small.clone())
    );
    let resized = key(1600, 2000, 1);
    let waiting = plan.next(
        &resized,
        None,
        Some(&small),
        now + Duration::from_millis(10),
    );
    assert!(waiting.request.is_none());
    assert!(waiting.wake_after.is_none());
    let decision = plan.next(
        &resized,
        Some(&small),
        Some(&small),
        now + Duration::from_millis(130),
    );
    assert_eq!(decision.request, Some(resized));
}

#[test]
fn retained_texture_rotates_into_the_new_page_geometry_before_the_new_raster_arrives() {
    let paper = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(800.0, 600.0));
    let mesh = page_mesh(egui::TextureId::Managed(7), paper, 0, PageRotation::Deg90);
    let uv: Vec<_> = mesh.vertices.iter().map(|vertex| vertex.uv).collect();
    assert_eq!(
        uv,
        vec![
            egui::pos2(0.0, 1.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 0.0),
            egui::pos2(1.0, 1.0)
        ]
    );
    assert_eq!(mesh.vertices[0].pos, paper.min);
    assert_eq!(mesh.vertices[2].pos, paper.max);
    assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
    for degrees in [0, 90, 180, 270] {
        let rotation = PageRotation::from_degrees(i64::from(degrees)).unwrap();
        let mesh = page_mesh(egui::TextureId::Managed(7), paper, degrees, rotation);
        assert_eq!(
            mesh.vertices
                .iter()
                .map(|vertex| vertex.uv)
                .collect::<Vec<_>>(),
            vec![
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(0.0, 1.0)
            ]
        );
    }
}
