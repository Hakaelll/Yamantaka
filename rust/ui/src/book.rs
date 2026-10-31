use crate::theme::Palette;
use eframe::egui::{
    self, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2, WidgetInfo,
    WidgetType,
};
use yamantaka_domain::Document;

mod material;

pub(crate) const CAPTION_HEIGHT: f32 = 102.0;
const COVER_HEADROOM: f32 = 32.0;
pub(crate) fn cover_height(width: f32) -> f32 {
    width * 1.34 + COVER_HEADROOM - 10.0
}

#[derive(Clone)]
pub(crate) struct PdfCover {
    pub texture: egui::TextureId,
    pub size: [u32; 2],
    pub path: std::path::PathBuf,
    pub fingerprint: String,
    pub page_count: u32,
}
pub(crate) fn cover_id(id: &str) -> egui::Id {
    egui::Id::new(("pdf-cover-surface", id))
}

#[derive(Clone, Copy, Default)]
pub struct BookPhysics {
    tilt: Vec2,
    velocity: Vec2,
    target: Vec2,
    lift: f32,
    lift_velocity: f32,
}
impl BookPhysics {
    pub fn step(&mut self, target: Vec2, seconds: f32, reduced: bool) -> bool {
        if reduced {
            *self = Self::default();
            return false;
        }
        self.target = target;
        let seconds = seconds.clamp(0.001, 0.032);
        let acceleration =
            (self.target - self.tilt) * (190.0 / 1.15) - self.velocity * (24.0 / 1.15);
        self.velocity += acceleration * seconds;
        self.tilt += self.velocity * seconds;
        self.tilt = self
            .tilt
            .clamp(egui::vec2(-1.08, -1.08), egui::vec2(1.08, 1.08));
        if (self.target - self.tilt).length() < 0.001 && self.velocity.length() < 0.01 {
            self.tilt = self.target;
            self.velocity = Vec2::ZERO;
            false
        } else {
            true
        }
    }
    pub fn pointer(normalized: Vec2) -> Vec2 {
        let weighted = |value: f32| value.signum() * value.abs().min(1.0).powf(1.28);
        egui::vec2(weighted(normalized.x), weighted(normalized.y))
    }
    pub fn emphasis(&mut self, target: f32, seconds: f32, reduced: bool) -> bool {
        if reduced {
            self.lift = 0.0;
            self.lift_velocity = 0.0;
            return false;
        }
        let seconds = seconds.clamp(0.001, 0.032);
        self.lift_velocity += ((target - self.lift) * 210.0 - self.lift_velocity * 29.0) * seconds;
        self.lift = (self.lift + self.lift_velocity * seconds).clamp(0.0, 1.05);
        if (target - self.lift).abs() < 0.001 && self.lift_velocity.abs() < 0.01 {
            self.lift = target;
            self.lift_velocity = 0.0;
            false
        } else {
            true
        }
    }
    fn project(&self, point: Pos2, depth: f32, book: Rect) -> Pos2 {
        let center = book.center();
        let x = point.x - center.x;
        let y = point.y - center.y;
        let rx = -0.08 - self.tilt.y * 0.052;
        let ry = -0.32 + self.tilt.x * 0.095;
        let rotated_x = x * ry.cos() + depth * ry.sin();
        let rotated_z = -x * ry.sin() + depth * ry.cos();
        let rotated_y = y * rx.cos() - rotated_z * rx.sin();
        let z = y * rx.sin() + rotated_z * rx.cos();
        let perspective = 850.0 / (850.0 - z);
        center + egui::vec2(rotated_x, rotated_y) * perspective * (1.0 + self.lift * 0.018)
            - egui::vec2(0.0, self.tilt.length() * 4.0 + self.lift * 7.0)
    }
}
fn polygon(painter: &egui::Painter, points: Vec<Pos2>, fill: Color32) {
    painter.add(egui::Shape::convex_polygon(points, fill, Stroke::NONE));
}

pub fn cover(
    ui: &mut egui::Ui,
    document: &Document,
    width: f32,
    selected: bool,
    reduced_motion: bool,
    palette: Palette,
    preview_pointer: Option<Vec2>,
) -> Response {
    cover_impl(
        ui,
        document,
        width,
        selected,
        reduced_motion,
        palette,
        preview_pointer,
        true,
    )
}

