use yamantaka_domain::{
    LibraryQuery, LibraryReadingState, LibraryRepository, LibrarySort, SortDirection,
};
use yamantaka_storage::Database;

fn catalog() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch(
        r#"
        INSERT INTO documents(
            id,path,normalized_path,file_name,title,author,source_title,source_author,
            title_override,author_override,page_count,file_size,file_modified_at,
            added_at,last_opened_at,fingerprint,reading_progress,is_favorite,is_missing
        ) VALUES
        ('a','a.pdf','a.pdf','a.pdf','Zulu','Raw','Source','Source author',
            'Alpha','Ada',20,1,'2026-01-01','2026-01-01','2026-01-06','a',0,1,0),
        ('b','b.pdf','b.pdf','b.pdf','alpha','Ada',NULL,NULL,
            NULL,NULL,20,1,'2026-01-01','2026-01-02','2026-01-05','b',0.5,1,0),
        ('c','c.pdf','c.pdf','c.pdf','Beta','Bob',NULL,NULL,
            NULL,NULL,5,1,'2026-01-01','2026-01-03','2026-01-04','c',0.995,0,1),
        ('d','d.pdf','d.pdf','d.pdf',NULL,NULL,'Gamma','Cara',
            NULL,NULL,30,1,'2026-01-01','2026-01-04','2026-01-03','d',1,1,0),
        ('e','e.pdf','e.pdf','e.pdf','Gamma','Bob',NULL,NULL,
            NULL,NULL,5,1,'2026-01-01','2026-01-05','2026-01-02','e',0,0,0),
        ('f','f.pdf','f.pdf','f.pdf','Beta','Bob',NULL,NULL,
            NULL,NULL,30,1,'2026-01-01','2026-01-06','2026-01-01','f',1,1,1);
        INSERT INTO document_topics VALUES
            ('a','science',1,'manual',1,'a','2026-01-01'),
            ('a','technology',1,'manual',1,'a','2026-01-01'),
            ('b','science',1,'manual',1,'b','2026-01-01'),
            ('c','technology',1,'manual',1,'c','2026-01-01'),
            ('d','religionMythology',1,'manual',1,'d','2026-01-01');
        INSERT INTO tags VALUES
            ('tag1','Tag one','tag one','red','2026-01-01','2026-01-01'),
            ('tag2','Tag two','tag two','blue','2026-01-01','2026-01-01');
        INSERT INTO document_tags VALUES ('a','tag1'),('a','tag2'),('b','tag2'),('c','tag1');
        INSERT INTO collections(id,name,normalized_name,created_at,updated_at) VALUES
            ('col1','One','one','2026-01-01','2026-01-01'),
            ('col2','Two','two','2026-01-01','2026-01-01');
        INSERT INTO collection_documents VALUES
            ('col1','a','2026-01-01'),('col1','b','2026-01-01'),
            ('col1','d','2026-01-01'),('col2','a','2026-01-01');
        INSERT INTO document_notes VALUES ('e','Review needle','2026-01-01');
        "#,
    )
    .unwrap();
    db
}

fn ids(db: &Database, query: &LibraryQuery) -> Vec<String> {
    db.query(query).unwrap().into_iter().map(|d| d.id).collect()
}

fn assert_matches(db: &Database, query: &LibraryQuery, expected: &[&str]) {
    assert_eq!(ids(db, query), expected);
    assert_eq!(db.count(query).unwrap(), expected.len() as u64);
}

#[test]
fn default_preserves_recent_order_and_includes_missing() {
    let db = catalog();
    assert_matches(
        &db,
        &LibraryQuery::default(),
        &["a", "b", "c", "d", "e", "f"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            include_missing: false,
            ..Default::default()
        },
        &["a", "b", "d", "e"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            missing_only: true,
            ..Default::default()
        },
        &["c", "f"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            missing_only: true,
            include_missing: false,
            ..Default::default()
        },
        &[],
    );
}

#[test]
fn reading_state_uses_zero_and_one_not_smart_collection_thresholds() {
    let db = catalog();
    for (state, expected) in [
        (LibraryReadingState::NotStarted, vec!["a", "e"]),
        (LibraryReadingState::InProgress, vec!["b", "c"]),
        (LibraryReadingState::Completed, vec!["d", "f"]),
    ] {
        assert_matches(
            &db,
            &LibraryQuery {
                reading_state: state,
                ..Default::default()
            },
            &expected,
        );
    }
}

