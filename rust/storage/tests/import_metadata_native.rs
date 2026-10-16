use rusqlite::Connection;
use std::{fs, path::PathBuf};
use yamantaka_domain::{
    Cancellation, LibraryQuery, LibraryRepository, NewDocument, PdfEngine,
    pdf_ticket::AsyncPdfEngine,
    search::{IndexAccess, SearchQuery, SearchRepository},
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

#[test]
fn native_import_filters_tool_metadata_preserves_sources_and_keeps_manual_choices() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("Estudio de óptica.pdf");
    fs::copy(
        repository.join("build/rust-fixtures/metadata-tool.pdf"),
        &source,
    )
    .unwrap();
    let before = fs::read(&source).unwrap();
    let worker = PdfWorker::start(
        &repository.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let info = worker
        .open(
            "import".into(),
            source.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    assert_eq!(info.title.as_deref(), Some("Microsoft Word - Thesis.docx"));
    assert_eq!(info.author.as_deref(), Some("Unknown"));
    let database = root.path().join("library.sqlite3");
    let mut library = Database::open(&database).unwrap();
    let import = NewDocument {
        path: source.clone(),
        info,
        source: worker
            .try_source_revision("import".into(), Cancellation::default())
            .unwrap()
            .wait()
            .unwrap(),
        analysis: yamantaka_domain::front_matter::ImportAnalysis::NotRequested,
    };
    let id = library.import(&import).unwrap();
    let document = library.get(&id).unwrap().unwrap();
    assert_eq!(document.display_title(), "Estudio de óptica.pdf");
    assert!(document.author.is_none());
    let raw = Connection::open(&database).unwrap();
    let source_values: (String, String) = raw
        .query_row(
            "SELECT source_title,source_author FROM documents WHERE id=?",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        source_values,
        ("Microsoft Word - Thesis.docx".into(), "Unknown".into())
    );
    let cancel = Cancellation::default();
    let mut index = library
        .begin_index(&id, false, IndexAccess::Enabled, &cancel)
        .unwrap();
    library
        .index_page(
            &mut index,
            0,
            "Synthetic acceptance body",
            IndexAccess::Enabled,
            &cancel,
        )
        .unwrap();
    library
        .finish_index(&mut index, IndexAccess::Enabled, &cancel)
        .unwrap();
    let hits = library
        .search(
            &SearchQuery {
                text: "Estudio".into(),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|hit| hit.title == "Estudio de óptica.pdf"));
    assert!(
        library
            .search(
                &SearchQuery {
                    text: "Thesis".into(),
                    ..Default::default()
                },
                &cancel
            )
            .unwrap()
            .is_empty()
    );
    raw.execute("UPDATE documents SET title_override='Título revisado',author_override='Autora revisada',reading_progress=0.5,last_page_index=0 WHERE id=?", [&id]).unwrap();
    assert_eq!(library.import(&import).unwrap(), id);
    let manual = library.get(&id).unwrap().unwrap();
    assert_eq!(manual.title.as_deref(), Some("Título revisado"));
    assert_eq!(manual.author.as_deref(), Some("Autora revisada"));
    assert_eq!(manual.reading_progress, 0.5);
    assert_eq!(library.count(&LibraryQuery::default()).unwrap(), 1);
    assert_eq!(fs::read(source).unwrap(), before);
}

#[test]
fn native_import_preserves_unicode_organizations_and_legitimate_short_titles() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = tempfile::tempdir().unwrap();
    let worker = PdfWorker::start(
        &repository.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let mut library = Database::in_memory_at(11).unwrap();
    for (fixture, title, author) in [
        (
            "metadata-unicode",
            "Óptica y señales",
            "Instituto de Óptica",
        ),
        ("metadata-legitimate", "1984", "Anonymous"),
    ] {
        let source = root.path().join(format!("{fixture}.pdf"));
        fs::copy(
            repository.join(format!("build/rust-fixtures/{fixture}.pdf")),
            &source,
        )
        .unwrap();
        let before = fs::read(&source).unwrap();
        let info = worker
            .open(
                fixture.into(),
                source.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let id = library
            .import(&NewDocument {
                path: source.clone(),
                info,
                source: worker
                    .try_source_revision(fixture.into(), Cancellation::default())
                    .unwrap()
                    .wait()
                    .unwrap(),
                analysis: yamantaka_domain::front_matter::ImportAnalysis::NotRequested,
            })
            .unwrap();
        let document = library.get(&id).unwrap().unwrap();
        assert_eq!(document.title.as_deref(), Some(title));
        assert_eq!(document.author.as_deref(), Some(author));
        assert_eq!(fs::read(source).unwrap(), before);
    }
}
