use crate::{Database, database::storage};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use std::io::Write;
use yamantaka_domain::study_book::{BookStudy, Operation, Source};
use yamantaka_domain::{Cancellation, Failure, Result};

pub(crate) const KEY: &str = "rust.study.v1:";

fn valid(book: &BookStudy) -> bool {
    fn title(text: &str) -> bool {
        !text.trim().is_empty() && text.len() <= 300 && !text.contains(['\r', '\n'])
    }
    fn source(source: &Option<Source>) -> bool {
        source.as_ref().is_none_or(|source| {
            !source.annotation.is_empty()
                && source.annotation.len() <= 256
                && source.point.iter().all(|value| (0.0..=1.0).contains(value))
        })
    }
    book.revision < i64::MAX as u64
        && book.sections.len() <= 200
        && book.cards.len() <= 500
        && book
            .sections
            .iter()
            .all(|s| title(&s.title) && s.text.len() <= 16_384 && source(&s.source))
        && book.cards.iter().all(|c| {
            title(&c.question)
                && c.answer.len() <= 16_384
                && (!c.answer.trim().is_empty() || c.source.is_some())
                && source(&c.source)
                && c.interval <= 4
        })
}

fn load(db: &rusqlite::Connection, key: &str) -> Result<BookStudy> {
    let json: Option<String> = db
        .query_row("SELECT value FROM app_metadata WHERE key=?", [key], |row| {
            row.get(0)
        })
        .optional()
        .map_err(storage)?;
    json.map_or_else(|| Ok(BookStudy::default()), |json| decode(&json))
}

pub(crate) fn decode(json: &str) -> Result<BookStudy> {
    if json.len() > 1_000_000 {
        return Err(Failure::Invalid(
            "Study notes exceed the storage limit".into(),
        ));
    }
    let book: BookStudy = serde_json::from_str(json).map_err(storage)?;
    if !valid(&book) {
        return Err(Failure::Invalid("Invalid study notes or cards".into()));
    }
    Ok(book)
}

impl Database {
    pub(crate) fn access_study_book(
        &mut self,
        document: &str,
        operation: Operation,
    ) -> Result<BookStudy> {
        let exists = self
            .connection
            .query_row("SELECT 1 FROM documents WHERE id=?", [document], |_| Ok(()))
            .optional()
            .map_err(storage)?
            .is_some();
        if !exists {
            return Err(Failure::Invalid(
                "The source book is no longer in the library".into(),
            ));
        }
        let key = format!("{KEY}{document}");
        match operation {
            Operation::Load => load(&self.connection, &key),
            Operation::Save(mut book) => {
                let tx = self
                    .connection
                    .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                    .map_err(storage)?;
                if load(&tx, &key)?.revision != book.revision {
                    return Err(Failure::Invalid(
                        "Study notes changed. Close and reopen the book study panel.".into(),
                    ));
                }
                book.revision += 1;
                let json = serde_json::to_string(&book).map_err(storage)?;
                decode(&json)?;
                tx.execute(
                    "INSERT INTO app_metadata(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
                    params![key, json, Utc::now().to_rfc3339()],
                ).map_err(storage)?;
                tx.commit().map_err(storage)?;
                Ok(book)
            }
            Operation::Export(path) => {
                let book = load(&self.connection, &key)?;
                self.publish_local_export(&path, &Cancellation::default(), |file| {
                    for section in &book.sections {
                        writeln!(file, "## {}\n\n{}\n", section.title, section.text)
                            .map_err(storage)?;
                        if let Some(source) = &section.source {
                            writeln!(file, "> PDF · p. {}\n", u64::from(source.page) + 1)
                                .map_err(storage)?;
                        }
                    }
                    Ok(())
                })?;
                Ok(book)
            }
        }
    }
}
