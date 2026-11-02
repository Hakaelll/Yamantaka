use crate::{Database, database::storage, preferences::Preferences};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params, types::ValueRef};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    duplicates::merge::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};
mod rows;
mod write;
fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
struct MergeStore {
    database: Database,
    preferences: Preferences,
    root: PathBuf,
}
pub(crate) fn factory(database: &Database, preferences: serde_json::Value) -> Result<Factory> {
    let path = database
        .file_path()
        .ok_or_else(|| invalid("Consolidation requires a file-backed library"))?;
    let preferences: Preferences = serde_json::from_value(preferences).map_err(storage)?;
    if preferences.version != 1 {
        return Err(invalid("Unsupported consolidation preferences"));
    }
    let root = path
        .parent()
        .ok_or_else(|| invalid("Library directory unavailable"))?
        .to_owned();
    Ok(Box::new(move || {
        Ok(Box::new(MergeStore {
            database: Database::open(&path)?,
            preferences,
            root,
        }) as Box<dyn Store>)
    }))
}
fn sources(db: &Connection, selection: &Selection) -> Result<()> {
    selection.validate()?;
    for source in [&selection.kept, &selection.incoming] {
        if crate::file_check::read_file(db, &source.document)? != source.file {
            return Err(invalid("Duplicate source changed; review again"));
        }
    }
    Ok(())
}
fn fields(db: &Connection, id: &str) -> Result<[Option<String>; 3]> {
    let mut statement=db.prepare("SELECT COALESCE(title_override,title,source_title,file_name),COALESCE(author_override,author,source_author),(SELECT note FROM document_notes WHERE document_id=documents.id) FROM documents WHERE id=?").map_err(storage)?;
    let mut rows = statement.query([id]).map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("Duplicate record disappeared"))?;
    let mut fields = [None, None, None];
    for (index, field) in fields.iter_mut().enumerate() {
        if let ValueRef::Text(bytes) = row.get_ref(index).map_err(storage)? {
            if bytes.len() > 262144 {
                return Err(invalid("Duplicate metadata exceeds supported bounds"));
            }
            let text = std::str::from_utf8(bytes).map_err(storage)?.trim();
            if !text.is_empty() {
                *field = Some(text.to_owned());
            }
        }
    }
    Ok(fields)
}
impl Store for MergeStore {
    fn prepare(
        &self,
        selection: Selection,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Arc<Plan>> {
        cancel.check()?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                2 * 1024 * 1024,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let tx = self
            .database
            .connection
            .unchecked_transaction()
            .map_err(storage)?;
        sources(&tx, &selection)?;
        let (revision, affected) = rows::revision(&tx, &selection, cancel)?;
        let left = fields(&tx, &selection.kept.document)?;
        let right = fields(&tx, &selection.incoming.document)?;
        let [lt, la, ln] = left;
        let [rt, ra, rn] = right;
        let fields = [
            Field {
                kept: lt,
                incoming: rt,
            },
            Field {
                kept: la,
                incoming: ra,
            },
            Field {
                kept: ln,
                incoming: rn,
            },
        ];
        cancel.check()?;
        Ok(Arc::new(Plan::new(
            selection, revision, fields, affected, lease,
        )))
    }
    fn apply(
        &mut self,
        plan: &Plan,
        choices: &Choices,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
        verify: &mut VerifySources<'_>,
    ) -> Result<Receipt> {
        cancel.check()?;
        let _lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                256 * 1024,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let guard = verify(&plan.selection)?;
        guard.verify()?;
        {
            let tx = self
                .database
                .connection
                .unchecked_transaction()
                .map_err(storage)?;
            sources(&tx, &plan.selection)?;
            if rows::revision(&tx, &plan.selection, cancel)?.0 != plan.revision {
                return Err(invalid("Duplicate data changed; review again"));
            }
        }
        crate::managed_imports::ordinary(&self.root, true)?;
        let backup_root = self.root.join("duplicate-backups");
        if !crate::managed_imports::present(&backup_root)? {
            std::fs::create_dir(&backup_root).map_err(storage)?;
        }
        crate::managed_imports::ordinary(&backup_root, true)?;
        let history = uuid::Uuid::new_v4().to_string();
        let backup = backup_root.join(format!("pre-merge-{history}.json"));
        if crate::managed_imports::present(&backup)? {
            return Err(invalid("Consolidation backup destination is occupied"));
        }
        self.database
            .export_backup(&backup, &self.preferences, cancel)?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(1);
        }
        let backup_guard = options.open(&backup).map_err(storage)?;
        let backup_sha256 = crate::identity::full_sha256(&backup, cancel)?;
        guard.verify()?;
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        sources(&tx, &plan.selection)?;
        if rows::revision(&tx, &plan.selection, cancel)?.0 != plan.revision {
            return Err(invalid("Duplicate data changed after backup; review again"));
        }
        let kept = &plan.selection.kept.document;
        let incoming = &plan.selection.incoming.document;
        let now = chrono::Utc::now().to_rfc3339();
        write::merge(&tx, plan, choices, &now, governor, cancel)?;
        let backup_name = backup
            .to_str()
            .ok_or_else(|| invalid("Backup path is not valid Unicode"))?;
        tx.execute("INSERT INTO duplicate_merge_history(id,kept_document_id,incoming_document_id,backup_path,status,created_at,updated_at) VALUES(?1,?2,?3,?4,'completed',?5,?5)",params![history,kept,incoming,backup_name,now]).map_err(storage)?;
        let digest: sha2::digest::Output<Sha256> = plan.selection.kept_proof.digest.into();
        let summary = serde_json::json!({"version":1,"keptDocument":kept,"incomingDocument":incoming,"sourceSha256":format!("{digest:x}"),"backupSha256":backup_sha256});
        let undo = serde_json::json!({"version":1,"backupPath":backup_name,"sha256":backup_sha256});
        tx.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES(?1,'rustDuplicateMerge.v1','completed',?2,2,2,?3,?4,?4,'completed','rollback','rust-duplicate-merge-v1')",params![history,summary.to_string(),undo.to_string(),now]).map_err(storage)?;
        crate::database::validate(&tx)?;
        guard.verify()?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        drop(backup_guard);
        Ok(Receipt {
            history,
            kept: kept.clone(),
            incoming: incoming.clone(),
            backup,
            backup_sha256,
        })
    }
}
