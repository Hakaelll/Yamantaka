use super::*;
use yamantaka_domain::{
    LibraryRepository,
    study_library::{self, Library, Operation, Outcome},
};

fn load(db: &rusqlite::Connection) -> Result<Library> {
    let json: Option<String> = db
        .query_row(
            "SELECT value FROM app_metadata WHERE key=?",
            [study_library::KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage)?;
    json.map_or_else(
        || Ok(Library::default()),
        |json| study_library::decode(&json),
    )
}

impl Database {
    pub(crate) fn access_study_library(&mut self, operation: Operation) -> Result<Outcome> {
        match operation {
            Operation::Load => load(&self.connection).map(Outcome::Library),
            Operation::Save(mut library) => {
                let tx = self
                    .connection
                    .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                    .map_err(storage)?;
                if load(&tx)?.revision != library.revision {
                    return Err(Failure::Invalid(
                        "Study library has changed. Reload before saving.".into(),
                    ));
                }
                library.revision += 1;
                let json = serde_json::to_string(&library).map_err(storage)?;
                study_library::decode(&json)?;
                tx.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",
                    params![study_library::KEY, json, Utc::now().to_rfc3339()]).map_err(storage)?;
                tx.commit().map_err(storage)?;
                Ok(Outcome::Library(library))
            }
            Operation::Resolve(passages) => {
                let targets = passages
                    .into_iter()
                    .map(|passage| {
                        let doc = self
                            .get(&passage.document)?
                            .filter(|doc| {
                                !doc.missing
                                    && doc.fingerprint == passage.fingerprint
                                    && passage.last_page < doc.page_count
                            })
                            .ok_or_else(|| {
                                Failure::Invalid("A passage source moved or changed".into())
                            })?;
                        Ok((passage, doc))
                    })
                    .collect::<Result<_>>()?;
                Ok(Outcome::Resolved(targets))
            }
            Operation::Export(index, destination) => {
                let library = load(&self.connection)?;
                let notebook = library
                    .notebooks
                    .get(index)
                    .ok_or_else(|| Failure::Invalid("Notebook no longer exists".into()))?;
                self.publish_local_export(&destination, &Cancellation::default(), |file| {
                    writeln!(file, "# {}\n", notebook.title).map_err(storage)?;
                    for passage in &notebook.entries {
                        let first = u64::from(passage.page) + 1;
                        let last = u64::from(passage.last_page) + 1;
                        let pages = if first == last {
                            first.to_string()
                        } else {
                            format!("{first}–{last}")
                        };
                        let text = if passage.crop.is_some() && passage.text.trim().is_empty() {
                            "Linked PDF crop"
                        } else {
                            &passage.text
                        };
                        writeln!(
                            file,
                            "## {} · p. {pages}\n\n{}\n",
                            passage.title.replace(['\r', '\n'], " "),
                            text
                        )
                        .map_err(storage)?;
                    }
                    Ok(())
                })?;
                Ok(Outcome::Exported)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::study_library::{Notebook, Passage, PassageLink};

    #[test]
    fn notebooks_and_passage_links_survive_a_v9_backup_and_restore() {
        let folder = tempfile::tempdir().unwrap();
        let mut db = Database::in_memory_at(11).unwrap();
        let passage = Passage {
            document: "scan".into(),
            fingerprint: "revision".into(),
            title: "Test PDF".into(),
            page: 2,
            last_page: 2,
            point: [0.2, 0.4],
            rects: vec![],
            text: "A passage".into(),
            annotation: None,
            crop: None,
        };
        let library = Library {
            revision: 0,
            notebooks: vec![Notebook {
                title: "Subject".into(),
                entries: vec![passage.clone()],
            }],
            links: vec![PassageLink {
                passages: [passage.clone(), passage],
                note: "Similarity".into(),
            }],
        };
        let stored = db
            .access_study_library(Operation::Save(library.clone()))
            .unwrap();
        assert!(matches!(
            stored,
            Outcome::Library(Library { revision: 1, .. })
        ));
        assert!(db.access_study_library(Operation::Save(library)).is_err());
        let markdown = folder.path().join("notebook.md");
        db.access_study_library(Operation::Export(0, markdown.clone()))
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(markdown).unwrap(),
            "# Subject\n\n## Test PDF · p. 3\n\nA passage\n\n"
        );
        let backup = folder.path().join("backup.json");
        let cancel = Cancellation::default();
        db.export_backup(
            &backup,
            &crate::preferences::Preferences::default(),
            &cancel,
        )
        .unwrap();
        let preview = crate::backup::preview(&backup, &cancel).unwrap();
        let active = folder.path().join("restored");
        crate::backup::install_preview(preview, &active, &cancel).unwrap();
        let mut db = Database::open(&active.join("yamantaka.sqlite3")).unwrap();
        let Outcome::Library(restored) = db.access_study_library(Operation::Load).unwrap() else {
            panic!("Expected restored study library")
        };
        assert_eq!(restored.notebooks[0].entries[0].text, "A passage");
        assert_eq!(restored.links[0].note, "Similarity");
    }
}
