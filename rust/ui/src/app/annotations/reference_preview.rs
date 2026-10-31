use super::*;

#[derive(Default)]
pub(in crate::app) struct MarksPreview {
    phase: u8,
    shared: bool,
    retained: bool,
}
impl MarksPreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 6)
    }
    pub fn phase(&self) -> u8 {
        self.phase
    }
}
impl YamantakaApp {
    pub fn preview_reference_marks(&mut self, shared: bool) {
        if self.demo && self.capture.is_some() {
            self.reference_marks_preview = MarksPreview {
                phase: 1,
                shared,
                retained: false,
            };
        }
    }
    pub(in crate::app) fn advance_reference_marks(&mut self) {
        if self.reference_marks_preview.ready() {
            return;
        }
        let Some(reference) = &mut self.reference else {
            return;
        };
        let document = if self.reference_marks_preview.shared {
            "demo-0"
        } else {
            "demo-1"
        };
        if reference.busy()
            || reference
                .pane
                .active
                .as_ref()
                .is_none_or(|(doc, _)| doc.id != document)
        {
            return;
        }
        let ready = reference.pane.annotation_paint.ready()
            && self.reader.annotation_paint.ready()
            && !reference.pane.loading;
        if self.reference_marks_preview.phase == 3
            && self.reader.annotations.saved
            && !self.reader.annotation_paint.ready()
        {
            assert_eq!(
                self.reader.annotation_paint.capture_summary()["retainedPages"],
                1
            );
            if self.reference_marks_preview.shared {
                assert_eq!(
                    reference.pane.annotation_paint.capture_summary()["retainedPages"],
                    1
                );
            }
            self.reference_marks_preview.retained = true;
        }
        match self.reference_marks_preview.phase {
            1 if self.reference_marks_preview.shared || self.reference_demo == 3 => {
                reference.set_page(0);
                self.reference_marks_preview.phase = 2;
            }
            2 if ready && self.reader.annotations.ready() => {
                assert_eq!(self.reader.annotation_paint.capture_summary()["marks"], 1);
                let summary = reference.pane.annotation_paint.capture_summary();
                assert_eq!(summary["document"], document);
                assert_eq!(summary["details"][0]["page"], 0);
                assert_eq!(
                    summary["marks"],
                    if self.reference_marks_preview.shared {
                        1
                    } else {
                        3
                    }
                );
                reference.pane.annotation_paint.invalidate();
                reference.pane.annotation_paint.request(
                    &self.runtime,
                    &self.governor,
                    document,
                    &[0],
                );
                reference.pane.annotation_paint.invalidate();
                reference.pane.annotation_paint.request(
                    &self.runtime,
                    &self.governor,
                    document,
                    &[0],
                );
                self.reference_marks_preview.phase = 7;
            }
            7 if ready && self.reader.annotations.ready() => {
                let expected = self
                    .reader
                    .annotations
                    .page
                    .as_ref()
                    .unwrap()
                    .entries()
                    .iter()
                    .find(|record| record.data().id == "synthetic-primary-mark")
                    .unwrap()
                    .revision()
                    .clone();
                self.annotation_command(
                    Some(AnnotationMutation::Edit {
                        id: "synthetic-primary-mark".into(),
                        expected,
                        kind: AnnotationKind::Highlight,
                        color: AnnotationColor::Blue,
                        note: Some("Updated in the main reading".into()),
                        review: AnnotationReviewState::Accepted,
                        tags: None,
                    }),
                    0,
                );
                self.reference_marks_preview.phase = 3;
            }
            3 if ready && self.reader.annotations.ready() && self.reader.annotations.saved => {
                assert!(self.reference_marks_preview.retained);
                let source = self.reader.annotation_paint.capture_summary();
                assert_eq!(source["details"][0]["colors"][0], "Blue");
                let summary = reference.pane.annotation_paint.capture_summary();
                assert_eq!(
                    summary["details"][0]["colors"][0],
                    if self.reference_marks_preview.shared {
                        "Blue"
                    } else {
                        "Green"
                    }
                );
                assert_eq!(
                    summary["marks"],
                    if self.reference_marks_preview.shared {
                        1
                    } else {
                        3
                    }
                );
                reference.set_page(1);
                self.reference_marks_preview.phase = 4;
            }
            4 if ready && reference.pane.page == 1 => {
                let summary = reference.pane.annotation_paint.capture_summary();
                assert_eq!(summary["details"][0]["page"], 1);
                assert_eq!(summary["marks"], 0);
                assert_eq!(summary["residentPages"], 1);
                assert_eq!(self.reader.page, 0);
                assert_eq!(self.reader.annotation_paint.capture_summary()["marks"], 1);
                reference.set_page(0);
                self.reference_marks_preview.phase = 5;
            }
            5 if ready && reference.pane.page == 0 => {
                let summary = reference.pane.annotation_paint.capture_summary();
                assert_eq!(summary["details"][0]["page"], 0);
                assert_eq!(
                    summary["marks"],
                    if self.reference_marks_preview.shared {
                        1
                    } else {
                        3
                    }
                );
                assert_eq!(
                    summary["details"][0]["colors"][0],
                    if self.reference_marks_preview.shared {
                        "Blue"
                    } else {
                        "Green"
                    }
                );
                assert_eq!(
                    self.reader.annotation_paint.capture_summary()["details"][0]["colors"][0],
                    "Blue"
                );
                self.reference_marks_preview.phase = 6;
            }
            _ => {}
        }
    }
}
