use super::*;
use crate::selection_repair_probe::{self, Fixture};
use std::{collections::BTreeMap, fs};
use yamantaka_storage::managed_imports;
#[derive(Clone)]
pub(super) struct Verification {
    pub source: Option<PathBuf>,
    pub books: usize,
    fixture: Fixture,
    before: BTreeMap<String, String>,
    install: bool,
}
impl Verification {
    pub(super) fn verify(
        &self,
        root: &Path,
        archive: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(root, self.fixture.root);
        if self.install {
            assert_eq!(fs::read(archive.unwrap())?, self.fixture.pointer);
            selection_repair_probe::verify_choice(
                &self.fixture,
                self.source.is_some(),
                &self.before,
            )?;
        } else {
            assert!(archive.is_none());
            assert_eq!(selection_repair_probe::hashes(root)?, self.before);
            assert_eq!(
                fs::read(root.join("selected-library.json"))?,
                self.fixture.pointer
            );
            assert!(managed_imports::recover_and_select(root).is_err());
        }
        Ok(())
    }
}
pub(super) fn run(args: &[OsString], capture: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mode = args
        .iter()
        .filter_map(|a| a.to_str())
        .find_map(|a| a.strip_prefix("--selection-recovery="))
        .unwrap();
    assert!(["original", "imported", "cancel"].contains(&mode));
    let parent = crate::qa_ui_root().join("../../build/demo-store/captures");
    let pointer = match mode {
        "original" => br#"{"version":99,"import":null}"#.to_vec(),
        "imported" => serde_json::to_vec(
            &serde_json::json!({"version":1,"import":uuid::Uuid::new_v4().to_string()}),
        )?,
        _ => b"damaged synthetic selection".to_vec(),
    };
    let fixture = selection_repair_probe::fixture(&parent, pointer)?;
    assert!(managed_imports::recover_and_select(&fixture.root).is_err());
    assert!(matches!(
        managed_imports::recovery_target(&fixture.root)?,
        Some(managed_imports::RecoveryTarget::LibrarySelection(_))
    ));
    let before = selection_repair_probe::hashes(&fixture.root)?;
    let verification = super::Verification {
        capture,
        backup: PathBuf::new(),
        original: Vec::new(),
        file: "",
        mode: if mode == "cancel" {
            "review".into()
        } else {
            "install".into()
        },
        requested: false,
        captured: false,
        scale: if args.iter().any(|a| a == "--text-scale=2") {
            2.0
        } else {
            1.0
        },
        pending: None,
        selection: Some(Verification {
            source: if mode == "imported" {
                Some(fixture.imported.clone())
            } else {
                None
            },
            books: if mode == "imported" { 3 } else { 2 },
            fixture: fixture.clone(),
            before,
            install: mode != "cancel",
        }),
    };
    super::super::run(
        fixture.root,
        FailedStage::LibrarySelection,
        !args.iter().any(|a| a == "--es"),
        args.iter().any(|a| a == "--software"),
        Some(verification),
    )
    .map(|_| ())
}