#[test]
fn effective_author_normalization_and_source_fallback_match_projection() {
    let db = catalog();
    assert_matches(
        &db,
        &LibraryQuery {
            authors: vec![" ADA ".into(), "Bob".into(), " ".into()],
            ..Default::default()
        },
        &["a", "b", "c", "e", "f"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            authors: vec!["Cara".into()],
            ..Default::default()
        },
        &["d"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            authors: vec!["Raw".into()],
            ..Default::default()
        },
        &[],
    );
    assert_eq!(db.get("a").unwrap().unwrap().author.as_deref(), Some("Ada"));
    assert_eq!(
        db.get("d").unwrap().unwrap().author.as_deref(),
        Some("Cara")
    );
}

#[test]
fn memberships_match_any_selected_value_without_duplicate_rows() {
    let db = catalog();
    assert_matches(
        &db,
        &LibraryQuery {
            topics: vec!["science".into(), "technology".into()],
            ..Default::default()
        },
        &["a", "b", "c"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            tag_ids: vec!["tag1".into(), "tag2".into()],
            ..Default::default()
        },
        &["a", "b", "c"],
    );
    assert_matches(
        &db,
        &LibraryQuery {
            collection_id: Some(" col1 ".into()),
            ..Default::default()
        },
        &["a", "b", "d"],
    );
    assert_eq!(
        db.count(&LibraryQuery {
            collection_id: Some(" ".into()),
            ..Default::default()
        })
        .unwrap(),
        6
    );
}

#[test]
fn all_filters_combine_and_count_ignores_only_pagination_and_order() {
    let db = catalog();
    let mut query = LibraryQuery {
        text: "ALPHA".into(),
        favorites_only: true,
        include_missing: false,
        reading_state: LibraryReadingState::InProgress,
        authors: vec!["Ada".into(), "Other".into()],
        topics: vec!["science".into()],
        tag_ids: vec!["tag2".into()],
        collection_id: Some("col1".into()),
        sort: LibrarySort::PageCount,
        sort_direction: SortDirection::Ascending,
        page_size: 1,
        ..Default::default()
    };
    assert_matches(&db, &query, &["b"]);
    query.offset = 1;
    assert!(ids(&db, &query).is_empty());
    assert_eq!(db.count(&query).unwrap(), 1);
    query.collection_id = Some("col2".into());
    assert_eq!(db.count(&query).unwrap(), 0);
}

#[test]
fn text_search_covers_effective_metadata_filename_note_and_topic_key() {
    let db = catalog();
    for (text, expected) in [
        ("  ALPHA  ", vec!["a", "b"]),
        ("Zulu", vec![]),
        ("Raw", vec![]),
        ("Cara", vec!["d"]),
        ("Gamma", vec!["d", "e"]),
        ("a.pdf", vec!["a"]),
        ("needle", vec!["e"]),
        ("religionMythology", vec!["d"]),
        ("  ", vec!["a", "b", "c", "d", "e", "f"]),
    ] {
        assert_matches(
            &db,
            &LibraryQuery {
                text: text.into(),
                ..Default::default()
            },
            &expected,
        );
    }
    assert_eq!(db.get("a").unwrap().unwrap().display_title(), "Alpha");
    assert_eq!(db.get("d").unwrap().unwrap().display_title(), "Gamma");
}

#[test]
fn like_metacharacters_are_literal_and_filter_values_cannot_inject_sql() {
    let db = catalog();
    db.execute_batch(r#"UPDATE documents SET title_override='Literal %_\ quote''s' WHERE id='a'"#)
        .unwrap();
    for text in ["%", "_", "\\", "quote's", "%_\\"] {
        assert_matches(
            &db,
            &LibraryQuery {
                text: text.into(),
                ..Default::default()
            },
            &["a"],
        );
    }
    let hostile = "' OR 1=1 --";
    for query in [
        LibraryQuery {
            text: hostile.into(),
            ..Default::default()
        },
        LibraryQuery {
            authors: vec![hostile.into()],
            ..Default::default()
        },
        LibraryQuery {
            topics: vec![hostile.into()],
            ..Default::default()
        },
        LibraryQuery {
            tag_ids: vec![hostile.into()],
            ..Default::default()
        },
        LibraryQuery {
            collection_id: Some(hostile.into()),
            ..Default::default()
        },
    ] {
        assert_matches(&db, &query, &[]);
    }
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 6);
}

