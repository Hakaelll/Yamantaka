use super::*;

pub(super) struct ReaderSurface {
    pub scale: f32,
    pub pages: HashMap<u32, ReaderPage>,
    pub layout: Option<Arc<ReaderLayout>>,
    pub scroll_anchor: bool,
    pub generation: u64,
    pub requested: Option<RenderKey>,
    pub retry: bool,
    pub cancel: Cancellation,
}

impl Default for ReaderSurface {
    fn default() -> Self {
        Self {
            scale: 1.0,
            pages: HashMap::new(),
            layout: None,
            scroll_anchor: true,
            generation: 0,
            requested: None,
            retry: false,
            cancel: Cancellation::default(),
        }
    }
}

impl ReaderSurface {
    pub fn accepts(&self, page: &yamantaka_domain::RenderedPage, cancel: &Cancellation) -> bool {
        self.pages
            .get(&page.key.page)
            .is_some_and(|slot| slot.accepts(page, cancel))
    }

    pub fn install(
        &mut self,
        page: &yamantaka_domain::RenderedPage,
        cancel: &Cancellation,
        image: PageImage,
    ) -> Option<bool> {
        let slot = self.pages.get_mut(&page.key.page)?;
        if !slot.accepts(page, cancel) {
            return None;
        }
        let replaced = slot.image_key.is_some();
        slot.image = Some(image);
        slot.image_key = Some(page.key.clone());
        Some(replaced)
    }

    pub fn reset_requests(&mut self) {
        self.requested = None;
        self.retry = false;
        self.scroll_anchor = true;
        for page in self.pages.values_mut() {
            page.cancel.cancel();
            page.requested = None;
        }
    }

    pub fn clear(&mut self) {
        self.cancel.cancel();
        for page in self.pages.values() {
            page.cancel.cancel();
        }
        self.pages.clear();
        self.layout = None;
        self.cancel = Cancellation::default();
        self.reset_requests();
    }
}

impl Drop for ReaderSurface {
    fn drop(&mut self) {
        self.cancel.cancel();
        for page in self.pages.values() {
            page.cancel.cancel();
        }
    }
}
