use super::*;

impl YamantakaApp {
    pub(super) fn apply_reader_destination(
        &mut self,
        document: &Document,
        info: &PdfInfo,
        viewport: [f64; 2],
    ) {
        let Some((id, destination)) = self.reader.reader_destination.take() else {
            return;
        };
        if id != document.id {
            return;
        }
        let result = (|| {
            let layout = self
                .reader
                .viewport
                .layout
                .as_ref()
                .ok_or_else(|| "Reader layout is unavailable".to_string())?;
            let navigation = self
                .reader
                .navigation
                .as_ref()
                .ok_or_else(|| "Reader navigation is unavailable".to_string())?;
            let mut preferences = self
                .reader
                .reader_preferences
                .clone()
                .ok_or_else(|| "Reader preferences are unavailable".to_string())?;
            let anchor = navigation.position();
            let space = info
                .pages
                .get(anchor.page() as usize)
                .ok_or_else(|| "Current page is unavailable".to_string())?
                .coordinates();
            let scroll = layout
                .scroll_for_anchor(
                    viewport,
                    anchor,
                    preferences.zoom_mode,
                    f64::from(self.reader.zoom),
                )
                .map_err(|error| error.to_string())?;
            let frame = layout
                .frame(
                    viewport,
                    anchor,
                    preferences.zoom_mode,
                    f64::from(self.reader.zoom),
                    scroll,
                    0.0,
                )
                .map_err(|error| error.to_string())?;
            let point = layout
                .pdf_point_at_viewport(&frame, anchor.page(), space)
                .map_err(|error| error.to_string())?;
            let resolved = yamantaka_domain::reader_destination::resolve(
                layout,
                destination,
                viewport,
                point,
                frame.zoom,
            )
            .map_err(|error| error.to_string())?;
            if !(resolved.zoom as f32).is_finite() || resolved.zoom as f32 <= 0.0 {
                return Err("Destination zoom cannot be represented by the reader".into());
            }
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = resolved.zoom;
            preferences.updated_at = None;
            preferences.validate().map_err(|error| error.to_string())?;
            Ok((preferences, resolved))
        })();
        match result {
            Ok((preferences, resolved)) => {
                self.reader.zoom = resolved.zoom as f32;
                self.reader.reader_preferences = Some(preferences.clone());
                if let Some(navigation) = &mut self.reader.navigation {
                    navigation.navigate_to(resolved.position);
                }
                self.apply_navigation();
                if !self.demo {
                    self.send(Action::SaveReaderPreferences(
                        document.id.clone(),
                        preferences,
                    ));
                }
            }
            Err(error) => self.error = Some(error),
        }
    }
}
