use super::Shortcut;
use crate::app::*;

impl YamantakaApp {
    pub(in crate::app) fn dispatch_reading_shortcut(
        &mut self,
        ctx: &egui::Context,
        shortcut: &Shortcut,
    ) -> bool {
        if matches!(
            shortcut,
            Shortcut::PreviousPage
                | Shortcut::NextPage
                | Shortcut::HistoryBack
                | Shortcut::HistoryForward
        ) {
            // Page keys must not also move egui's spatial keyboard focus.
            ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
        if matches!(shortcut, Shortcut::DocumentFind) {
            self.reading_focus.cancel_canvas_request();
        }
        if matches!(shortcut, Shortcut::SwitchPane) {
            if self.reference.is_some() {
                if !self.reading_focus.reference()
                    && self.reader.panel
                    && !self.reader_focus
                    && ctx.content_rect().width() < 620.0
                {
                    self.reader.panel = false;
                    self.reader.find.stop();
                }
                egui::Popup::close_all(ctx);
                self.reading_focus.switch(ctx);
                ctx.request_repaint();
            }
            return true;
        }
        if self.reading_focus.reference() {
            if matches!(shortcut, Shortcut::CloseTab) {
                self.request_reference_close();
                return true;
            }
            let Some(reference) = &mut self.reference else {
                return false;
            };
            if !matches!(
                shortcut,
                Shortcut::DocumentFind
                    | Shortcut::PreviousMatch
                    | Shortcut::NextMatch
                    | Shortcut::PreviousPage
                    | Shortcut::NextPage
                    | Shortcut::ZoomIn
                    | Shortcut::ZoomOut
                    | Shortcut::HistoryBack
                    | Shortcut::HistoryForward
                    | Shortcut::FitPage
            ) {
                return false;
            }
            if reference.busy() {
                return true;
            }
            match shortcut {
                Shortcut::HistoryBack | Shortcut::HistoryForward => {
                    let action = if matches!(shortcut, Shortcut::HistoryBack) {
                        navigation::PositionAction::Back
                    } else {
                        navigation::PositionAction::Forward
                    };
                    if reference
                        .pane
                        .navigation
                        .as_mut()
                        .is_some_and(|nav| action.apply(nav))
                    {
                        reference.reference_navigation_changed();
                    }
                }
                Shortcut::DocumentFind => {
                    reference.pane.reader_panel_tab = outline::ReaderPanelTab::Find;
                    reference.search_visible = false;
                    reference.navigator_requested = true;
                }
                Shortcut::PreviousMatch | Shortcut::NextMatch => {
                    if let Some((document, info)) = &reference.pane.active
                        && let Some(target) = reference.pane.find.step(
                            &self.runtime,
                            document,
                            info,
                            matches!(shortcut, Shortcut::NextMatch),
                        )
                    {
                        reference.apply_reference_find(target);
                    }
                }
                Shortcut::PreviousPage | Shortcut::NextPage => {
                    if let Some(nav) = &mut reference.pane.navigation
                        && if matches!(shortcut, Shortcut::NextPage) {
                            nav.next_page()
                        } else {
                            nav.previous()
                        }
                    {
                        reference.reference_navigation_changed();
                    }
                }
                Shortcut::ZoomIn | Shortcut::ZoomOut => {
                    let zoom = (reference.pane.viewport.scale
                        * if matches!(shortcut, Shortcut::ZoomIn) {
                            1.2
                        } else {
                            1.0 / 1.2
                        })
                    .clamp(0.2, 8.0);
                    reference.set_zoom(ZoomMode::Custom, zoom);
                }
                Shortcut::FitPage => reference.set_zoom(ZoomMode::FitPage, reference.pane.zoom),
                _ => unreachable!(),
            }
            ctx.request_repaint();
            return true;
        }
        match shortcut {
            Shortcut::HistoryBack | Shortcut::HistoryForward => {
                let action = if matches!(shortcut, Shortcut::HistoryBack) {
                    navigation::PositionAction::Back
                } else {
                    navigation::PositionAction::Forward
                };
                if self
                    .reader
                    .navigation
                    .as_mut()
                    .is_some_and(|nav| action.apply(nav))
                {
                    self.apply_navigation();
                }
            }
            Shortcut::ZoomIn | Shortcut::ZoomOut => {
                let zoom = self.reader.viewport.scale
                    * if matches!(shortcut, Shortcut::ZoomIn) {
                        1.2
                    } else {
                        1.0 / 1.2
                    };
                self.set_reader_zoom(zoom);
            }
            Shortcut::FitPage => {
                if let Some(mut preferences) = self.reader.reader_preferences.clone() {
                    preferences.zoom_mode = ZoomMode::FitPage;
                    self.apply_reader_preferences(preferences);
                }
            }
            _ => return false,
        }
        ctx.request_repaint();
        true
    }
}
