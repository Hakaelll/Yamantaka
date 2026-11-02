use super::*;
use yamantaka_domain::duplicates::merge::{Choice, Field};
impl YamantakaApp {
    pub(in crate::app) fn duplicate_merge_dialog(&mut self, ctx: &egui::Context) {
        if !self.duplicates.merge.open() {
            return;
        }
        let mut state = std::mem::take(&mut self.duplicates.merge);
        let p = Palette::new(self.dark);
        let mut apply = false;
        let mut close = false;
        let compact = ctx.content_rect().height() < 500.0;
        let modal=egui::Modal::new(egui::Id::new("duplicate-consolidation")).frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(16)).show(ctx,|ui|{
            ui.set_width((ctx.content_rect().width()-64.0).clamp(180.0,720.0));
            let top=ui.cursor().top();
            theme::display(ui,self.tr("Consolidar copias","Consolidate copies"),if compact{20.0}else{26.0});
            ui.label(RichText::new(self.tr("Se unen los registros. Los PDF originales no se modifican.","Library records are combined. Original PDFs remain unchanged.")).size(13.0).color(p.secondary));
            let header=ui.cursor().top()-top;
            egui::ScrollArea::vertical().id_salt("duplicate-consolidation-fields").max_height((ctx.content_rect().height()-header-180.0).clamp(32.0,500.0)).show(ui,|ui|{
                ui.add_enabled_ui(state.pending.is_none(),|ui|{
                    if let Some(plan)=&state.plan {
                        ui.label(format!("{} {}",self.tr("Se conserva:","Keep:"),plan.selection.kept.title));
                        ui.label(format!("{} {}",self.tr("Se retira el registro:","Retire record:"),plan.selection.incoming.title));
                        ui.add_space(12.0);
                        field_choice(ui,self.tr("Título","Title"),&plan.title,&mut state.choices.title,self.english,p);
                        field_choice(ui,self.tr("Autor","Author"),&plan.author,&mut state.choices.author,self.english,p);
                        field_choice(ui,self.tr("Nota de biblioteca","Library note"),&plan.note,&mut state.choices.note,self.english,p);
                        ui.add_space(12.0);
                        for (table,es,en) in [("annotations","Anotaciones","Annotations"),("bookmarks","Marcadores","Bookmarks"),("reading_sessions","Sesiones","Sessions"),("collection_documents","Pertenencias a colecciones","Collection memberships"),("workspace_tabs","Lecturas en espacios","Workspace readings")] {
                            let count=plan.affected.get(table).copied().unwrap_or(0);if count>0{ui.label(format!("{}: {count}",self.tr(es,en)));}
                        }
                        ui.add_space(8.0);ui.label(RichText::new(self.tr("En marcadores o lecturas repetidas prevalece el registro conservado. Se prioriza la portada manual. Se guardará un backup antes de unir.","The kept record wins for duplicate bookmarks or workspace readings. Manual covers take priority. A backup is saved before consolidation.")).size(13.0).color(p.secondary));
                    }
                });
                if state.pending.is_some(){ui.label(self.tr(if state.writing(){"Preparando backup y consolidación…"}else{"Preparando revisión…"},if state.writing(){"Preparing backup and consolidation…"}else{"Preparing review…"}));}
                if let Some(notice)=&state.notice {ui.label(notice);}
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui|{
                let cancel=theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p);
                if state.focus_cancel{cancel.request_focus();state.focus_cancel=false;}
                if cancel.clicked(){state.cancel();close=true;}
                if state.pending.is_none() && state.plan.is_some() && theme::navigation(ui,self.tr("Consolidar","Consolidate"),false,p).clicked(){apply=true;}
            });
        });
        if modal.should_close() {
            state.cancel();
            close = true;
        }
        if close && state.pending.is_none() {
            state.plan = None;
        }
        self.duplicates.merge = state;
        if apply {
            self.apply_duplicate_merge();
        }
    }
}
fn field_choice(
    ui: &mut egui::Ui,
    label: &str,
    field: &Field,
    choice: &mut Choice,
    english: bool,
    p: Palette,
) {
    ui.add_space(8.0);
    ui.label(RichText::new(label).size(13.0).color(p.secondary));
    if field.conflict() {
        for (kind, value, prefix) in [
            (
                Choice::Kept,
                field.kept.as_deref(),
                if english {
                    "Kept copy"
                } else {
                    "Copia conservada"
                },
            ),
            (
                Choice::Incoming,
                field.incoming.as_deref(),
                if english { "Other copy" } else { "Otra copia" },
            ),
        ] {
            let text = format!("{prefix}: {}", value.unwrap_or(""));
            if theme::bounded_choice(ui, &text, *choice == kind, p).clicked() {
                *choice = kind;
            }
        }
    } else {
        ui.label(
            field
                .resolve(*choice)
                .unwrap_or(if english { "Empty" } else { "Vacío" }),
        );
    }
}
