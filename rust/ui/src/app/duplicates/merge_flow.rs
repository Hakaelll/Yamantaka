use super::*;
use yamantaka_application::duplicates::merge_job::{Command, MergeReply, MergeRequest, Outcome};
use yamantaka_domain::{
    Failure, Result,
    duplicates::merge::{Choices, Plan, Receipt, Selection},
};
#[derive(Default)]
pub(super) struct MergeState {
    pub(super) pending: Option<(Cancellation, bool)>,
    pub(super) plan: Option<Arc<Plan>>,
    pub(super) choices: Choices,
    pub(super) notice: Option<String>,
    pub(super) receipt: Option<Arc<Receipt>>,
    pub(super) kept: Option<String>,
    pub(super) focus_cancel: bool,
    dismiss_after: bool,
    pub(super) close_after: bool,
    pub(super) preview: Option<(u8, u8)>,
}
impl MergeState {
    pub(super) fn writing(&self) -> bool {
        self.pending.as_ref().is_some_and(|(_, writing)| *writing)
    }
    pub(super) fn open(&self) -> bool {
        self.plan.is_some() || self.pending.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none()
    }
    pub(super) fn cancel(&mut self) {
        self.dismiss_after = true;
        if let Some((token, _)) = &self.pending {
            token.cancel();
        }
        if !self.writing() {
            self.plan = None;
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.is_some(),"writing":self.writing(),"reviewed":self.plan.is_some(),"merged":self.receipt.is_some(),"titleIncoming":self.choices.title==yamantaka_domain::duplicates::merge::Choice::Incoming,"notice":self.notice})
    }
}
impl YamantakaApp {
    fn duplicate_preflight(&self, selection: &Selection) -> Result<()> {
        let ids = [
            selection.kept.document.as_str(),
            selection.incoming.document.as_str(),
        ];
        if self.tabs.busy()
            || self.opening.busy()
            || self.editor_writes_pending()
            || self.library_selection.writing()
            || self.library_tags.writing()
        {
            return Err(Failure::Invalid(
                self.tr(
                    "Termina los cambios pendientes antes de revisar la consolidación.",
                    "Finish pending changes before reviewing consolidation.",
                )
                .into(),
            ));
        }
        if self
            .tabs
            .model
            .tabs()
            .iter()
            .any(|tab| ids.contains(&tab.document()))
            || self
                .reader
                .active
                .as_ref()
                .is_some_and(|(doc, _)| ids.contains(&doc.id.as_str()))
            || self.reference.as_ref().is_some_and(|view| {
                view.pane
                    .active
                    .as_ref()
                    .is_some_and(|(doc, _)| ids.contains(&doc.id.as_str()))
            })
        {
            return Err(Failure::Invalid(self.tr("Cierra las lecturas de estas dos copias desde Herramientas antes de unirlas. Sus posiciones se conservarán.","Close readings of both copies from Tools before consolidating. Their positions will be retained.").into()));
        }
        if !self
            .sessions
            .lifecycle
            .settled_for(&ids.map(str::to_owned))?
        {
            return Err(Failure::Invalid(
                self.tr(
                    "Terminando de guardar la lectura. Vuelve a intentarlo.",
                    "Finishing saved reading changes. Try again.",
                )
                .into(),
            ));
        }
        self.merge_continuity_preflight(&ids)
    }
    pub(in crate::app) fn review_duplicate_pair(
        &mut self,
        group: usize,
        kept: &str,
        incoming: &str,
    ) {
        if self.duplicates.merge.pending.is_some() {
            return;
        }
        let result = (|| -> Result<()> {
            let group = self
                .duplicates
                .report
                .as_ref()
                .and_then(|report| report.groups.get(group))
                .ok_or_else(|| Failure::Invalid("Search for duplicate copies again".into()))?;
            let selection = Selection::from_group(group, kept, incoming)?;
            self.duplicate_preflight(&selection)?;
            self.submit_duplicate_merge(Command::Review(Box::new(selection)))
        })();
        if let Err(error) = result {
            self.duplicates.merge.notice = Some(error.to_string());
        }
    }
    fn submit_duplicate_merge(&mut self, command: Command) -> Result<()> {
        let writing = matches!(command, Command::Apply { .. });
        let cancel = Cancellation::default();
        let preferences = serde_json::to_value(&self.preferences)
            .map_err(|_| Failure::Invalid("Preferences could not be prepared".into()))?;
        self.runtime
            .submit(Action::MergeDuplicates(Box::new(MergeRequest {
                command,
                preferences,
                governor: self.governor.clone(),
                cancel: cancel.clone(),
            })))?;
        self.duplicates.merge.pending = Some((cancel, writing));
        self.duplicates.merge.dismiss_after = false;
        self.duplicates.merge.notice = None;
        self.duplicates.merge.focus_cancel = true;
        Ok(())
    }
    pub(in crate::app) fn apply_duplicate_merge(&mut self) {
        let Some(plan) = self.duplicates.merge.plan.clone() else {
            return;
        };
        let result = (|| -> Result<()> {
            self.duplicate_preflight(&plan.selection)?;
            if !self.demo {
                self.preferences
                    .save(&self.store.join("preferences.json"))?;
            }
            self.library_covers.pause(&[
                plan.selection.kept.document.clone(),
                plan.selection.incoming.document.clone(),
            ]);
            self.submit_duplicate_merge(Command::Apply {
                plan,
                choices: self.duplicates.merge.choices,
            })
        })();
        if let Err(error) = result {
            self.duplicates.merge.notice = Some(error.to_string());
        }
    }
    pub(in crate::app) fn accept_duplicate_merge(
        &mut self,
        ctx: &egui::Context,
        reply: MergeReply,
    ) {
        if !self
            .duplicates
            .merge
            .pending
            .as_ref()
            .is_some_and(|(token, _)| token.same_request(&reply.cancel))
        {
            return;
        }
        self.duplicates.merge.pending = None;
        match reply.result {
            Ok(Outcome::Reviewed(plan)) if !reply.cancel.is_cancelled()=>{self.duplicates.merge.plan=Some(plan);self.duplicates.merge.choices=Choices::default();}
            Ok(Outcome::Reviewed(_))=>{self.duplicates.merge.plan=None;}
            Ok(Outcome::Merged(receipt))=>{
                // A committed write remains real if cancellation arrived after commit.
                if let Some(plan)=&self.duplicates.merge.plan {
                    let old=plan.selection.incoming.file.path.clone();
                    if self.preferences.get("lastDocumentPath").and_then(serde_json::Value::as_str).is_some_and(|path|std::path::Path::new(path)==old) {
                        self.preferences.rust.insert("lastDocumentPath".into(),plan.selection.kept.file.path.to_string_lossy().into_owned().into());
                    }
                }
                if let Err(error)=self.reconcile_merge_continuity(&receipt.incoming,&receipt.kept){self.error=Some(error.to_string());}
                for id in [&receipt.incoming,&receipt.kept]{self.invalidate_library_cover(ctx,id);}
                let selected=self.library_selection.model.ids().iter().filter(|id|**id!=receipt.incoming).cloned().collect::<Vec<_>>();self.library_selection.model.retain(&selected);
                self.documents.retain(|document|document.id!=receipt.incoming);
                self.search.invalidate_removed(std::slice::from_ref(&receipt.incoming));
                self.selected=Some(receipt.kept.clone());self.offset=0;
                self.removal_views_changed();self.relations.leave();
                if !self.demo && let Err(error)=self.preferences.save(&self.store.join("preferences.json")){self.error=Some(error.to_string());}
                self.duplicates.merge.receipt=Some(Arc::new(receipt));self.duplicates.merge.plan=None;
                self.duplicates.merge.notice=Some(self.tr("Registros unidos. Los PDF originales se conservan.","Records consolidated. Original PDFs are preserved.").into());
                self.duplicates.report=None;self.refresh();self.start_duplicate_discovery();
            }
            Err(Failure::QueueFull)=>self.duplicates.merge.notice=Some(self.tr("Hay trabajo en segundo plano. Termínalo o cancélalo y vuelve a confirmar.","Background work is still active. Finish or cancel it, then confirm again.").into()),
            Err(Failure::Cancelled)=>self.duplicates.merge.notice=Some(self.tr("Consolidación cancelada.","Consolidation cancelled.").into()),
            Err(Failure::Invalid(_))=>self.duplicates.merge.notice=Some(self.tr("Los datos o archivos cambiaron. Cancela esta revisión y vuelve a buscar copias.","Data or files changed. Cancel this review and search for copies again.").into()),
            Err(_)=>self.duplicates.merge.notice=Some(self.tr("No se pudo unir. Se han conservado los datos y cualquier copia de seguridad terminada.","Could not consolidate. Data and any completed backup have been preserved.").into()),
        }
        if self.duplicates.merge.dismiss_after {
            self.duplicates.merge.plan = None;
            self.duplicates.merge.dismiss_after = false;
        }
    }
    pub(in crate::app) fn cancel_duplicate_merge_close(&mut self) {
        self.duplicates.merge.close_after = false;
    }
    pub(in crate::app) fn guard_duplicate_merge_close(&mut self, ctx: &egui::Context) {
        if self.duplicates.merge.pending.is_some()
            && ctx.input(|input| input.viewport().close_requested())
        {
            self.duplicates.merge.close_after = true;
            self.duplicates.merge.cancel();
            self.runtime.wake();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if self.duplicates.merge.close_after && self.duplicates.merge.pending.is_none() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub fn preview_duplicate_merge(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.enable_continuity_capture();
            self.preview_duplicates(true);
            self.duplicates.merge.preview = Some((mode, 0));
        }
    }
    pub(in crate::app) fn pump_duplicate_merge_preview(&mut self, ctx: &egui::Context) {
        let Some((mode, phase)) = self.duplicates.merge.preview else {
            return;
        };
        if phase == 0
            && self.duplicates.pending.is_none()
            && self.duplicates.merge.pending.is_none()
            && let Some(report) = &self.duplicates.report
            && let Some(group) = report.groups.first()
        {
            let kept = group.sources[0].catalogue.document.clone();
            let incoming = group.sources[1].catalogue.document.clone();
            self.review_duplicate_pair(0, &kept, &incoming);
            if self.duplicates.merge.pending.is_some() {
                self.duplicates.merge.preview = Some((mode, 1));
            }
        } else if phase == 1
            && self.duplicates.merge.plan.is_some()
            && self.duplicates.merge.pending.is_none()
        {
            if mode == 1 {
                self.duplicates.merge.preview = None;
            } else if mode == 3 {
                self.duplicates.merge.plan = None;
                self.duplicates.merge.notice = Some(
                    self.tr("Consolidación cancelada.", "Consolidation cancelled.")
                        .into(),
                );
                self.duplicates.merge.preview = None;
            } else {
                self.duplicates.merge.choices.title =
                    yamantaka_domain::duplicates::merge::Choice::Incoming;
                self.apply_duplicate_merge();
                self.duplicates.merge.preview = Some((mode, 2));
            }
        } else if phase == 2
            && self.duplicates.merge.pending.is_none()
            && self.duplicates.pending.is_none()
        {
            if mode == 4 && self.duplicates.merge.receipt.is_some() {
                self.undo_duplicate_merge(ctx);
            }
            self.duplicates.merge.preview = None;
        }
    }
    pub(in crate::app) fn undo_duplicate_merge(&mut self, ctx: &egui::Context) {
        if let Some(receipt) = &self.duplicates.merge.receipt {
            self.review_duplicate_undo(ctx, receipt.backup.clone(), receipt.backup_sha256.clone());
        }
    }
}
