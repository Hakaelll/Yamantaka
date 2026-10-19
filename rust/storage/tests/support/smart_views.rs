use yamantaka_domain::{
    Cancellation, LibraryQuery, LibraryReadingState, LibraryRepository, LibrarySmartView as View,
    collections::{CollectionChange, CollectionQuery},
};
use yamantaka_storage::Database;

pub fn verify(db: &mut Database) {
    db.execute_batch("UPDATE documents SET reading_progress=0.995,added_at='2026-10-05T12:00:00Z',last_opened_at='2026-10-05T14:00:00+02:00';
        INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,is_missing,is_favorite) VALUES
        ('a','a.pdf','a.pdf','a.pdf','A','Study',3,100,'2026-10-19','2026-10-05T11:59:59Z','2026-10-05T11:59:59Z','a',0.994999,0,0),
        ('b','b.pdf','b.pdf','b.pdf','B','Study',3,100,'2026-10-19','2026-10-19T12:00:00Z','','b',0,0,0),
        ('c','c.pdf','c.pdf','c.pdf','C','Study',3,100,'2026-10-19','2026-10-20T12:00:00Z','invalid','c',1,1,0),
        ('d','d.pdf','d.pdf','d.pdf','D','Study',3,100,'2026-10-19','invalid','2026-10-06T12:00:00Z','d',0.5,0,1);
        INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('smart-a','d',0,'note','violet','d','2026-10-19','2026-10-19'),('smart-b','d',1,'note','violet','d','2026-10-19','2026-10-19');
        INSERT INTO bookmarks(id,document_id,page_index,created_at,updated_at) VALUES('smart-a','d',0,'2026-10-19','2026-10-19'),('smart-b','d',1,'2026-10-19','2026-10-19'),('smart-original','original',0,'2026-10-19','2026-10-19');").unwrap();
    let reference_time = chrono::DateTime::parse_from_rfc3339("2026-10-19T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc)
        .into();
    for (view, expected) in [
        (View::InProgress, vec!["a", "d"]),
        (View::NotStarted, vec!["b"]),
        (View::Finished, vec!["c", "original"]),
        (View::Annotated, vec!["d"]),
        (View::Bookmarked, vec!["d", "original"]),
        (View::Missing, vec!["c"]),
        (View::RecentlyOpened, vec!["d", "original"]),
        (View::RecentlyAdded, vec!["b", "c", "original"]),
    ] {
        assert_eq!(View::from_key(view.key()), Some(view));
        let query = LibraryQuery {
            smart_view: Some(view),
            reference_time,
            ..Default::default()
        };
        let mut actual: Vec<_> = db
            .query(&query)
            .unwrap()
            .into_iter()
            .map(|doc| doc.id)
            .collect();
        actual.sort();
        assert_eq!(actual, expected, "{view:?}");
        assert_eq!(db.count(&query).unwrap(), expected.len() as u64);
    }
    assert!(View::from_key("future-view").is_none());
    let query = LibraryQuery {
        smart_view: Some(View::Bookmarked),
        reference_time,
        page_size: 1,
        ..Default::default()
    };
    let first = db.query(&query).unwrap();
    let second = db
        .query(&LibraryQuery {
            offset: 1,
            ..query.clone()
        })
        .unwrap();
    assert_eq!(
        (first.len(), second.len(), db.count(&query).unwrap()),
        (1, 1, 2)
    );
    assert_ne!(first[0].id, second[0].id);
    let combined = LibraryQuery {
        favorites_only: true,
        authors: vec!["study".into()],
        text: "D".into(),
        ..query
    };
    assert_eq!(db.query(&combined).unwrap()[0].id, "d");
    assert_eq!(db.count(&combined).unwrap(), 1);
    let cancel = Cancellation::default();
    let id = db
        .change_collection(
            &CollectionChange::Create {
                name: "Smart view combination".into(),
                icon: "study".into(),
            },
            &cancel,
        )
        .unwrap();
    let collection = db
        .collections(
            &CollectionQuery {
                id: Some(id.clone()),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap()
        .entries
        .remove(0);
    let scoped = LibraryQuery {
        collection_id: Some(id),
        ..combined
    };
    assert_eq!(db.count(&scoped).unwrap(), 0);
    db.change_collection(
        &CollectionChange::SetMembership {
            expected: (&collection).into(),
            document: "d".into(),
            included: true,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(db.count(&scoped).unwrap(), 1);
    let strict = LibraryQuery {
        reading_state: LibraryReadingState::Completed,
        ..Default::default()
    };
    assert_eq!(db.query(&strict).unwrap()[0].id, "c");
    assert_eq!(db.count(&strict).unwrap(), 1);
    db.integrity().unwrap();
}
