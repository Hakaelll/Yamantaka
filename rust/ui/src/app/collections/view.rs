use super::*;
use crate::app::settings::quiet;

impl YamantakaApp {
    pub(in crate::app) fn collections_view(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let cramped = ui.available_width() < 700.0;
        let controls_layout = if cramped {
            egui::Layout::top_down(egui::Align::Min)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };
        egui::ScrollArea::vertical().id_salt("collection-manager").animated(!self.reduced_motion).show(ui,|ui| {
            egui::Frame::new().inner_margin(egui::Margin::same(if cramped {16} else {40})).show(ui,|ui| {
                ui.set_max_width(ui.available_width().min(1100.0));
                ui.horizontal_wrapped(|ui| {
                    theme::display(ui,self.tr("Colecciones","Collections"),if cramped {26.0} else {38.0});
                    if theme::navigation(ui,self.tr("Volver a la biblioteca","Back to library"),false,p).clicked() {self.show_surface(Surface::Library);}
                });
                ui.add_space(8.0);
                if let Some(document)=&self.collections.context {
                    quiet(ui,&format!("{} {}",self.tr("Organizar:","Organize:"),document.display_title()),p);
                } else {quiet(ui,self.tr("Reúne tus lecturas por tema o proyecto.","Keep readings together by subject or project."),p);}
                ui.add_space(if cramped {16.0} else {28.0});
                ui.add_enabled_ui(self.collections.writing.is_none() && !self.library_selection.writing(),|ui| {
                    let label=ui.label(self.tr("Nueva colección","New collection"));
                    ui.allocate_ui_with_layout(egui::vec2(ui.available_width(),36.0),controls_layout,|ui| {
                        let width=ui.available_width().min(400.0);
                        ui.add_sized([width,36.0],egui::TextEdit::singleline(&mut self.collections.draft).char_limit(60).background_color(p.surface).margin(egui::vec2(10.0,8.0))).labelled_by(label.id);
                        let enabled=!self.collections.draft.trim().is_empty() && self.collections.draft.trim().encode_utf16().count()<=60;
                        if ui.add_enabled_ui(enabled,|ui|theme::control(ui,self.tr("Crear","Create"),Some(theme::Icon::Plus),false,true,p)).inner.clicked() {
                            self.change_collection(CollectionChange::Create{name:self.collections.draft.clone(),icon:"folder".into()});
                        }
                    });
                });
                ui.add_space(24.0);
                let label=ui.label(self.tr("Buscar colecciones","Search collections"));
                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(),36.0),controls_layout,|ui| {
                    if ui.add_sized([ui.available_width().min(400.0),36.0],egui::TextEdit::singleline(&mut self.collections.query.text).char_limit(256).background_color(p.surface).margin(egui::vec2(10.0,8.0))).labelled_by(label.id).changed() {
                        self.collections.query.offset=0;self.load_collections();
                    }
                    if theme::navigation(ui,self.tr("Actualizar","Refresh"),false,p).clicked() {self.load_collections();}
                });
                if self.collections.failed || self.collections.write_failed {
                    ui.add_space(12.0);
                    Self::file_status_label(ui,self.tr("No se pudo completar la operación. Revisa el nombre y actualiza antes de reintentar.","The operation could not finish. Check the name and refresh before retrying."),p);
                }
                let busy=self.library_selection.writing() || self.collections.writing.is_some() || self.collections.pending.is_some() || self.collections.failed;
                if self.collections.writing.is_some() || self.library_selection.writing() {quiet(ui,self.tr("Guardando cambios…","Saving changes…"),p);}
                ui.add_space(16.0);ui.separator();
                let entries=self.collections.entries.clone();
                for entry in &entries {
                    ui.push_id(&entry.id,|ui| {
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            let (rect,_) = ui.allocate_exact_size(egui::vec2(20.0,24.0),egui::Sense::hover());
                            theme::icon(ui.painter(),rect.center(),collection_icon(&entry.icon),p.secondary);
                            ui.label(RichText::new(&entry.name).size(18.0).strong());
                        });
                        let count=format!("{} {}{}",entry.document_count,if entry.document_count==1 {self.tr("libro","book")} else {self.tr("libros","books")},if entry.pinned {self.tr(" · Fijada"," · Pinned")} else {""});
                        quiet(ui,&count,p);
                        let actions=ui.add_enabled_ui(!busy,|ui| {
                            ui.horizontal_wrapped(|ui| {
                                if self.collections.context.is_some() {
                                    let mut included=entry.contains_document;
                                    if ui.checkbox(&mut included,self.tr("Incluye este libro","Includes this book")).changed() {
                                        self.change_collection(CollectionChange::SetMembership{expected:entry.into(),document:self.collections.query.document.clone().unwrap(),included});
                                    }
                                }
                                if theme::navigation(ui,self.tr("Ver libros","View books"),false,p).clicked() {self.apply_collection_filter(entry);}
                                let more=theme::icon_button(ui,self.tr("Acciones de colección","Collection actions"),theme::Icon::Menu,p);
                                let mut popup=egui::Popup::menu(&more);
                                if self.collections.preview_menu && entry.name=="Reading seminar" {popup=popup.open(true);}
                                popup.show(|ui| {
                                    ui.set_max_width((ui.ctx().content_rect().width()-24.0).max(160.0));
                                    egui::ScrollArea::vertical().id_salt(("collection-actions",&entry.id)).max_height((ui.ctx().content_rect().height()-32.0).max(40.0)).show(ui,|ui| {
                                    if theme::navigation(ui,if entry.pinned {self.tr("Desfijar","Unpin")} else {self.tr("Fijar","Pin")},entry.pinned,p).clicked() {
                                        self.change_collection(CollectionChange::Pin{expected:entry.into(),pinned:!entry.pinned});ui.close();
                                    }
                                    if theme::navigation(ui,self.tr("Mover antes","Move earlier"),false,p).on_hover_text(self.tr("Ordena dentro de las colecciones fijadas o sin fijar, incluso fuera de esta página o búsqueda.","Reorder within pinned or unpinned collections, including outside this page or search.")).clicked() {
                                        self.change_collection(CollectionChange::Move{expected:entry.into(),direction:CollectionDirection::Earlier});ui.close();
                                    }
                                    if theme::navigation(ui,self.tr("Mover después","Move later"),false,p).clicked() {
                                        self.change_collection(CollectionChange::Move{expected:entry.into(),direction:CollectionDirection::Later});ui.close();
                                    }
                                    if theme::navigation(ui,self.tr("Cambiar icono","Change icon"),false,p).clicked() {self.collections.icon_editing=Some(entry.id.clone());self.collections.editing=None;self.collections.deleting=None;ui.close();}
                                    if theme::navigation(ui,self.tr("Cambiar nombre","Rename"),false,p).clicked() {self.collections.editing=Some((entry.id.clone(),entry.name.clone()));self.collections.deleting=None;self.collections.icon_editing=None;ui.close();}
                                    if theme::navigation(ui,self.tr("Eliminar colección…","Delete collection…"),false,p).clicked() {self.collections.deleting=Some(entry.clone());self.collections.editing=None;self.collections.icon_editing=None;ui.close();}
                                    });
                                });
                            });
                        });
                        if self.collections.icon_editing.as_ref()==Some(&entry.id) {
                            let editor=ui.add_enabled_ui(!busy,|ui| {
                                ui.label(self.tr("Icono de la colección","Collection icon"));
                                ui.horizontal_wrapped(|ui| {
                                    for icon in COLLECTION_ICONS {
                                        let label=match icon {"folder"=>self.tr("Carpeta","Folder"),"study"=>self.tr("Estudio","Study"),"reference"=>self.tr("Referencia","Reference"),"archive"=>self.tr("Archivo","Archive"),_=>self.tr("Estrella","Star")};
                                        if theme::control(ui,label,Some(collection_icon(icon)),entry.icon==icon,false,p).clicked() {
                                            self.change_collection(CollectionChange::Icon{expected:entry.into(),icon:icon.into()});
                                        }
                                    }
                                    if theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p).clicked() {self.collections.icon_editing=None;}
                                });
                            });
                            if cramped && self.demo && self.capture.is_some() && self.collections.preview.is_none() {ui.scroll_to_rect(editor.response.rect,Some(egui::Align::Max));}
                        }
                        if self.collections.editing.as_ref().is_some_and(|(id,_)|*id==entry.id) {
                            ui.add_enabled_ui(!busy,|ui| {
                                let label=ui.label(self.tr("Nombre de la colección","Collection name"));
                                let draft=&mut self.collections.editing.as_mut().unwrap().1;
                                ui.add_sized([ui.available_width().min(400.0),36.0],egui::TextEdit::singleline(draft).char_limit(60).background_color(p.surface).margin(egui::vec2(10.0,8.0))).labelled_by(label.id);
                                ui.horizontal_wrapped(|ui| {
                                    if theme::navigation(ui,self.tr("Guardar nombre","Save name"),false,p).clicked() {
                                        let name=self.collections.editing.as_ref().unwrap().1.clone();
                                        self.change_collection(CollectionChange::Rename{expected:entry.into(),name});
                                    }
                                    if theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p).clicked() {self.collections.editing=None;}
                                });
                            });
                        }
                        if self.collections.deleting.as_ref().is_some_and(|selected|selected.id==entry.id) {
                            ui.add_space(10.0);
                            ui.label(self.tr("Se elimina la colección y su agrupación. Tus libros y PDF se conservan.","This removes the collection and its grouping. Your books and PDFs are kept."));
                            ui.add_enabled_ui(!busy,|ui|ui.horizontal_wrapped(|ui| {
                                if theme::navigation(ui,self.tr("Eliminar colección","Delete collection"),false,p).clicked() {
                                    let expected=self.collections.deleting.as_ref().unwrap().into();
                                    self.change_collection(CollectionChange::Delete{expected});
                                }
                                if theme::navigation(ui,self.tr("Conservar colección","Keep collection"),false,p).clicked() {self.collections.deleting=None;}
                            }));
                        }
                        if cramped && self.demo && self.capture.is_some() && self.collections.preview.is_none() && self.collections.icon_editing.is_none() {ui.scroll_to_rect(actions.response.rect,Some(egui::Align::Center));}
                        ui.add_space(14.0);ui.separator();
                    });
                }
                if self.collections.loaded && self.collections.entries.is_empty() {
                    ui.add_space(24.0);
                    ui.label(self.tr("No hay colecciones que coincidan.","No matching collections."));
                }
                ui.add_space(14.0);
                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(),36.0),controls_layout,|ui| {
                    let offset=self.collections.query.offset;
                    if ui.add_enabled_ui(!busy && offset>0,|ui|theme::navigation(ui,self.tr("Anterior","Previous"),false,p)).inner.clicked() {self.collections.query.offset=offset.saturating_sub(80);self.load_collections();}
                    let first=if self.collections.total==0 {0} else {offset+1};
                    ui.label(format!("{first}–{} / {}",offset+self.collections.entries.len(),self.collections.total));
                    if ui.add_enabled_ui(!busy && ((offset+self.collections.entries.len()) as u64)<self.collections.total,|ui|theme::navigation(ui,self.tr("Siguiente","Next"),false,p)).inner.clicked() {self.collections.query.offset=offset+80;self.load_collections();}
                });
                ui.add_space(20.0);
            });
        });
    }
}

fn collection_icon(value: &str) -> theme::Icon {
    match value {
        "study" => theme::Icon::Book,
        "reference" => theme::Icon::Bookmark,
        "archive" => theme::Icon::Archive,
        "star" => theme::Icon::Star,
        _ => theme::Icon::Folder,
    }
}
