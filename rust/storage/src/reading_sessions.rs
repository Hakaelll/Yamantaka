use crate::{Database, database::storage};
use rusqlite::{OptionalExtension, params};
use yamantaka_domain::{Cancellation, Failure, Result, reading_sessions::*};
const CONTEXTS: &str = "reading_session_contexts_v1";

fn timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|v| v.with_timezone(&chrono::Utc))
        .map_err(|_| Failure::Invalid("Reading session timestamp is invalid".into()))
}
fn identity(id: &str, document: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || document.is_empty()
    {
        return Err(Failure::Invalid(
            "Reading session identity is invalid".into(),
        ));
    }
    Ok(())
}
impl ReadingSessionRepository for Database {
    fn start_reading_session(
        &mut self,
        start: &ReadingSessionStart<'_>,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        identity(start.id, start.document)?;
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
        identity(finish.id, finish.document)?;
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
