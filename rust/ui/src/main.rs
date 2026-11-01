#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod backup_worker;
mod book;
mod diagnostics_worker;
mod graphics_recovery;
mod legacy_import_worker;
mod managed_pages;
mod page_presenter;
mod restore_worker;
mod rule_simulation_fixture;
mod startup_recovery;
mod theme;
mod window_state;
use eframe::egui;
use std::{path::PathBuf, sync::mpsc};
use yamantaka_application::runtime::Runtime;
use yamantaka_domain::memory::{GlobalMemoryLimits, MemoryGovernor};
use yamantaka_domain::reader_preferences::{
    PageRotation, ReaderMode, ReaderPreferences, ReaderTheme, ZoomMode,
};
use yamantaka_domain::{LibraryRepository, PageLayout, PdfEngine, pdf_ticket::AsyncPdfEngine};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::{Database, preferences::Preferences};
use yamantaka_windows::{Activation, AppPaths, Instance};

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let args: Vec<_> = std::env::args_os().skip(1).collect();
            if graphics_recovery::can_retry(error.as_ref(), &args) {
                eprintln!("Graphics failed; trying the alternate compositor once.");
                match graphics_recovery::restart(
                    &args,
                    error.downcast_ref::<graphics_recovery::RuntimeFault>(),
                ) {
                    Ok(status) => {
                        return if status.success() {
                            std::process::ExitCode::SUCCESS
                        } else {
                            std::process::ExitCode::FAILURE
                        };
                    }
                    Err(restart_error) => eprintln!(
                        "Could not start the alternate graphics compositor: {restart_error}"
                    ),
                }
            }
            eprintln!("{error}");
            // Release GUI binaries have no console: make a failed startup
            // visible. Synthetic capture failures instead return a nonzero code.
            if !std::env::args_os().any(|arg| arg == "--demo") {
                yamantaka_windows::show_startup_error(&error.to_string());
            }
            std::process::ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let demo = args.iter().any(|arg| arg == "--demo");
    let settings = args.iter().any(|arg| arg == "--settings");
    let protected = args.iter().any(|arg| arg == "--protected");
    let cover_rebuild = args.iter().any(|arg| arg == "--cover-rebuild");
    let reader = protected
        || args.iter().any(|arg| {
            arg == "--reader"
                || arg == "--bookmarks"
                || arg == "--outline"
                || arg == "--thumbnails"
                || arg == "--find"
                || arg == "--links"
        });
    let light = args.iter().any(|arg| arg == "--light");
    let tilt = args
        .iter()
        .any(|arg| arg == "--tilt" || arg == "--tilt-negative");
    let capture = args.iter().find_map(|arg| {
        arg.to_str()
            .and_then(|s| s.strip_prefix("--capture="))
            .map(PathBuf::from)
    });
    let executable = std::env::current_exe()?;
    if demo
        && args.iter().any(|arg| {
            arg.to_str()
                .is_some_and(|s| s.starts_with("--startup-recovery="))
        })
    {
        return startup_recovery::preview::run(
            &args,
            capture.ok_or("Recovery verification requires a capture path")?,
        );
    }
    let paths = AppPaths::discover(&executable)?;
    let initial: Vec<PathBuf> = args
        .iter()
        .filter(|arg| !arg.to_string_lossy().starts_with("--"))
        .map(PathBuf::from)
        .collect();
    let primary_instance = if !demo {
        match Instance::acquire(&paths.activations, &initial)? {
            Activation::Primary(instance) => Some(instance),
            Activation::Forwarded => return Ok(()),
        }
    } else {
        None
    };
    let requested_safe = args.iter().any(|arg| arg == "--safe-mode");
    let mut startup = None;
    let demo_resume = if demo && capture.is_some() {
        graphics_recovery::demo_store(&args)?
    } else {
        None
    };
    let (database, mut prefs, store) = if let Some(store) = demo_resume {
        (
            Database::open(&store.join("export-preview.sqlite3"))?,
            Preferences::load(&store.join("preferences.json"))?,
            store,
        )
    } else if demo {
        let mut store = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/demo-store");
        // Captures have their own instance identity so QA does not forward to
        // or interfere with a demonstration window the user is reviewing.
        if capture.is_some() {
            store = store.join("captures").join(format!(
                "{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
        } else if args.iter().any(|arg| arg == "--demo-window") {
            store = store.join("previews").join(format!(
                "{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
        }
        let mut db = if args.iter().any(|arg| arg == "--demo-window")
            || capture.is_some()
                && args.iter().any(|arg| {
                    arg.to_str().is_some_and(|value| {
                        value.starts_with("--bulk-remove=")
                            || value.starts_with("--backup-export=")
                            || value == "--restore-preview"
                            || value == "--restore-apply"
                            || value == "--workspaces"
                    }) || arg == "--rule-simulation"
                        || arg == "--export-json"
                        || arg == "--reference-export-json"
                        || arg == "--reference-export-md"
                        || arg == "--automatic-cover"
                        || arg == "--cover-picker-save"
                        || arg == "--cover-picker-cancel"
                        || arg == "--cover-picker-failed-save"
                        || arg == "--sessions-preview"
                        || arg == "--export-markdown"
                        || arg == "--global-notes-export-json"
                        || arg == "--global-notes-export-markdown"
                        || arg == "--metadata-review"
                        || arg == "--file-check"
                        || arg == "--relocate"
                        || arg == "--library-check"
                        || arg == "--import-recovery"
                        || arg == "--cover-capacity-retry"
                        || arg == "--reference-bookmarks"
                        || arg == "--diagnostics"
                        || arg == "--graphics-fail-runtime"
                }) {
            std::fs::create_dir_all(&store).map_err(|_| {
                yamantaka_domain::Failure::Storage("Synthetic export store could not open".into())
            })?;
            Database::open(&store.join("export-preview.sqlite3"))?
        } else {
            Database::in_memory_at(11)?
        };
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(if args.iter().any(|arg| arg == "--metadata-reanalyze") {
                if args.iter().any(|arg| arg == "--metadata-evidence-later") {
                    "../../build/rust-fixtures/front-matter-evidence-later.pdf"
                } else if args.iter().any(|arg| arg == "--metadata-reanalyze-limit") {
                    "../../build/rust-fixtures/front-matter-work-limit.pdf"
                } else {
                    "../../build/rust-fixtures/front-matter-missing.pdf"
                }
            } else if protected || cover_rebuild {
                "../../build/rust-fixtures/protected.pdf"
            } else if args.iter().any(|arg| arg == "--links") {
                if args.iter().any(|arg| arg == "--rejected-link") {
                    "../../build/rust-fixtures/rejected-links.pdf"
                } else if args.iter().any(|arg| arg == "--external-links") {
                    "../../build/rust-fixtures/external-links.pdf"
                } else {
                    "../../build/rust-fixtures/links.pdf"
                }
            } else if args.iter().any(|arg| arg == "--find-scanned") {
                "../../build/rust-fixtures/scanned.pdf"
            } else if args.iter().any(|arg| arg == "--thumbnails-long") {
                "../../build/rust-fixtures/5000-pages.pdf"
            } else if args.iter().any(|arg| {
                arg == "--outline" || arg == "--thumbnails" || arg == "--find" || arg == "--links"
            }) {
                "../../build/rust-fixtures/outline.pdf"
            } else {
                "../../build/rust-fixtures/textual.pdf"
            })
            .canonicalize()?;
        let names = [
            ("On the Nature of Things", "Lucretius"),
            ("The Architecture of Thought", "Study collection"),
            ("Ways of Seeing", "John Berger"),
            ("A Field Guide to the Universe", "Study collection"),
            ("The Art of Attention", "Study collection"),
            ("Essays on Light", "Study collection"),
            ("The Shape of Ideas", "Study collection"),
            ("A Reader's Notebook", "Study collection"),
            ("Collected Fragments", "Study collection"),
            ("The Geometry of Silence", "Study collection"),
        ];
        for (index, (title, author)) in names.iter().enumerate() {
            let automatic_first = index == 0 && args.iter().any(|arg| arg == "--automatic-cover");
            let cover_fixture = if automatic_first {
                Some(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../build/rust-fixtures/automatic-cover.pdf"),
                )
            } else {
                args.iter().any(|arg| arg == "--pdf-covers").then(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                        "../../build/rust-fixtures/covers/cover-{index}.pdf"
                    ))
                })
            };
            let reference_protected =
                (index == 1 && args.iter().any(|arg| arg == "--reference-protected")).then(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../build/rust-fixtures/protected.pdf")
                });
            let reference_links = (index == 1 && args.iter().any(|arg| arg == "--reference-links"))
                .then(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                        if args.iter().any(|arg| arg == "--reference-link-external") {
                            "../../build/rust-fixtures/external-links.pdf"
                        } else {
                            "../../build/rust-fixtures/links.pdf"
                        },
                    )
                });
            let reference_outline =
                (index == 1 && args.iter().any(|arg| arg == "--reference-outline")).then(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../build/rust-fixtures/outline.pdf")
                });
            let reference_text =
                (index == 1 && args.iter().any(|arg| arg == "--reference-text-empty")).then(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../build/rust-fixtures/scanned.pdf")
                });
            let document_path = reference_protected
                .as_ref()
                .or(reference_text.as_ref())
                .or(reference_outline.as_ref())
                .or(reference_links.as_ref())
                .or(cover_fixture.as_ref())
                .unwrap_or(&fixture)
                .canonicalize()?;
            let path = document_path.to_string_lossy().replace('\'', "''");
            let title = if index == 0
                && args.iter().any(|arg| {
                    arg == "--outline"
                        || arg == "--thumbnails"
                        || arg == "--find"
                        || arg == "--links"
                }) {
                "The art of attention"
            } else {
                title
            };
            let title = title.replace('\'', "''");
            let pages = if automatic_first { 7 } else { 2 };
            let progress = if automatic_first {
                4.0 / 7.0
            } else if index == 0 {
                0.5
            } else {
                0.0
            };
            let last = if automatic_first { 3 } else { 0 };
            db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,last_page_index) VALUES('demo-{index}','{path}','demo-{index}','fixture.pdf','{title}','{author}',{pages},900,'2026-10-08','2026-10-08','2026-10-08','synthetic',{progress},{last})"))?;
            if index == 0
                && args
                    .iter()
                    .any(|arg| arg == "--compare-pdf" || arg == "--compare-different")
            {
                let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&document_path)?;
                db.execute_batch(&format!(
                    "UPDATE documents SET fingerprint='{fingerprint}' WHERE id='demo-0'"
                ))?;
            }
        }
        if args.iter().any(|arg| arg == "--reference-chooser") {
            let path = fixture.to_string_lossy().replace('\'', "''");
            for index in 0..100 {
                db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('chooser-{index:03}','{path}','chooser-{index:03}','reference.pdf','Reference catalogue entry {index:03}','Study collection',2,900,'2000-01-01','2000-01-01','2000-01-01','synthetic');"))?;
            }
        }
        if capture.is_some()
            && args
                .iter()
                .any(|arg| arg == "--file-check" || arg == "--relocate")
        {
            let original = db.get("demo-0")?.unwrap();
            let copy = store.join("file-check.pdf");
            std::fs::copy(&original.path, &copy)?;
            let path = copy.canonicalize()?;
            let metadata = path.metadata()?;
            let modified = chrono::DateTime::<chrono::Utc>::from(metadata.modified()?).to_rfc3339();
            let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&path)?;
            let encoded = path.to_string_lossy().replace('\'', "''");
            let normalized = path.to_string_lossy().to_lowercase().replace('\'', "''");
            db.execute_batch(&format!("UPDATE documents SET path='{encoded}',normalized_path='{normalized}',file_size={},file_modified_at='{modified}',fingerprint='{fingerprint}' WHERE id='demo-0'",metadata.len()))?;
            if args.iter().any(|arg| {
                arg == "--file-changed"
                    || arg == "--file-restored"
                    || (arg == "--relocate" && !args.iter().any(|arg| arg == "--relocation-legacy"))
            }) {
                let hex = yamantaka_storage::identity::full_sha256(
                    &path,
                    &yamantaka_domain::Cancellation::default(),
                )?;
                let mut digest = [0u8; 32];
                for (index, byte) in digest.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)?;
                }
                let snapshot = db.file_snapshot("demo-0")?;
                db.apply_file_check(
                    &snapshot,
                    yamantaka_domain::file_check::FileCheckResult::Available {
                        digest,
                        modified,
                        baseline_created: true,
                    },
                    &yamantaka_domain::Cancellation::default(),
                )?;
            }
            if args.iter().any(|arg| arg == "--file-missing") {
                std::fs::remove_file(&path)?;
            }
            if args.iter().any(|arg| arg == "--file-changed") {
                use std::io::Write;
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&path)?
                    .write_all(b"\n% synthetic changed-file fixture\n")?;
            }
            if args.iter().any(|arg| arg == "--file-restored") {
                db.execute_batch("UPDATE documents SET is_missing=1 WHERE id='demo-0'")?;
            }
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--library-check") {
            for (index, document) in db.query(&Default::default())?.into_iter().enumerate() {
                let path = store.join(format!("library-check-{index}.pdf"));
                std::fs::copy(&document.path, &path)?;
                let path = path.canonicalize()?;
                let metadata = path.metadata()?;
                let modified =
                    chrono::DateTime::<chrono::Utc>::from(metadata.modified()?).to_rfc3339();
                let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&path)?;
                let encoded = path.to_string_lossy().replace('\'', "''");
                let normalized = path.to_string_lossy().to_lowercase().replace('\'', "''");
                db.execute_batch(&format!("UPDATE documents SET path='{encoded}',normalized_path='{normalized}',file_size={},file_modified_at='{modified}',fingerprint='{fingerprint}' WHERE id='{}'",metadata.len(),document.id))?;
                match index {
                    0 => std::fs::remove_file(&path)?,
                    1 => {
                        use std::io::{Seek, SeekFrom, Write};
                        let mut file = std::fs::OpenOptions::new().write(true).open(&path)?;
                        file.seek(SeekFrom::End(-1))?;
                        file.write_all(b"!")?;
                    }
                    2 => db.execute_batch(&format!(
                        "UPDATE documents SET file_modified_at='unknown' WHERE id='{}'",
                        document.id
                    ))?,
                    3 => {
                        std::fs::remove_file(&path)?;
                        std::fs::create_dir(&path)?;
                    }
                    _ => {}
                }
            }
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--relocate") {
            let original = db.get("demo-0")?.unwrap();
            let candidate = store.join("relocated.pdf");
            std::fs::rename(&original.path, &candidate)?;
            if args.iter().any(|arg| arg == "--relocation-mismatch") {
                use std::io::{Seek, SeekFrom, Write};
                let mut file = std::fs::OpenOptions::new().write(true).open(&candidate)?;
                file.seek(SeekFrom::End(-1))?;
                file.write_all(b"!")?;
            }
            db.execute_batch("UPDATE documents SET is_missing=1 WHERE id='demo-0'")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--import-recovery") {
            let first = db.get("demo-0")?.unwrap();
            let second = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../build/rust-fixtures/front-matter-wrapped-title.pdf")
                .canonicalize()?;
            let source = yamantaka_domain::import_recovery::ImportSource::Files(vec![
                first.path.clone(),
                second,
            ])
            .absolute()?;
            let job = db.queue_import(&source)?.unwrap();
            db.checkpoint_import(&job, &first.path, &first.id)?;
            drop(db);
            db = Database::open(&store.join("export-preview.sqlite3"))?;
        }
        if args.iter().any(|arg| {
            arg.to_str()
                .is_some_and(|value| value.starts_with("--bulk-covers="))
        }) {
            db.execute_batch("INSERT INTO document_covers SELECT id,1,'','','2026-10-20','2026-10-08',1,'manual',1,'contain',0.75 FROM documents WHERE id IN ('demo-0','demo-1','demo-2');")?;
        }
        if args.iter().any(|arg| arg == "--bulk-remove=last-page") {
            db.execute_batch("WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<70) INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('extra-%03d',x),path,printf('extra-%03d',x),'fixture.pdf','Synthetic last-page fixture',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM n JOIN documents ON documents.id='demo-0';")?;
        }
        if args.iter().any(|arg| arg == "--bulk-remove=failure") {
            db.execute_batch("CREATE TRIGGER synthetic_remove_failure BEFORE DELETE ON documents WHEN OLD.id='demo-0' BEGIN SELECT RAISE(ABORT,'synthetic removal failure'); END;")?;
        }
        if cover_rebuild {
            let metadata = fixture.metadata()?;
            let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&fixture)?;
            let modified = chrono::DateTime::<chrono::Utc>::from(metadata.modified()?).to_rfc3339();
            let source = fixture
                .canonicalize()?
                .to_string_lossy()
                .replace('\'', "''");
            let normalized = source.to_lowercase();
            db.execute_batch(&format!("UPDATE documents SET path='{source}',normalized_path='{normalized}',file_name='protected.pdf',fingerprint='{fingerprint}',file_size={},file_modified_at='{modified}' WHERE id='demo-0';",metadata.len()))?;
        }
        if args.iter().any(|arg| arg == "--automatic-cover-manual") {
            db.execute_batch("INSERT INTO document_covers VALUES('demo-0',0,'','','2026-10-13','2026-10-08',1,'manual',4,'contain',0.7142857142857143);")?;
        }
        if args.iter().any(|arg| arg == "--metadata-review") {
            db.execute_batch("UPDATE documents SET source_title='Untitled',source_author='Unknown' WHERE id='demo-0'; INSERT INTO document_enrichment(document_id,fingerprint,analyzer_version,status,suggested_title,suggested_author) VALUES('demo-0','synthetic',2,'review','Geometry of Light','Ada Lovelace');")?;
            if args.iter().any(|arg| arg == "--metadata-reset") {
                db.execute_batch("UPDATE documents SET title_override='Manual geometry',author_override='Manual author' WHERE id='demo-0'")?;
            }
            if args.iter().any(|arg| arg == "--metadata-failed") {
                db.execute_batch("CREATE TRIGGER synthetic_metadata_failure BEFORE UPDATE OF title_override ON documents BEGIN SELECT RAISE(ABORT,'Synthetic metadata save failure'); END;")?;
            }
            if args.iter().any(|arg| arg == "--metadata-reanalyze") {
                let native_source = fixture.metadata()?;
                let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&fixture)?;
                let modified =
                    chrono::DateTime::<chrono::Utc>::from(native_source.modified()?).to_rfc3339();
                let pages = if args.iter().any(|arg| arg == "--metadata-reanalyze-limit") {
                    1
                } else {
                    2
                };
                db.execute_batch(&format!("UPDATE documents SET fingerprint='{fingerprint}',file_size={},file_modified_at='{modified}',page_count={pages} WHERE id='demo-0'; UPDATE document_enrichment SET fingerprint='{fingerprint}' WHERE document_id='demo-0';",native_source.len()))?;
                if args.iter().any(|arg| arg == "--metadata-reanalyze-manual") {
                    db.execute_batch("UPDATE documents SET title_override='Manual geometry',author_override='Manual author' WHERE id='demo-0'")?;
                }
            }
        }
        if args
            .iter()
            .any(|arg| arg == "--automatic-cover-stale-version")
        {
            db.execute_batch("INSERT INTO document_covers VALUES('demo-0',0,'','','2026-10-13','2026-10-08',0.79,'automatic',2,'contain',0.7142857142857143);")?;
        }
        if args
            .iter()
            .any(|arg| arg == "--automatic-cover-stale-source")
        {
            db.execute_batch("INSERT INTO document_covers VALUES('demo-0',0,'','','2026-10-13','2026-10-07',0.79,'automatic',3,'contain',0.7142857142857143);")?;
        }
        if args
            .iter()
            .any(|arg| arg == "--cover-picker-restore-failed")
        {
            db.execute_batch("CREATE TRIGGER synthetic_restore_rejection BEFORE UPDATE OF thumbnail_path ON documents WHEN OLD.id='demo-0' BEGIN SELECT RAISE(ABORT,'synthetic restore rejection'); END;")?;
        }
        if args.iter().any(|arg| arg == "--cover-picker-failed-save") {
            db.execute_batch("CREATE TRIGGER synthetic_cover_rejection BEFORE INSERT ON document_covers BEGIN SELECT RAISE(ABORT,'synthetic cover save rejection'); END;")?;
        }
        if args.iter().any(|arg| arg == "--saved-cover-page") {
            db.execute_batch("INSERT INTO document_covers VALUES('demo-0',1,'','','2026-10-13T00:00:00Z','2026-10-08',1,'manual',1,'contain',0.75);")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--saved-rules") {
            let actions = (0..8)
                .map(|index| serde_json::json!({"kind":"addTag","value":format!("Study {index}")}))
                .collect::<Vec<_>>();
            db.execute_batch(&format!("INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) VALUES('legacy-rule','Inherited study rule',1,'{{\"kind\":\"extension\",\"value\":\".pdf\"}}','{}','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z'),('future-rule','Future saved rule',1,' {{\"kind\":\"futureTrigger\",\"value\":null,\"vendor\":7}} ','[{{\"kind\":\"futureAction\",\"value\":\"Keep me\"}}]','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z');",serde_json::to_string(&actions)?))?;
            if args.iter().any(|arg| arg == "--saved-rules-many") {
                db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<79) INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) SELECT printf('paged-rule-%03d',x),printf('Study rule %03d',x),1,'{\"kind\":\"extension\",\"value\":\".pdf\"}','[{\"kind\":\"addTag\",\"value\":\"Study\"}]','2026-10-31T00:00:00Z','2026-10-31T00:00:00Z' FROM n;")?;
            }
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--rule-simulation") {
            rule_simulation_fixture::seed(&db, &store, &fixture, &args)?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--revision-groups") {
            db.execute_batch("INSERT INTO document_revision_groups(id,name,current_document_id,created_at,updated_at) VALUES('legacy-revisions','Inherited editions','demo-1','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z'); INSERT INTO document_revision_members(group_id,document_id,added_at) VALUES('legacy-revisions','demo-0','2026-10-30T00:00:00Z'),('legacy-revisions','demo-1','2026-10-30T00:00:00Z');")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--relations") {
            db.execute_batch("INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,source_anchor_kind,source_anchor_id,target_anchor_kind,target_anchor_id,created_at) VALUES('legacy-relation','demo-2','demo-0','sourceOf','annotation','legacy-annotation','page','1','2026-10-30T00:00:00Z');")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--reference-notes") {
            db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) VALUES('source-note-a','demo-0',0,'pageComment','violet','Source note A','[]',0,'synthetic','attached','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z','pending'),('source-note-b','demo-0',1,'pageComment','violet','Source note B','[]',0,'synthetic','attached','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z','pending');")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--quarantine") {
            db.execute_batch("UPDATE documents SET restore_blocked=1,last_failure_code='inheritedUnknownFailure' WHERE id IN ('demo-0','demo-1')")?;
            std::fs::create_dir_all(&store)?;
            let path = store.join("quarantine-broken.pdf");
            std::fs::write(&path, b"This is not a valid PDF.")?;
            db.execute_batch(&format!(
                "UPDATE documents SET path='{}' WHERE id='demo-2'",
                path.to_string_lossy().replace('\'', "''")
            ))?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--reference-note-bulk") {
            db.execute_batch("WITH RECURSIVE notes(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM notes WHERE n<81) INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) SELECT printf('reference-bulk-%03d',n),'demo-1',n%2,'pageComment','violet',printf('Reference bulk note %03d',n),'[]',0,'synthetic','attached','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z','pending' FROM notes; INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('reference-existing-tag','Existing study','existing study','blue','2026-10-30T00:00:00Z','2026-10-30T00:00:00Z'); INSERT INTO annotation_tags(annotation_id,tag_id) SELECT id,'reference-existing-tag' FROM annotations WHERE document_id='demo-1';")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--reference-marks") {
            db.execute_batch(r#"
                INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state)
                VALUES('synthetic-primary-mark','demo-0',0,'highlight','yellow','Main passage','[{"left":0.08,"top":0.112,"right":0.55,"bottom":0.135}]',0,'synthetic','attached','2026-10-30','2026-10-30','pending'),
                ('synthetic-reference-highlight','demo-1',0,'highlight','green','Reference highlight','[{"left":0.08,"top":0.112,"right":0.55,"bottom":0.135}]',0,'synthetic','attached','2026-10-30','2026-10-30','pending'),
                ('synthetic-reference-underline','demo-1',0,'underline','green','Reference underline','[{"left":0.08,"top":0.16,"right":0.55,"bottom":0.18}]',0,'synthetic','attached','2026-10-30','2026-10-30','pending'),
                ('synthetic-reference-strike','demo-1',0,'strikeout','green','Reference strikeout','[{"left":0.08,"top":0.21,"right":0.55,"bottom":0.23}]',0,'synthetic','attached','2026-10-30','2026-10-30','pending');
            "#)?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--annotation-many") {
            db.execute_batch(r#"WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180)
                INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state)
                SELECT printf('synthetic-mark-%03d',x),'demo-0',0,'highlight','violet','Synthetic mark',
                    printf('[{"left":%.4f,"top":%.4f,"right":%.4f,"bottom":%.4f}]',0.08+(x%6)*0.14,0.12+(x/6)*0.023,0.18+(x%6)*0.14,0.13+(x/6)*0.023),
                    0,'synthetic','attached','2026-10-09T12:00:00Z','2026-10-09T12:00:00Z','pending' FROM n;"#)?;
        }
        if capture.is_some()
            && args
                .iter()
                .any(|arg| arg == "--annotation-filter" || arg == "--annotation-filter-empty")
        {
            db.execute_batch("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('synthetic-study','Study','study','violet','2026-10-10T12:00:00Z','2026-10-10T12:00:00Z'); INSERT INTO annotation_tags(annotation_id,tag_id) SELECT id,'synthetic-study' FROM annotations WHERE id<'synthetic-mark-020';")?;
        }
        if capture.is_some()
            && args.iter().any(|arg| {
                arg.to_str()
                    .is_some_and(|value| value.starts_with("--global-notes"))
            })
        {
            db.execute_batch("UPDATE annotations SET document_id='demo-1',document_fingerprint=(SELECT fingerprint FROM documents WHERE id='demo-1') WHERE id>='synthetic-mark-002';")?;
            if args.iter().any(|arg| arg == "--global-notes-passage") {
                db.execute_batch("UPDATE annotations SET selected_text='A personal library grows through the connections we make between its pages. Returning to a passage can reveal a question we did not notice on the first reading.',note='Synthetic study note: compare this idea with the next chapter, then write a short explanation in your own words.' WHERE id='synthetic-mark-000';")?;
            }
        }
        if capture.is_some()
            && args.iter().any(|arg| {
                arg == "--history"
                    || arg == "--history-next"
                    || arg == "--history-restore"
                    || arg == "--history-invalid"
                    || arg == "--history-delete"
            })
        {
            db.execute_batch("WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) SELECT printf('synthetic-session-%03d',x),CASE WHEN x%2=0 THEN 'demo-0' ELSE 'demo-1' END,'2026-10-09T09:00:00Z','2026-10-09T10:00:00Z',0,1,120+x*11 FROM n; INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES('synthetic-live','demo-0','2026-10-10T09:00:00Z',0,0,0);")?;
        }
        if capture.is_some()
            && args.iter().any(|arg| {
                arg == "--history"
                    || arg == "--history-restore"
                    || arg == "--history-invalid"
                    || arg == "--history-delete"
            })
        {
            db.execute_batch(r#"INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','{"synthetic-session-180":{"zoomLevel":1.75,"pageLayout":"continuous","readingMode":"document","readingTheme":"sepia","scrollOffset":1500,"panelSection":null,"future":{"retained":true}}}','now');"#)?;
            if args.iter().any(|arg| arg == "--history-invalid") {
                db.execute_batch("UPDATE app_metadata SET value=json_set(value,'$.\"synthetic-session-180\".readingTheme','future') WHERE key='reading_session_contexts_v1';")?;
            }
        }
        if args
            .iter()
            .any(|arg| arg == "--goal-editor" || arg == "--goal-save" || arg == "--goal-clear")
        {
            db.execute_batch("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_goals_v1','{\"demo-0\":{\"pages\":12,\"minutes\":30,\"future\":{\"kept\":true}},\"demo-1\":{\"pages\":8},\"future\":false}','synthetic');")?;
        }
        if capture.is_some()
            && args.iter().any(|arg| {
                arg.to_str()
                    .is_some_and(|value| value.starts_with("--smart-view="))
            })
        {
            db.execute_batch("UPDATE documents SET added_at='2000-01-01T00:00:00Z',last_opened_at=''; UPDATE documents SET added_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),last_opened_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='demo-0'; INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('smart-a','demo-0',0,'note','violet','synthetic','2026-10-19','2026-10-19'),('smart-b','demo-0',1,'note','violet','synthetic','2026-10-19','2026-10-19'); INSERT INTO bookmarks(id,document_id,page_index,created_at,updated_at) VALUES('smart-a','demo-0',0,'2026-10-19','2026-10-19'),('smart-b','demo-0',1,'2026-10-19','2026-10-19');")?;
        }
        if capture.is_some()
            && args
                .iter()
                .filter_map(|arg| arg.to_str())
                .any(|arg| arg.starts_with("--grouping="))
        {
            db.execute_batch("UPDATE documents SET author=CASE WHEN id IN ('demo-0','demo-2') THEN 'Study collective' WHEN id='demo-1' THEN NULL ELSE author END; INSERT INTO collections VALUES('grouping-collection','Personal study','personal study','study',0,1,'2026-10-21','2026-10-21'); INSERT INTO collection_documents VALUES('grouping-collection','demo-0','2026-10-21'),('grouping-collection','demo-2','2026-10-21'); INSERT INTO tags VALUES('grouping-tag','Research','research','violet','2026-10-21','2026-10-21'); INSERT INTO document_tags VALUES('demo-0','grouping-tag'),('demo-2','grouping-tag');")?;
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--tag-management=failure") {
            db.execute_batch("CREATE TRIGGER reject_tag_rename BEFORE UPDATE OF name ON tags WHEN NEW.name='Rejected name' BEGIN SELECT RAISE(ABORT,'synthetic tag failure'); END;")?;
        }
        if capture.is_some()
            && args
                .iter()
                .any(|arg| arg == "--tag-management=paged-picker")
        {
            for index in 0..80 {
                db.execute_batch(&format!("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('tag-target-{index:03}','Z target {index:03}','z target {index:03}','blue','synthetic','synthetic')"))?;
            }
        }
        if capture.is_some() && args.iter().any(|arg| arg == "--library-tags") {
            db.execute_batch("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('legacy-study','Study','study','violet','2026-10-19','2026-10-19'); INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('tag-annotation','demo-0',0,'note','violet','synthetic','2026-10-19','2026-10-19'); INSERT INTO annotation_tags(annotation_id,tag_id) VALUES('tag-annotation','legacy-study');")?;
        }
        if capture.is_some()
            && args.iter().any(|arg| {
                arg.to_str()
                    .is_some_and(|value| value.starts_with("--bulk-organization="))
            })
        {
            for index in 0..80 {
                db.execute_batch(&format!("INSERT INTO collections(id,name,normalized_name,icon,sort_order,is_pinned,created_at,updated_at) VALUES('bulk-c-{index}','Example collection {index:03}','example collection {index:03}','folder',{index},0,'2026-10-20','2026-10-20'); INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('bulk-t-{index}','Example tag {index:03}','example tag {index:03}','violet','2026-10-20','2026-10-20');"))?;
            }
            db.execute_batch("INSERT INTO collections(id,name,normalized_name,icon,sort_order,is_pinned,created_at,updated_at) VALUES('bulk-study-collection','Study','study','study',80,0,'2026-10-20','2026-10-20'); INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('bulk-study-tag','Study','study','violet','2026-10-20','2026-10-20'); INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('bulk-note','demo-0',0,'note','violet','synthetic','2026-10-20','2026-10-20'); INSERT INTO annotation_tags(annotation_id,tag_id) VALUES('bulk-note','bulk-study-tag');")?;
            if args.iter().any(|arg| arg == "--bulk-long-target") {
                db.execute_batch("UPDATE tags SET name='Study — littérature française, manuscrits et annotations',normalized_name='study — littérature française, manuscrits et annotations' WHERE id='bulk-study-tag'")?;
            }
            if args.iter().any(|arg| arg == "--bulk-organization=remove") {
                db.execute_batch("INSERT INTO collection_documents(collection_id,document_id,added_at) SELECT 'bulk-study-collection',id,'2026-10-20' FROM documents WHERE id IN ('demo-0','demo-1','demo-2','demo-3')")?;
            }
        }
        let mut reading = ReaderPreferences::new("demo-0");
        if args
            .iter()
            .any(|arg| arg == "--bookmarks" || arg == "--reference-bookmarks")
        {
            db.execute_batch("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES('demo-bookmark-1','demo-0',0,'A passage to revisit','2026-10-09','2026-10-09'),('demo-bookmark-2','demo-0',1,'Notes for tomorrow','2026-10-09','2026-10-09');")?;
        }
        for arg in &args {
            if let Some(value) = arg
                .to_str()
                .and_then(|arg| arg.strip_prefix("--reader-theme="))
            {
                reading.reading_theme = ReaderTheme::from_persisted(value)?;
            }
            if let Some(value) = arg
                .to_str()
                .and_then(|arg| arg.strip_prefix("--reader-zoom="))
            {
                reading.zoom_mode = ZoomMode::Custom;
                reading.zoom_level = value.parse()?;
            }
            if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--rotation=")) {
                reading.page_rotation = PageRotation::from_degrees(value.parse()?)?;
            }
        }
        if args.iter().any(|arg| arg == "--fit-width") {
            reading.zoom_mode = ZoomMode::FitWidth;
        }
        if args.iter().any(|arg| arg == "--facing") {
            reading.page_layout = PageLayout::Spread;
        }
        if args.iter().any(|arg| arg == "--continuous") {
            reading.page_layout = PageLayout::Continuous;
        }
        if args.iter().any(|arg| arg == "--book-mode") {
            reading.view_mode = ReaderMode::Book;
        }
        db.save_reader_preferences("demo-0", &reading)?;
        if capture.is_some()
            && args
                .iter()
                .filter_map(|arg| arg.to_str())
                .any(|arg| arg.starts_with("--legacy-import="))
        {
            std::fs::create_dir_all(store.join("flutter-source"))?;
            db.snapshot(&store.join("flutter-source/yamantaka.sqlite3"))?;
            db.snapshot(&store.join("yamantaka.sqlite3"))?;
            std::fs::write(
                store.join("flutter-source/shared_preferences.json"),
                r#"{"flutter.darkMode":true,"flutter.languageCode":"en","custom":42}"#,
            )?;
            Preferences::default().save(&store.join("preferences.json"))?;
        }
        if capture.is_some()
            && args
                .iter()
                .any(|arg| arg == "--restore-preview" || arg == "--restore-apply")
        {
            db.export_backup(
                &store.join("synthetic-backup.json"),
                &Preferences::default(),
                &yamantaka_domain::Cancellation::default(),
            )?;
        }
        (db, Preferences::default(), store)
    } else {
        let active_store = yamantaka_storage::managed_imports::recover_and_select(&paths.store)?;
        std::fs::create_dir_all(&active_store)?;
        startup = Some(yamantaka_storage::startup::StartupRun::begin(
            &active_store,
            requested_safe,
        ));
        match startup_recovery::load(&active_store) {
            Ok((database, preferences)) => (database, preferences, active_store),
            Err(stage) => {
                let installed = startup_recovery::run(
                    active_store,
                    stage,
                    args.iter().any(|arg| arg == "--en"),
                    args.iter().any(|arg| arg == "--software"),
                    None,
                )?;
                if !installed && let Some(startup) = startup.take() {
                    startup.finish()?;
                }
                return Ok(());
            }
        }
    };
    if demo && args.iter().any(|arg| arg == "--sessions-disabled") {
        prefs
            .rust
            .insert("recordReadingSessions".into(), false.into());
    }
    if demo
        && capture.is_some()
        && args
            .iter()
            .any(|arg| arg == "--safe-mode" || arg == "--safe-failures" || arg == "--safe-unknown")
    {
        prefs.rust.insert("globalIndexEnabled".into(), true.into());
    }
    let activations = if demo {
        store.join("activations")
    } else {
        paths.activations
    };
    let instance = if let Some(instance) = primary_instance {
        instance
    } else {
        let Activation::Primary(instance) = Instance::acquire(&activations, &initial)? else {
            return Ok(());
        };
        instance
    };
    let bundled = executable.parent().unwrap().join("pdfium.dll");
    if demo
        && capture.is_some()
        && args
            .iter()
            .any(|arg| arg == "--safe-failures" || arg == "--safe-unknown")
    {
        std::fs::create_dir_all(&store)?;
        let fixture = if args.iter().any(|arg| arg == "--safe-unknown") {
            serde_json::json!({"version":99,"synthetic":true})
        } else {
            serde_json::json!({"version":1,"clean":false,"failures":2,"token":uuid::Uuid::new_v4().to_string()})
        };
        std::fs::write(
            store.join("rust-startup-v1.json"),
            serde_json::to_vec(&fixture)?,
        )?;
    }
    let startup = startup
        .unwrap_or_else(|| yamantaka_storage::startup::StartupRun::begin(&store, requested_safe));
    let startup_mode = startup.mode();
    let library = if bundled.exists() || !cfg!(debug_assertions) {
        bundled
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/native/pdfium-7881/bin/pdfium.dll")
    };
    // The 72 MiB page category includes CPU pages, the replacement texture and
    // its staging buffer. Keep the CPU cache a bounded subset of that ledger.
    let governor = MemoryGovernor::new(GlobalMemoryLimits::default());
    let cover_cache_root = if demo {
        store.join("cover-cache-v1")
    } else {
        paths.cache.join("covers-v1")
    };
    let cache_megabytes = prefs
        .get("maxCacheMegabytes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(256)
        .clamp(64, 1024);
    let engine = PdfWorker::start_with_cover_cache(
        &library,
        MemoryLimits {
            pages: 24 * 1024 * 1024,
            tiles: 8 * 1024 * 1024,
            ..MemoryLimits::default()
        },
        governor.clone(),
        if startup_mode.is_safe() {
            None
        } else {
            Some(std::sync::Arc::new(
                yamantaka_storage::cover_cache::CoverCache::new(
                    cover_cache_root.clone(),
                    cache_megabytes * 1024 * 1024,
                ),
            ))
        },
    )?;
    if demo && capture.is_some() && args.iter().any(|arg| arg == "--cache-maintenance") {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/rust-fixtures/textual.pdf");
        let cover = engine
            .try_cover(
                "cache-fixture".into(),
                fixture,
                1,
                0,
                yamantaka_domain::Cancellation::default(),
            )?
            .wait()?;
        drop(cover);
        std::fs::write(
            cover_cache_root.join("notes.txt"),
            b"Synthetic neighboring file preserved by cache cleanup",
        )?;
    }
    let dark = if demo {
        !light
    } else {
        prefs
            .get("darkMode")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    };
    if demo {
        if args.iter().any(|arg| arg == "--search") {
            prefs.rust.insert("globalIndexEnabled".into(), true.into());
        }
        if args.iter().any(|arg| arg == "--reduce-motion") {
            prefs.rust.insert("reduceMotion".into(), true.into());
        }
        if let Some(scale) = args
            .iter()
            .find_map(|arg| {
                arg.to_str()
                    .and_then(|arg| arg.strip_prefix("--text-scale="))
                    .and_then(|value| value.parse::<f64>().ok())
            })
            .filter(|value| value.is_finite())
        {
            prefs
                .rust
                .insert("textScale".into(), scale.clamp(1.0, 2.0).into());
        }
    }
    let mut saved_window_placement = prefs.get("nativeWindowPlacement").cloned();
    let window_verification =
        demo && capture.is_some() && args.iter().any(|arg| arg == "--window-placement");
    let window_unknown = args.iter().any(|arg| arg == "--window-placement-unknown");
    let window_capture = if window_verification {
        capture.as_ref().map(|path| (store.clone(), path.clone()))
    } else {
        None
    };
    if window_verification
        && let Some(placement) = window_state::preview::seed(
            &store,
            window_unknown,
            args.iter().any(|arg| arg == "--window-placement-backup"),
        )?
    {
        saved_window_placement = Some(placement);
    }
    let english = if demo {
        !args.iter().any(|arg| arg == "--es")
    } else {
        prefs
            .get("languageCode")
            .and_then(serde_json::Value::as_str)
            == Some("en")
    };
    let software = args
        .iter()
        .any(|arg| arg == "--opengl" || arg == "--software");
    let icon = image::load_from_memory(include_bytes!(
        "../../../assets/branding/yamantaka-logo-purple.png"
    ))?
    .to_rgba8();
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(if startup_mode.is_safe() {
                if english {
                    "Yamantaka · Safe mode"
                } else {
                    "Yamantaka · Modo seguro"
                }
            } else {
                "Yamantaka"
            })
            .with_inner_size([1440.0, 960.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(egui::IconData {
                rgba: icon.as_raw().clone(),
                width: icon.width(),
                height: icon.height(),
            }),
        renderer: if software {
            eframe::Renderer::Glow
        } else {
            eframe::Renderer::Wgpu
        },
        ..Default::default()
    };
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
        if demo
            && capture.is_some()
            && !software
            && args.iter().any(|arg| arg == "--graphics-fail-startup")
        {
            setup.native_adapter_selector = Some(std::sync::Arc::new(|_, _| {
                Err("Synthetic adapter selection failure".into())
            }));
        }
    }
    let legacy_window = !demo && !startup_mode.is_safe();
    if legacy_window {
        let number = |key| {
            prefs
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .filter(|value| value.is_finite())
        };
        if let (Some(width), Some(height)) = (number("windowWidth"), number("windowHeight")) {
            options.viewport = options.viewport.with_inner_size([
                width.clamp(640.0, 16_384.0) as f32,
                height.clamp(480.0, 16_384.0) as f32,
            ]);
        }
        if let (Some(x), Some(y)) = (number("windowX"), number("windowY")) {
            options.viewport = options.viewport.with_position([
                x.clamp(-100_000.0, 100_000.0) as f32,
                y.clamp(-100_000.0, 100_000.0) as f32,
            ]);
        }
    }
    if let Some(size) = args
        .iter()
        .find_map(|arg| arg.to_str().and_then(|s| s.strip_prefix("--size=")))
        && let Some((width, height)) = size.split_once('x')
        && let (Ok(width), Ok(height)) = (width.parse::<f32>(), height.parse::<f32>())
    {
        options.viewport = options.viewport.with_inner_size([width, height]);
    }
    let instance = std::rc::Rc::new(instance);
    let app_instance = instance.clone();
    let restore_handover: app::RestoreHandover = Default::default();
    let ui_restore_handover = restore_handover.clone();
    let restore_smoke = if demo {
        capture
            .as_ref()
            .filter(|_| args.iter().any(|arg| arg == "--restore-apply"))
            .map(|path| path.with_extension("restore-result.json"))
    } else {
        None
    };
    let graphics = graphics_recovery::State::default();
    let app_graphics = graphics.clone();
    let recovery_demo_store =
        (demo && capture.is_some() && args.iter().any(|arg| arg == "--graphics-fail-runtime"))
            .then(|| store.clone());
    if recovery_demo_store.is_some() && !graphics_recovery::runtime_fallback() {
        prefs.rust.insert("restoreLastSession".into(), false.into());
        prefs.save(&store.join("preferences.json"))?;
    }
    let app_result = eframe::run_native(
        "Yamantaka",
        options,
        Box::new(move |cc| {
            if demo
                && capture.is_some()
                && args.iter().any(|arg| arg == "--graphics-fail-application")
            {
                return Err("Synthetic application creation failure".into());
            }
            theme::load_fonts(&cc.egui_ctx);
            theme::install(&cc.egui_ctx, dark);
            for (argument, key) in [
                ("--library-view-menu", "preview-library-view-menu"),
                ("--library-search-menu", "preview-library-search-menu"),
            ] {
                if demo && args.iter().any(|arg| arg == argument) {
                    cc.egui_ctx
                        .data_mut(|data| data.insert_temp(egui::Id::new(key), true));
                }
            }
            let notify = cc.egui_ctx.clone();
            if demo && args.iter().any(|arg| arg == "--cover-capacity-retry") {
                let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../build/rust-fixtures/textual.pdf");
                for id in ["capacity-reader-a", "capacity-reader-b"] {
                    engine.open(
                        id.into(),
                        source.clone(),
                        None,
                        yamantaka_domain::Cancellation::default(),
                    )?;
                }
                engine.set_visible_documents(&[
                    "capacity-reader-a".into(),
                    "capacity-reader-b".into(),
                ])?;
            }
            let backup_source = database.file_path();
            let runtime = Runtime::start(database, engine, move || notify.request_repaint())?;
            let presenter = cc.wgpu_render_state.clone().map(|state| {
                app_graphics.install(&state.device, &cc.egui_ctx);
                page_presenter::PagePresenter::new(state, governor.clone(), cc.egui_ctx.clone())
            });
            let (tx, rx) = mpsc::channel();
            let ctx = cc.egui_ctx.clone();
            let watch = app_instance.watch(move |paths| {
                let _ = tx.send(paths);
                ctx.request_repaint();
            })?;
            let preview_device_fault = demo
                && capture.is_some()
                && args.iter().any(|arg| arg == "--graphics-fail-runtime");
            let mut app = app::YamantakaApp::new(
                runtime,
                prefs,
                store.clone(),
                dark,
                english,
                demo,
                settings,
                capture,
                rx,
                app_instance,
                watch,
                governor,
                presenter,
                startup_mode,
            );
            app.graphics_recovery = app_graphics.clone();
            if preview_device_fault && let Some(state) = &cc.wgpu_render_state {
                app.preview_graphics_loss(state.device.clone());
                if args.iter().any(|arg| arg == "--graphics-fail-checkpoint") {
                    app_graphics.checkpoint_failed();
                }
            }
            if demo && startup_mode.is_safe() {
                app.preview_safe_index();
                app.preview_safe_dismiss(args.iter().any(|arg| arg == "--safe-dismiss"));
            }
            if (!demo || window_verification) && !startup_mode.is_safe() {
                let mut state = window_state::State::open(
                    &store,
                    cc,
                    legacy_window,
                    saved_window_placement.as_ref(),
                );
                if window_verification && !window_unknown {
                    state
                        .as_mut()
                        .expect("Native window state must open")
                        .start_verification();
                }
                app.configure_window_state(state);
            }
            app.configure_restore(
                backup_source.as_ref().map(|_| store.clone()),
                ui_restore_handover,
            );
            app.configure_diagnostics(
                backup_source.clone(),
                if demo {
                    store.clone()
                } else {
                    paths.store.clone()
                },
            );
            app.configure_backup(
                backup_source,
                if demo {
                    store.clone()
                } else {
                    paths.store.clone()
                },
            );
            app.configure_legacy_import(
                if demo {
                    store.clone()
                } else {
                    paths.store.clone()
                },
                yamantaka_storage::migration::LegacySource {
                    database: paths.legacy_installed.join("yamantaka.sqlite3"),
                    preferences: paths.legacy_installed.join("shared_preferences.json"),
                },
            );
            cc.egui_ctx.set_zoom_factor(app.text_scale());
            if args
                .iter()
                .any(|arg| arg == "--palette-preview" || arg == "--annotation-history")
            {
                app.preview_command_palette(
                    if args.iter().any(|arg| arg == "--annotation-history") {
                        4
                    } else if args.iter().any(|arg| arg == "--palette-execute") {
                        1
                    } else if args.iter().any(|arg| arg == "--palette-recent") {
                        2
                    } else if args.iter().any(|arg| arg == "--palette-empty") {
                        3
                    } else {
                        0
                    },
                );
            }

            if args.iter().any(|arg| arg == "--legacy-reader-commands") {
                app.preview_legacy_reader_commands(
                    args.iter().any(|arg| arg == "--page-jump-review"),
                );
            }

            if args
                .iter()
                .any(|arg| arg == "--shortcut-settings" || arg == "--custom-shortcuts")
            {
                app.preview_binding_settings(
                    &cc.egui_ctx,
                    if args.iter().any(|arg| arg == "--custom-shortcuts") {
                        2
                    } else if args.iter().any(|arg| arg == "--shortcut-conflict") {
                        1
                    } else {
                        0
                    },
                );
            }
            if tilt {
                app.preview_tilt(args.iter().any(|arg| arg == "--tilt-negative"));
            }
            if demo {
                if args.iter().any(|arg| arg == "--reading-queue") {
                    app.preview_reading_queue();
                }
                if args.iter().any(|arg| arg == "--workspaces") {
                    app.preview_workspaces();
                }
                if args
                    .iter()
                    .any(|arg| arg == "--restore-preview" || arg == "--restore-apply")
                {
                    app.preview_restore(
                        &cc.egui_ctx,
                        store.join("synthetic-backup.json"),
                        args.iter().any(|arg| arg == "--restore-apply"),
                    );
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|arg| arg.strip_prefix("--backup-export="))
                {
                    app.preview_backup(&cc.egui_ctx, mode);
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|arg| arg.strip_prefix("--legacy-import="))
                {
                    app.preview_legacy_import(
                        &cc.egui_ctx,
                        yamantaka_storage::migration::LegacySource {
                            database: store.join("flutter-source/yamantaka.sqlite3"),
                            preferences: store.join("flutter-source/shared_preferences.json"),
                        },
                        mode,
                    );
                }
                if args.iter().any(|arg| arg == "--cover-capacity-retry") {
                    app.preview_cover_capacity();
                }
                if !graphics_recovery::continuity_resume()
                    && args.iter().any(|arg| arg == "--metadata-review")
                {
                    app.preview_metadata(
                        if args.iter().any(|arg| arg == "--metadata-source-stale") {
                            "source-stale"
                        } else if args.iter().any(|arg| arg == "--metadata-source-open") {
                            "source"
                        } else if args.iter().any(|arg| arg == "--metadata-source-return") {
                            "source-return"
                        } else if args.iter().any(|arg| arg == "--metadata-source-draft") {
                            "source-draft"
                        } else if args.iter().any(|arg| arg == "--metadata-reference-visible") {
                            "reference"
                        } else if args.iter().any(|arg| arg == "--metadata-reanalyze") {
                            "reanalyze"
                        } else if args
                            .iter()
                            .any(|arg| arg == "--metadata-save" || arg == "--metadata-failed")
                        {
                            "save"
                        } else if args.iter().any(|arg| arg == "--metadata-reset") {
                            "reset"
                        } else if args.iter().any(|arg| arg == "--metadata-dismiss") {
                            "dismiss"
                        } else {
                            "open"
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--cover-picker") {
                    app.preview_cover_editor("open");
                }
                if args.iter().any(|arg| arg == "--cover-picker-restore") {
                    app.preview_cover_editor("restore");
                }
                if args
                    .iter()
                    .any(|arg| arg == "--cover-picker-restore-failed")
                {
                    app.preview_cover_editor("failed-restore");
                }
                if args.iter().any(|arg| arg == "--cover-picker-save") {
                    app.preview_cover_editor("save");
                }
                if args.iter().any(|arg| arg == "--cover-picker-cancel") {
                    app.preview_cover_editor("cancel");
                }
                if args.iter().any(|arg| arg == "--cover-picker-failed-save") {
                    app.preview_cover_editor("failed-save");
                }
                if args.iter().any(|arg| arg == "--tabs-close-others") {
                    app.preview_bulk_tab_close(false);
                }
                if args.iter().any(|arg| arg == "--tabs-close-right") {
                    app.preview_bulk_tab_close(true);
                }
                if args.iter().any(|arg| arg == "--tabs-inactive-close") {
                    app.preview_tab_closure(4);
                }
                if args.iter().any(|arg| arg == "--tabs-close-reopen") {
                    app.preview_tab_closure(1);
                }
                if args.iter().any(|arg| arg == "--tabs-last-reopen") {
                    app.preview_tab_closure(2);
                }
                if args.iter().any(|arg| arg == "--tabs-last-close") {
                    app.preview_tab_closure(3);
                }
                if args.iter().any(|arg| arg == "--tabs") {
                    app.preview_reader_tabs();
                }
                if args.iter().any(|arg| arg == "--sessions-preview") {
                    app.preview_sessions();
                }
                if let Some(key) = args.iter().find_map(|arg| {
                    arg.to_str()
                        .and_then(|value| value.strip_prefix("--smart-view="))
                }) {
                    app.preview_smart_view(key, args.iter().any(|arg| arg == "--smart-menu"));
                }
                if args.iter().any(|arg| arg == "--reset-library-filters") {
                    app.preview_reset_library_filters();
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|value| value.strip_prefix("--bulk-covers="))
                {
                    app.preview_cover_batch(mode);
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|value| value.strip_prefix("--bulk-remove="))
                {
                    app.preview_library_removal(mode);
                }
                if args.iter().any(|arg| arg == "--library-selection") {
                    app.preview_library_selection(
                        args.iter().any(|arg| arg == "--selection-favorite"),
                        args.iter().any(|arg| arg == "--selection-menu"),
                    );
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|value| value.strip_prefix("--bulk-organization="))
                {
                    app.preview_bulk_organization(mode);
                }
                if let Some(mode) = args
                    .iter()
                    .filter_map(|arg| arg.to_str())
                    .find_map(|arg| arg.strip_prefix("--grouping="))
                {
                    app.preview_grouping(mode, args.iter().any(|arg| arg == "--fold-group"));
                }
                if args.iter().any(|arg| arg == "--library-tags") {
                    let mode = args
                        .iter()
                        .filter_map(|arg| arg.to_str())
                        .find_map(|value| value.strip_prefix("--tag-management="));
                    app.preview_tags(match mode {
                        Some("rename") => 2,
                        Some("color") => 3,
                        Some("picker") => 4,
                        Some("merge-review") => 5,
                        Some("merge") => 6,
                        Some("delete-review") => 7,
                        Some("delete") => 8,
                        Some("failure") => 9,
                        Some("cancel") => 10,
                        Some("actions") => 11,
                        Some("rename-editor") => 12,
                        Some("paged-picker") => 14,
                        _ => u8::from(args.iter().any(|arg| arg == "--tag-filter")),
                    });
                }
                if args.iter().any(|arg| arg == "--collections") {
                    app.preview_collections(
                        if args.iter().any(|arg| arg == "--collection-filter") {
                            1
                        } else if args.iter().any(|arg| arg == "--collection-delete-review") {
                            2
                        } else if args.iter().any(|arg| arg == "--collection-icon-review") {
                            3
                        } else if args.iter().any(|arg| arg == "--collection-order") {
                            4
                        } else if args.iter().any(|arg| arg == "--collection-order-search") {
                            5
                        } else if args.iter().any(|arg| arg == "--collection-actions") {
                            6
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--cache-maintenance") {
                    app.preview_cover_cache(
                        &cc.egui_ctx,
                        args.iter().any(|arg| arg == "--clear-cover-cache"),
                    );
                }
                if args.iter().any(|arg| arg == "--privacy") {
                    app.preview_privacy(&cc.egui_ctx);
                }
                if args.iter().any(|arg| arg == "--quarantine") {
                    app.preview_quarantine(
                        if args.iter().any(|arg| arg == "--quarantine-review") {
                            1
                        } else if args.iter().any(|arg| arg == "--quarantine-release") {
                            2
                        } else if args.iter().any(|arg| arg == "--quarantine-retry") {
                            3
                        } else if args.iter().any(|arg| arg == "--quarantine-auto") {
                            4
                        } else if args.iter().any(|arg| arg == "--quarantine-failure") {
                            5
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--diagnostics") {
                    app.preview_diagnostics(
                        &cc.egui_ctx,
                        if args.iter().any(|arg| arg == "--diagnostics-export") {
                            1
                        } else if args.iter().any(|arg| arg == "--diagnostics-occupied") {
                            2
                        } else if args.iter().any(|arg| arg == "--diagnostics-cancel") {
                            3
                        } else if args.iter().any(|arg| arg == "--diagnostics-error") {
                            4
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| {
                    arg == "--goal-editor" || arg == "--goal-save" || arg == "--goal-clear"
                }) {
                    app.preview_goal(if args.iter().any(|arg| arg == "--goal-save") {
                        2
                    } else if args.iter().any(|arg| arg == "--goal-clear") {
                        3
                    } else {
                        1
                    });
                }
                app.preview_layout(
                    args.iter().any(|arg| arg == "--list"),
                    args.iter().any(|arg| arg == "--details"),
                    args.iter().any(|arg| arg == "--text-panel"),
                );
                if args
                    .iter()
                    .any(|arg| arg == "--compare-pdf" || arg == "--compare-different")
                {
                    app.preview_file_comparison(
                        args.iter().any(|arg| arg == "--compare-different"),
                    );
                }
                if args.iter().any(|arg| arg == "--inherited-journals") {
                    app.preview_inherited_journals(
                        if args.iter().any(|arg| arg == "--inherited-recovery-unknown") {
                            8
                        } else if args.iter().any(|arg| arg == "--inherited-recovery-fail") {
                            10
                        } else if args.iter().any(|arg| arg == "--inherited-recovery-undo") {
                            7
                        } else if args.iter().any(|arg| arg == "--inherited-recovery-apply") {
                            6
                        } else if args.iter().any(|arg| arg == "--inherited-recovery") {
                            if args.iter().any(|arg| arg == "--text-scale=2") {
                                9
                            } else {
                                5
                            }
                        } else if args.iter().any(|arg| arg == "--inherited-paged") {
                            4
                        } else if args.iter().any(|arg| arg == "--inherited-incompatible") {
                            2
                        } else if args.iter().any(|arg| arg == "--inherited-retry") {
                            1
                        } else if args.iter().any(|arg| arg == "--text-scale=2") {
                            3
                        } else {
                            0
                        },
                    );
                } else if args.iter().any(|arg| arg == "--rule-simulation") {
                    app.preview_rule_simulation(
                        if args.iter().any(|arg| arg == "--rule-import-context") {
                            if args.iter().any(|arg| arg == "--rule-import-no-context") {
                                14
                            } else if args.iter().any(|arg| arg == "--rule-import-undo") {
                                13
                            } else if args.iter().any(|arg| arg == "--rule-import-apply") {
                                12
                            } else {
                                11
                            }
                        } else if args.iter().any(|arg| arg == "--rule-execute-fail") {
                            10
                        } else if args.iter().any(|arg| arg == "--rule-execute-undo") {
                            if args.iter().any(|arg| arg == "--text-scale=2") {
                                9
                            } else {
                                8
                            }
                        } else if args.iter().any(|arg| arg == "--rule-execute") {
                            7
                        } else if args.iter().any(|arg| arg == "--rule-review-missing") {
                            6
                        } else if args.iter().any(|arg| arg == "--rule-review") {
                            if args.iter().any(|arg| arg == "--text-scale=2") {
                                5
                            } else {
                                4
                            }
                        } else if args.iter().any(|arg| arg == "--rule-simulation-duplicates") {
                            1
                        } else if args.iter().any(|arg| arg == "--rule-simulation-cancel") {
                            2
                        } else if args.iter().any(|arg| arg == "--text-scale=2") {
                            3
                        } else {
                            0
                        },
                    );
                } else if args.iter().any(|arg| arg == "--saved-rules") {
                    app.preview_automation_rules(
                        if args.iter().any(|arg| arg == "--saved-rules-many") {
                            3
                        } else if args.iter().any(|arg| arg == "--saved-rules-review") {
                            2
                        } else if args.iter().any(|arg| arg == "--saved-rules-change") {
                            1
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--revision-groups") {
                    app.preview_revision_groups(
                        if args.iter().any(|arg| arg == "--revision-current") {
                            2
                        } else if args.iter().any(|arg| arg == "--revision-create") {
                            1
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--relations") {
                    app.preview_relations(
                        args.iter().any(|arg| arg == "--relations-review"),
                        if args.iter().any(|arg| arg == "--relation-location-open") {
                            2
                        } else if args.iter().any(|arg| arg == "--relation-locations") {
                            1
                        } else {
                            0
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--annotation-many") {
                    app.preview_annotation_list();
                }
                if args
                    .iter()
                    .any(|arg| arg == "--export-json" || arg == "--export-markdown")
                {
                    app.preview_annotation_export(
                        args.iter().any(|arg| arg == "--export-markdown"),
                    );
                }
                if args
                    .iter()
                    .any(|arg| arg == "--annotation-filter" || arg == "--annotation-filter-empty")
                {
                    app.preview_annotation_filters(
                        args.iter().any(|arg| arg == "--annotation-filter-empty"),
                    );
                }
                if args
                    .iter()
                    .any(|arg| arg == "--annotation-bulk" || arg == "--annotation-bulk-save")
                {
                    app.preview_annotation_bulk(
                        args.iter().any(|arg| arg == "--annotation-bulk-save"),
                    );
                }
                if args.iter().any(|arg| {
                    arg == "--annotation-bulk-tag" || arg == "--annotation-bulk-tag-save"
                }) {
                    app.preview_annotation_tag(
                        args.iter().any(|arg| arg == "--annotation-bulk-tag-save"),
                    );
                }
                if args.iter().any(|arg| {
                    arg.to_str()
                        .is_some_and(|value| value.starts_with("--global-notes"))
                }) {
                    let action = if args.iter().any(|arg| arg == "--global-notes-export-json") {
                        "export-json"
                    } else if args
                        .iter()
                        .any(|arg| arg == "--global-notes-export-markdown")
                    {
                        "export-markdown"
                    } else if args.iter().any(|arg| arg == "--global-notes-open") {
                        "open"
                    } else if args.iter().any(|arg| arg == "--global-notes-filters") {
                        "filters"
                    } else if args.iter().any(|arg| arg == "--global-notes-review") {
                        "review"
                    } else if args.iter().any(|arg| arg == "--global-notes-tag") {
                        "tag"
                    } else if args.iter().any(|arg| arg == "--global-notes-empty") {
                        "empty"
                    } else {
                        "list"
                    };
                    app.preview_global_notes(action);
                }
                if args.iter().any(|arg| {
                    arg == "--history"
                        || arg == "--history-next"
                        || arg == "--history-restore"
                        || arg == "--history-invalid"
                        || arg == "--history-delete"
                        || arg == "--history-empty"
                }) {
                    app.preview_history(
                        args.iter().any(|arg| arg == "--history-next"),
                        args.iter().any(|arg| arg == "--history-restore"),
                        args.iter().any(|arg| arg == "--history-delete"),
                    );
                }
                if args.iter().any(|arg| arg == "--page-note") {
                    app.preview_page_note(args.iter().any(|arg| arg == "--annotation-save"));
                }
                if args.iter().any(|arg| arg == "--annotation-conflict") {
                    app.preview_annotation_conflict();
                }
                if args.iter().any(|arg| arg == "--close-review") {
                    app.preview_editor_close(args.iter().any(|arg| arg == "--close-review-return"));
                }
                if args.iter().any(|arg| arg == "--annotations") {
                    app.preview_annotations(
                        args.iter().any(|arg| arg == "--annotation-save"),
                        args.iter().any(|arg| arg == "--annotation-multi"),
                    );
                }
                if args.iter().any(|arg| arg == "--selection") {
                    app.preview_selection(args.iter().any(|value| value == "--selection-keyboard"));
                }
                if args.iter().any(|arg| arg == "--links") {
                    app.preview_links(args.iter().any(|arg| arg == "--links-activate"));
                }
                if args.iter().any(|arg| arg == "--find") {
                    app.find_preview_closed = args.iter().any(|arg| arg == "--find-closed");
                    app.preview_find(
                        if args.iter().any(|arg| arg == "--find-absent") {
                            "absent_phrase_731"
                        } else {
                            "reading"
                        },
                        if args.iter().any(|arg| arg == "--find-previous") {
                            Some(false)
                        } else if args.iter().any(|arg| arg == "--find-next") {
                            Some(true)
                        } else {
                            None
                        },
                    );
                }
                if args.iter().any(|arg| arg == "--bookmarks") {
                    app.preview_bookmarks();
                }
                if args.iter().any(|arg| arg == "--outline") {
                    app.preview_outline(if args.iter().any(|arg| arg == "--outline-jump") {
                        Some(3)
                    } else {
                        None
                    });
                }
                if args.iter().any(|arg| arg == "--thumbnails") {
                    app.preview_thumbnails(args.iter().find_map(|arg| {
                        arg.to_str()
                            .and_then(|arg| arg.strip_prefix("--thumbnail-page="))
                            .and_then(|page| page.parse().ok())
                    }));
                }
                if args.iter().any(|arg| arg == "--list-states") {
                    app.preview_list_states();
                }
            }
            if demo && args.iter().any(|arg| arg == "--open-replacement-failure") {
                app.preview_failed_replacement();
            }
            if demo && args.iter().any(|arg| arg == "--import-protected") {
                app.preview_import_password(
                    if args.iter().any(|arg| arg == "--import-password-unlock") {
                        "unlock"
                    } else if args.iter().any(|arg| arg == "--import-password-wrong") {
                        "wrong"
                    } else if args.iter().any(|arg| arg == "--import-password-cancel") {
                        "cancel"
                    } else {
                        "prompt"
                    },
                );
            }
            if demo && args.iter().any(|arg| arg == "--cover-rebuild-menu") {
                app.preview_inspector_actions();
            }
            if demo && cover_rebuild {
                app.preview_cover_rebuild();
            }
            if demo && protected {
                app.preview_password(if args.iter().any(|arg| arg == "--password-unlock") {
                    "unlock"
                } else if args.iter().any(|arg| arg == "--password-wrong") {
                    "wrong"
                } else {
                    "prompt"
                });
            }
            if demo && reader && !graphics_recovery::continuity_resume() {
                app.preview_reader();
                if args.iter().any(|arg| arg == "--nearby-turn") {
                    app.preview_nearby_turn();
                }
                if args.iter().any(|arg| arg == "--reference") {
                    app.preview_reference();
                    if args.iter().any(|arg| arg == "--reference-regions") {
                        app.preview_reference_regions();
                    }
                    if args.iter().any(|arg| arg == "--reference-notes") {
                        app.preview_reference_notes(
                            !args.iter().any(|arg| arg == "--reference-book"),
                            args.iter().any(|arg| arg == "--reference-notes-editor"),
                        );
                    }
                    if args.iter().any(|arg| {
                        arg == "--reference-export-json" || arg == "--reference-export-md"
                    }) {
                        app.preview_reference_annotation_export(
                            args.iter().any(|arg| arg == "--reference-export-md"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-note-bulk") {
                        app.preview_reference_note_bulk(
                            args.iter().any(|arg| arg == "--reference-note-bulk-save"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-note-filters") {
                        app.preview_reference_note_filters(
                            args.iter()
                                .any(|arg| arg == "--reference-note-filters-empty"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-marks") {
                        app.preview_reference_marks(
                            !args.iter().any(|arg| arg == "--reference-book"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-selection") {
                        app.preview_reference_selection(
                            args.iter().any(|arg| arg == "--reference-selection-shared"),
                            args.iter().any(|arg| arg == "--reference-selection-close"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-sync") {
                        app.preview_reference_sync();
                    }
                    if args.iter().any(|arg| arg == "--reference-bookmarks") {
                        app.preview_reference_bookmarks(
                            !args.iter().any(|arg| arg == "--reference-book"),
                        );
                    }
                    if args.iter().any(|arg| arg == "--reference-chooser") {
                        app.preview_reference_chooser();
                    }
                    if args.iter().any(|arg| arg == "--reference-book") {
                        app.preview_reference_book(
                            args.iter().any(|arg| arg == "--reference-close"),
                            args.iter().any(|arg| arg == "--reference-swap"),
                            args.iter().any(|arg| arg == "--reference-find"),
                        );
                        if args.iter().any(|arg| {
                            arg == "--reference-shortcuts" || arg == "--reference-history"
                        }) {
                            app.preview_reference_shortcuts(
                                args.iter().any(|arg| arg == "--reference-shortcuts-close"),
                                args.iter().any(|arg| arg == "--reference-history"),
                                args.iter().any(|arg| arg == "--history-primary"),
                            );
                        }
                        if args.iter().any(|arg| arg == "--reference-text") {
                            app.preview_reference_text();
                        }
                        if args.iter().any(|arg| arg == "--reference-links") {
                            app.preview_reference_links();
                        }
                        if args.iter().any(|arg| arg == "--reference-thumbnails") {
                            app.preview_reference_thumbnails();
                        }
                        if args.iter().any(|arg| arg == "--reference-outline") {
                            app.preview_reference_outline();
                        }
                        if args.iter().any(|arg| arg == "--reference-protected") {
                            app.preview_reference_password(
                                args.iter().any(|arg| arg == "--reference-password-cancel"),
                                args.iter().any(|arg| arg == "--reference-password-import"),
                            );
                        }
                    }
                }
                if args.iter().any(|arg| arg == "--reader-options") {
                    app.preview_reader_options(usize::from(
                        args.iter().any(|arg| arg == "--layout-options"),
                    ));
                }
            }
            if demo
                && args
                    .iter()
                    .any(|arg| arg == "--search" || arg == "--search-disabled")
            {
                app.preview_search(args.iter().any(|arg| arg == "--search"));
                if args.iter().any(|arg| arg == "--search-open-page") {
                    app.preview_search_action("open");
                }
                if args.iter().any(|arg| arg == "--search-excluded") {
                    app.preview_search_action("exclude");
                }
            }
            if demo && args.iter().any(|arg| arg == "--library-check") {
                app.preview_library_check(
                    if args.iter().any(|arg| arg == "--library-check-cancel") {
                        1
                    } else if args.iter().any(|arg| arg == "--library-check-missing") {
                        2
                    } else if args.iter().any(|arg| arg == "--library-check-modified") {
                        3
                    } else {
                        0
                    },
                );
            }
            if demo && args.iter().any(|arg| arg == "--relocate") {
                app.preview_relocation(
                    store.join("relocated.pdf"),
                    args.iter().any(|arg| arg == "--relocation-confirm"),
                );
            }
            if demo && args.iter().any(|arg| arg == "--file-check") {
                app.preview_file_check();
            }
            if demo && args.iter().any(|arg| arg == "--import-recovery") {
                app.preview_import_recovery(if args.iter().any(|arg| arg == "--resume-import") {
                    1
                } else if args.iter().any(|arg| arg == "--discard-import") {
                    2
                } else {
                    0
                });
            }
            if demo && args.iter().any(|arg| arg == "--folder-import") {
                let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../build/rust-fixtures/folder-import");
                std::fs::create_dir_all(root.join("nested/empty"))?;
                for (source, target) in [
                    ("textual", "first.pdf"),
                    ("front-matter-missing", "nested/second.PDF"),
                    ("front-matter-wrapped-title", "nested/third.pdf"),
                ] {
                    std::fs::copy(
                        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join(format!("../../build/rust-fixtures/{source}.pdf")),
                        root.join(target),
                    )?;
                }
                std::fs::write(
                    root.join("nested/ignored.txt"),
                    b"Synthetic folder import fixture",
                )?;
                app.import_folder(root);
                if args.iter().any(|arg| arg == "--import-reference") {
                    app.preview_import_reference();
                }
            }
            if demo && args.iter().any(|arg| arg == "--tools-menu") {
                app.preview_tools_menu();
            }
            if !initial.is_empty() && !graphics_recovery::continuity_resume() {
                app.import(initial);
            }
            Ok(Box::new(app))
        }),
    );
    if graphics.faulted() {
        graphics.handover()?;
        startup.finish()?;
        drop(instance);
        return Err(Box::new(graphics_recovery::RuntimeFault {
            demo_store: recovery_demo_store,
        }));
    }
    app_result?;
    if let Some(commit) = restore_handover.borrow_mut().take() {
        let safety = yamantaka_storage::backup::install_preview(
            commit.preview,
            &commit.store,
            &yamantaka_domain::Cancellation::default(),
        )?;
        if let Some(result) = restore_smoke {
            let database = Database::open_read_only(&commit.store.join("yamantaka.sqlite3"))?;
            let books = database.count(&yamantaka_domain::LibraryQuery::default())?;
            std::fs::write(
                result,
                serde_json::to_vec(
                    &serde_json::json!({"synthetic":true,"installed":true,"books":books,"safetyExists":safety.is_dir()}),
                )?,
            )?;
        } else if !demo {
            yamantaka_windows::show_restore_complete(&safety, english);
        }
    }
    if let Some((window_store, capture)) = window_capture {
        window_state::preview::verify_saved(&window_store, &capture, window_unknown)?;
    }
    startup.finish()?;
    drop(instance);
    Ok(())
}
