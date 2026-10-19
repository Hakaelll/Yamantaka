mod preview;
use super::*;
pub(super) use preview::OrganizationPreview;
use yamantaka_application::library_bulk::BulkAction;
use yamantaka_domain::{
    collections::{CollectionPage, CollectionQuery},
    tags::{TagPage, TagQuery},
};
#[derive(Clone, Copy)]
pub(super) enum TargetKind {
    Tag,
    Collection(bool),
}
pub(super) struct Target {
    name: String,
    documents: u64,
    annotation_uses: Option<u64>,
    action: BulkAction,
}
pub(super) struct BulkPicker {
    kind: TargetKind,
    ids: Vec<String>,
    text: String,
    offset: usize,
    entries: Vec<Target>,
    total: u64,
    pending: Option<Cancellation>,
    loaded: bool,
    failed: bool,
}
impl BulkPicker {
    fn new(kind: TargetKind, ids: Vec<String>) -> Self {
        Self {
            kind,
            ids,
            text: String::new(),
            offset: 0,
            entries: Vec::new(),
            total: 0,
            pending: None,
            loaded: false,
            failed: false,
        }
    }

    pub(super) fn ready(&self) -> bool {
        self.pending.is_none()
    }
}
impl Drop for BulkPicker {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(super) fn open_bulk_picker(&mut self, kind: TargetKind) {
        if self.library_selection.writing() || self.library_selection.model.ids().is_empty() {
            return;
        }
        let mut picker = BulkPicker::new(kind, self.library_selection.model.ids().to_vec());
        self.load_bulk_targets(&mut picker);
        self.bulk_picker = Some(picker);
    }
    fn load_bulk_targets(&mut self, picker: &mut BulkPicker) {
        if let Some(cancel) = picker.pending.take() {
            cancel.cancel();
        }
        picker.loaded = false;
        let cancel = Cancellation::default();
        let action = match picker.kind {
            TargetKind::Tag => Action::Tags(
                TagQuery {
                    text: picker.text.clone(),
                    offset: picker.offset,
                    ..Default::default()
                },
                cancel.clone(),
            ),
            TargetKind::Collection(_) => Action::Collections(
                CollectionQuery {
                    text: picker.text.clone(),
                    offset: picker.offset,
                    ..Default::default()
                },
                cancel.clone(),
            ),
        };
        match self.runtime.submit(action) {
            Ok(()) => {
                picker.pending = Some(cancel);
                picker.failed = false;
            }
            Err(_) => picker.failed = true,
        }
    }
    pub(super) fn bulk_target_matches(&self, cancel: &Cancellation) -> bool {
        self.bulk_picker
            .as_ref()
            .and_then(|picker| picker.pending.as_ref())
            .is_some_and(|scope| scope.same_request(cancel) && !scope.is_cancelled())
    }
    pub(super) fn accept_bulk_tags(&mut self, result: yamantaka_domain::Result<TagPage>) {
        let result = result.map(|page| {
            (
                page.entries
                    .into_iter()
                    .map(|tag| Target {
                        name: tag.name,
                        documents: tag.document_count,
                        annotation_uses: Some(tag.annotation_count),
                        action: BulkAction::Tag(tag.id),
                    })
                    .collect(),
                page.total,
            )
        });
        self.accept_bulk_targets(result);
    }
    pub(super) fn accept_bulk_collections(
        &mut self,
        result: yamantaka_domain::Result<CollectionPage>,
    ) {
        let included = matches!(
            self.bulk_picker.as_ref().map(|picker| picker.kind),
            Some(TargetKind::Collection(true))
        );
        let result = result.map(|page| {
            (
                page.entries
                    .into_iter()
                    .map(|collection| Target {
                        action: BulkAction::Collection {
                            expected: (&collection).into(),
                            included,
                        },
                        name: collection.name,
                        documents: collection.document_count,
                        annotation_uses: None,
                    })
                    .collect(),
                page.total,
            )
        });
        self.accept_bulk_targets(result);
    }
    fn accept_bulk_targets(&mut self, result: yamantaka_domain::Result<(Vec<Target>, u64)>) {
        if let Some(picker) = &mut self.bulk_picker {
            picker.pending = None;
            picker.loaded = true;
            match result {
                Ok((entries, total)) => {
                    picker.entries = entries;
                    picker.total = total;
                    picker.failed = false;
                }
                Err(_) => picker.failed = true,
            }
        }
    }
    pub(super) fn bulk_picker_summary(&self) -> serde_json::Value {
        match &self.bulk_picker {
            Some(picker) => {
                serde_json::json!({"open":true,"selected":picker.ids.len(),"visible":picker.entries.len(),"total":picker.total,"offset":picker.offset,"busy":picker.pending.is_some(),"failed":picker.failed,"annotationUses":picker.entries.iter().filter_map(|entry|entry.annotation_uses).sum::<u64>()})
            }
            None => serde_json::json!({"open":false}),
        }
    }
    pub(super) fn bulk_target_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut picker) = self.bulk_picker.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut close = false;
        let title = match picker.kind {
            TargetKind::Tag => self.tr("Asignar etiqueta", "Assign tag"),
            TargetKind::Collection(true) => self.tr("Añadir a colección", "Add to collection"),
            TargetKind::Collection(false) => {
                self.tr("Quitar de colección", "Remove from collection")
            }
        };
        let modal=egui::Modal::new(egui::Id::new("bulk-organization-target"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(16))
            .show(ctx,|ui| {
                ui.set_width((ctx.content_rect().width()-64.0).clamp(160.0,480.0));
                ui.set_height((ctx.content_rect().height()-64.0).clamp(160.0,520.0));
                ui.spacing_mut().item_spacing.y=if compact {4.0} else {8.0};
                theme::display(ui,title,if compact {20.0} else {22.0});
                ui.label(RichText::new(format!("{} {}",picker.ids.len(),if picker.ids.len()==1 {self.tr("libro seleccionado","selected book")} else {self.tr("libros seleccionados","selected books")})).size(if compact {12.0} else {14.0}));
                let search=self.tr("Buscar nombre","Search name");
                let response=ui.add(egui::TextEdit::singleline(&mut picker.text).hint_text(search).char_limit(4096).desired_width(ui.available_width()));
                theme::record_control_bounds(ui,search,response.rect);
                if response.changed() {picker.offset=0;self.load_bulk_targets(&mut picker);}
                let list_height=(ui.available_height()-44.0).max(40.0);
                egui::ScrollArea::vertical().id_salt("bulk-target-list").max_height(list_height).min_scrolled_height(40.0).auto_shrink([false,false]).show(ui,|ui| {
                    if picker.failed {
                        ui.label(self.tr("No se pudo completar la solicitud. Reintenta para actualizar los destinos.","Could not complete the request. Retry to refresh the destinations."));
                        if theme::navigation(ui,self.tr("Reintentar","Retry"),false,p).clicked() {self.load_bulk_targets(&mut picker);}
                    } else if picker.loaded && picker.entries.is_empty() {
                        ui.label(self.tr("No hay coincidencias. Puedes crear etiquetas y colecciones desde Herramientas.","No matches. You can create tags and collections from Tools."));
                    }
                    for target in &picker.entries {
                        let label=format!("{} · {} {}",target.name,target.documents,if target.documents==1 {self.tr("libro","book")} else {self.tr("libros","books")});
                        if ui.add_enabled_ui(picker.pending.is_none() && !picker.failed,|ui|theme::bounded_navigation(ui,&label,p)).inner.clicked() {
                            if self.submit_library_bulk(target.action.clone(),picker.ids.clone()) {close=true;} else {picker.failed=true;}
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add_enabled_ui(picker.pending.is_none() && picker.offset>0,|ui|theme::icon_button(ui,self.tr("Anterior","Previous"),theme::Icon::Left,p)).inner.clicked() {picker.offset=picker.offset.saturating_sub(80);self.load_bulk_targets(&mut picker);}
                    if ui.add_enabled_ui(picker.pending.is_none() && picker.offset+picker.entries.len()<picker.total as usize,|ui|theme::icon_button(ui,self.tr("Siguiente","Next"),theme::Icon::Right,p)).inner.clicked() {picker.offset+=80;self.load_bulk_targets(&mut picker);}
                    if ui.available_width()>120.0 {ui.label(format!("{}/{}",picker.offset/80+1,picker.total.div_ceil(80).max(1)));}
                    if theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p).clicked() {close=true;}
                });
            });
        close |= modal.should_close();
        if !close {
            self.bulk_picker = Some(picker);
        }
    }
}
