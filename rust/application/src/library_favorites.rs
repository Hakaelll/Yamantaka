//! One catalogue write per actor iteration, with a fixed target snapshot.
use yamantaka_domain::{Cancellation, Failure, LibraryRepository, MAX_LIBRARY_PAGE, Result};
pub struct FavoriteRequest {
    pub ids: Vec<String>,
    pub value: bool,
    pub cancel: Cancellation,
}
pub struct FavoriteReport {
    pub succeeded: Vec<String>,
    pub failed: Vec<String>,
    pub remaining: Vec<String>,
}
pub struct FavoriteReply {
    pub cancel: Cancellation,
    pub result: Result<FavoriteReport>,
}
pub(crate) struct FavoriteJob {
    request: FavoriteRequest,
    next: usize,
    succeeded: Vec<String>,
    failed: Vec<String>,
}
impl FavoriteJob {
    pub(crate) fn start(mut request: FavoriteRequest) -> Result<Self> {
        if request.ids.is_empty()
            || request.ids.len() > MAX_LIBRARY_PAGE
            || request
                .ids
                .iter()
                .any(|id| id.trim().is_empty() || id.len() > 4096)
        {
            return Err(Failure::Invalid(
                "Select between one and 80 catalogue books".into(),
            ));
        }
        let mut unique = Vec::with_capacity(request.ids.len());
        for id in request.ids {
            if !unique.contains(&id) {
                unique.push(id);
            }
        }
        request.ids = unique;
        Ok(Self {
            request,
            next: 0,
            succeeded: Vec::new(),
            failed: Vec::new(),
        })
    }
    pub(crate) fn step(&mut self, repository: &mut impl LibraryRepository) -> bool {
        if self.request.cancel.is_cancelled() || self.next == self.request.ids.len() {
            return true;
        }
        let id = &self.request.ids[self.next];
        let result = repository.set_favorite(id, self.request.value);
        if result.is_ok() {
            self.succeeded.push(id.clone());
        } else {
            self.failed.push(id.clone());
        }
        self.next += 1;
        self.next == self.request.ids.len()
    }
    pub(crate) fn reply(self) -> FavoriteReply {
        FavoriteReply {
            cancel: self.request.cancel,
            result: Ok(FavoriteReport {
                succeeded: self.succeeded,
                failed: self.failed,
                remaining: self.request.ids[self.next..].to_vec(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{LibraryQuery, library_selection::LibrarySelection};
    use yamantaka_storage::Database;
    #[test]
    fn page_selection_and_bulk_favorites_keep_fixed_targets_and_partial_results() {
        let page: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
        let mut selection = LibrarySelection::default();
        selection.select(&page, "a", true, false);
        selection.select(&page, "c", false, true);
        assert_eq!(selection.ids(), &["a", "b", "c"]);
        selection.select(&page, "b", true, false);
        selection.select(&page, "d", true, false);
        assert_eq!(selection.ids(), &["a", "c", "d"]);
        let targets = selection.ids().to_vec();
        selection.retain(&["a".into(), "b".into()]);
        assert_eq!(selection.ids(), &["a"]);
        selection.all(&page);
        assert!(selection.mode);
        assert_eq!(selection.ids(), &["a", "b", "c", "d"]);
        selection.clear();
        assert!(!selection.mode);
        let mut db = Database::in_memory_at(11).unwrap();
        for id in &page {
            db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,last_page_index,title_override) VALUES('{id}','{id}.pdf','{id}.pdf','fixture.pdf','Title','Author',2,900,'2026-10-20','2026-10-20','2026-10-20','synthetic',0.5,1,'My title')")).unwrap();
        }
        let cancel = Cancellation::default();
        let mut job = FavoriteJob::start(FavoriteRequest {
            ids: targets,
            value: true,
            cancel: cancel.clone(),
        })
        .unwrap();
        assert!(!job.step(&mut db));
        assert!(db.get("a").unwrap().unwrap().favorite);
        assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 4);
        cancel.cancel();
        assert!(job.step(&mut db));
        let report = job.reply().result.unwrap();
        assert_eq!(report.succeeded, ["a"]);
        assert!(report.failed.is_empty());
        assert_eq!(report.remaining, ["c", "d"]);
        assert!(!db.get("c").unwrap().unwrap().favorite);
        let mut job = FavoriteJob::start(FavoriteRequest {
            ids: vec!["a".into(), "missing".into(), "d".into(), "a".into()],
            value: false,
            cancel: Cancellation::default(),
        })
        .unwrap();
        while !job.step(&mut db) {}
        let report = job.reply().result.unwrap();
        assert_eq!(report.succeeded, ["a", "d"]);
        assert_eq!(report.failed, ["missing"]);
        assert!(report.remaining.is_empty());
        let doc = db.get("a").unwrap().unwrap();
        assert!(!doc.favorite);
        assert_eq!(doc.display_title(), "My title");
        assert_eq!(doc.last_page_index, 1);
        assert_eq!(doc.reading_progress, 0.5);
        assert!(
            FavoriteJob::start(FavoriteRequest {
                ids: vec!["a".into(); 81],
                value: true,
                cancel: Cancellation::default()
            })
            .is_err()
        );
        let cancel = Cancellation::default();
        cancel.cancel();
        let mut job = FavoriteJob::start(FavoriteRequest {
            ids: page,
            value: true,
            cancel,
        })
        .unwrap();
        assert!(job.step(&mut db));
        assert_eq!(job.reply().result.unwrap().remaining, ["a", "b", "c", "d"]);
    }
}
