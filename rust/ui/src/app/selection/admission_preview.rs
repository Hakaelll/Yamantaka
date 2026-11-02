use super::*;
fn failed(page: u32) -> text::TextState {
    let mut state = text::TextState::default();
    let cancel = state.begin("synthetic-admission", page).unwrap();
    state.accept(
        "synthetic-admission",
        page,
        &cancel,
        Err("Synthetic extraction failure".into()),
    );
    state
}
impl YamantakaApp {
    pub fn preview_selection_admission(&mut self) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        let mut state = SelectionState {
            enabled: true,
            document: Some("synthetic-admission".into()),
            gesture: SelectionGesture::Selected(
                SelectionPosition::new(0, 0).unwrap(),
                SelectionPosition::new(99999, 0).unwrap(),
            ),
            ..Default::default()
        };
        state.pages.insert(0, failed(0));
        assert_eq!(state.next_pages(&[0]), Vec::<u32>::new());
        assert_eq!(state.next_pages(&[17]), vec![17]);
        state.retry_page(0);
        assert_eq!(state.next_pages(&[17]), vec![17, 0]);
        state.pages.insert(88, failed(88));
        state.gesture = SelectionGesture::Selected(
            SelectionPosition::new(0, 0).unwrap(),
            SelectionPosition::new(2, 1).unwrap(),
        );
        assert_eq!(state.next_pages(&[0]), vec![0, 1]);
        for page in [1, 2] {
            let mut pending = text::TextState::default();
            pending.begin("synthetic-admission", page).unwrap();
            state.pages.insert(page, pending);
        }
        assert_eq!(state.next_pages(&[0]), Vec::<u32>::new());
        self.selection_admission_verified = Some(
            serde_json::json!({"synthetic":true,"selectedPages":100000,"failedRangeBackgroundRequests":0,"visiblePageStillAdmitted":true,"retryResumesBackgroundRequests":true,"unrelatedFailureDoesNotBlockSelection":true,"twoPendingRequestsStopAdmission":true}),
        );
    }
}
