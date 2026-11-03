//! Isolated native acceptance scenarios; never uses the installed library.
use super::*;
use std::{ffi::OsString, time::Duration};
use yamantaka_domain::{LibraryQuery, LibraryRepository};
mod pending;
mod selection;

#[derive(Clone)]
pub struct Verification {
    capture: PathBuf,
    backup: PathBuf,
    original: Vec<u8>,
    file: &'static str,
    mode: String,
    requested: bool,
    captured: bool,
    scale: f32,
    pending: Option<pending::Verification>,
    selection: Option<selection::Verification>,
}
impl Verification {
    pub fn verify_installed(
        &self,
        store: &Path,
        safety: &Path,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Some(selection) = &self.selection {
            selection.verify(store, Some(safety))?;
            return self.result(true);
        }
        if let Some(pending) = &self.pending {
            pending.verify(store, Some(safety))?;
            return self.result(true);
        }
        assert_eq!(self.mode, "install");
        assert_eq!(std::fs::read(safety.join(self.file))?, self.original);
        let database = Database::open_read_only(&store.join("yamantaka.sqlite3"))?;
        assert_eq!(database.count(&LibraryQuery::default())?, 2);
        Preferences::load(&store.join("preferences.json"))?;
        self.result(true)
    }
    pub fn verify_unchanged(
        &self,
        store: &Path,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Some(selection) = &self.selection {
            selection.verify(store, None)?;
            return self.result(false);
        }
        if let Some(pending) = &self.pending {
            pending.verify(store, None)?;
            return self.result(false);
        }
        assert_ne!(
            self.mode, "install",
            "Installation did not reach the handover"
        );
        assert_eq!(std::fs::read(store.join(self.file))?, self.original);
        self.result(false)
    }
    fn result(&self, installed: bool) -> std::result::Result<(), Box<dyn std::error::Error>> {
        std::fs::write(
            self.capture.with_extension("result.json"),
            serde_json::to_vec(&serde_json::json!({
                "synthetic":true,"originalPreserved":true,"installed":installed
            }))?,
        )?;
        Ok(())
    }
}
pub fn run(
    args: &[OsString],
    capture: PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if args.iter().any(|arg| {
        arg.to_str()
            .is_some_and(|s| s.starts_with("--selection-recovery="))
    }) {
        return selection::run(args, capture);
    }
    if args.iter().any(|arg| {
        arg.to_str()
            .is_some_and(|s| s.starts_with("--interrupted-recovery="))
    }) {
        return pending::run(args, capture);
    }
    let root = crate::qa_ui_root()
        .join("../../build/demo-store/captures")
        .join(format!("startup-{}", uuid::Uuid::new_v4()));
    let store = root.join("active");
    std::fs::create_dir_all(&store)?;
    let file = if args.iter().any(|arg| arg == "--broken-library") {
        "yamantaka.sqlite3"
    } else {
        "preferences.json"
    };
    let original = if args.iter().any(|arg| arg == "--future-preferences") {
        br#"{"version":99,"legacy":{},"rust":{"synthetic":true}}"#.to_vec()
    } else {
        b"Synthetic unreadable state: retained byte for byte".to_vec()
    };
    std::fs::write(store.join(file), &original)?;
    let stage = match load(&store) {
        Ok(_) => panic!("Broken startup unexpectedly succeeded"),
        Err(stage) => stage,
    };
    assert_eq!(std::fs::read(store.join(file))?, original);
    if file == "preferences.json" {
        assert!(matches!(stage, FailedStage::Preferences));
        assert!(!store.join("yamantaka.sqlite3").exists());
    } else {
        assert!(matches!(stage, FailedStage::Library));
    }
    let backup = root.join("synthetic-backup.json");
    let mode = args
        .iter()
        .find_map(|arg| arg.to_str()?.strip_prefix("--startup-recovery="))
        .unwrap()
        .to_owned();
    std::fs::write(
        &backup,
        if mode == "invalid" {
            "not JSON"
        } else {
            include_str!("../../../storage/tests/fixtures/populated-backup-v9.json")
        },
    )?;
    let verification = Verification {
        selection: None,
        pending: None,
        capture,
        backup,
        original,
        file,
        mode,
        requested: false,
        captured: false,
        scale: if args.iter().any(|arg| arg == "--text-scale=2") {
            2.0
        } else {
            1.0
        },
    };
    super::run(
        store,
        stage,
        !args.iter().any(|arg| arg == "--es"),
        args.iter().any(|arg| arg == "--software"),
        Some(verification),
    )
    .map(|_| ())
}
impl Recovery {
    pub(super) fn start_preview(&mut self, ctx: &egui::Context) {
        if let Some(preview) = &self.verification {
            ctx.set_zoom_factor(preview.scale);
            let command = if let Some(selection) = &preview.selection {
                Command::ReviewSelection {
                    root: self.store.clone(),
                    source: selection.source.clone(),
                }
            } else if let Some(pending) = &preview.pending {
                Command::ReviewPending {
                    active: self.store.clone(),
                    previous: pending.previous.clone(),
                }
            } else {
                Command::Review(preview.backup.clone())
            };
            self.start(command, ctx);
        }
    }
    pub(super) fn advance_preview(&mut self, ctx: &egui::Context) {
        if self.job.is_some() {
            return;
        }
        if let Some(preview) = &self.verification
            && preview.mode == "install"
            && preview.captured
            && let Some(review) = self.review.clone()
        {
            self.start(Command::Prepare(review), ctx);
        }
    }
    pub(super) fn capture_preview(&mut self, ctx: &egui::Context) {
        let Some(preview) = &mut self.verification else {
            return;
        };
        if preview.captured || self.job.is_some() {
            return;
        }
        assert_eq!(self.failed, preview.mode == "invalid");
        if !self.failed {
            assert_eq!(
                self.review.as_ref().unwrap().counts()["documents"],
                preview.selection.as_ref().map_or_else(
                    || preview.pending.as_ref().map_or(2, |p| p.books),
                    |s| s.books
                )
            );
        }
        if !preview.requested {
            preview.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(screenshot) = screenshot {
            let bytes: Vec<u8> = screenshot
                .pixels
                .iter()
                .flat_map(egui::Color32::to_array)
                .collect();
            image::save_buffer(
                &preview.capture,
                &bytes,
                screenshot.width() as u32,
                screenshot.height() as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let rect = |r: egui::Rect| [r.left(), r.top(), r.right(), r.bottom()];
            let controls = ctx.data(|data| {
                data.get_temp::<Vec<theme::ControlBounds>>(egui::Id::new("capture-controls"))
                    .unwrap_or_default()
            });
            std::fs::write(preview.capture.with_extension("controls.json"), serde_json::to_vec(&serde_json::json!({
                "synthetic":true,"captureProcessId":std::process::id(),"viewport":rect(ctx.content_rect()),"pixelsPerPoint":ctx.pixels_per_point(),
                "controls":controls.into_iter().map(|c|serde_json::json!({"label":c.label,"rect":rect(c.rect),"clip":rect(c.clip),"enabled":c.enabled})).collect::<Vec<_>>()
            })).unwrap()).unwrap();
            preview.captured = true;
            if preview.mode != "install" {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
