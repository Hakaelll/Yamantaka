use super::YamantakaApp;
use crate::{
    book,
    theme::{self, Palette},
};
use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, StrokeKind};
use yamantaka_application::runtime::Action;
use yamantaka_domain::Document;

const WIDE: f32 = 760.0;
const ACTIONS: f32 = 88.0;

impl YamantakaApp {
    /// Inside the library's existing scroll area; catalog pages are bounded at 80.
    pub(super) fn list_documents(
        &mut self,
        ui: &mut egui::Ui,
        documents: Vec<Document>,
        p: Palette,
    ) {
        let width = ui.available_width();
        let wide = width >= WIDE;
        if wide {
            let (header, _) = ui.allocate_exact_size(egui::vec2(width, 36.0), Sense::hover());
            if ui.is_rect_visible(header) {
                let columns = Columns::new(
                    header,
                    self.library_column("progress") || self.library_column("status"),
                    self.library_column("pages"),
                );
                line_label(
                    ui,
                    "heading-title",
                    columns.title,
                    self.tr("Documento", "Document"),
                    12.0,
                    p.secondary,
                    false,
                );
                if self.library_column("progress") || self.library_column("status") {
                    line_label(
                        ui,
                        "heading-progress",
                        columns.progress,
                        self.tr("Lectura", "Reading"),
                        12.0,
                        p.secondary,
                        false,
                    );
                }
                if self.library_column("pages") {
                    line_label(
                        ui,
                        "heading-pages",
                        columns.pages,
                        self.tr("Páginas", "Pages"),
                        12.0,
                        p.secondary,
                        false,
                    );
                }
                line_label(
                    ui,
                    "heading-actions",
                    columns.actions,
                    self.tr("Acciones", "Actions"),
                    12.0,
                    p.secondary,
                    false,
                );
                ui.painter()
                    .hline(header.x_range(), header.bottom(), Stroke::new(1.0, p.line));
            }
        }
        ui.add_space(4.0);
        for document in documents {
            ui.push_id(("library-list", &document.id), |ui| {
                self.document_list_row(ui, &document, width, wide, p);
            });
        }
    }

