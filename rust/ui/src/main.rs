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
use yamantaka_domain::{LibraryRepository, PageLayout};
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
            arg == "--reader" || arg == "--bookmarks" || arg == "--outline" || arg == "--thumbnails"
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
            store = store.join("captures").join(std::process::id().to_string());
        } else if args.iter().any(|arg| arg == "--demo-window") {
            store = store.join("previews").join(std::process::id().to_string());
        }
        let mut db = Database::in_memory_at(11)?;
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(if protected {
                "../../build/rust-fixtures/protected.pdf"
            } else if args.iter().any(|arg| arg == "--thumbnails-long") {
                "../../build/rust-fixtures/5000-pages.pdf"
            } else if args
                .iter()
                .any(|arg| arg == "--outline" || arg == "--thumbnails")
            {
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
            let path = fixture.to_string_lossy().replace('\'', "''");
            let title = if index == 0
                && args
                    .iter()
                    .any(|arg| arg == "--outline" || arg == "--thumbnails")
            {
                "The art of attention"
            } else {
                title
            };
            let title = title.replace('\'', "''");
            db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,last_page_index) VALUES('demo-{index}','{path}','demo-{index}','fixture.pdf','{title}','{author}',2,900,'2026-10-08','2026-10-08','2026-10-08','synthetic',{},0)",if index==0{0.5}else{0.0}))?;
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
    let engine = PdfWorker::start_with_governor(
        &library,
        MemoryLimits {
            pages: 24 * 1024 * 1024,
            tiles: 8 * 1024 * 1024,
            ..MemoryLimits::default()
        },
        governor.clone(),
    )?;
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
                runtime, prefs, store, dark, english, demo, settings, capture, rx, instance, watch,
                governor, presenter,
            );
            cc.egui_ctx.set_zoom_factor(app.text_scale());
            if tilt {
                app.preview_tilt();
            }
            if demo {
                app.preview_layout(
                    args.iter().any(|arg| arg == "--list"),
                    args.iter().any(|arg| arg == "--details"),
                    args.iter().any(|arg| arg == "--text-panel"),
                );
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
            if !initial.is_empty() {
                app.import(initial);
            }
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
