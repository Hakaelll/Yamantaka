use eframe::egui::{self, Color32, Rect, Shape, Stroke};

/// Bound cloth, hinge and edge light share the jacket plane and rigid projection.
pub(super) fn jacket(
    book: Rect,
    color: Color32,
    ink: Color32,
    tilt: f32,
    seed: u32,
    width: f32,
) -> Vec<Shape> {
    let mut mesh = egui::epaint::Mesh::default();
    let columns = [0.0, 0.014, 0.032, 0.05, 0.07, 0.085, 0.12, 0.38, 0.7, 1.0];
    let spine_light = [0.58, 0.75, 1.02, 1.13, 0.83, 0.7, 1.0, 1.0, 1.0, 1.0];
    for row in 0..=8 {
        let y = row as f32 / 8.0;
        for (column, x) in columns.iter().enumerate() {
            let light = (0.11 * (1.0 - x) + tilt * 0.065).clamp(0.0, 0.18);
            let shade = color
                .lerp_to_gamma(ink, light)
                .gamma_multiply((1.0 - y * 0.2) * spine_light[column]);
            mesh.colored_vertex(book.min + book.size() * egui::vec2(*x, y), shade);
        }
    }
    for row in 0..8 {
        for column in 0..columns.len() - 1 {
            let a = (row * columns.len() + column) as u32;
            let b = a + columns.len() as u32;
            mesh.add_triangle(a, a + 1, b + 1);
            mesh.add_triangle(a, b + 1, b);
        }
    }
    let mut shapes = vec![Shape::mesh(mesh)];
    // Fixed weave in book coordinates: bounded detail, no random frame-to-frame shimmer.
    if width >= 170.0 {
        let start = book.left() + book.width() * 0.12;
        for row in 0..64 {
            let y = book.top() + book.height() * (row as f32 + 0.5) / 64.0;
            let alpha = 3 + (seed.rotate_left(row % 32) & 3) as u8;
            shapes.push(Shape::line_segment(
                [egui::pos2(start, y), egui::pos2(book.right() - 1.0, y)],
                Stroke::new(0.5, Color32::from_white_alpha(alpha)),
            ));
        }
        for column in 0..32 {
            let x = start + (book.right() - start) * (column as f32 + 0.5) / 32.0;
            shapes.push(Shape::line_segment(
                [
                    egui::pos2(x, book.top() + 1.0),
                    egui::pos2(x, book.bottom() - 1.0),
                ],
                Stroke::new(0.5, Color32::from_black_alpha(5)),
            ));
        }
    }
    shapes.push(Shape::line_segment(
        [book.right_top(), book.right_bottom()],
        Stroke::new(0.8, Color32::from_white_alpha(34)),
    ));
    shapes
}
