use super::*;
use yamantaka_domain::{Failure, reading_sessions::ReadingSessionRepository};
use yamantaka_storage::Database;

fn reply(request: ReadingHistoryRequest, db: &Database) -> ReadingHistoryReply {
    ReadingHistoryReply {
        result: db
            .reading_history(&request.query, &request.governor, &request.cancel)
            .map(Arc::new),
        query: request.query,
        cancel: request.cancel,
    }
}
#[test]
fn latest_query_and_token_own_the_history_destination() {
    let governor = MemoryGovernor::default();
    let db = Database::in_memory_at(11).unwrap();
    let mut state = HistoryState::default();
    let first = state.request(0, &governor);
    let stale = reply(first, &db);
    let current = state.request(80, &governor);
    assert!(stale.cancel.is_cancelled());
    state.accept(stale);
    assert!(state.page.is_none());
    assert!(state.pending.is_some());
    let wrong = ReadingHistoryReply {
        query: ReadingHistoryQuery::default(),
        cancel: current.cancel.clone(),
        result: Err(Failure::MemoryBudget),
    };
    state.accept(wrong);
    assert!(state.error.is_none());
    assert!(state.pending.is_some());
    state.accept(reply(current, &db));
    assert_eq!(state.page.as_ref().unwrap().offset(), 80);
    assert!(state.pending.is_none());
    let late = reply(state.request(0, &governor), &db);
    state.leave();
    state.accept(late);
    assert!(state.page.is_none());
    assert!(state.pending.is_none());
}
#[test]
fn failed_refresh_preserves_readings_and_can_be_retried() {
    let governor = MemoryGovernor::default();
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('study','study.pdf','study.pdf','study.pdf','Study',2,1,'now','now','now','study'); INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) VALUES('session','study','2026-10-09T09:00:00Z','2026-10-09T10:00:00Z',0,1,600);").unwrap();
    let mut state = HistoryState::default();
    let request = state.request(0, &governor);
    state.accept(reply(request, &db));
    let retained = state.page.as_ref().unwrap().clone();
    assert_eq!(retained.entries().len(), 1);
    assert!(governor.snapshot().usage.total > 0);
    let request = state.request(80, &governor);
    state.accept(ReadingHistoryReply {
        query: request.query,
        cancel: request.cancel,
        result: Err(Failure::MemoryBudget),
    });
    assert!(Arc::ptr_eq(&retained, state.page.as_ref().unwrap()));
    assert!(state.error.is_some());
    assert!(state.pending.is_none());
    let request = state.request(0, &governor);
    assert!(state.error.is_none());
    state.accept(reply(request, &db));
    state.leave();
    assert_eq!(Arc::strong_count(&retained), 1);
    drop(retained);
    assert_eq!(governor.snapshot().usage.total, 0);
}
