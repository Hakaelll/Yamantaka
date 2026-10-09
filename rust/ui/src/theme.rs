use eframe::egui::{self, Color32, FontFamily, FontId, TextStyle};

#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub ink: Color32,
    pub secondary: Color32,
    pub line: Color32,
    pub accent: Color32,
}
#[derive(Clone)]
pub struct ControlBounds {
    pub label: String,
    pub rect: egui::Rect,
    pub clip: egui::Rect,
    pub enabled: bool,
}
impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                canvas: Color32::from_rgb(15, 16, 18),
                surface: Color32::from_rgb(22, 23, 26),
                raised: Color32::from_rgb(32, 33, 37),
                ink: Color32::from_rgb(239, 237, 232),
                secondary: Color32::from_rgb(167, 168, 176),
                line: Color32::from_rgb(50, 51, 57),
                accent: Color32::from_rgb(189, 163, 245),
            }
        } else {
            Self {
                canvas: Color32::from_rgb(247, 245, 239),
                surface: Color32::from_rgb(255, 253, 248),
                raised: Color32::from_rgb(235, 232, 224),
                ink: Color32::from_rgb(33, 34, 39),
                secondary: Color32::from_rgb(100, 100, 109),
                line: Color32::from_rgb(211, 207, 198),
                accent: Color32::from_rgb(103, 72, 167),
            }
        }
    }
}
pub fn install(ctx: &egui::Context, dark: bool) {
    let p = Palette::new(dark);
    let selected = if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    let mut style = (*ctx.style_of(selected)).clone();
    style.visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    style.visuals.panel_fill = p.canvas;
    style.visuals.window_fill = p.surface;
    style.visuals.extreme_bg_color = p.canvas;
    style.visuals.faint_bg_color = p.surface;
    style.visuals.override_text_color = Some(p.ink);
    style.visuals.selection.bg_fill = p.raised;
    style.visuals.selection.stroke = egui::Stroke::new(1.5, p.accent);
    style.visuals.widgets.inactive.bg_fill = p.surface;
    style.visuals.widgets.inactive.weak_bg_fill = p.surface;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, p.secondary);
    style.visuals.widgets.hovered.bg_fill = p.raised;
    style.visuals.widgets.hovered.weak_bg_fill = p.raised;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, p.secondary);
    style.visuals.widgets.active.bg_fill = p.raised;
    style.visuals.widgets.active.weak_bg_fill = p.raised;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, p.accent);
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.secondary);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.line);
    for visual in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        visual.corner_radius = egui::CornerRadius::same(7);
        visual.fg_stroke = egui::Stroke::new(1.0, p.ink);
    }
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    style.spacing.interact_size = egui::vec2(40.0, 40.0);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(30.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(13.0, FontFamily::Proportional),
    );
    ctx.set_style_of(selected, style);
    ctx.set_theme(selected);
}
pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in [
        "C:/Windows/Fonts/SegUIVar.ttf",
        "C:/Windows/Fonts/segoeui.ttf",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "Yamantaka UI".into(),
                egui::FontData::from_owned(bytes).into(),
            );
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "Yamantaka UI".into());
            break;
        }
    }
    let jacket = if let Ok(bytes) = std::fs::read("C:/Windows/Fonts/segoeuib.ttf") {
        fonts.font_data.insert(
            "Yamantaka Jacket".into(),
            egui::FontData::from_owned(bytes).into(),
        );
        let mut family = fonts.families[&FontFamily::Proportional].clone();
        family.insert(0, "Yamantaka Jacket".into());
        family
    } else {
        fonts.families[&FontFamily::Proportional].clone()
    };
    fonts
        .families
        .insert(FontFamily::Name("Jacket".into()), jacket);
    fonts.font_data.insert(
        "Source Serif Display".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/source-serif/SourceSerif4Display-Regular.ttf"
        ))
        .into(),
    );
    let mut editorial = fonts.families[&FontFamily::Proportional].clone();
    editorial.insert(0, "Source Serif Display".into());
    fonts
        .families
        .insert(FontFamily::Name("Editorial".into()), editorial);
    ctx.set_fonts(fonts);
}

pub fn display(ui: &mut egui::Ui, text: &str, size: f32) -> egui::Response {
    ui.label(egui::RichText::new(text).size(size))
}

#[derive(Clone, Copy)]
pub enum Icon {
    Menu,
    Book,
    Add,
    Search,
    Grid,
    List,
    Settings,
    Left,
    Right,
    Panel,
    Minus,
    Plus,
    Fit,
    Sort,
}

