use crate::{Database, database::storage};
use rusqlite::{OptionalExtension, params};
use yamantaka_domain::{Cancellation, Failure, Result, reading_sessions::*};
const CONTEXTS: &str = "reading_session_contexts_v1";
mod context_pruning;
mod goals;
mod history;

fn timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|v| v.with_timezone(&chrono::Utc))
        .map_err(|_| Failure::Invalid("Reading session timestamp is invalid".into()))
}

impl ReadingSessionRepository for Database {
    fn reading_goal(
        &self,
        document: &str,
        cancel: &Cancellation,
    ) -> Result<Option<yamantaka_domain::reading_sessions::goals::ReadingGoal>> {
        goals::load(self, document, cancel)
    }
    fn save_reading_goal(
        &mut self,
        document: &str,
        goal: Option<yamantaka_domain::reading_sessions::goals::ReadingGoal>,
        cancel: &Cancellation,
    ) -> Result<Option<yamantaka_domain::reading_sessions::goals::ReadingGoal>> {
        goals::save(self, document, goal, cancel)
    }
    fn delete_reading_session(
        &mut self,
        id: &str,
        document: &str,
        cancel: &Cancellation,
    ) -> Result<bool> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM reading_sessions WHERE id=?1 AND document_id=?2 AND ended_at IS NOT NULL)",params![id,document],|r|r.get(0)).map_err(storage)?;
        if !exists {
            cancel.check()?;
            return Ok(false);
        }
        context_pruning::prune(
            &tx,
            "SELECT id FROM reading_sessions WHERE id=?1 AND ended_at IS NOT NULL",
            id,
        )?;
        let deleted = tx.execute("DELETE FROM reading_sessions WHERE id=?1 AND document_id=?2 AND ended_at IS NOT NULL",params![id,document]).map_err(storage)?;
        if deleted != 1 {
            return Err(Failure::Invalid(
                "Reading session deletion was not applied; original data is preserved".into(),
            ));
        }
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(true)
    }

    fn reading_history(
        &self,
        query: &history::ReadingHistoryQuery,
        governor: &yamantaka_domain::memory::MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<history::ReadingHistoryPage> {
        history::read(self, query, governor, cancel)
    }
    fn trim_reading_history(&mut self, limit: usize, cancel: &Cancellation) -> Result<u64> {
        cancel.check()?;
        let limit = limit.clamp(25, 1000) as i64;
        let tx = self.connection.transaction().map_err(storage)?;
        let removed = "SELECT id FROM reading_sessions WHERE ended_at IS NOT NULL ORDER BY started_at DESC,id DESC LIMIT -1 OFFSET ?1";
        let count: i64 = tx
            .query_row(&format!("SELECT count(*) FROM ({removed})"), [limit], |r| {
                r.get(0)
            })
            .map_err(storage)?;
        if count == 0 {
            cancel.check()?;
            return Ok(0);
        }
        context_pruning::prune(&tx, removed, limit)?;
        let deleted = tx
            .execute(
                &format!("DELETE FROM reading_sessions WHERE id IN ({removed})"),
                [limit],
            )
            .map_err(storage)?;
        if i64::try_from(deleted).ok() != Some(count) {
            return Err(Failure::Invalid(
                "Reading history pruning was not applied; original data is preserved".into(),
            ));
        }
        cancel.check()?;
        tx.commit().map_err(storage)?;
        u64::try_from(count)
            .map_err(|_| Failure::Invalid("Reading history count is invalid".into()))
    }
    fn start_reading_session(
        &mut self,
        start: &ReadingSessionStart<'_>,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        validate_session_identity(start.id, start.document)?;
        let date =
            timestamp(start.started_at)?.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        let tx = self.connection.transaction().map_err(storage)?;
        tx.execute("INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES(?1,?2,?3,?4,?4,0) ON CONFLICT(id) DO NOTHING",params![start.id,start.document,date,start.page]).map_err(storage)?;
        let stored: (String, String, u32) = tx
            .query_row(
                "SELECT document_id,started_at,start_page FROM reading_sessions WHERE id=?",
                [start.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(storage)?;
        if stored != (start.document.to_owned(), date, start.page) {
            return Err(Failure::Invalid(
                "Reading session identity is already occupied".into(),
            ));
        }
        cancel.check()?;
        tx.commit().map_err(storage)
    }
    fn finish_reading_session(
        &mut self,
        finish: &ReadingSessionFinish<'_>,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        validate_session_identity(finish.id, finish.document)?;
        let ended = timestamp(finish.ended_at)?;
        let date = ended.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        let seconds = finish.active_seconds.min(86400) as i64;
        let context = finish
            .context
            .map(|context| {
                context.validate()?;
                serde_json::to_string(context).map_err(storage)
            })
            .transpose()?;
        let path = format!("$.\"{}\"", finish.id);
        let tx = self.connection.transaction().map_err(storage)?;
        let started: Option<String> = tx
            .query_row(
                "SELECT started_at FROM reading_sessions WHERE id=? AND document_id=?",
                params![finish.id, finish.document],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        let Some(started) = started else {
            return Err(Failure::Invalid("Reading session is unavailable".into()));
        };
        if ended < timestamp(&started)? {
            return Err(Failure::Invalid(
                "Reading session ends before it starts".into(),
            ));
        }
        if context.is_some() {
            let valid:Option<bool>=tx.query_row("SELECT CASE WHEN json_valid(value) THEN json_type(value)='object' AND COALESCE(json_type(value,?2),'object')='object' ELSE 0 END FROM app_metadata WHERE key=?1",params![CONTEXTS,path],|r|r.get(0)).optional().map_err(storage)?;
            if valid == Some(false) {
                return Err(Failure::Invalid(
                    "Stored reading session contexts require recovery".into(),
                ));
            }
        }
        let changed=tx.execute("UPDATE reading_sessions SET ended_at=?3,end_page=?4,active_seconds=?5 WHERE id=?1 AND document_id=?2 AND (ended_at IS NULL OR (ended_at=?3 AND end_page=?4 AND active_seconds=?5))",params![finish.id,finish.document,date,finish.page,seconds]).map_err(storage)?;
        if changed != 1 {
            return Err(Failure::Invalid(
                "Reading session has already ended with different values".into(),
            ));
        }
        if let Some(context) = context {
            tx.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES(?1,json_object(?2,json(?3)),?4) ON CONFLICT(key) DO UPDATE SET value=json_set(app_metadata.value,?5,json_patch(COALESCE(json_extract(app_metadata.value,?5),'{}'),json(?3))),updated_at=excluded.updated_at",params![CONTEXTS,finish.id,context,date,path]).map_err(storage)?;
        }
        cancel.check()?;
        tx.commit().map_err(storage)
    }
}
