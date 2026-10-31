use super::*;

impl YamantakaApp {
    pub fn preview_diagnostics(&mut self, ctx: &egui::Context, mode: u8) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.preview_privacy(ctx);
        self.diagnostics.open = true;
        self.diagnostics.preview = 1;
        self.diagnostics.preview_mode = mode;
        if mode == 4
            && let Some((source, _)) = &mut self.diagnostics.source
        {
            *source = source.with_file_name("unavailable-diagnostics.sqlite3");
        }
        self.start_diagnostics(ctx, None);
        if mode == 3
            && let Some(job) = &self.diagnostics.job
        {
            job.cancel();
        }
    }
    pub(super) fn advance_diagnostics_preview(&mut self, ctx: &egui::Context) {
        if !self.demo
            || self.diagnostics.job.is_some()
            || !matches!(self.diagnostics.preview, 1 | 2)
        {
            return;
        }
        let mode = self.diagnostics.preview_mode;
        let destination = self
            .capture
            .as_ref()
            .unwrap()
            .with_extension("diagnostics.json");
        if self.diagnostics.preview == 1 {
            if mode >= 3 {
                assert!(self.diagnostics.error.is_some());
                assert!(self.diagnostics.snapshot.is_none());
                if mode == 3 {
                    assert!(matches!(self.diagnostics.error, Some(Failure::Cancelled)));
                }
            } else {
                let snapshot = self
                    .diagnostics
                    .snapshot
                    .as_ref()
                    .expect("Diagnostic snapshot");
                assert_eq!(snapshot.report.database.counts.documents, 10);
                assert!(snapshot.report.database.quick_check_ok);
                assert!(snapshot.report.database.foreign_keys_ok);
                assert!(snapshot.json.len() < 16 * 1024);
                let value: serde_json::Value = serde_json::from_str(&snapshot.json).unwrap();
                assert_eq!(value["contains_private_content"], false);
                assert_eq!(value["format"], "yamantaka-rust-safe-diagnostics");
                if mode == 1 || mode == 2 {
                    if mode == 2 {
                        std::fs::write(&destination, b"previous complete report").unwrap();
                    }
                    self.start_diagnostics(ctx, Some(destination));
                    self.diagnostics.preview = 2;
                    return;
                }
            }
        } else if mode == 1 {
            assert!(self.diagnostics.saved);
            assert_eq!(
                std::fs::read_to_string(destination).unwrap(),
                self.diagnostics.snapshot.as_ref().unwrap().json.as_ref()
            );
        } else {
            assert!(matches!(
                self.diagnostics.error,
                Some(Failure::DestinationOccupied)
            ));
            assert_eq!(
                std::fs::read(destination).unwrap(),
                b"previous complete report"
            );
        }
        self.diagnostics.preview = 90;
        self.diagnostics.verified = true;
    }
}