#[test]
fn all_sort_modes_and_directions_have_stable_id_ties() {
    let db = catalog();
    for (sort, ascending, descending, default_direction) in [
        (
            LibrarySort::LastOpened,
            vec!["f", "e", "d", "c", "b", "a"],
            vec!["a", "b", "c", "d", "e", "f"],
            SortDirection::Descending,
        ),
        (
            LibrarySort::Title,
            vec!["a", "b", "c", "f", "d", "e"],
            vec!["d", "e", "c", "f", "a", "b"],
            SortDirection::Ascending,
        ),
        (
            LibrarySort::Author,
            vec!["a", "b", "c", "e", "f", "d"],
            vec!["d", "c", "e", "f", "a", "b"],
            SortDirection::Ascending,
        ),
        (
            LibrarySort::Added,
            vec!["a", "b", "c", "d", "e", "f"],
            vec!["f", "e", "d", "c", "b", "a"],
            SortDirection::Descending,
        ),
        (
            LibrarySort::Progress,
            vec!["a", "e", "b", "c", "d", "f"],
            vec!["d", "f", "c", "b", "a", "e"],
            SortDirection::Descending,
        ),
        (
            LibrarySort::PageCount,
            vec!["c", "e", "a", "b", "d", "f"],
            vec!["d", "f", "a", "b", "c", "e"],
            SortDirection::Descending,
        ),
    ] {
        for direction in [
            SortDirection::Default,
            SortDirection::Ascending,
            SortDirection::Descending,
        ] {
            let effective = if direction == SortDirection::Default {
                default_direction
            } else {
                direction
            };
            let expected = if effective == SortDirection::Ascending {
                &ascending
            } else {
                &descending
            };
            let query = LibraryQuery {
                sort,
                sort_direction: direction,
                ..Default::default()
            };
            assert_matches(&db, &query, expected);
            // Ties straddle page boundaries; concatenating windows must retain every ID once.
            let paged: Vec<_> = (0..3)
                .flat_map(|page| {
                    ids(
                        &db,
                        &LibraryQuery {
                            offset: page * 2,
                            page_size: 2,
                            ..query.clone()
                        },
                    )
                })
                .collect();
            assert_eq!(&paged, expected);
        }
    }
    // A date tie keeps id ASC even when the primary direction changes.
    db.execute_batch("UPDATE documents SET last_opened_at='2026-01-01',added_at='2026-01-01'")
        .unwrap();
    for sort in [LibrarySort::LastOpened, LibrarySort::Added] {
        for sort_direction in [SortDirection::Ascending, SortDirection::Descending] {
            assert_matches(
                &db,
                &LibraryQuery {
                    sort,
                    sort_direction,
                    ..Default::default()
                },
                &["a", "b", "c", "d", "e", "f"],
            );
        }
    }
}

#[test]
fn page_size_is_capped_at_eighty_and_large_offsets_do_not_wrap() {
    let db = Database::in_memory_at(11).unwrap();
    for index in 0..165 {
        db.execute_batch(&format!(
            "INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('{index:03}','{index}.pdf','{index}.pdf','{index}.pdf',1,1,'2026-01-01','2026-01-01','2026-01-01','{index}')"
        )).unwrap();
    }
    for page_size in [0, 80, 81, usize::MAX] {
        let query = LibraryQuery {
            page_size,
            ..Default::default()
        };
        assert_eq!(ids(&db, &query).len(), 80);
        assert_eq!(db.count(&query).unwrap(), 165);
    }
    let second = LibraryQuery {
        offset: 80,
        page_size: 1000,
        ..Default::default()
    };
    assert_eq!(ids(&db, &second).first().map(String::as_str), Some("080"));
    assert_eq!(ids(&db, &second).len(), 80);
    let end = LibraryQuery {
        offset: 160,
        ..Default::default()
    };
    assert_eq!(ids(&db, &end).len(), 5);
    let beyond = LibraryQuery {
        offset: usize::MAX,
        ..Default::default()
    };
    assert!(ids(&db, &beyond).is_empty());
    assert_eq!(db.count(&beyond).unwrap(), 165);
}
