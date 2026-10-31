//! Native commands target only the isolated acceptance window.
use super::*;
use std::time::Instant;

pub struct Cycle {
    phase: u8,
    baseline: Option<[i32; 4]>,
    since: Instant,
}
impl State {
    pub fn start_verification(&mut self) {
        self.verification = Some(Cycle {
            phase: 0,
            baseline: None,
            since: Instant::now(),
        });
    }
    pub fn advance_verification(&mut self, ctx: &egui::Context) {
        let Some(cycle) = &mut self.verification else {
            return;
        };
        let settled = self.pending.is_none() && self.last.is_some() && self.last == self.saved;
        let (full, max) = ctx.input(|i| {
            (
                i.viewport().fullscreen.unwrap_or(false),
                i.viewport().maximized.unwrap_or(false),
            )
        });
        match cycle.phase {
            0 if settled => {
                let placement = self.last.unwrap();
                assert!(
                    placement.normal()[0].unsigned_abs() < 100_000,
                    "Off-screen placement was not adjusted"
                );
                cycle.baseline = Some(placement.normal());
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                cycle.phase = 1;
            }
            1 if settled && max && self.last.unwrap().maximized() => {
                assert_eq!(self.last.unwrap().normal(), cycle.baseline.unwrap());
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                cycle.phase = 2;
                cycle.since = Instant::now();
            }
            2 if full && cycle.since.elapsed() > Duration::from_millis(250) => {
                assert_eq!(self.last, self.saved);
                assert!(self.last.unwrap().maximized());
                assert_eq!(self.last.unwrap().normal(), cycle.baseline.unwrap());
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                cycle.phase = 3;
            }
            3 if !full => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                cycle.phase = 4;
            }
            4 if !full && !max && settled && !self.last.unwrap().maximized() => {
                assert_eq!(self.last.unwrap().normal(), cycle.baseline.unwrap());
                cycle.phase = 5;
            }
            _ => (),
        }
        if cycle.phase != 5 {
            ctx.request_repaint_after(Duration::from_millis(80));
        }
    }
    pub fn verification_ready(&self) -> bool {
        self.verification
            .as_ref()
            .is_none_or(|cycle| cycle.phase == 5)
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"phase":self.verification.as_ref().map(|c|c.phase),"last":self.last.map(Placement::encode),"saved":self.saved.map(Placement::encode)})
    }
}
pub fn seed(store: &Path, unknown: bool, from_backup: bool) -> Result<Option<serde_json::Value>> {
    std::fs::create_dir_all(store)
        .map_err(|_| Failure::Storage("Could not create synthetic window store".into()))?;
    let mut preferences = Preferences::default();
    if unknown {
        preferences.version = 99;
    }
    preferences.rust.insert("nativePlacement".into(), serde_json::json!({"version":1,"normal":[-500000,-500000,-499100,-499300],"maximized":false}));
    if from_backup {
        let placement = preferences.rust.remove("nativePlacement").unwrap();
        preferences
            .rust
            .insert("nativeWindowPlacement".into(), placement.clone());
        let database = yamantaka_storage::Database::in_memory_at(11)?;
        let path = store.join("synthetic-window-backup.json");
        database.export_backup(&path, &preferences, &Default::default())?;
        let restored = yamantaka_storage::backup::preview(&path, &Default::default())?;
        assert_eq!(
            restored.preferences.get("nativeWindowPlacement"),
            Some(&placement)
        );
        assert!(!store.join("window-state-v1.json").exists());
        return Ok(Some(placement));
    }
    preferences.save(&store.join("window-state-v1.json"))?;
    std::fs::copy(
        store.join("window-state-v1.json"),
        store.join("window-state-source.json"),
    )
    .map_err(|_| Failure::Storage("Could not preserve synthetic window state".into()))?;
    Ok(None)
}
pub fn verify_saved(
    store: &Path,
    capture: &Path,
    unknown: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let controls: serde_json::Value =
        serde_json::from_slice(&std::fs::read(capture.with_extension("controls.json"))?)?;
    if unknown {
        assert_eq!(
            std::fs::read(store.join("window-state-v1.json"))?,
            std::fs::read(store.join("window-state-source.json"))?
        );
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(store.join("window-state-v1.json"))?)?;
        assert_eq!(value["version"], 99);
        assert_eq!(value["rust"]["nativePlacement"]["normal"][0], -500000);
        assert!(controls["windowPlacement"].is_null());
    } else {
        let preferences = Preferences::load(&store.join("window-state-v1.json"))?;
        assert_eq!(
            preferences.rust["nativePlacement"],
            controls["windowPlacement"]["saved"]
        );
        assert_eq!(controls["windowPlacement"]["phase"], 5);
        assert_eq!(
            controls["windowPlacementBackupSetting"],
            controls["windowPlacement"]["saved"]
        );
    }
    Ok(())
}
