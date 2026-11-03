use super::*;
use yamantaka_domain::Result;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum VisualProfile {
    #[default]
    Standard,
    ElectronicInk,
}

impl VisualProfile {
    pub(super) fn load(preferences: &Preferences) -> Self {
        match preferences
            .get("visualProfile")
            .and_then(serde_json::Value::as_str)
        {
            Some("electronicInk") => Self::ElectronicInk,
            _ => Self::Standard,
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::ElectronicInk => "electronicInk",
        }
    }

    pub(super) fn effective(self, saved: &ReaderPreferences) -> ReaderPreferences {
        let mut effective = saved.clone();
        if self == Self::ElectronicInk {
            effective.page_layout = yamantaka_domain::PageLayout::Single;
            effective.view_mode = yamantaka_domain::reader::ReaderMode::Document;
            effective.reading_theme = ReaderTheme::Light;
        }
        effective
    }
}

impl reader_pane::ReaderPane {
    pub(super) fn effective_preferences(&self) -> Option<ReaderPreferences> {
        self.reader_preferences
            .as_ref()
            .map(|saved| self.visual_profile.effective(saved))
    }

    pub(super) fn set_visual_profile(
        &mut self,
        profile: VisualProfile,
        direction: ReadingDirection,
    ) -> Result<()> {
        if self.visual_profile == profile {
            return Ok(());
        }
        self.visual_profile = profile;
        self.restoration.scroll = None;
        self.rebuild_layout(direction)?;
        self.viewport.generation += 1;
        self.viewport.reset_requests();
        Ok(())
    }

    pub(super) fn profile_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "profile":self.visual_profile.name(),
            "storedLayout":self.reader_preferences.as_ref().map(|p|p.persisted_layout_name()),
            "storedMode":self.reader_preferences.as_ref().map(|p|p.view_mode.persisted_name()),
            "storedTheme":self.reader_preferences.as_ref().map(|p|p.reading_theme.persisted_name()),
            "effectiveLayout":self.effective_preferences().map(|p|p.persisted_layout_name().to_owned()),
            "effectiveMode":self.effective_preferences().map(|p|p.view_mode.persisted_name()),
            "effectiveTheme":self.effective_preferences().map(|p|p.reading_theme.persisted_name()),
            "navigationLayout":self.navigation.as_ref().map(|n|format!("{:?}",n.layout())),
            "flatPages":self.viewport.flat_pages
        })
    }
}

impl YamantakaApp {
    pub(super) fn electronic_ink(&self) -> bool {
        self.reader.visual_profile == VisualProfile::ElectronicInk
    }

    pub(super) fn reader_profile_settings(&mut self, ui: &mut egui::Ui, p: Palette) -> bool {
        ui.separator();
        ui.add_space(16.0);
        ui.label(RichText::new(self.tr("Perfil del lector", "Reader profile")).strong());
        ui.label(RichText::new(self.tr("Tinta electrónica usa página única, papel blanco y cambios sin animación. Conserva los ajustes de cada documento.", "Electronic ink uses single pages, white paper and changes without animation. Your document settings are preserved.")).color(p.secondary));
        let mut selected = self.reader.visual_profile;
        let available = !self.opening.busy()
            && self.reference_mount.is_none()
            && self.reference.as_ref().is_none_or(|r| !r.busy());
        ui.add_enabled_ui(available, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(
                    &mut selected,
                    VisualProfile::Standard,
                    self.tr("Estándar", "Standard"),
                );
                let response = ui.radio_value(
                    &mut selected,
                    VisualProfile::ElectronicInk,
                    self.tr("Tinta electrónica", "Electronic ink"),
                );
                theme::record_control_bounds(
                    ui,
                    self.tr("Tinta electrónica", "Electronic ink"),
                    response.rect,
                );
                if ui.ctx().data(|data| {
                    data.get_temp::<bool>(egui::Id::new("preview-reader-profile-settings"))
                        .unwrap_or(false)
                }) {
                    response.scroll_to_me(Some(egui::Align::Center));
                }
            });
        });
        if selected == self.reader.visual_profile {
            return false;
        }
        if let Err(error) = self.apply_visual_profile(selected) {
            self.error = Some(error.to_string());
            return false;
        }
        true
    }

    fn apply_visual_profile(&mut self, selected: VisualProfile) -> Result<()> {
        let direction = self.reading_direction();
        self.reader
            .set_visual_profile(selected, direction)
            .and_then(|_| {
                if let Some(reference) = &mut self.reference {
                    reference.pane.set_visual_profile(selected, direction)?;
                }
                Ok(())
            })?;
        self.preferences
            .rust
            .insert("visualProfile".into(), selected.name().into());
        Ok(())
    }

    pub fn preview_profile_return(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reader_profile_preview = 1;
        }
    }

    pub(super) fn advance_profile_return(&mut self) {
        if self.reader_profile_preview != 1
            || self.opening.busy()
            || self.reader.reader_preferences.is_none()
            || self.reader.viewport.pages.is_empty()
            || self
                .reader
                .viewport
                .pages
                .values()
                .any(|page| page.image.is_none())
        {
            return;
        }
        assert!(self.electronic_ink());
        let path = self.store.join("visual-profile-verification.json");
        self.preferences
            .save(&path)
            .expect("Save synthetic profile");
        assert!(
            VisualProfile::load(&Preferences::load(&path).expect("Reload synthetic profile"))
                == VisualProfile::ElectronicInk
        );
        let before = self.reader.reader_preferences.clone();
        self.apply_visual_profile(VisualProfile::Standard)
            .expect("Synthetic profile return");
        assert_eq!(before, self.reader.reader_preferences);
        self.preferences
            .save(&path)
            .expect("Save synthetic standard profile");
        assert!(
            VisualProfile::load(
                &Preferences::load(&path).expect("Reload synthetic standard profile")
            ) == VisualProfile::Standard
        );
        self.reader_profile_preview = 2;
    }
}
