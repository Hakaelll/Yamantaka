use crate::{Database, database::storage};
use rusqlite::TransactionBehavior;
use yamantaka_domain::{Cancellation, Failure, Result};

pub(crate) fn remove(database: &mut Database, id: &str, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if id.trim().is_empty() || id.len() > 4096 {
        return Err(Failure::Invalid("Choose a valid library document".into()));
    }
    let tx = database
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    // FTS5 has no document foreign key. Retire cached text even when search is off.
    tx.execute("DELETE FROM library_search WHERE document_id=?", [id])
        .map_err(storage)?;
    if tx
        .execute("DELETE FROM documents WHERE id=?", [id])
        .map_err(storage)?
        != 1
    {
        return Err(Failure::Invalid(
            "Document is no longer in the library".into(),
        ));
    }
    cancel.check()?;
    tx.commit().map_err(storage)
}