pub fn miniature(
    ui: &mut egui::Ui,
    document: &Document,
    width: f32,
    selected: bool,
    reduced_motion: bool,
    palette: Palette,
) -> Response {
    cover_impl(
        ui,
        document,
        width,
        selected,
        reduced_motion,
        palette,
        None,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn cover_impl(
    ui: &mut egui::Ui,
    document: &Document,
    width: f32,
    selected: bool,
    reduced_motion: bool,
    palette: Palette,
    preview_pointer: Option<Vec2>,
    caption: bool,
) -> Response {
    let height = if caption {
        cover_height(width)
    } else {
        width * 1.34
    };
    let english = ui.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("yamantaka-english"))
            .unwrap_or(false)
    });
    let author = document
        .author
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(if english {
            "Author not specified"
        } else {
            "Autor sin especificar"
        });
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height + if caption { CAPTION_HEIGHT } else { 8.0 }),
        Sense::click(),
    );
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::Button,
            ui.is_enabled(),
            selected,
            format!(
                "{}, {}, {} {}, {:.0}%",
                document.display_title(),
                author,
                document.page_count,
                if english { "pages" } else { "páginas" },
                document.reading_progress.clamp(0.0, 1.0) * 100.0,
            ),
        )
    });
    if !ui.is_rect_visible(rect) {
        ui.ctx()
            .data_mut(|data| data.remove::<BookPhysics>(response.id));
        return response;
    }
    ui.ctx().data_mut(|data| {
        let mut requests = data
            .get_temp::<Vec<Document>>(egui::Id::new("pdf-cover-requests"))
            .unwrap_or_default();
        if requests.len() < 16 && !requests.iter().any(|doc| doc.id == document.id) {
            requests.push(document.clone());
        }
        data.insert_temp(egui::Id::new("pdf-cover-requests"), requests);
    });
    let book = Rect::from_min_size(
        rect.min + egui::vec2(11.0, if caption { COVER_HEADROOM } else { 10.0 }),
        Vec2::new(width - 20.0, width * 1.34 - 14.0),
    );
    let mut physics = ui.ctx().data_mut(|data| {
        data.get_temp::<BookPhysics>(response.id)
            .unwrap_or_default()
    });
    let pointer = preview_pointer
        .map(BookPhysics::pointer)
        .unwrap_or_else(|| {
            response
                .hover_pos()
                .filter(|pos| book.contains(*pos))
                .map(|pos| {
                    BookPhysics::pointer(
                        ((pos - book.center()) / (book.size() * 0.5))
                            .clamp(egui::vec2(-1.0, -1.0), Vec2::splat(1.0)),
                    )
                })
                .unwrap_or(Vec2::ZERO)
        });
    let tilt_moving = physics.step(pointer, ui.input(|i| i.stable_dt), reduced_motion);
    let hovering_book =
        response.hover_pos().is_some_and(|pos| book.contains(pos)) || preview_pointer.is_some();
    let lift_moving = physics.emphasis(
        if !hovering_book {
            0.0
        } else if caption {
            1.0
        } else {
            0.2
        },
        ui.input(|i| i.stable_dt),
        reduced_motion,
    );
    if tilt_moving || lift_moving {
        ui.ctx().request_repaint();
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(response.id, physics));
    let pdf_cover = ui
        .ctx()
        .data(|data| data.get_temp::<PdfCover>(cover_id(&document.id)))
        .filter(|cover| {
            cover.path == document.path
                && cover.fingerprint == document.fingerprint
                && cover.page_count == document.page_count
                && !document.missing
        });
    let depth = (width * 0.13).clamp(12.0, 30.0);
    let project = |point| physics.project(point, depth * 0.5, book);
    if ui.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("capture-control-bounds"))
            .unwrap_or(false)
    }) {
        let corners = [
            book.left_top(),
            book.right_top(),
            book.left_bottom(),
            book.right_bottom(),
        ];
        let mut points = [Pos2::ZERO; 8];
        for (index, point) in corners.into_iter().enumerate() {
            points[index] = project(point);
            points[index + 4] = physics.project(point, -depth * 0.5, book);
        }
        crate::theme::record_control_bounds(
            ui,
            &format!("Book volume · {}", document.id),
            Rect::from_points(&points),
        );
        crate::theme::record_control_bounds(ui, &format!("Book slot · {}", document.id), rect);
    }
    let painter = ui.painter();
    let seed = document.id.bytes().fold(2166136261_u32, |hash, b| {
        hash.wrapping_mul(16777619) ^ b as u32
    });
    let colors = [
        Color32::from_rgb(43, 61, 81),
        Color32::from_rgb(83, 46, 57),
        Color32::from_rgb(44, 71, 58),
        Color32::from_rgb(153, 106, 69),
        Color32::from_rgb(58, 49, 79),
        Color32::from_rgb(33, 74, 77),
    ];
    let ink = Color32::from_rgb(230, 222, 207);
    let color = colors[seed as usize % colors.len()];
    painter.add(
        egui::epaint::Shadow {
            offset: [6, (10.0 + physics.lift * 9.0) as i8],
            blur: (22.0 + physics.lift * 10.0) as u8,
            spread: 0,
            color: Color32::from_black_alpha(if palette.canvas.r() < 100 { 110 } else { 65 }),
        }
        .as_shape(book.shrink(3.0), 2),
    );
    painter.add(
        egui::epaint::Shadow {
            offset: [2, 4],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha((80.0 * (1.0 - physics.lift * 0.5)) as u8),
        }
        .as_shape(book.shrink(3.0), 2),
    );
    let back = |point| physics.project(point, -depth * 0.5, book);
    polygon(
        painter,
        vec![
            back(book.left_top()),
            back(book.right_top()),
            back(book.right_bottom()),
            back(book.left_bottom()),
        ],
        color.gamma_multiply(0.55),
    );
    let paper = book.shrink(0.8);
    let paper_front = |point| physics.project(point, depth * 0.5 - 1.0, book);
    let paper_back = |point| physics.project(point, -depth * 0.5 + 1.0, book);
    for (a, b, shade) in [
        (
            paper.left_top(),
            paper.right_top(),
            Color32::from_rgb(217, 211, 195),
        ),
        (
            paper.right_top(),
            paper.right_bottom(),
            Color32::from_rgb(198, 191, 175),
        ),
        (
            paper.right_bottom(),
            paper.left_bottom(),
            Color32::from_rgb(177, 171, 157),
        ),
    ] {
        polygon(
            painter,
            vec![paper_back(a), paper_back(b), paper_front(b), paper_front(a)],
            shade,
        );
    }
    for n in 1..7 {
        let z = -depth * 0.5 + depth * n as f32 / 7.0;
        painter.line_segment(
            [
                physics.project(paper.right_top(), z, book),
                physics.project(paper.right_bottom(), z, book),
            ],
            Stroke::new(0.45, Color32::from_rgb(152, 146, 131)),
        );
    }
    let spine = [
        back(book.left_top()),
        project(book.left_top()),
        project(book.left_bottom()),
        back(book.left_bottom()),
    ];
    if let Some(cover) = &pdf_cover {
        painter.add(material::pdf_spine(spine, cover));
    } else {
        polygon(painter, spine.to_vec(), color.gamma_multiply(0.48));
    }
    let jacket = if let Some(cover) = pdf_cover {
        material::pdf_jacket(book, cover)
    } else {
        let mut jacket = material::jacket(book, color, ink, physics.tilt.x, seed, width);
        jacket.push(egui::Shape::line_segment(
            [
                book.left_top() + egui::vec2(2.0, 1.5),
                book.right_top() + egui::vec2(-1.0, 1.5),
            ],
            Stroke::new(1.1, Color32::from_white_alpha(44)),
        ));
        jacket.push(egui::Shape::line_segment(
            [
                egui::pos2(book.left() + book.width() * 0.085, book.top() + 2.0),
                egui::pos2(book.left() + book.width() * 0.085, book.bottom() - 2.0),
            ],
            Stroke::new(1.8, Color32::from_black_alpha(40)),
        ));
        jacket.push(egui::Shape::line_segment(
            [
                book.min + egui::vec2(book.width() * 0.07, 2.0),
                Pos2::new(book.left() + book.width() * 0.07, book.bottom() - 2.0),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(26)),
        ));
        let small = width < 170.0;
        let titleless = width <= 60.0;
        let miniature = width < 110.0;
        let inner = book.shrink(if titleless {
            3.0
        } else if miniature {
            7.0
        } else if small {
            14.0
        } else {
            22.0
        });
        let family = seed % 6;
        let (title_region, art_region, author_region) = jacket_regions(
            inner,
            family,
            !small && document.author.is_some(),
            titleless,
        );
        if !titleless {
            let family_font = if matches!(family, 0 | 3 | 5) {
                "Jacket"
            } else {
                "Editorial"
            };
            let mut size = (width * if matches!(family, 1 | 4) { 0.115 } else { 0.10 }).clamp(
                if miniature {
                    7.0
                } else if small {
                    11.0
                } else {
                    13.0
                },
                if family == 3 { 27.0 } else { 25.0 },
            );
            let title = loop {
                let mut job = egui::text::LayoutJob::simple(
                    document.display_title().to_owned(),
                    FontId::new(size, egui::FontFamily::Name(family_font.into())),
                    ink,
                    title_region.width(),
                );
                job.wrap.max_rows = 3;
                job.wrap.overflow_character = Some('…');
                let galley = painter.layout_job(job);
                if galley.size().y <= title_region.height()
                    || size
                        <= if miniature {
                            7.0
                        } else if small {
                            10.0
                        } else {
                            12.0
                        }
                {
                    break galley;
                }
                size -= 1.0;
            };
            let x = if family == 5 {
                title_region.center().x - title.size().x * 0.5
            } else if family == 1 {
                title_region.right() - title.size().x
            } else {
                title_region.left()
            };
            let y = if family == 2 {
                title_region.center().y - title.size().y * 0.5
            } else {
                title_region.top()
            };
            jacket.push(egui::Shape::galley(egui::pos2(x, y), title, ink));
        }
        let art_height = art_region.height();
        let center = art_region.center();
        let radius =
            (art_region.width() * if family == 1 { 0.47 } else { 0.38 }).min(art_height * 0.43);
        let art = ink.gamma_multiply(0.78);
        let foil = Color32::from_rgb(201, 168, 117);
        let line = |a, b| egui::Shape::line_segment([a, b], Stroke::new(1.0, art));
        match family {
            0 => {
                jacket.push(egui::Shape::circle_filled(
                    center,
                    radius * 0.85,
                    color.lerp_to_gamma(foil, 0.38),
                ));
                for n in 0..6 {
                    let c = center + egui::vec2((n as f32 - 2.5) * art_region.width() * 0.022, 0.0);
                    let points = (0..96)
                        .map(|n| {
                            let angle = n as f32 * std::f32::consts::TAU / 96.0;
                            c + egui::vec2(angle.cos(), angle.sin()) * radius
                        })
                        .collect();
                    jacket.push(egui::Shape::closed_line(points, Stroke::new(0.85, foil)));
                }
            }
            1 => {
                let r = radius;
                let points = [
                    center + egui::vec2(0.0, -r),
                    center + egui::vec2(r, 0.0),
                    center + egui::vec2(0.0, r),
                    center + egui::vec2(-r, 0.0),
                ]
                .into_iter()
                .collect();
                jacket.push(egui::Shape::closed_line(points, Stroke::new(1.0, art)));
                jacket.push(line(
                    center + egui::vec2(0.0, -r),
                    center + egui::vec2(0.0, r),
                ));
                jacket.push(egui::Shape::convex_polygon(
                    vec![
                        center + egui::vec2(0.0, -r),
                        center + egui::vec2(r, 0.0),
                        center + egui::vec2(0.0, r),
                    ],
                    foil.gamma_multiply(0.88),
                    Stroke::NONE,
                ));
                jacket.push(egui::Shape::convex_polygon(
                    vec![
                        center + egui::vec2(0.0, -r),
                        center + egui::vec2(-r, 0.0),
                        center + egui::vec2(0.0, r),
                    ],
                    color.gamma_multiply(0.65),
                    Stroke::new(0.8, foil),
                ));
            }
            2 => {
                let a = radius * 0.85;
                let top = center - egui::vec2(0.0, a);
                let left = center - egui::vec2(a, 0.0);
                let right = center + egui::vec2(a, 0.0);
                let bottom = center + egui::vec2(0.0, a);
                jacket.push(egui::Shape::convex_polygon(
                    vec![top, left, center, right],
                    Color32::from_rgb(198, 185, 147),
                    Stroke::NONE,
                ));
                jacket.push(egui::Shape::convex_polygon(
                    vec![left, center, bottom],
                    Color32::from_rgb(128, 136, 104),
                    Stroke::NONE,
                ));
                for (a, b) in [
                    (top, left),
                    (top, right),
                    (left, center),
                    (right, center),
                    (center, bottom),
                    (left, bottom),
                    (right, bottom),
                ] {
                    jacket.push(line(a, b));
                }
            }
            3 => {
                let r = Rect::from_center_size(
                    center,
                    egui::vec2(art_region.width() * 0.94, art_height * 0.90),
                );
                for n in 0..4 {
                    let left = r.left() + r.width() * n as f32 / 4.0;
                    jacket.push(egui::Shape::rect_filled(
                        Rect::from_min_max(
                            egui::pos2(left, r.top()),
                            egui::pos2(left + r.width() / 4.0 - 2.0, r.bottom()),
                        ),
                        0.0,
                        foil.gamma_multiply(0.35 + n as f32 * 0.15),
                    ));
                }
            }
            4 => {
                for n in 0..9 {
                    let points = (0..48)
                        .map(|sample| {
                            let t = sample as f32 / 47.0;
                            center
                                + egui::vec2(
                                    (t - 0.5) * art_region.width() * 0.98,
                                    (n as f32 - 4.0) * art_height / 14.0
                                        + (t * std::f32::consts::PI * 1.4 + n as f32 * 0.13).sin()
                                            * art_height
                                            * 0.16,
                                )
                        })
                        .collect();
                    jacket.push(egui::Shape::line(
                        points,
                        Stroke::new(
                            if n % 3 == 0 { 2.0 } else { 0.8 },
                            if n % 3 == 0 { foil } else { art },
                        ),
                    ));
                }
            }
            _ => {
                jacket.push(egui::Shape::circle_filled(
                    center,
                    radius * 0.38,
                    color.lerp_to_gamma(foil, 0.5),
                ));
                for n in 0..16 {
                    let angle = n as f32 * std::f32::consts::TAU / 16.0;
                    jacket.push(line(
                        center + egui::vec2(angle.cos(), angle.sin()) * radius * 0.43,
                        center + egui::vec2(angle.cos(), angle.sin()) * radius,
                    ));
                }
                jacket.push(egui::Shape::circle_stroke(
                    center,
                    radius * 0.43,
                    Stroke::new(1.0, foil),
                ));
            }
        }
        if let (Some(author), Some(region)) = (&document.author, author_region) {
            let mut job = egui::text::LayoutJob::simple(
                author.clone(),
                FontId::proportional((width * 0.06).clamp(12.0, 14.0)),
                ink,
                region.width(),
            );
            job.wrap.max_rows = 1;
            job.wrap.overflow_character = Some('…');
            let galley = painter.layout_job(job);
            let x = if family == 1 {
                region.right() - galley.size().x
            } else if family == 5 {
                region.center().x - galley.size().x * 0.5
            } else {
                region.left()
            };
            jacket.push(egui::Shape::galley(
                egui::pos2(x, region.top()),
                galley,
                ink,
            ));
        }
        jacket
    };
    let shapes = jacket
        .into_iter()
        .map(|shape| egui::epaint::ClippedShape {
            clip_rect: book.expand(32.0),
            shape,
        })
        .collect();
    for primitive in ui.ctx().tessellate(shapes, ui.ctx().pixels_per_point()) {
        if let egui::epaint::Primitive::Mesh(mut mesh) = primitive.primitive {
            for vertex in &mut mesh.vertices {
                vertex.pos = project(vertex.pos);
            }
            painter.add(egui::Shape::mesh(mesh));
        }
    }
    if selected {
        let edge = book.expand(4.0);
        painter.add(egui::Shape::closed_line(
            [
                edge.left_top(),
                edge.right_top(),
                edge.right_bottom(),
                edge.left_bottom(),
            ]
            .map(project)
            .to_vec(),
            Stroke::new(1.5, palette.accent),
        ));
        painter.circle_filled(
            project(book.right_top() + egui::vec2(-10.0, 12.0)),
            5.0,
            palette.accent,
        );
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            6.0,
            Stroke::new(2.0, palette.accent),
            StrokeKind::Inside,
        );
    }
    if !caption {
        return response;
    }
    let title_rect = Rect::from_min_size(
        egui::pos2(rect.left() + 10.0, rect.top() + height + 5.0),
        egui::vec2(width - 20.0, 42.0),
    );
    let mut title = egui::text::LayoutJob::simple(
        document.display_title().to_owned(),
        FontId::proportional(16.0),
        palette.ink,
        title_rect.width(),
    );
    title.wrap.max_rows = 2;
    title.wrap.overflow_character = Some('…');
    painter.galley(title_rect.min, painter.layout_job(title), palette.ink);
    let mut author_job = egui::text::LayoutJob::simple(
        author.to_owned(),
        FontId::proportional(13.0),
        palette.secondary,
        title_rect.width(),
    );
    author_job.wrap.max_rows = 1;
    author_job.wrap.overflow_character = Some('…');
    painter.galley(
        egui::pos2(title_rect.left(), title_rect.bottom() + 3.0),
        painter.layout_job(author_job),
        palette.secondary,
    );
    painter.text(
        egui::pos2(title_rect.left(), rect.bottom() - 20.0),
        egui::Align2::LEFT_CENTER,
        format!(
            "{} {}  ·  {}%",
            document.page_count,
            if english { "pages" } else { "páginas" },
            (document.reading_progress * 100.0).round()
        ),
        FontId::proportional(12.0),
        palette.secondary,
    );
    if document.reading_progress > 0.0 {
        let track = Rect::from_min_size(
            egui::pos2(title_rect.left(), rect.bottom() - 5.0),
            egui::vec2(title_rect.width(), 2.0),
        );
        painter.rect_filled(track, 1.0, palette.line);
        painter.rect_filled(
            Rect::from_min_size(
                track.min,
                egui::vec2(
                    track.width() * document.reading_progress.clamp(0.0, 1.0) as f32,
                    track.height(),
                ),
            ),
            1.0,
            palette.accent,
        );
    }
    response
}
/// The six jackets share volume and physics, while type and artwork occupy
/// distinct, non-overlapping regions. Miniatures communicate by color and art.
fn jacket_regions(
    inner: Rect,
    family: u32,
    author: bool,
    titleless: bool,
) -> (Rect, Rect, Option<Rect>) {
    if titleless {
        return (Rect::from_min_size(inner.min, Vec2::ZERO), inner, None);
    }
    let author_height = if author { 20.0 } else { 0.0 };
    let usable = Rect::from_min_max(
        inner.min,
        egui::pos2(inner.right(), inner.bottom() - author_height),
    );
    let author_region = author
        .then(|| Rect::from_min_max(egui::pos2(inner.left(), inner.bottom() - 16.0), inner.max));
    let gap = if inner.height() > 100.0 { 10.0 } else { 4.0 };
    let title_fraction = match family {
        0 => 0.46,
        1 => 0.46,
        2 => 0.45,
        3 => 0.49,
        4 => 0.44,
        _ => 0.48,
    };
    let title_height = (usable.height() - gap) * title_fraction;
    let art_height = usable.height() - gap - title_height;
    let at = |y: f32, h: f32| {
        Rect::from_min_size(egui::pos2(usable.left(), y), egui::vec2(usable.width(), h))
    };
    let (title, art) = match family {
        0 => (
            at(usable.bottom() - title_height, title_height),
            at(usable.top(), art_height),
        ),
        1 => (
            at(usable.top(), title_height),
            Rect::from_min_size(
                egui::pos2(usable.left(), usable.top() + title_height + gap),
                egui::vec2(usable.width() * 0.72, art_height),
            ),
        ),
        2 => (
            at(usable.bottom() - title_height, title_height),
            Rect::from_min_size(
                egui::pos2(usable.left() + usable.width() * 0.2, usable.top()),
                egui::vec2(usable.width() * 0.8, art_height),
            ),
        ),
        3 | 4 => (
            at(usable.top(), title_height),
            at(usable.top() + title_height + gap, art_height),
        ),
        _ => (
            at(usable.bottom() - title_height, title_height),
            at(usable.top(), art_height),
        ),
    };
    (title, art, author_region)
}