pub fn icon(painter: &egui::Painter, center: egui::Pos2, icon: Icon, color: Color32) {
    let s = egui::Stroke::new(1.5, color);
    let v = |x, y| center + egui::vec2(x, y);
    let line = |a, b| painter.line_segment([a, b], s);
    match icon {
        Icon::Menu => {
            for y in [-4.5, 0.0, 4.5] {
                line(v(-6.0, y), v(6.0, y));
            }
        }
        Icon::Add | Icon::Plus => {
            line(v(-5.0, 0.0), v(5.0, 0.0));
            line(v(0.0, -5.0), v(0.0, 5.0));
        }
        Icon::Minus => {
            line(v(-5.0, 0.0), v(5.0, 0.0));
        }
        Icon::Sort => {
            for (y, length) in [(-4.5, 10.0), (0.0, 7.0), (4.5, 4.0)] {
                line(v(-6.0, y), v(-6.0 + length, y));
            }
            line(v(6.0, -4.0), v(6.0, 5.0));
            line(v(3.5, 2.5), v(6.0, 5.0));
            line(v(6.0, 5.0), v(8.5, 2.5));
        }
        Icon::Left | Icon::Right => {
            let direction = if matches!(icon, Icon::Left) {
                -1.0
            } else {
                1.0
            };
            line(v(-2.0 * direction, -5.0), v(3.0 * direction, 0.0));
            line(v(3.0 * direction, 0.0), v(-2.0 * direction, 5.0));
        }
        Icon::Search => {
            painter.circle_stroke(v(-1.5, -1.5), 5.0, s);
            line(v(2.5, 2.5), v(6.0, 6.0));
        }
        Icon::Grid => {
            for x in [-5.5, 1.5] {
                for y in [-5.5, 1.5] {
                    painter.rect_stroke(
                        egui::Rect::from_min_size(v(x, y), egui::vec2(4.0, 4.0)),
                        0.5,
                        s,
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
        Icon::List => {
            for y in [-4.5, 0.0, 4.5] {
                line(v(-2.0, y), v(6.0, y));
                painter.circle_filled(v(-5.5, y), 1.0, color);
            }
        }
        Icon::Book => {
            painter.add(egui::Shape::line(
                vec![
                    v(-7.0, -5.0),
                    v(-3.0, -6.0),
                    v(0.0, -4.0),
                    v(3.0, -6.0),
                    v(7.0, -5.0),
                    v(7.0, 6.0),
                    v(3.0, 5.0),
                    v(0.0, 7.0),
                    v(-3.0, 5.0),
                    v(-7.0, 6.0),
                    v(-7.0, -5.0),
                ],
                s,
            ));
            line(v(0.0, -4.0), v(0.0, 7.0));
        }
        Icon::Panel => {
            painter.rect_stroke(
                egui::Rect::from_center_size(center, egui::vec2(14.0, 12.0)),
                2.0,
                s,
                egui::StrokeKind::Inside,
            );
            line(v(-2.0, -5.0), v(-2.0, 5.0));
        }
        Icon::Fit => {
            for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                line(v(x * 6.0, y * 2.0), v(x * 6.0, y * 6.0));
                line(v(x * 6.0, y * 6.0), v(x * 2.0, y * 6.0));
            }
        }
        Icon::Settings => {
            for n in 0..8 {
                let a = n as f32 * std::f32::consts::TAU / 8.0;
                line(
                    v(a.cos() * 5.0, a.sin() * 5.0),
                    v(a.cos() * 7.5, a.sin() * 7.5),
                );
            }
            painter.circle_stroke(center, 5.5, s);
            painter.circle_stroke(center, 2.0, s);
        }
    }
}

/// Own control geometry, including semantics, bounded motion and distinct focus.
pub fn control(
    ui: &mut egui::Ui,
    label: &str,
    symbol: Option<Icon>,
    selected: bool,
    primary: bool,
    p: Palette,
) -> egui::Response {
    control_impl(ui, label, symbol, selected, primary, false, p)
}

pub fn icon_button(ui: &mut egui::Ui, label: &str, symbol: Icon, p: Palette) -> egui::Response {
    control_impl(ui, label, Some(symbol), false, false, true, p).on_hover_text(label)
}

pub fn icon_toggle(
    ui: &mut egui::Ui,
    label: &str,
    symbol: Icon,
    selected: bool,
    p: Palette,
) -> egui::Response {
    control_impl(ui, label, Some(symbol), selected, false, true, p).on_hover_text(label)
}

pub fn primary_icon(ui: &mut egui::Ui, label: &str, symbol: Icon, p: Palette) -> egui::Response {
    control_impl(ui, label, Some(symbol), false, true, true, p).on_hover_text(label)
}

fn control_impl(
    ui: &mut egui::Ui,
    label: &str,
    symbol: Option<Icon>,
    selected: bool,
    primary: bool,
    icon_only: bool,
    p: Palette,
) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        if icon_only {
            String::new()
        } else {
            label.to_owned()
        },
        FontId::proportional(14.0),
        Color32::PLACEHOLDER,
    );
    let width = if icon_only {
        40.0
    } else {
        (galley.size().x + if symbol.is_some() { 52.0 } else { 28.0 }).max(40.0)
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 40.0), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    let reduced = ui.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("yamantaka-reduced-motion"))
            .unwrap_or(false)
    });
    let hover = if reduced || response.has_focus() {
        f32::from(response.hovered())
    } else {
        ui.ctx().animate_bool_with_time_and_easing(
            response.id.with("hover"),
            response.hovered(),
            0.14,
            |t| 1.0 - (1.0 - t).powi(3),
        )
    };
    let foreground = if primary {
        if p.canvas.r() < 100 {
            Color32::from_rgb(25, 19, 36)
        } else {
            Color32::WHITE
        }
    } else if selected || response.hovered() {
        p.ink
    } else {
        p.secondary
    };
    let fill = if primary {
        p.accent.lerp_to_gamma(p.ink, hover * 0.10)
    } else if selected {
        p.raised
    } else {
        Color32::TRANSPARENT.lerp_to_gamma(p.raised, hover)
    };
    let rect = rect.translate(egui::vec2(
        0.0,
        f32::from(response.is_pointer_button_down_on()),
    ));
    ui.painter().rect_filled(
        rect,
        8.0,
        if ui.is_enabled() {
            fill
        } else {
            fill.gamma_multiply(0.35)
        },
    );
    let foreground = if ui.is_enabled() {
        foreground
    } else {
        foreground.gamma_multiply(0.4)
    };
    let text_x = if let Some(symbol) = symbol {
        icon(
            ui.painter(),
            egui::pos2(rect.left() + 20.0, rect.center().y),
            symbol,
            foreground,
        );
        rect.left() + 36.0
    } else {
        rect.left() + 14.0
    };
    ui.painter().galley(
        egui::pos2(text_x, rect.center().y - galley.size().y * 0.5),
        galley,
        foreground,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            8.0,
            egui::Stroke::new(2.0, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    if selected {
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 12.0, rect.bottom() - 4.0),
                egui::pos2(rect.right() - 12.0, rect.bottom() - 4.0),
            ],
            egui::Stroke::new(1.5, p.accent),
        );
    }
    if ui.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("capture-control-bounds"))
            .unwrap_or(false)
    }) {
        ui.ctx().data_mut(|data| {
            let controls = data
                .get_temp_mut_or_default::<Vec<ControlBounds>>(egui::Id::new("capture-controls"));
            controls.push(ControlBounds {
                label: label.to_owned(),
                rect,
                clip: ui.clip_rect(),
                enabled: ui.is_enabled(),
            });
        });
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn navigation(ui: &mut egui::Ui, label: &str, selected: bool, p: Palette) -> egui::Response {
    control(ui, label, None, selected, false, p)
}

/// The paper sample communicates a real reader setting, including its selected
/// state; keyboard focus remains a separate outer outline.
pub fn reading_environment(
    ui: &mut egui::Ui,
    label: &str,
    paper: Color32,
    selected: bool,
    p: Palette,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(122.0, 48.0), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 7, p.raised);
    }
    let sample =
        egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 8.0), egui::vec2(22.0, 30.0));
    ui.painter().rect_filled(sample, 3, paper);
    let sample_ink = if paper.r() < 80 {
        Color32::from_rgb(224, 222, 213)
    } else {
        Color32::from_rgb(75, 71, 64)
    };
    for y in [8.0, 13.0, 18.0] {
        ui.painter().line_segment(
            [
                sample.min + egui::vec2(5.0, y),
                sample.min + egui::vec2(17.0, y),
            ],
            egui::Stroke::new(1.0, sample_ink),
        );
    }
    ui.painter().text(
        rect.min + egui::vec2(40.0, 24.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        p.ink,
    );
    if selected {
        ui.painter()
            .circle_filled(rect.right_center() - egui::vec2(9.0, 0.0), 2.5, p.accent);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            7,
            egui::Stroke::new(2.0, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    if ui.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("capture-control-bounds"))
            .unwrap_or(false)
    }) {
        ui.ctx().data_mut(|data| {
            data.get_temp_mut_or_default::<Vec<ControlBounds>>(egui::Id::new("capture-controls"))
                .push(ControlBounds {
                    label: label.to_owned(),
                    rect,
                    clip: ui.clip_rect(),
                    enabled: ui.is_enabled(),
                })
        });
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
