use super::*;
impl YamantakaApp {
    pub(super) fn run_palette_command(&mut self, ctx: &egui::Context, command: Command, id: &str) {
        let global = matches!(
            command,
            Command::Settings
                | Command::Library
                | Command::History
                | Command::Notes
                | Command::Recent(_)
        ) || matches!(command,Command::Shortcut(shortcut) if shortcuts::available_outside_reader(shortcut));
        if !global && self.palette_document().is_none() {
            self.close_command_palette(ctx);
            return;
        }
        self.command_palette.last = Some(id.into());
        if matches!(command, Command::Info) {
            self.command_palette.info = true;
            return;
        }
        let recent = if let Command::Recent(index) = command {
            self.command_palette
                .recents
                .get(index)
                .filter(|document| !document.missing)
                .map(|document| document.id.clone())
        } else {
            None
        };
        let target = self
            .command_palette
            .context
            .as_ref()
            .map(|(target, _)| *target)
            .unwrap_or(ReadingTarget::Primary);
        self.close_command_palette(ctx);
        match command {
            Command::Shortcut(shortcut) => self.execute_shortcut(ctx, shortcut),
            Command::Settings => self.show_surface(Surface::Settings),
            Command::Library => self.show_surface(Surface::Library),
            Command::History => self.show_history(),
            Command::Notes => self.show_global_notes(),
            Command::Recent(_) => {
                if let Some(id) = recent {
                    let cancel = self.open_intent.lookup(id.clone(), None);
                    self.send(Action::GetDocument(id, cancel));
                }
            }
            Command::Info => unreachable!(),
            command => self.run_palette_reader(ctx, target, command),
        }
    }
    fn run_palette_reader(&mut self, ctx: &egui::Context, target: ReadingTarget, command: Command) {
        if let Command::Panel(tab) = command {
            self.execute_shortcut(ctx, Shortcut::Navigator);
            if target == ReadingTarget::Reference {
                if let Some(reference) = &mut self.reference {
                    reference.stop_navigator_requests();
                    reference.pane.reader_panel_tab = tab;
                    reference.navigator_requested = true;
                }
            } else {
                self.reader.find.stop();
                self.reader.reader_panel_tab = tab;
            }
            return;
        }
        if target == ReadingTarget::Reference {
            let Some(reference) = &mut self.reference else {
                return;
            };
            match command {
                Command::First => reference.set_page(0),
                Command::Last => {
                    if let Some(nav) = &reference.pane.navigation {
                        reference.set_page(nav.page_count().saturating_sub(1));
                    }
                }
                Command::Position(action) => {
                    if let Some(nav) = &mut reference.pane.navigation
                        && action.apply(nav)
                    {
                        reference.reference_navigation_changed();
                    }
                }
                Command::Layout(layout) => {
                    if let Some(preferences) = &mut reference.pane.reader_preferences {
                        preferences.page_layout = layout;
                    }
                    let direction = reference
                        .pane
                        .navigation
                        .as_ref()
                        .map_or(ReadingDirection::Ltr, ReaderNavigation::direction);
                    if let Err(error) = reference.pane.rebuild_layout(direction) {
                        self.error = Some(error.to_string());
                    }
                    reference.pane.viewport.reset_requests();
                }
                Command::Theme(theme) => {
                    if let Some(preferences) = &mut reference.pane.reader_preferences {
                        preferences.reading_theme = theme;
                    }
                }
                _ => unreachable!(),
            }
        } else {
            match command {
                Command::First => self.navigate(0),
                Command::Last => {
                    if let Some(nav) = &self.reader.navigation {
                        self.navigate(nav.page_count().saturating_sub(1));
                    }
                }
                Command::Position(action) => {
                    if let Some(nav) = &mut self.reader.navigation
                        && action.apply(nav)
                    {
                        self.apply_navigation();
                    }
                }
                Command::Layout(layout) => {
                    if let Some(mut preferences) = self.reader.reader_preferences.clone() {
                        preferences.page_layout = layout;
                        self.apply_reader_preferences(preferences);
                    }
                }
                Command::Theme(theme) => {
                    if let Some(mut preferences) = self.reader.reader_preferences.clone() {
                        preferences.reading_theme = theme;
                        self.apply_reader_preferences(preferences);
                    }
                }
                _ => unreachable!(),
            }
        }
        ctx.request_repaint();
    }
}
