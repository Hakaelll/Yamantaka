use super::*;

#[derive(Default)]
pub(super) struct Probe {
    stage: u8,
    expect_empty: bool,
    fail_once: bool,
    expect_failure: bool,
    failure_seen: bool,
    target_page: u32,
    prepared_pages: Vec<u32>,
    adoption_paints: u64,
    adopted_without_image: bool,
    empty_before: u64,
    empty_after: u64,
}
impl Probe {
    pub(super) fn begin_adoption(&mut self, paints: u64) {
        self.prepared_pages.clear();
        self.adoption_paints = paints;
    }
    pub(super) fn record_prepared(&mut self, page: u32) {
        self.prepared_pages.push(page);
    }
    pub(super) fn inject_failure(&mut self, document: &str) -> bool {
        if self.stage == 2 && document == "demo-1" {
            std::mem::take(&mut self.fail_once)
        } else {
            false
        }
    }
    pub(super) fn ready(&self) -> bool {
        matches!(self.stage, 0 | 3)
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"stage":self.stage,"adoptedWithoutImage":self.adopted_without_image,"failureSeen":self.failure_seen,"targetPage":self.target_page,"preparedPages":self.prepared_pages,"emptyPagePaints":self.empty_after.saturating_sub(self.empty_before)})
    }
}
impl YamantakaApp {
    pub fn preview_first_raster(&mut self, expect_empty: bool, fail_once: bool, page_two: bool) {
        if self.demo && self.capture.is_some() {
            self.opening.first_raster_probe = Probe {
                stage: 1,
                expect_empty,
                fail_once,
                expect_failure: fail_once,
                target_page: u32::from(page_two),
                ..Default::default()
            };
        }
    }
    pub(in crate::app) fn advance_first_raster_probe(&mut self) {
        let image = self
            .reader
            .viewport
            .pages
            .values()
            .any(|slot| slot.image.is_some());
        match self.opening.first_raster_probe.stage {
            1 if image
                && !self.reader.loading
                && self.reader.reader_preferences.is_some()
                && !self.opening.busy() =>
            {
                let document = self.documents.get(1).cloned().unwrap();
                self.opening.first_raster_probe.empty_before = self.reader.viewport.empty_paints;
                assert!(self.open(document));
                self.open_intent
                    .set_page("demo-1".into(), self.opening.first_raster_probe.target_page);
                self.opening.first_raster_probe.stage = 2;
            }
            2 if self.opening.first_raster_probe.expect_failure
                && !self.opening.first_raster_probe.failure_seen
                && self.error.is_some()
                && !self.opening.busy() =>
            {
                assert!(image);
                assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                self.opening.first_raster_probe.failure_seen = true;
                self.error = None;
                let document = self.documents.get(1).cloned().unwrap();
                assert!(self.open(document));
                self.open_intent
                    .set_page("demo-1".into(), self.opening.first_raster_probe.target_page);
            }
            2 if self
                .reader
                .active
                .as_ref()
                .is_some_and(|(document, _)| document.id == "demo-1") =>
            {
                let probe = &mut self.opening.first_raster_probe;
                probe.adopted_without_image |= !image;
                if image
                    && self.reader.viewport.paint_count > probe.adoption_paints
                    && !self.reader.loading
                    && self.reader.reader_preferences.is_some()
                    && !self.opening.busy()
                {
                    let probe = &mut self.opening.first_raster_probe;
                    probe.empty_after = self.reader.viewport.empty_paints;
                    assert_eq!(
                        probe.adopted_without_image || probe.empty_after > probe.empty_before,
                        probe.expect_empty
                    );
                    if !probe.expect_empty {
                        assert_eq!(probe.empty_before, probe.empty_after);
                    }
                    assert_eq!(self.reader.page, probe.target_page);
                    assert_eq!(probe.failure_seen, probe.expect_failure);
                    probe.stage = 3;
                }
            }
            _ => {}
        }
    }
}
