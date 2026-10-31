use crate::{Database, database::storage, relations::exists};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use yamantaka_domain::{Cancellation, Failure, MAX_LIBRARY_PAGE, Result, revisions::*};
fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
fn offset(value: usize) -> Result<i64> {
    i64::try_from(value).map_err(|_| invalid("Revision offset is too large"))
}
fn next_revision(previous: &str) -> String {
    let mut now = chrono::Utc::now();
    if let Ok(previous) = chrono::DateTime::parse_from_rfc3339(previous)
        && now <= previous
    {
        now = previous.with_timezone(&chrono::Utc) + chrono::Duration::nanoseconds(1);
    }
    now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
impl Database {
    pub(crate) fn revision_page(
        &self,
        query: &RevisionQuery,
        cancel: &Cancellation,
    ) -> Result<RevisionPage> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        exists(&tx, &query.document)?;
        let total = tx
            .query_row(
                "SELECT count(*) FROM document_revision_members WHERE document_id=?",
                [&query.document],
                |row| row.get::<_, i64>(0),
            )
            .map_err(storage)?;
        let entries = {
            let mut statement = tx.prepare("SELECT g.id,g.name,g.current_document_id,COALESCE(d.title_override,d.title,d.source_title,d.file_name),g.created_at,g.updated_at,(SELECT count(*) FROM document_revision_members m WHERE m.group_id=g.id) FROM document_revision_groups g JOIN document_revision_members own ON own.group_id=g.id LEFT JOIN document_revision_members cm ON cm.group_id=g.id AND cm.document_id=g.current_document_id LEFT JOIN documents d ON d.id=cm.document_id WHERE own.document_id=? ORDER BY g.updated_at DESC,g.id LIMIT ? OFFSET ?").map_err(storage)?;
            let rows = statement
                .query_map(
                    params![
                        query.document,
                        MAX_LIBRARY_PAGE as i64,
                        offset(query.offset)?
                    ],
                    |row| {
                        Ok(RevisionGroup {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            current: row.get(2)?,
                            current_title: row.get(3)?,
                            created_at: row.get(4)?,
                            updated_at: row.get(5)?,
                            member_count: row.get::<_, i64>(6)?.max(0) as u64,
                        })
                    },
                )
                .map_err(storage)?;
            let mut entries = Vec::new();
            for row in rows {
                cancel.check()?;
                entries.push(row.map_err(storage)?);
            }
            entries
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(RevisionPage {
            entries,
            total: total.max(0) as u64,
        })
    }
    pub(crate) fn revision_member_page(
        &self,
        query: &RevisionMemberQuery,
        cancel: &Cancellation,
    ) -> Result<RevisionMemberPage> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let (current, updated_at): (Option<String>, String) = tx
            .query_row(
                "SELECT current_document_id,updated_at FROM document_revision_groups WHERE id=?",
                [&query.group],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage)?
            .ok_or_else(|| invalid("Revision group is no longer available"))?;
        let total = tx
            .query_row(
                "SELECT count(*) FROM document_revision_members WHERE group_id=?",
                [&query.group],
                |row| row.get::<_, i64>(0),
            )
            .map_err(storage)?;
        let entries = {
            let mut statement = tx.prepare("SELECT d.id,COALESCE(d.title_override,d.title,d.source_title,d.file_name),d.is_missing FROM document_revision_members m JOIN documents d ON d.id=m.document_id WHERE m.group_id=? ORDER BY m.added_at,m.document_id LIMIT ? OFFSET ?").map_err(storage)?;
            let rows = statement
                .query_map(
                    params![query.group, MAX_LIBRARY_PAGE as i64, offset(query.offset)?],
                    |row| {
                        Ok(RevisionMember {
                            document: row.get(0)?,
                            title: row.get(1)?,
                            missing: row.get(2)?,
                        })
                    },
                )
                .map_err(storage)?;
            let mut entries = Vec::new();
            for row in rows {
                cancel.check()?;
                entries.push(row.map_err(storage)?);
            }
            entries
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(RevisionMemberPage {
            group: query.group.clone(),
            current,
            updated_at,
            entries,
            total: total.max(0) as u64,
        })
    }
    pub(crate) fn apply_revision(
        &mut self,
        change: &RevisionChange,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = match change {
            RevisionChange::Create {
                name,
                members,
                current,
            } => {
                let name = name.trim();
                if name.is_empty() || name.chars().count() > 120 {
                    return Err(invalid(
                        "Use a revision group name between 1 and 120 characters",
                    ));
                }
                let mut unique = Vec::new();
                for member in members {
                    if !unique.contains(member) {
                        unique.push(member.clone());
                    }
                }
                if !(2..=80).contains(&unique.len()) {
                    return Err(invalid("A revision group needs between 2 and 80 documents"));
                }
                if current.as_ref().is_some_and(|id| !unique.contains(id)) {
                    return Err(invalid("The current revision must belong to the group"));
                }
                for member in &unique {
                    cancel.check()?;
                    exists(&tx, member)?;
                }
                let id = uuid::Uuid::new_v4().to_string();
                let now = next_revision("");
                tx.execute("INSERT INTO document_revision_groups(id,name,current_document_id,created_at,updated_at) VALUES(?,?,?,?,?)",params![id,name,current,now,now]).map_err(storage)?;
                for member in unique {
                    cancel.check()?;
                    tx.execute("INSERT INTO document_revision_members(group_id,document_id,added_at) VALUES(?,?,?)",params![id,member,now]).map_err(storage)?;
                }
                id
            }
            RevisionChange::SetCurrent { expected, document } => {
                let revision = tx.query_row("SELECT updated_at,current_document_id FROM document_revision_groups WHERE id=?",[&expected.id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,Option<String>>(1)?))).optional().map_err(storage)?;
                if revision.as_ref().is_none_or(|(updated, current)| {
                    updated != &expected.updated_at || current != &expected.current
                }) {
                    return Err(invalid(
                        "Revision group changed; reload before choosing its current edition",
                    ));
                }
                if !tx.query_row("SELECT EXISTS(SELECT 1 FROM document_revision_members WHERE group_id=? AND document_id=?)",params![expected.id,document],|row|row.get::<_,bool>(0)).map_err(storage)? { return Err(invalid("The current revision must belong to the group")); }
                tx.execute("UPDATE document_revision_groups SET current_document_id=?,updated_at=? WHERE id=?",params![document,next_revision(&expected.updated_at),expected.id]).map_err(storage)?;
                expected.id.clone()
            }
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}