    fn document_list_row(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        width: f32,
        wide: bool,
        p: Palette,
    ) {
        let summary_y = 64.0
            + if self.library_column("author") {
                30.0
            } else {
                0.0
            };
        let has_summary = self.library_column("progress") || self.library_column("pages");
        let footer_y = summary_y + if has_summary { 42.0 } else { 12.0 };
        let height = if wide { 92.0 } else { footer_y + 48.0 };
        let (row, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
        // No hidden book meshes, texture work, animation IDs or text galleys.
        if !ui.is_rect_visible(row) && self.library_selection.focus.as_deref() != Some(&document.id)
        {
            return;
        }
        let selected = self.library_selection.model.contains(&document.id);
        let progress = if document.reading_progress.is_finite() {
            document.reading_progress.clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        let percent = format!("{:.0}%", progress * 100.0);
        let current_page = if document.page_count == 0 {
            0
        } else {
            document
                .last_page_index
                .saturating_add(1)
                .min(document.page_count)
        };
        let page_position = format!(
            "{} {} {} {}",
            self.tr("Página", "Page"),
            current_page,
            self.tr("de", "of"),
            document.page_count
        );
        let author = document
            .author
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.tr("Autor sin especificar", "Author not specified"));
        let author = if document.favorite {
            format!("{author} · {}", self.tr("Favorito", "Favorite"))
        } else {
            author.to_owned()
        };
        let metadata = if document.missing {
            self.tr("Archivo no disponible", "File unavailable")
                .to_owned()
        } else if progress >= 1.0 {
            self.tr("Lectura completada", "Reading completed")
                .to_owned()
        } else if progress > 0.0 {
            page_position.clone()
        } else {
            self.tr("Por empezar", "Not started").to_owned()
        };
        let content = if wide {
            Rect::from_min_max(
                row.min + egui::vec2(72.0, 0.0),
                egui::pos2(row.right() - ACTIONS - 24.0, row.bottom()),
            )
        } else {
            Rect::from_min_max(row.min, egui::pos2(row.right(), row.top() + 130.0))
        };
        let response = ui.interact(content, ui.id().with("select"), Sense::click());
        if self.preview_list_focus && document.id == "demo-1" {
            response.request_focus();
            self.preview_list_focus = false;
        }
        let semantic_label = format!(
            "{}, {}, {}, {} {}, {}",
            document.display_title(),
            author,
            percent,
            document.page_count,
            self.tr("páginas", "pages"),
            metadata
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                ui.is_enabled(),
                selected,
                &semantic_label,
            )
        });
        let hovered = ui.rect_contains_pointer(row);
        let fill = if selected {
            p.raised
        } else if hovered {
            p.surface
        } else {
            p.canvas
        };
        ui.painter()
            .rect_filled(row.shrink2(egui::vec2(0.0, 2.0)), 8.0, fill);
        if selected {
            ui.painter().circle_filled(
                egui::pos2(
                    row.left() + 6.0,
                    if wide {
                        row.center().y
                    } else {
                        row.top() + 28.0
                    },
                ),
                3.0,
                p.accent,
            );
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                row.shrink(2.0),
                8.0,
                Stroke::new(2.0, p.accent),
                StrokeKind::Inside,
            );
        }
        ui.painter()
            .hline(row.x_range(), row.bottom(), Stroke::new(1.0, p.line));

        if wide {
            let columns = Columns::new(
                row,
                self.library_column("progress") || self.library_column("status"),
                self.library_column("pages"),
            );
            let cover_rect =
                Rect::from_min_size(row.min + egui::vec2(8.0, 5.0), egui::vec2(54.0, 82.0));
            let mut cover_ui = bounded_child(ui, "miniature", cover_rect);
            let cover = book::miniature(
                &mut cover_ui,
                document,
                54.0,
                selected,
                self.reduced_motion,
                p,
            );
            self.library_book_response(ui, &cover, document);
            cover
                .on_hover_text(&semantic_label)
                .context_menu(|ui| self.list_context_menu(ui, document, p));
            line_label(
                ui,
                "title",
                at_y(columns.title, row.top() + 14.0, 24.0),
                document.display_title(),
                16.0,
                p.ink,
                true,
            );
            if self.library_column("author") {
                line_label(
                    ui,
                    "author",
                    at_y(columns.title, row.top() + 44.0, 20.0),
                    &author,
                    13.0,
                    p.secondary,
                    false,
                );
            }
            if document.missing {
                line_label(
                    ui,
                    "missing",
                    at_y(columns.title, row.top() + 68.0, 18.0),
                    &metadata,
                    12.0,
                    p.accent,
                    false,
                );
            }
            if self.library_column("progress") {
                line_label(
                    ui,
                    "progress-label",
                    at_y(columns.progress, row.top() + 16.0, 22.0),
                    &format!("{percent} {}", self.tr("leído", "read")),
                    14.0,
                    p.ink,
                    false,
                );
                progress_track(
                    ui,
                    Rect::from_min_size(
                        egui::pos2(columns.progress.left(), row.top() + 46.0),
                        egui::vec2(columns.progress.width() - 16.0, 3.0),
                    ),
                    progress,
                    p,
                );
            }
            if self.library_column("status") {
                line_label(
                    ui,
                    "position",
                    at_y(
                        columns.progress,
                        row.top()
                            + if self.library_column("progress") {
                                58.0
                            } else {
                                30.0
                            },
                        22.0,
                    ),
                    &metadata,
                    12.0,
                    p.secondary,
                    false,
                );
            }
            if self.library_column("pages") {
                line_label(
                    ui,
                    "pages",
                    at_y(columns.pages, row.top() + 30.0, 24.0),
                    &document.page_count.to_string(),
                    16.0,
                    p.ink,
                    false,
                );
            }
            self.list_actions(
                ui,
                document,
                at_y(columns.actions, row.top() + 26.0, 40.0),
                p,
            );
        } else {
            let body = row.shrink2(egui::vec2(12.0, 0.0));
            let title = at_y(body, row.top() + 14.0, 44.0);
            two_line_title(ui, title, document.display_title(), p.ink);
            if self.library_column("author") {
                line_label(
                    ui,
                    "author",
                    at_y(body, row.top() + 64.0, 20.0),
                    &author,
                    13.0,
                    p.secondary,
                    false,
                );
            }
            let mut summary = Vec::new();
            if self.library_column("progress") {
                summary.push(format!("{percent} {}", self.tr("leído", "read")));
            }
            if self.library_column("pages") {
                summary.push(format!(
                    "{} {}",
                    document.page_count,
                    self.tr("páginas", "pages")
                ));
            }
            if !summary.is_empty() {
                line_label(
                    ui,
                    "summary",
                    at_y(body, row.top() + summary_y, 22.0),
                    &summary.join(" · "),
                    13.0,
                    p.ink,
                    false,
                );
            }
            if self.library_column("progress") {
                progress_track(
                    ui,
                    Rect::from_min_size(
                        egui::pos2(body.left(), row.top() + summary_y + 28.0),
                        egui::vec2(body.width(), 3.0),
                    ),
                    progress,
                    p,
                );
            }
            let action_rect = Rect::from_min_size(
                egui::pos2(body.right() - ACTIONS, row.top() + footer_y),
                egui::vec2(ACTIONS, 40.0),
            );
            let status_rect = Rect::from_min_max(
                egui::pos2(body.left(), row.top() + footer_y + 10.0),
                egui::pos2(action_rect.left() - 8.0, row.top() + footer_y + 30.0),
            );
            if self.library_column("status") || document.missing {
                line_label(
                    ui,
                    "position",
                    status_rect,
                    &metadata,
                    12.0,
                    if document.missing {
                        p.accent
                    } else {
                        p.secondary
                    },
                    false,
                );
            }
            self.list_actions(ui, document, action_rect, p);
        }
        self.library_book_response(ui, &response, document);
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(format!(
                "{semantic_label}\n{}",
                self.tr(
                    "Doble clic o Intro para leer",
                    "Double-click or Enter to read"
                )
            ))
            .context_menu(|ui| self.list_context_menu(ui, document, p));
    }

    fn list_actions(&mut self, ui: &mut egui::Ui, document: &Document, rect: Rect, p: Palette) {
        let mut actions = bounded_child(ui, "actions", rect);
        actions.spacing_mut().item_spacing.x = 8.0;
        actions.horizontal(|ui| {
            let read = format!("{}: {}", self.tr("Leer", "Read"), document.display_title());
            if theme::icon_button(ui, &read, theme::Icon::Right, p).clicked() {
                self.selected = Some(document.id.clone());
                self.open(document.clone());
            }
            let details = format!(
                "{}: {}",
                self.tr("Ver ficha", "Book details"),
                document.display_title()
            );
            if theme::icon_button(ui, &details, theme::Icon::Panel, p).clicked() {
                self.selected = Some(document.id.clone());
                self.inspector = true;
            }
        });
    }

    pub(super) fn list_context_menu(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        if theme::control(
            ui,
            self.tr("Leer documento", "Read document"),
            Some(theme::Icon::Right),
            false,
            false,
            p,
        )
        .clicked()
        {
            self.selected = Some(document.id.clone());
            self.open(document.clone());
            ui.close();
        }
        if theme::control(
            ui,
            self.tr("Ver ficha", "Book details"),
            Some(theme::Icon::Panel),
            false,
            false,
            p,
        )
        .clicked()
        {
            self.selected = Some(document.id.clone());
            self.inspector = true;
            ui.close();
        }
        let favorite = if document.favorite {
            self.tr("Quitar de favoritos", "Remove from favorites")
        } else {
            self.tr("Añadir a favoritos", "Add to favorites")
        };
        if ui
            .add_enabled_ui(!self.library_selection.writing(), |ui| {
                theme::navigation(ui, favorite, document.favorite, p)
            })
            .inner
            .clicked()
        {
            self.send(Action::Favorite(document.id.clone(), !document.favorite));
            self.refresh();
            ui.close();
        }
        if theme::navigation(
            ui,
            self.tr("Mostrar en Explorer", "Show in Explorer"),
            false,
            p,
        )
        .clicked()
        {
            if let Err(error) = yamantaka_windows::reveal(&document.path) {
                self.error = Some(error.to_string());
            }
            ui.close();
        }
    }
}

