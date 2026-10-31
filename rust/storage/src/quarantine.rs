use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, params, types::ValueRef};
use sha2::{Digest, Sha256};
use yamantaka_domain::{Cancellation, Document, Failure, LibraryRepository, Result, quarantine::*};

fn revision(db: &Connection, id: &str) -> Result<QuarantineRevision> {
    db.query_row("SELECT path,fingerprint,file_size,file_modified_at,page_count,restore_blocked,last_failure_code,is_missing FROM documents WHERE id=?", [id], |row| {
        let mut hash = Sha256::new();
        for index in 0..8 {
            match row.get_ref(index)? {
                ValueRef::Null => hash.update([0]),
                ValueRef::Integer(value) => { hash.update([1]); hash.update(value.to_le_bytes()); }
                ValueRef::Real(value) => { hash.update([2]); hash.update(value.to_bits().to_le_bytes()); }
                ValueRef::Text(bytes) => { hash.update([3]); hash.update((bytes.len() as u64).to_le_bytes()); hash.update(bytes); }
                ValueRef::Blob(bytes) => { hash.update([4]); hash.update((bytes.len() as u64).to_le_bytes()); hash.update(bytes); }
            }
        }
        Ok(QuarantineRevision { document: id.into(), state: hash.finalize().into() })
    }).optional().map_err(storage)?.ok_or_else(|| Failure::Invalid("Quarantined document no longer exists".into()))
}
fn read_page(
    database: &Database,
    tx: &Connection,
    offset: usize,
    cancel: &Cancellation,
) -> Result<QuarantinePage> {
    let total: i64 = tx
        .query_row(
            "SELECT count(*) FROM documents WHERE restore_blocked=1",
            [],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(80)
        .map_err(|_| Failure::MemoryBudget)?;
    {
        let mut statement = tx.prepare("SELECT id,last_failure_code='pdfOpenFailed' FROM documents WHERE restore_blocked=1 ORDER BY last_opened_at DESC,id LIMIT 80 OFFSET ?").map_err(storage)?;
        let mut rows = statement
            .query([i64::try_from(offset).map_err(storage)?])
            .map_err(storage)?;
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let id: String = row.get(0).map_err(storage)?;
            let native: Option<bool> = row.get(1).map_err(storage)?;
            let document = database
                .get(&id)?
                .ok_or_else(|| Failure::Invalid("Quarantined document no longer exists".into()))?;
            entries.push(QuarantinedDocument {
                document,
                reason: if native == Some(true) {
                    QuarantineReason::PdfOpenFailed
                } else {
                    QuarantineReason::InheritedOrUnknown
                },
                revision: revision(tx, &id)?,
            });
        }
    }
    cancel.check()?;
    Ok(QuarantinePage {
        entries,
        total: u64::try_from(total).map_err(storage)?,
        offset,
    })
}
impl QuarantineRepository for Database {
    fn quarantined_documents(
        &self,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<QuarantinePage> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let page = read_page(self, &tx, offset, cancel)?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }
    fn automatic_restore_allowed(&self, document: &str, cancel: &Cancellation) -> Result<bool> {
        cancel.check()?;
        let allowed = self
            .connection
            .query_row(
                "SELECT restore_blocked=0 AND is_missing=0 FROM documents WHERE id=?",
                [document],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?
            .unwrap_or(false);
        cancel.check()?;
        Ok(allowed)
    }
    fn record_reader_health(
        &mut self,
        document: &Document,
        health: ReaderHealth,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let (blocked, code) = match health {
            ReaderHealth::Opened => (false, None),
            ReaderHealth::Failed(QuarantineReason::PdfOpenFailed) => (true, Some("pdfOpenFailed")),
            ReaderHealth::Failed(QuarantineReason::InheritedOrUnknown) => {
                return Err(Failure::Invalid(
                    "Only classified reader failures may enter quarantine".into(),
                ));
            }
        };
        if !blocked && tx.query_row("SELECT EXISTS(SELECT 1 FROM documents WHERE id=? AND path=? AND fingerprint=? AND page_count=? AND restore_blocked=0 AND last_failure_code IS NULL)",params![document.id,document.path.to_string_lossy(),document.fingerprint,document.page_count],|row|row.get::<_,bool>(0)).map_err(storage)? {
            cancel.check()?;
            return tx.commit().map_err(storage);
        }
        if tx.execute("UPDATE documents SET restore_blocked=?,last_failure_code=? WHERE id=? AND path=? AND fingerprint=? AND page_count=?", params![blocked,code,document.id,document.path.to_string_lossy(),document.fingerprint,document.page_count]).map_err(storage)? != 1 {
            return Err(Failure::Invalid("The reader source changed; quarantine state was preserved".into()));
        }
        cancel.check()?;
        tx.commit().map_err(storage)
    }
    fn release_quarantine(
        &mut self,
        expected: &QuarantineRevision,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<QuarantinePage> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        if revision(&tx, &expected.document)?.state != expected.state {
            return Err(Failure::Invalid(
                "Quarantine changed after review; review it again".into(),
            ));
        }
        tx.execute("UPDATE documents SET restore_blocked=0,last_failure_code=NULL WHERE id=? AND restore_blocked=1", [&expected.document]).map_err(storage)?;
        cancel.check()?;
        let page = read_page(self, &tx, offset, cancel)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }
}
