use super::*;
use yamantaka_domain::memory::ByteLease;
struct PendingUndo {
    data: Undo,
    _leases: Vec<ByteLease>,
}
pub(super) fn reading(db: &Connection, id: &str) -> Result<LegacyReadingState> {
    db.query_row("SELECT last_page_index,reading_progress,preferred_view_mode,preferred_theme,last_opened_at FROM documents WHERE id=?",[id],|row|Ok(LegacyReadingState {page:row.get(0)?,progress:row.get(1)?,mode:row.get(2)?,theme:row.get(3)?,opened:row.get(4)?})).map_err(storage)
}
fn document_budget(db: &Connection, id: &str, governor: &MemoryGovernor) -> Result<ByteLease> {
    let mut query = db
        .prepare("SELECT * FROM documents WHERE id=?")
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("Document no longer exists"))?;
    let mut bytes = id.len();
    for column in ["preferred_view_mode", "preferred_theme", "last_opened_at"] {
        bytes = bytes
            .checked_add(optional_text(row, column)?.map_or(0, str::len))
            .ok_or(Failure::MemoryBudget)?;
    }
    super::super::super::plan::reserve(
        governor,
        bytes
            .checked_mul(64)
            .and_then(|bytes| bytes.checked_add(65536))
            .ok_or(Failure::MemoryBudget)?,
    )
}
fn remove_queue(
    db: &Connection,
    document: &str,
    target: &str,
    before: &rows::WorkspaceState,
) -> Result<()> {
    let tab: Option<String> = db
        .query_row(
            "SELECT id FROM workspace_tabs WHERE workspace_id=? AND document_id=?",
            params![target, document],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage)?;
    let Some(tab) = tab else {
        return Ok(());
    };
    let split = if let Some(raw) = &before.split {
        let mut value: serde_json::Value = serde_json::from_str(raw).map_err(|_| {
            invalid("Workspace split data needs review before removing its queue entry")
        })?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| invalid("Workspace split data must be an object"))?;
        if object
            .get("secondaryTabId")
            .and_then(serde_json::Value::as_str)
            == Some(&tab)
        {
            object.insert("secondaryTabId".into(), serde_json::Value::Null);
            object.insert("enabled".into(), false.into());
            Some(serde_json::to_string(&value).map_err(storage)?)
        } else {
            Some(raw.clone())
        }
    } else {
        None
    };
    db.execute(
        "DELETE FROM workspace_tabs WHERE workspace_id=? AND document_id=?",
        params![target, document],
    )
    .map_err(storage)?;
    db.execute("UPDATE workspaces SET active_tab_id=CASE WHEN active_tab_id IS NOT NULL AND EXISTS(SELECT 1 FROM workspace_tabs WHERE workspace_id=?1 AND id=active_tab_id) THEN active_tab_id ELSE (SELECT id FROM workspace_tabs WHERE workspace_id=?1 ORDER BY sort_order,id LIMIT 1) END,split_json=?2 WHERE id=?1",params![target,split]).map_err(storage)?;
    touch_workspace(db, target)
}
fn apply_document(
    db: &Connection,
    plan: &LegacyRecoveryPlan,
    doc: &LegacyDocument,
    context: &ExecutionContext,
) -> Result<PendingUndo> {
    if record(db, "documents", &doc.id)? != doc.revision {
        return Err(invalid(
            "A reviewed document changed; prepare recovery again",
        ));
    }
    if doc.revision.is_none() {
        return Ok(PendingUndo {
            data: Undo::Targets {
                entries: Vec::new(),
            },
            _leases: Vec::new(),
        });
    }
    let mut leases = vec![document_budget(db, &doc.id, &context.governor)?];
    for change in &doc.changes {
        if let LegacyChange::RemoveMembership { target, revision } = change {
            let target = plan
                .targets
                .get(*target as usize)
                .ok_or_else(|| invalid("Invalid recovery destination"))?;
            if membership(db, target.kind, &doc.id, &target.id)? != *revision {
                return Err(invalid(
                    "A reviewed membership changed; prepare recovery again",
                ));
            }
        }
    }
    let old = reading(db, &doc.id)?;
    let favorite: bool = db
        .query_row(
            "SELECT is_favorite FROM documents WHERE id=?",
            [&doc.id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let mut undo = DocumentUndo {
        document: doc.id.clone(),
        identity: document_identity(db, &doc.id)?,
        favorite: None,
        reading: None,
        removed: Vec::new(),
        workspaces: Vec::new(),
    };
    for change in &doc.changes {
        context.cancel.check()?;
        match change {
            LegacyChange::RestoreFavorite { value, .. } => {
                db.execute(
                    "UPDATE documents SET is_favorite=? WHERE id=?",
                    params![value, doc.id],
                )
                .map_err(storage)?;
            }
            LegacyChange::RestoreProgress {
                page,
                page_count,
                mode,
                theme,
                ..
            } => {
                let progress = if *page_count == 0 {
                    0.0
                } else if *page_count == 1 {
                    1.0
                } else {
                    *page as f64 / (*page_count - 1) as f64
                };
                db.execute("UPDATE documents SET last_page_index=?,reading_progress=?,preferred_view_mode=?,preferred_theme=?,last_opened_at=? WHERE id=?",params![page,progress,mode.persisted_name(),theme.persisted_name(),stamp(""),doc.id]).map_err(storage)?;
            }
            LegacyChange::RemoveMembership { target, .. } => {
                let target = &plan.targets[*target as usize];
                if let Some((removed, lease)) =
                    rows::capture(db, target.kind, &doc.id, &target.id, &context.governor)?
                {
                    leases.push(lease);
                    match target.kind {
                        LegacyTargetKind::Tag => {
                            db.execute(
                                "DELETE FROM document_tags WHERE document_id=? AND tag_id=?",
                                params![doc.id, target.id],
                            )
                            .map_err(storage)?;
                        }
                        LegacyTargetKind::Collection => {
                            db.execute("DELETE FROM collection_documents WHERE document_id=? AND collection_id=?",params![doc.id,target.id]).map_err(storage)?;
                        }
                        LegacyTargetKind::Workspace => {
                            leases.push(rows::workspace_budget(db, &target.id, &context.governor)?);
                            let before = rows::workspace(db, &target.id, &context.cancel)?;
                            remove_queue(db, &doc.id, &target.id, &before)?;
                            let after = rows::workspace(db, &target.id, &context.cancel)?;
                            undo.workspaces.push(rows::WorkspaceUndo {
                                id: target.id.clone(),
                                before,
                                after,
                            });
                        }
                    }
                    undo.removed.push(removed);
                }
            }
        }
    }
    let new = reading(db, &doc.id)?;
    let new_favorite: bool = db
        .query_row(
            "SELECT is_favorite FROM documents WHERE id=?",
            [&doc.id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if old != new {
        undo.reading = Some((old, new));
    }
    if favorite != new_favorite {
        undo.favorite = Some((favorite, new_favorite));
    }
    crate::search::refresh_document_tags(db, &doc.id, context.access)?;
    Ok(PendingUndo {
        data: Undo::LegacyDocument(undo),
        _leases: leases,
    })
}
fn version(db: &Connection) -> Result<i64> {
    db.query_row("PRAGMA data_version", [], |row| row.get(0))
        .map_err(storage)
}
fn guards(db: &Connection, plan: &LegacyRecoveryPlan, expected: &[Option<[u8; 32]>]) -> Result<()> {
    for (target, expected) in plan.targets.iter().zip(expected) {
        if record(db, table(target.kind), &target.id)? != *expected {
            return Err(invalid("A recovery destination changed; review again"));
        }
    }
    Ok(())
}
pub(super) fn recover(
    db: &mut Database,
    plan: &LegacyRecoveryPlan,
    context: &ExecutionContext,
) -> Result<Arc<JournalEntry>> {
    context.cancel.check()?;
    journal::indexes(&db.connection)?;
    let mut size = plan
        .journal
        .revision
        .id
        .len()
        .checked_add(plan.journal.rule_name.len())
        .ok_or(Failure::MemoryBudget)?;
    for value in [
        Some(plan.before.status.as_str()),
        Some(plan.before.phase.as_str()),
        Some(plan.before.capability.as_str()),
        plan.before.error.as_deref(),
        plan.before.interruption.as_deref(),
    ] {
        size = size
            .checked_add(value.map_or(0, str::len))
            .ok_or(Failure::MemoryBudget)?;
    }
    let _lease = super::super::super::plan::reserve(
        &context.governor,
        size.checked_mul(16)
            .and_then(|size| size.checked_add(65536))
            .ok_or(Failure::MemoryBudget)?,
    )?;
    let mut expected = plan
        .targets
        .iter()
        .map(|target| target.revision)
        .collect::<Vec<_>>();
    let (root, mut observed) = {
        let tx = db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        available(&tx, &plan.journal.revision)?;
        guards(&tx, plan, &expected)?;
        let root = uuid::Uuid::new_v4().to_string();
        let now = stamp("");
        let summary = serde_json::json!({"version":1,"ruleName":plan.journal.rule_name,"legacyJournalId":plan.journal.revision.id});
        tx.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES(?,?,'running',?,?,0,'{\"version\":1}',?,?,'recovering_legacy','rollback',?)",params![root,journal::ROOT,summary.to_string(),plan.documents.len() as i64,now,now,FORMAT]).map_err(storage)?;
        journal::child(
            &tx,
            &root,
            0,
            &Undo::LegacyJournal(JournalUndo {
                id: plan.journal.revision.id.clone(),
                before: plan.before.clone(),
                after: None,
            }),
        )?;
        let observed = version(&tx)?;
        context.cancel.check()?;
        tx.commit().map_err(storage)?;
        (root, observed)
    };
    let start = Instant::now();
    let work = (|| -> Result<()> {
        for (index, document) in plan.documents.iter().enumerate() {
            context.cancel.check()?;
            if start.elapsed() >= Duration::from_secs(30) {
                return Err(invalid("Inherited recovery time limit reached"));
            }
            let tx = db
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage)?;
            let current = version(&tx)?;
            if current != observed {
                journal::verify_kind(&tx, &plan.journal.revision, journal::LEGACY)?;
                observed = current;
            }
            guards(&tx, plan, &expected)?;
            let receipt = apply_document(&tx, plan, document, context)?;
            journal::child(&tx, &root, index + 1, &receipt.data)?;
            journal::update(
                &tx,
                &root,
                "running",
                "recovering_legacy",
                Some(index + 1),
                None,
                true,
            )?;
            for (target, expected) in plan.targets.iter().zip(&mut expected) {
                *expected = record(&tx, table(target.kind), &target.id)?;
            }
            context.cancel.check()?;
            tx.commit().map_err(storage)?;
            drop(receipt);
            context.watch.publish((index + 1) as u64);
        }
        context.cancel.check()?;
        let tx = db
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        journal::verify_kind(&tx, &plan.journal.revision, journal::LEGACY)?;
        tx.execute("UPDATE operation_journal SET status='rolledBack',phase='legacy_receipts_recovered',resume_capability='none',error_code=NULL,interruption_reason=NULL,updated_at=? WHERE id=? AND operation_kind=?",params![stamp(&plan.journal.revision.updated_at),plan.journal.revision.id,journal::LEGACY]).map_err(storage)?;
        let undo = Undo::LegacyJournal(JournalUndo {
            id: plan.journal.revision.id.clone(),
            before: plan.before.clone(),
            after: Some(super::super::super::plan::record(
                &tx,
                "operation_journal",
                &plan.journal.revision.id,
            )?),
        });
        tx.execute(
            "UPDATE operation_journal SET undo_json=? WHERE id=? AND operation_kind=?",
            params![
                serde_json::to_string(&undo).map_err(storage)?,
                format!("{root}:00000000"),
                journal::ITEM
            ],
        )
        .map_err(storage)?;
        journal::update(&tx, &root, "completed", "completed", None, None, true)?;
        context.cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(())
    })();
    if let Err(error) = work {
        let cancelled = matches!(error, Failure::Cancelled);
        let rollback = ExecutionContext {
            governor: context.governor.clone(),
            cancel: Cancellation::default(),
            watch: context.watch.clone(),
            access: context.access,
        };
        super::super::undo::rollback(
            &mut db.connection,
            &root,
            &rollback,
            if cancelled { "cancelled" } else { "failed" },
            Some(if cancelled {
                "user_cancelled"
            } else {
                "legacy_recovery_failed"
            }),
        )?;
    }
    journal::entry(&db.connection, &root, &context.governor)
}