struct Columns {
    title: Rect,
    progress: Rect,
    pages: Rect,
    actions: Rect,
}
impl Columns {
    fn new(rect: Rect, show_progress: bool, show_pages: bool) -> Self {
        let actions = Rect::from_min_max(
            egui::pos2(rect.right() - ACTIONS - 12.0, rect.top()),
            egui::pos2(rect.right() - 12.0, rect.bottom()),
        );
        let mut right = actions.left() - 24.0;
        let pages = Rect::from_min_size(
            egui::pos2(right - if show_pages { 80.0 } else { 0.0 }, rect.top()),
            egui::vec2(if show_pages { 80.0 } else { 0.0 }, rect.height()),
        );
        if show_pages {
            right = pages.left() - 24.0;
        }
        let progress = Rect::from_min_size(
            egui::pos2(right - if show_progress { 156.0 } else { 0.0 }, rect.top()),
            egui::vec2(if show_progress { 156.0 } else { 0.0 }, rect.height()),
        );
        if show_progress {
            right = progress.left() - 24.0;
        }
        let title = Rect::from_min_max(
            rect.min + egui::vec2(72.0, 0.0),
            egui::pos2(right, rect.bottom()),
        );
        Self {
            title,
            progress,
            pages,
            actions,
        }
    }
}

