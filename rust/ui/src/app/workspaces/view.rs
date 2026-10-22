use super::*;
impl YamantakaApp {
    pub(in crate::app) fn workspace_dialog(&mut self, ctx: &egui::Context) {
        if !self.workspaces.open {
            return;
        }
        let mut state = std::mem::take(&mut self.workspaces);
        let p = Palette::new(self.dark);
        let busy = state.pending.is_some()
            || self.tabs.busy()
            || self.opening.busy()
            || self.reader.reader_preferences_loading
            || self.library_selection.removing();
        let mut command = None;
        let mut close = false;
        let modal=egui::Modal::new(egui::Id::new("saved-workspaces"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(16))
            .show(ctx,|ui| {
                ui.set_width((ctx.content_rect().width()-56.0).clamp(180.0,620.0));
                theme::display(ui,self.tr("Espacios de trabajo","Workspaces"),24.0);
                ui.label(self.tr("Guarda tus lecturas y su cola para retomarlas después.","Save your readings and their queue to return to them later."));
                if let Some(error)=&state.error {ui.colored_label(p.accent,error);}
                if let Some(notice)=&state.notice {ui.label(notice);}
                if let Some(undo)=&state.undo {
                    ui.add_enabled_ui(!busy,|ui| {
                        if theme::bounded_navigation(ui,self.tr("Deshacer eliminación","Undo deletion"),p).clicked() {
                            command=Some(WorkspaceCommand::Change(WorkspaceChange::UndoDelete {token:undo.token.clone()}));
                        }
                    });
                }
                egui::ScrollArea::vertical().max_height((ctx.content_rect().height()-210.0).clamp(40.0,480.0)).show(ui,|ui| {
                    ui.add_enabled_ui(!busy,|ui| {
                        ui.label(self.tr("Nuevo espacio","New workspace"));
                        ui.add(egui::TextEdit::singleline(&mut state.draft).hint_text(self.tr("Nombre del espacio","Workspace name")).char_limit(80).desired_width(ui.available_width()));
                        ui.add_enabled_ui(!state.draft.trim().is_empty() && !self.tabs.model.tabs().is_empty(),|ui| {
                            if theme::bounded_navigation(ui,self.tr("Guardar lecturas actuales","Save current readings"),p).clicked() {
                                self.sync_tab_context();
                                command=Some(WorkspaceCommand::Change(WorkspaceChange::Create {name:state.draft.clone(),tabs:Box::new(self.tabs.model.clone()),view:Box::new(self.capture_workspace_view())}));
                            }
                        });
                        ui.add_space(12.0);
                        ui.separator();
                        if state.entries.is_empty() && state.pending.is_none() {ui.label(self.tr("Aún no hay espacios guardados.","No saved workspaces yet."));}
                        for entry in &state.entries {
                            ui.push_id(&entry.id,|ui| {
                                ui.add_space(8.0);
                                let name: String=entry.name.chars().take(160).collect();
                                ui.label(RichText::new(name).size(18.0).color(p.ink));
                                let description=format!("{} {}{}",entry.readings,self.tr("lecturas","readings"),if entry.is_default {self.tr(" · Predeterminado"," · Default")} else {""});
                                ui.label(RichText::new(description).color(p.secondary));
                                ui.horizontal_wrapped(|ui| {
                                    if theme::bounded_navigation(ui,self.tr("Abrir","Open"),p).clicked() {command=Some(WorkspaceCommand::Load(entry.into()));}
                                    let action=theme::bounded_navigation(ui,self.tr("Opciones","Options"),p);
                                    egui::Popup::menu(&action).show(|ui| {
                                        for (label,kind) in [(self.tr("Renombrar","Rename"),0),(self.tr("Duplicar","Duplicate"),1),(self.tr("Actualizar con las lecturas actuales","Update from current readings"),2),(self.tr("Hacer predeterminado","Make default"),3),(self.tr("Eliminar","Delete"),4)] {
                                            if theme::bounded_navigation(ui,label,p).clicked() {
                                                match kind {
                                                    0|1=>state.edit=Some((entry.clone(),entry.name.clone(),kind==1)),
                                                    2|4=>{state.confirm=Some((entry.clone(),kind==2));state.focus_cancel=true;},
                                                    _=>command=Some(WorkspaceCommand::Change(WorkspaceChange::SetDefault {expected:entry.into()})),
                                                }
                                                ui.close();
                                            }
                                        }
                                    });
                                });
                                if let Some((source,draft,duplicate))=&mut state.edit && source.id==entry.id {
                                    ui.label(if *duplicate {self.tr("Nombre de la copia","Copy name")} else {self.tr("Nuevo nombre","New name")});
                                    ui.add(egui::TextEdit::singleline(draft).char_limit(80).desired_width(ui.available_width()));
                                    let mut cancel_edit=false;
                                    ui.horizontal_wrapped(|ui| {
                                        ui.add_enabled_ui(!draft.trim().is_empty(),|ui| {
                                            if theme::bounded_navigation(ui,self.tr("Guardar","Save"),p).clicked() {
                                                let expected=WorkspaceRevision::from(&*source);
                                                command=Some(WorkspaceCommand::Change(if *duplicate {WorkspaceChange::Duplicate {expected,name:draft.clone()}} else {WorkspaceChange::Rename {expected,name:draft.clone()}}));
                                            }
                                        });
                                        if theme::bounded_navigation(ui,self.tr("Cancelar","Cancel"),p).clicked() {cancel_edit=true;}
                                    });
                                    if cancel_edit {state.edit=None;}
                                }
                                ui.add_space(8.0);
                                ui.separator();
                            });
                        }
                    });
                });
                if let Some((entry,replace))=&state.confirm {
                    ui.add_space(8.0);
                    ui.label(if *replace {self.tr("Se reemplazarán las lecturas guardadas de este espacio.","This replaces the workspace's saved readings.")} else {self.tr("Se eliminará el espacio guardado. Puedes deshacer la última eliminación durante esta sesión. Los PDF y las lecturas abiertas se conservan.","The saved workspace will be deleted. The last deletion can be undone during this session. PDFs and open readings are preserved.")});
                    let mut cancel_confirm=false;
                    ui.add_enabled_ui(!busy,|ui| ui.horizontal_wrapped(|ui| {
                        let cancel=theme::bounded_navigation(ui,self.tr("Cancelar","Cancel"),p);
                        if state.focus_cancel {cancel.request_focus();state.focus_cancel=false;}
                        if cancel.clicked() {cancel_confirm=true;}
                        if theme::bounded_navigation(ui,self.tr("Confirmar","Confirm"),p).clicked() {
                            let expected=entry.into();
                            command=Some(WorkspaceCommand::Change(if *replace {
                                self.sync_tab_context();
                                WorkspaceChange::Replace {expected,tabs:Box::new(self.tabs.model.clone()),view:Some(Box::new(self.capture_workspace_view()))}
                            } else {WorkspaceChange::Delete {expected}}));
                        }
                    }));
                    if cancel_confirm {state.confirm=None;}
                }
                ui.horizontal_wrapped(|ui| {
                    let response=theme::navigation(ui,self.tr("Cerrar","Close"),false,p);
                    if state.focus_close {response.request_focus();state.focus_close=false;}
                    if response.clicked() {close=true;}
                    ui.add_enabled_ui(!busy,|ui| {
                        if theme::bounded_navigation(ui,self.tr("Actualizar lista","Refresh list"),p).clicked() {command=Some(WorkspaceCommand::List {offset:state.offset});}
                        ui.add_enabled_ui(state.offset>0,|ui| {if theme::bounded_navigation(ui,self.tr("Anteriores","Previous"),p).clicked() {state.offset=state.offset.saturating_sub(80);command=Some(WorkspaceCommand::List {offset:state.offset});}});
                        ui.add_enabled_ui((state.offset+state.entries.len())<(state.total as usize),|ui| {if theme::bounded_navigation(ui,self.tr("Siguientes","Next"),p).clicked() {state.offset+=80;command=Some(WorkspaceCommand::List {offset:state.offset});}});
                    });
                });
            });
        if close || modal.should_close() {
            if state.loading
                && let Some(cancel) = state.pending.take()
            {
                cancel.cancel();
            }
            state.open = false;
            state.edit = None;
            state.confirm = None;
        }
        self.workspaces = state;
        if let Some(command) = command {
            self.workspace_request(command);
        }
    }
}
