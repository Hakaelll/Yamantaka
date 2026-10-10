use yamantaka_domain::{
    Failure, Result,
    reading_sessions::{ReaderSessionContext, restoration::ReaderSessionRestoration},
};

/// Decode only known fields. The governed history page retains the original
/// object, including extension fields; decoding does not rewrite stored data.
pub fn decode_session_context(json: Option<&str>) -> Result<Option<ReaderSessionRestoration>> {
    json.map(|value| {
        let context: ReaderSessionContext = serde_json::from_str(value).map_err(|_| {
            Failure::Invalid(
                "Stored reading context requires recovery; original data is preserved".into(),
            )
        })?;
        context.try_into()
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::reader_preferences::{
        CoverPolicy, PageRotation, ReaderMode, ReaderPreferences, ReaderTheme, ZoomMode,
    };
    use yamantaka_domain::{PageLayout, reading_sessions::restoration::SessionPanel};

    #[test]
    fn partial_and_null_contexts_use_flutter_defaults_without_rewriting_source() {
        assert!(decode_session_context(None).unwrap().is_none());
        for raw in [
            "{}",
            r#"{"zoomLevel":null,"pageLayout":null,"readingMode":null,"readingTheme":null,"scrollOffset":null,"panelSection":null,"future":{"nested":[1,true]}}"#,
        ] {
            let context = decode_session_context(Some(raw)).unwrap().unwrap();
            let result = context.apply(&ReaderPreferences::new("study")).unwrap();
            assert_eq!(result.zoom_level, 1.0);
            assert_eq!(result.zoom_mode, ZoomMode::Custom);
            assert_eq!(result.page_layout, PageLayout::Single);
            assert_eq!(result.view_mode, ReaderMode::Document);
            assert_eq!(result.reading_theme, ReaderTheme::Ivory);
            assert_eq!(result.scroll_offset, 0.0);
            assert_eq!(context.panel(), None);
        }
    }
    #[test]
    fn restoration_preserves_choices_outside_the_session_snapshot() {
        let mut current = ReaderPreferences::new("Étude 学习");
        current.cover_policy = CoverPolicy::NoCover;
        current.page_rotation = PageRotation::Deg270;
        current.horizontal_margin = 33.0;
        current.vertical_margin = 17.0;
        current.controls_auto_hide = true;
        current.updated_at = Some("2026-10-10T00:00:00Z".into());
        let original = current.clone();
        let raw = r#"{"zoomLevel":1.75,"pageLayout":"spread","readingMode":"book","readingTheme":"sepia","scrollOffset":234.5,"panelSection":"annotations","future":{"large":"preserved elsewhere"}}"#;
        let context = decode_session_context(Some(raw)).unwrap().unwrap();
        let restored = context.apply(&current).unwrap();
        assert_eq!(current, original);
        assert_eq!(restored.document_id, current.document_id);
        assert_eq!(restored.cover_policy, current.cover_policy);
        assert_eq!(restored.page_rotation, current.page_rotation);
        assert_eq!(restored.horizontal_margin, current.horizontal_margin);
        assert_eq!(restored.vertical_margin, current.vertical_margin);
        assert_eq!(restored.controls_auto_hide, current.controls_auto_hide);
        assert_eq!(restored.zoom_level, 1.75);
        assert_eq!(restored.persisted_layout_name(), "spread");
        assert_eq!(restored.view_mode, ReaderMode::Book);
        assert_eq!(restored.reading_theme, ReaderTheme::Sepia);
        assert_eq!(restored.scroll_offset, 234.5);
        assert_eq!(context.panel(), Some(SessionPanel::Annotations));
        assert_eq!(restored.updated_at, None);
    }
    #[test]
    fn invalid_or_future_values_fail_without_silent_reset() {
        for raw in [
            "null",
            "[]",
            "{",
            r#"{"zoomLevel":"1"}"#,
            r#"{"zoomLevel":0}"#,
            r#"{"zoomLevel":-1}"#,
            r#"{"zoomLevel":1e300}"#,
            r#"{"zoomLevel":1e-300}"#,
            r#"{"scrollOffset":-1}"#,
            r#"{"scrollOffset":1e300}"#,
            r#"{"pageLayout":"future"}"#,
            r#"{"readingMode":"future"}"#,
            r#"{"readingTheme":"future"}"#,
            r#"{"panelSection":"future"}"#,
            r#"{"panelSection":12}"#,
        ] {
            assert!(decode_session_context(Some(raw)).is_err(), "{raw}");
        }
    }
    #[test]
    fn unknown_extensions_are_ignored_and_known_text_is_bounded_before_copying() {
        let raw = format!(
            r#"{{"future":{{"large":"{}"}},"zoomLevel":1.25}}"#,
            "x".repeat(400_000)
        );
        let restored = decode_session_context(Some(&raw))
            .unwrap()
            .unwrap()
            .apply(&ReaderPreferences::new("study"))
            .unwrap();
        assert_eq!(restored.zoom_level, 1.25);
        for (field, limit) in [
            ("pageLayout", 128),
            ("readingMode", 128),
            ("readingTheme", 128),
            ("panelSection", 512),
        ] {
            let raw = format!(r#"{{"{field}":"{}"}}"#, "x".repeat(limit + 1));
            let Err(Failure::Invalid(error)) = decode_session_context(Some(&raw)) else {
                panic!("Oversized field was accepted")
            };
            assert!(error.contains("requires recovery"));
        }
        assert!(decode_session_context(Some(r#"{"zoomLevel":1,"zoomLevel":2}"#)).is_err());
        let escaped = r#"{"panelSection":"anno\u0074ations"}"#;
        assert_eq!(
            decode_session_context(Some(escaped))
                .unwrap()
                .unwrap()
                .panel(),
            Some(SessionPanel::Annotations)
        );
    }
    #[test]
    fn all_supported_panels_and_layout_spellings_are_explicit() {
        for (name, panel) in [
            ("outline", SessionPanel::Outline),
            ("thumbnails", SessionPanel::Thumbnails),
            ("search", SessionPanel::Search),
            ("bookmarks", SessionPanel::Bookmarks),
            ("annotations", SessionPanel::Annotations),
            ("text", SessionPanel::Text),
        ] {
            let raw = format!(r#"{{"panelSection":"{name}"}}"#);
            assert_eq!(
                decode_session_context(Some(&raw)).unwrap().unwrap().panel(),
                Some(panel)
            );
        }
        for name in ["single", "facing", "spread", "continuous"] {
            let raw = format!(r#"{{"pageLayout":"{name}"}}"#);
            let context = decode_session_context(Some(&raw)).unwrap().unwrap();
            assert_eq!(
                context
                    .apply(&ReaderPreferences::new("study"))
                    .unwrap()
                    .persisted_layout_name(),
                name
            );
        }
    }
}