fn at_y(rect: Rect, y: f32, height: f32) -> Rect {
    Rect::from_min_size(
        egui::pos2(rect.left(), y),
        egui::vec2(rect.width().max(0.0), height),
    )
}

fn bounded_child(ui: &mut egui::Ui, salt: &str, rect: Rect) -> egui::Ui {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(salt)
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(ui.clip_rect().intersect(rect));
    child
}

fn line_label(
    ui: &mut egui::Ui,
    salt: &str,
    rect: Rect,
    text: &str,
    size: f32,
    color: Color32,
    strong: bool,
) {
    if rect.width() <= 0.0 {
        return;
    }
    let mut child = bounded_child(ui, salt, rect);
    let text_style = RichText::new(text).size(size).color(color);
    child
        .add(
            egui::Label::new(if strong {
                text_style.strong()
            } else {
                text_style
            })
            .truncate()
            .selectable(false),
        )
        .on_hover_text(text);
}

fn two_line_title(ui: &mut egui::Ui, rect: Rect, text: &str, color: Color32) {
    let mut child = bounded_child(ui, "title", rect);
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        egui::FontId::proportional(16.0),
        color,
        rect.width(),
    );
    job.wrap.max_rows = 2;
    job.wrap.overflow_character = Some('…');
    child
        .add(egui::Label::new(job).selectable(false))
        .on_hover_text(text);
}

fn progress_track(ui: &egui::Ui, rect: Rect, progress: f32, p: Palette) {
    ui.painter().rect_filled(rect, 2.0, p.line);
    if progress > 0.0 {
        let fill =
            Rect::from_min_size(rect.min, egui::vec2(rect.width() * progress, rect.height()));
        ui.painter().rect_filled(fill, 2.0, p.accent);
    }
}
