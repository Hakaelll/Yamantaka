use super::*;
use std::ops::Bound::{Excluded, Unbounded};

impl ReferenceView {
    pub(super) fn toggle_difference(&mut self, page: u32) {
        if !self.differences.insert(page) {
            self.differences.remove(&page);
        }
    }

    pub(super) fn adjacent_difference(
        &self,
        primary: &ReaderNavigation,
        forward: bool,
    ) -> Option<u32> {
        let page = primary.position().page();
        if forward {
            self.differences
                .range((Excluded(page), Unbounded))
                .next()
                .copied()
                .filter(|page| *page < primary.page_count())
        } else {
            self.differences.range(..page).next_back().copied()
        }
    }

    pub(super) fn difference_options(
        &mut self,
        ui: &mut egui::Ui,
        primary: &ReaderNavigation,
        english: bool,
        p: Palette,
    ) {
        let tr = |es, en| if english { en } else { es };
        let page = primary.position().page();
        ui.separator();
        ui.label(
            RichText::new(tr(
                "Diferencias · marcas manuales",
                "Differences · manual marks",
            ))
            .color(p.secondary)
            .size(12.0),
        );
        ui.label(format!(
            "{} {} · {} {}",
            tr("Página principal", "Main page"),
            page + 1,
            self.differences.len(),
            tr("marcas", "marks")
        ));
        let marked = self.differences.contains(&page);
        if theme::navigation(
            ui,
            if marked {
                tr("Quitar marca", "Remove mark")
            } else {
                tr("Marcar esta página", "Mark this page")
            },
            marked,
            p,
        )
        .clicked()
        {
            self.toggle_difference(page);
            ui.ctx().request_repaint();
        }
        for (forward, es, en) in [
            (false, "Anterior diferencia", "Previous difference"),
            (true, "Siguiente diferencia", "Next difference"),
        ] {
            let target = self.adjacent_difference(primary, forward);
            ui.add_enabled_ui(target.is_some(), |ui| {
                if theme::navigation(ui, tr(es, en), false, p).clicked() {
                    self.difference_target = target;
                }
            });
        }
    }
}
