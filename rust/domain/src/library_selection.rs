//! Selection is scoped to the current bounded catalogue page.
#[derive(Default)]
pub struct LibrarySelection {
    pub mode: bool,
    ids: Vec<String>,
    anchor: Option<String>,
}
impl LibrarySelection {
    pub fn ids(&self) -> &[String] {
        &self.ids
    }
    pub fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|value| value == id)
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn retain(&mut self, page: &[String]) {
        self.ids.retain(|id| page.contains(id));
        if self.anchor.as_ref().is_some_and(|id| !page.contains(id)) {
            self.anchor = None;
        }
    }
    pub fn all(&mut self, page: &[String]) {
        self.ids = page.iter().take(crate::MAX_LIBRARY_PAGE).cloned().collect();
        self.mode = true;
        self.anchor = None;
    }
    pub fn select(&mut self, page: &[String], id: &str, additive: bool, range: bool) {
        let page = &page[..page.len().min(crate::MAX_LIBRARY_PAGE)];
        if !page.iter().any(|value| value == id) {
            return;
        }
        if self.mode || additive || range {
            if range
                && let Some(start) = self
                    .anchor
                    .as_ref()
                    .and_then(|anchor| page.iter().position(|value| value == anchor))
            {
                let end = page.iter().position(|value| value == id).unwrap();
                if !additive {
                    self.ids.clear();
                }
                for value in &page[start.min(end)..=start.max(end)] {
                    if !self.contains(value) {
                        self.ids.push(value.clone());
                    }
                }
            } else if self.contains(id) && (self.mode || additive) {
                self.ids.retain(|value| value != id);
            } else if !self.contains(id) {
                self.ids.push(id.to_owned());
            }
            self.anchor = Some(id.to_owned());
        }
    }
}
