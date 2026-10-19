use super::*;
use crate::app::settings::quiet;

fn color(value: &str, p: Palette) -> Color32 {
    if let Some(hex) = value.strip_prefix('#').filter(|hex| hex.len() == 6)
        && let Ok(rgb) = u32::from_str_radix(hex, 16)
    {
        return Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    }
    match value {
        "violet" | "purple" => p.accent,
        "blue" => Color32::from_rgb(78, 142, 214),
        "green" => Color32::from_rgb(82, 156, 110),
        "red" => Color32::from_rgb(217, 89, 99),
        "orange" => Color32::from_rgb(219, 142, 66),
        _ => p.secondary,
    }
}
impl YamantakaApp {
    pub(in crate::app) fn tags_view(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let cramped = ui.available_width() < 700.0;
        let controls_layout = if cramped {
            egui::Layout::top_down(egui::Align::Min)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };
        egui::ScrollArea::vertical().id_salt("tag-manager").auto_shrink([false,false]).animated(!self.reduced_motion).show(ui,|ui| {
            egui::Frame::new().inner_margin(egui::Margin::same(if cramped {16} else {40})).show(ui,|ui| {
                ui.set_max_width(ui.available_width().min(1100.0));
                ui.horizontal_wrapped(|ui| {
                    theme::display(ui,self.tr("Etiquetas","Tags"),if cramped {26.0} else {38.0});
                    if theme::navigation(ui,self.tr("Volver a la biblioteca","Back to library"),false,p).clicked() {self.show_surface(Surface::Library);}
                });
                ui.add_space(8.0);
                if let Some(document)=&self.library_tags.context {quiet(ui,&format!("{} {}",self.tr("Organizar:","Organize:"),document.display_title()),p);}
                else {quiet(ui,self.tr("Conecta tus lecturas con etiquetas compartidas.","Connect your readings with shared tags."),p);}
                ui.add_space(if cramped {16.0} else {28.0});
                ui.add_enabled_ui(self.library_tags.writing.is_none() && !self.library_selection.writing(),|ui| {
                    let label=ui.label(self.tr("Nombre de la etiqueta","Tag name"));
                    ui.with_layout(controls_layout,|ui| {
                        let response=ui.add_sized([ui.available_width().min(400.0),36.0],egui::TextEdit::singleline(&mut self.library_tags.draft).char_limit(60).background_color(p.surface).margin(egui::vec2(10.0,8.0))).labelled_by(label.id);
                        theme::record_control_bounds(ui,self.tr("Nombre de la etiqueta","Tag name"),response.rect);
                        let valid=!self.library_tags.draft.trim().is_empty()&&self.library_tags.draft.trim().encode_utf16().count()<=60;
                        if ui.add_enabled_ui(valid,|ui|theme::control(ui,if self.library_tags.context.is_some() {self.tr("Crear y asignar","Create and assign")} else {self.tr("Crear etiqueta","Create tag")},Some(theme::Icon::Plus),false,valid,p)).inner.clicked() {
                            let name=self.library_tags.draft.clone();
                            let change=if let Some(document)=self.library_tags.query.document.clone() {TagChange::EnsureForDocument{name,document}} else {TagChange::Ensure{name}};
                            self.change_tag(change);
                        }
                    });
                });
                ui.add_space(24.0);
                let label=ui.label(self.tr("Buscar etiquetas","Search tags"));
                ui.with_layout(controls_layout,|ui| {
                    let response=ui.add_sized([ui.available_width().min(400.0),36.0],egui::TextEdit::singleline(&mut self.library_tags.query.text).char_limit(256).background_color(p.surface).margin(egui::vec2(10.0,8.0))).labelled_by(label.id);
                    theme::record_control_bounds(ui,self.tr("Buscar etiquetas","Search tags"),response.rect);
                    if response.changed() {self.library_tags.query.ids.clear();self.library_tags.query.offset=0;self.load_tags();}
                    if theme::navigation(ui,self.tr("Actualizar","Refresh"),false,p).clicked() {self.load_tags();}
                });
                if !self.library_tags.query.ids.is_empty() {
                    ui.add_space(8.0);
                    quiet(ui,self.tr("Mostrando la etiqueta creada o reutilizada.","Showing the created or reused tag."),p);
                    if theme::navigation(ui,self.tr("Mostrar todas las etiquetas","Show all tags"),false,p).clicked() {self.library_tags.query.ids.clear();self.library_tags.query.text.clear();self.library_tags.query.offset=0;self.load_tags();}
                }
                ui.add_space(12.0);
                quiet(ui,self.tr("Al seleccionar varias etiquetas, se muestran libros con cualquiera de ellas.","Selecting several tags shows books with any of them."),p);
                ui.with_layout(controls_layout,|ui| {
                    let count=self.library_tags.filters.len();
                    ui.label(format!("{}: {count}",self.tr("Etiquetas para filtrar","Tag filters")));
                    if ui.add_enabled_ui(count>0,|ui|theme::navigation(ui,self.tr("Ver libros","View books"),false,p)).inner.clicked() {self.show_surface(Surface::Library);}
                    if ui.add_enabled_ui(count>0,|ui|theme::navigation(ui,self.tr("Quitar filtros","Clear filters"),false,p)).inner.clicked() {self.clear_tag_filters();self.offset=0;self.refresh();}
                });
                if self.library_tags.failed||self.library_tags.write_failed {
                    Self::file_status_label(ui,self.tr("No se pudo completar la operación. Revisa el nombre y actualiza antes de reintentar.","The operation could not finish. Check the name and refresh before retrying."),p);
                }
                if self.library_tags.writing.is_some() || self.library_selection.writing() {quiet(ui,self.tr("Guardando cambios…","Saving changes…"),p);}
                ui.add_space(16.0);ui.separator();
                let busy=self.library_selection.writing()||self.library_tags.writing.is_some()||self.library_tags.pending.is_some()||self.library_tags.failed;
                let entries=self.library_tags.entries.clone();
                for tag in &entries {
                    ui.push_id(&tag.id,|ui| {
                        ui.add_space(14.0);
                        let heading=ui.horizontal(|ui| {
                            let (rect,_)=ui.allocate_exact_size(egui::vec2(10.0,10.0),egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(),4.0,color(&tag.color,p));
                            ui.label(RichText::new(&tag.name).size(18.0).strong());
                        });
                        let counts=format!("{} {} · {} {}",tag.document_count,if tag.document_count==1 {self.tr("libro","book")} else {self.tr("libros","books")},tag.annotation_count,if tag.annotation_count==1 {self.tr("anotación","annotation")} else {self.tr("anotaciones","annotations")});
                        quiet(ui,&counts,p);
                        let choices=ui.add_enabled_ui(!busy,|ui|ui.horizontal_wrapped(|ui| {
                            if let Some(document)=self.library_tags.query.document.clone() {
                                let mut included=tag.contains_document;
                                let response=ui.checkbox(&mut included,self.tr("En este libro","On this book"));
                                let accessible=format!("{}: {}",self.tr("En este libro","On this book"),tag.name);
                                response.widget_info(||egui::WidgetInfo::selected(egui::WidgetType::Checkbox,ui.is_enabled(),included,&accessible));
                                theme::record_control_bounds(ui,&accessible,response.rect);
                                if response.changed() {self.change_tag(TagChange::SetMembership{tag:tag.id.clone(),document,included});}
                            }
                            let mut selected=self.library_tags.filters.iter().any(|(id,_)|*id==tag.id);
                            let response=ui.checkbox(&mut selected,self.tr("Filtrar biblioteca","Filter library"));
                            let accessible=format!("{}: {}",self.tr("Filtrar biblioteca","Filter library"),tag.name);
                            response.widget_info(||egui::WidgetInfo::selected(egui::WidgetType::Checkbox,ui.is_enabled(),selected,&accessible));
                            theme::record_control_bounds(ui,&accessible,response.rect);
                            if response.changed() {self.toggle_tag_filter(tag);}
                        }));
                        if cramped&&self.demo&&self.capture.is_some()&&tag.contains_document&&self.library_tags.preview.is_none() {ui.scroll_to_rect(heading.response.rect.union(choices.response.rect),Some(egui::Align::Center));}
                        ui.add_space(14.0);ui.separator();
                    });
                }
                if self.library_tags.loaded&&self.library_tags.entries.is_empty() {ui.add_space(24.0);ui.label(self.tr("No hay etiquetas que coincidan.","No matching tags."));}
                ui.add_space(14.0);
                ui.with_layout(controls_layout,|ui| {
                    let offset=self.library_tags.query.offset;
                    if ui.add_enabled_ui(!busy&&offset>0,|ui|theme::navigation(ui,self.tr("Anterior","Previous"),false,p)).inner.clicked() {self.library_tags.query.offset=offset.saturating_sub(80);self.load_tags();}
                    let first=if self.library_tags.total==0 {0} else {offset+1};
                    ui.label(format!("{first}–{} / {}",offset+self.library_tags.entries.len(),self.library_tags.total));
                    if ui.add_enabled_ui(!busy&&((offset+self.library_tags.entries.len()) as u64)<self.library_tags.total,|ui|theme::navigation(ui,self.tr("Siguiente","Next"),false,p)).inner.clicked() {self.library_tags.query.offset=offset+80;self.load_tags();}
                });
                ui.add_space(20.0);
            });
        });
    }
}
