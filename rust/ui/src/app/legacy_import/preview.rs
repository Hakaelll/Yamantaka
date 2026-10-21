use super::*;
impl YamantakaApp {
    pub fn preview_legacy_import(&mut self, ctx: &egui::Context, source: LegacySource, mode: &str) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.legacy_import.source = Some(source);
        self.legacy_import.verification = Some(Verification {
            closed: mode != "blocked",
            cancel_copy: mode == "cancel",
        });
        self.legacy_import.open = true;
        self.legacy_import.focus_cancel = true;
        self.legacy_import.preview = Some(match mode {
            "files" => 0,
            "copy" => 2,
            "activate" => 3,
            "stale" => 4,
            "cancel" => 5,
            "blocked" => 6,
            "original" => 7,
            _ => 1,
        });
        self.preview_storage(ctx);
        if mode != "files" {
            self.legacy_import.inspect(ctx);
        }
    }
    pub(super) fn drive_legacy_import_preview(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.legacy_import.preview else {
            return;
        };
        if self.legacy_import.busy() {
            return;
        }
        if mode == 0
            || mode == 1 && self.legacy_import.review.is_some()
            || mode == 6 && matches!(self.legacy_import.error, Some(Failure::LegacyRunning))
            || mode == 5 && matches!(self.legacy_import.error, Some(Failure::Cancelled))
            || mode == 8
                && matches!(
                    self.legacy_import.error,
                    Some(Failure::MigrationSourceChanged)
                )
        {
            if mode == 5 || mode == 8 {
                assert!(!self.legacy_import.target.as_ref().unwrap().path().exists());
            }
            self.legacy_import.preview = None;
            self.legacy_import.verified = true;
            return;
        }
        if mode == 4 && self.legacy_import.review.is_some() {
            let source = self.legacy_import.source.as_ref().unwrap();
            let db = yamantaka_storage::Database::open(&source.database).unwrap();
            db.execute_batch("UPDATE documents SET title='Changed after review'")
                .unwrap();
            self.legacy_import.preview = Some(8);
            self.legacy_import.copy(ctx);
            return;
        }
        if self.legacy_import.receipt.is_none() && self.legacy_import.review.is_some() {
            self.legacy_import.copy(ctx);
            return;
        }
        if self.legacy_import.receipt.is_some() && mode == 2 {
            let target = self.legacy_import.target.as_ref().unwrap();
            yamantaka_storage::migration::verify_capture(&target.path(), &Cancellation::default())
                .unwrap();
            assert_eq!(
                yamantaka_storage::managed_imports::active_store(
                    self.legacy_import.root.as_ref().unwrap()
                )
                .unwrap(),
                *self.legacy_import.root.as_ref().unwrap()
            );
            self.legacy_import.preview = None;
            self.legacy_import.verified = true;
            return;
        }
        if self.legacy_import.receipt.is_some()
            && !self.legacy_import.selected_next
            && !self.legacy_import.original_next
        {
            self.legacy_import.activate(ctx);
            return;
        }
        if self.legacy_import.selected_next && mode == 7 {
            let root = self.legacy_import.root.as_ref().unwrap().clone();
            self.legacy_import
                .begin(Command::Original(root), Pending::Original, ctx);
            return;
        }
        if self.legacy_import.selected_next || self.legacy_import.original_next {
            let root = self.legacy_import.root.as_ref().unwrap();
            let selected = yamantaka_storage::managed_imports::active_store(root).unwrap();
            if mode == 7 {
                assert_eq!(&selected, root);
            } else {
                assert_eq!(selected, self.legacy_import.target.as_ref().unwrap().path());
            }
            assert!(root.join("yamantaka.sqlite3").is_file());
            self.legacy_import.preview = None;
            self.legacy_import.verified = true;
        }
    }
}
