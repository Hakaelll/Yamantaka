#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod book;
mod page_presenter;
mod theme;
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
    let tilt = args.iter().any(|arg| arg == "--tilt");
    let capture = args.iter().find_map(|arg| {
        arg.to_str()
            .and_then(|s| s.strip_prefix("--capture="))
            .map(PathBuf::from)
    });
    let executable = std::env::current_exe()?;
    let paths = AppPaths::discover(&executable)?;
    let initial: Vec<PathBuf> = args
        .iter()
        .filter(|arg| !arg.to_string_lossy().starts_with("--"))
        .map(PathBuf::from)
        .collect();
    let (database, mut prefs, store) = if demo {
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
        let mut db = if capture.is_some()
            && args.iter().any(|arg| {
                arg == "--export-json"
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
            } else if protected {
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
            let document_path = cover_fixture.as_ref().unwrap_or(&fixture).canonicalize()?;
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
        let mut reading = ReaderPreferences::new("demo-0");
        if args.iter().any(|arg| arg == "--bookmarks") {
            db.execute_batch("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES('demo-bookmark-1','demo-0',0,'A passage to revisit','2026-10-09','2026-10-09'),('demo-bookmark-2','demo-0',1,'Notes for tomorrow','2026-10-09','2026-10-09');")?;
        }
        for arg in &args {
            if let Some(value) = arg
                .to_str()
                .and_then(|arg| arg.strip_prefix("--reader-theme="))
            {
                reading.reading_theme = ReaderTheme::from_persisted(value)?;
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
        (db, Preferences::default(), store)
    } else {
        yamantaka_storage::backup::recover_interrupted_restore(&paths.store)?;
        std::fs::create_dir_all(&paths.store)?;
        (
            Database::open(&paths.store.join("yamantaka.sqlite3"))?,
            Preferences::load(&paths.store.join("preferences.json"))?,
            paths.store.clone(),
        )
    };
    if demo && args.iter().any(|arg| arg == "--sessions-disabled") {
        prefs
            .rust
            .insert("recordReadingSessions".into(), false.into());
    }
    let activations = if demo {
        store.join("activations")
    } else {
        paths.activations
    };
    let Activation::Primary(instance) = Instance::acquire(&activations, &initial)? else {
        return Ok(());
    };
    let bundled = executable.parent().unwrap().join("pdfium.dll");
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
        Some(std::sync::Arc::new(
            yamantaka_storage::cover_cache::CoverCache::new(
                cover_cache_root.clone(),
                cache_megabytes * 1024 * 1024,
            ),
        )),
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
    let english = if demo {
        !args.iter().any(|arg| arg == "--es")
    } else {
        prefs
            .get("languageCode")
            .and_then(serde_json::Value::as_str)
            == Some("en")
    };
    let software = args.iter().any(|arg| arg == "--opengl");
    let icon = image::load_from_memory(include_bytes!(
        "../../../assets/branding/yamantaka-logo-purple.png"
    ))?
    .to_rgba8();
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
    }
    if let Some(size) = args
        .iter()
        .find_map(|arg| arg.to_str().and_then(|s| s.strip_prefix("--size=")))
        && let Some((width, height)) = size.split_once('x')
        && let (Ok(width), Ok(height)) = (width.parse::<f32>(), height.parse::<f32>())
    {
        options.viewport = options.viewport.with_inner_size([width, height]);
    }
    eframe::run_native(
        "Yamantaka",
        options,
        Box::new(move |cc| {
            theme::load_fonts(&cc.egui_ctx);
            theme::install(&cc.egui_ctx, dark);
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
            let runtime = Runtime::start(database, engine, move || notify.request_repaint())?;
            let presenter = cc.wgpu_render_state.clone().map(|state| {
                page_presenter::PagePresenter::new(state, governor.clone(), cc.egui_ctx.clone())
            });
            let (tx, rx) = mpsc::channel();
            let ctx = cc.egui_ctx.clone();
            let watch = instance.watch(move |paths| {
                let _ = tx.send(paths);
                ctx.request_repaint();
            })?;
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
                instance,
                watch,
                governor,
                presenter,
            );
            cc.egui_ctx.set_zoom_factor(app.text_scale());
            if tilt {
                app.preview_tilt();
            }
            if demo {
                if args.iter().any(|arg| arg == "--cover-capacity-retry") {
                    app.preview_cover_capacity();
                }
                if args.iter().any(|arg| arg == "--metadata-review") {
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
                if args.iter().any(|arg| arg == "--collections") {
                    app.preview_collections(
                        if args.iter().any(|arg| arg == "--collection-filter") {
                            1
                        } else if args.iter().any(|arg| arg == "--collection-delete-review") {
                            2
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
            if demo && protected {
                app.preview_password(if args.iter().any(|arg| arg == "--password-unlock") {
                    "unlock"
                } else if args.iter().any(|arg| arg == "--password-wrong") {
                    "wrong"
                } else {
                    "prompt"
                });
            }
            if demo && reader {
                app.preview_reader();
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
            if !initial.is_empty() {
                app.import(initial);
            }
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
