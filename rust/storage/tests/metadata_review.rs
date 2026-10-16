use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository,
    metadata_review::{ManualMetadata, MetadataMutation},
    search::*,
};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,author,source_title,source_author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,is_favorite) VALUES('a','a.pdf','a.pdf','a.pdf','Current title','Current author','Raw title','Raw author',2,1,'now','now','now','fa',0.5,1); INSERT INTO document_enrichment(document_id,fingerprint,analyzer_version,status,suggested_title,suggested_author) VALUES('a','fa',2,'review','Geometry of Light','Ada Lovelace'); INSERT INTO document_covers(document_id,source_page_index,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,confidence,selection_mode,algorithm_version,crop_mode,page_aspect_ratio) VALUES('a',1,'small','large','now','now',1.0,'manual',1,'contain',0.75);").unwrap();
    db
}
fn save(title: &str, author: &str) -> MetadataMutation {
    MetadataMutation::Save(ManualMetadata::new(title, author).unwrap())
}
fn query(text: &str) -> SearchQuery {
    SearchQuery {
        text: text.into(),
        ..Default::default()
    }
}

#[test]
fn manual_corrections_accept_proposals_preserve_sources_and_reset_explicitly() {
    let mut db = library();
    let cancel = Cancellation::default();
    let before = db.metadata_review("a", &cancel).unwrap();
    assert_eq!(before.suggested_title.as_deref(), Some("Geometry of Light"));
    let after = db
        .change_metadata(
            &before,
            &save("Geometry of Light", "Ada Lovelace"),
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(after.title, "Geometry of Light");
    assert_eq!(after.author, "Ada Lovelace");
    assert_eq!(after.title_override.as_deref(), Some("Geometry of Light"));
    assert!(after.suggested_title.is_none() && after.suggested_author.is_none());
    assert_eq!(after.source_title.as_deref(), Some("Raw title"));
    assert_eq!(after.source_author.as_deref(), Some("Raw author"));
    db.execute_batch(
        "UPDATE documents SET title='Reanalyzed title',author='Reanalyzed author' WHERE id='a'",
    )
    .unwrap();
    assert_eq!(
        db.get("a").unwrap().unwrap().display_title(),
        "Geometry of Light"
    );
    let fresh = db.metadata_review("a", &cancel).unwrap();
    let reset = db
        .change_metadata(
            &fresh,
            &MetadataMutation::ResetOverrides,
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(reset.title, "Reanalyzed title");
    assert_eq!(reset.author, "Reanalyzed author");
    assert!(reset.title_override.is_none() && reset.author_override.is_none());
    let document = db.get("a").unwrap().unwrap();
    assert_eq!(document.reading_progress, 0.5);
    assert!(document.favorite);
    assert_eq!(db.cover_source_page("a").unwrap(), Some(1));
    assert!(db.search(&query("Geometry"), &cancel).unwrap().is_empty());
}

#[test]
fn stale_reviews_changed_analysis_and_cancellation_never_overwrite_newer_metadata() {
    let mut db = library();
    let cancel = Cancellation::default();
    let stale = db.metadata_review("a", &cancel).unwrap();
    db.execute_batch(
        "UPDATE document_enrichment SET suggested_title='New proposal' WHERE document_id='a'",
    )
    .unwrap();
    assert!(
        db.change_metadata(
            &stale,
            &save("Old editor", ""),
            IndexAccess::Disabled,
            &cancel
        )
        .is_err()
    );
    let fresh = db.metadata_review("a", &cancel).unwrap();
    let stopped = Cancellation::default();
    stopped.cancel();
    assert!(matches!(
        db.change_metadata(
            &fresh,
            &save("Cancelled", ""),
            IndexAccess::Enabled,
            &stopped
        ),
        Err(Failure::Cancelled)
    ));
    assert_eq!(db.metadata_review("a", &cancel).unwrap(), fresh);
    let next = db
        .change_metadata(
            &fresh,
            &save("Manual title", ""),
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(next.author, "");
    assert_eq!(next.author_override.as_deref(), Some(""));
    assert!(db.get("a").unwrap().unwrap().author.is_none());
    assert!(
        db.change_metadata(
            &fresh,
            &save("Stale", "Other"),
            IndexAccess::Disabled,
            &cancel
        )
        .is_err()
    );
}

#[test]
fn metadata_and_opt_in_search_refresh_commit_together_and_respect_exclusions() {
    let mut db = library();
    let cancel = Cancellation::default();
    let mut session = db
        .begin_index("a", false, IndexAccess::Enabled, &cancel)
        .unwrap();
    for page in 0..2 {
        db.index_page(
            &mut session,
            page,
            "Synthetic body",
            IndexAccess::Enabled,
            &cancel,
        )
        .unwrap();
    }
    db.finish_index(&mut session, IndexAccess::Enabled, &cancel)
        .unwrap();
    let before = db.metadata_review("a", &cancel).unwrap();
    db.change_metadata(
        &before,
        &save("Manual optics", "Reviewed institute"),
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    let hits = db.search(&query("Synthetic"), &cancel).unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| hit.title == "Manual optics"));
    assert!(!db.search(&query("institute"), &cancel).unwrap().is_empty());
    assert!(db.search(&query("Current"), &cancel).unwrap().is_empty());
    assert!(!db.needs_index("a").unwrap());
    db.exclude_from_index("a", true, &cancel).unwrap();
    let before = db.metadata_review("a", &cancel).unwrap();
    db.change_metadata(
        &before,
        &save("Excluded optics", "Author"),
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    assert!(db.search(&query("Excluded"), &cancel).unwrap().is_empty());
}

#[test]
fn failed_ignored_or_mutated_writes_roll_back_manual_values_and_proposals() {
    for trigger in [
        "CREATE TRIGGER broken BEFORE UPDATE OF title_override ON documents BEGIN SELECT RAISE(IGNORE); END;",
        "CREATE TRIGGER broken AFTER UPDATE OF title_override ON documents BEGIN UPDATE documents SET source_title='Changed' WHERE id=new.id; END;",
        "CREATE TRIGGER broken BEFORE UPDATE ON document_enrichment BEGIN SELECT RAISE(IGNORE); END;",
        "DROP TABLE library_search;",
    ] {
        let mut db = library();
        let cancel = Cancellation::default();
        let before = db.metadata_review("a", &cancel).unwrap();
        db.execute_batch(trigger).unwrap();
        assert!(
            db.change_metadata(
                &before,
                &save("Corrected", "Author"),
                IndexAccess::Enabled,
                &cancel
            )
            .is_err()
        );
        assert_eq!(db.metadata_review("a", &cancel).unwrap(), before);
    }
}

#[test]
fn dismissing_proposals_preserves_overrides_and_obsolete_analysis_is_not_offered() {
    let mut db = library();
    let cancel = Cancellation::default();
    db.execute_batch("UPDATE documents SET title_override='Manual decision'; UPDATE document_enrichment SET fingerprint='obsolete'").unwrap();
    let before = db.metadata_review("a", &cancel).unwrap();
    assert!(before.suggested_title.is_none());
    let after = db
        .change_metadata(
            &before,
            &MetadataMutation::DismissSuggestions,
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(after.title, "Manual decision");
    db.execute_batch("UPDATE document_enrichment SET fingerprint='fa',status='review'")
        .unwrap();
    assert!(
        db.metadata_review("a", &cancel)
            .unwrap()
            .suggested_title
            .is_none()
    );
    db.execute_batch("UPDATE documents SET source_title=CAST(zeroblob(70000) AS TEXT)")
        .unwrap();
    assert!(db.metadata_review("a", &cancel).is_err());
}
