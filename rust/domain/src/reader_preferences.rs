//! Lossless adapters for Flutter's per-document v11 reading preferences.
pub use crate::reader::ReaderMode;
use crate::{Failure, PageLayout, Result};

macro_rules! persisted_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn persisted_name(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
            pub fn from_persisted(value: &str) -> Result<Self> {
                match value {
                    $($value => Ok(Self::$variant)),+,
                    _ => Err(Failure::Invalid(format!(
                        "Unknown {} value {value:?}; stored preference is preserved", stringify!($name)
                    ))),
                }
            }
        }
    };
}
persisted_enum!(ReaderTheme {
    Light => "light", Ivory => "ivory", Sepia => "sepia", Gray => "gray",
    Dark => "dark", Black => "black"
});
persisted_enum!(ZoomMode { Custom => "custom", FitWidth => "fitWidth", FitPage => "fitPage" });
persisted_enum!(CoverPolicy { FirstPageIsCover => "firstPageIsCover", NoCover => "noCover" });
persisted_enum!(InitialSide { Left => "left", Right => "right" });

impl ReaderMode {
    pub fn persisted_name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Book => "book",
        }
    }
    pub fn from_persisted(value: &str) -> Result<Self> {
        match value {
            "document" => Ok(Self::Document),
            "book" => Ok(Self::Book),
            _ => Err(Failure::Invalid(format!(
                "Unknown reader mode {value:?}; stored preference is preserved"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageRotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}
impl PageRotation {
    pub fn rotate_point(self, [x, y]: [f64; 2]) -> [f64; 2] {
        match self {
            Self::Deg0 => [x, y],
            Self::Deg90 => [1.0 - y, x],
            Self::Deg180 => [1.0 - x, 1.0 - y],
            Self::Deg270 => [y, 1.0 - x],
        }
    }
    pub fn unrotate_point(self, point: [f64; 2]) -> [f64; 2] {
        match self {
            Self::Deg0 => Self::Deg0,
            Self::Deg90 => Self::Deg270,
            Self::Deg180 => Self::Deg180,
            Self::Deg270 => Self::Deg90,
        }
        .rotate_point(point)
    }
    pub fn degrees(self) -> u16 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }
    pub fn from_degrees(value: i64) -> Result<Self> {
        match value {
            0 => Ok(Self::Deg0),
            90 => Ok(Self::Deg90),
            180 => Ok(Self::Deg180),
            270 => Ok(Self::Deg270),
            _ => Err(Failure::Invalid(
                "Reader rotation must be 0, 90, 180 or 270 degrees".into(),
            )),
        }
    }
}

/// Logical points. Large but finite legacy values remain valid without clamping;
/// values beyond the supported page extent fail recoverably rather than reset.
pub const MAX_READER_MARGIN: f64 = 16_384.0;

pub fn parse_layout_name(name: &str) -> Result<PageLayout> {
    match name {
        "single" => Ok(PageLayout::Single),
        "facing" | "spread" => Ok(PageLayout::Spread),
        "continuous" => Ok(PageLayout::Continuous),
        _ => Err(Failure::Invalid(format!(
            "Unknown reader layout {name:?}; stored preference is preserved"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReaderPreferences {
    pub document_id: String,
    pub view_mode: ReaderMode,
    pub reading_theme: ReaderTheme,
    pub zoom_mode: ZoomMode,
    pub zoom_level: f64,
    pub page_layout: PageLayout,
    pub cover_policy: CoverPolicy,
    pub initial_side: InitialSide,
    pub page_rotation: PageRotation,
    pub horizontal_margin: f64,
    pub vertical_margin: f64,
    pub controls_auto_hide: bool,
    pub scroll_offset: f64,
    /// Exact ISO text read from Flutter. None receives a UTC timestamp on save.
    pub updated_at: Option<String>,
    // Preserve accepted historical `spread` spelling while the layout is unchanged.
    original_layout: Option<(PageLayout, String)>,
}
impl ReaderPreferences {
    /// Flutter opens Book as a facing spread; the persisted raw layout remains
    /// untouched until the user explicitly changes the reading mode/layout.
    pub fn effective_layout(&self) -> PageLayout {
        if self.view_mode == ReaderMode::Book {
            PageLayout::Spread
        } else {
            self.page_layout
        }
    }
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            view_mode: ReaderMode::Document,
            reading_theme: ReaderTheme::Ivory,
            zoom_mode: ZoomMode::FitPage,
            zoom_level: 1.0,
            page_layout: PageLayout::Single,
            cover_policy: CoverPolicy::NoCover,
            initial_side: InitialSide::Left,
            page_rotation: PageRotation::Deg0,
            horizontal_margin: 32.0,
            vertical_margin: 24.0,
            controls_auto_hide: true,
            scroll_offset: 0.0,
            updated_at: None,
            original_layout: None,
        }
    }
    pub fn load_layout_name(&mut self, name: &str) -> Result<()> {
        let layout = parse_layout_name(name)?;
        self.page_layout = layout;
        self.original_layout = Some((layout, name.into()));
        Ok(())
    }
    pub fn persisted_layout_name(&self) -> &str {
        if let Some((layout, name)) = &self.original_layout
            && *layout == self.page_layout
        {
            return name;
        }
        match self.page_layout {
            PageLayout::Single => "single",
            PageLayout::Spread => "facing",
            PageLayout::Continuous => "continuous",
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.document_id.is_empty() {
            return Err(Failure::Invalid(
                "Reader preferences require a document ID".into(),
            ));
        }
        if !self.zoom_level.is_finite() || self.zoom_level <= 0.0 {
            return Err(Failure::Invalid(
                "Reader zoom must be finite and positive".into(),
            ));
        }
        for margin in [self.horizontal_margin, self.vertical_margin] {
            if !margin.is_finite() || !(0.0..=MAX_READER_MARGIN).contains(&margin) {
                return Err(Failure::Invalid(
                    "Reader margins must be finite and within 0..=16384 logical points".into(),
                ));
            }
        }
        if !self.scroll_offset.is_finite() {
            return Err(Failure::Invalid(
                "Reader scroll offset must be finite".into(),
            ));
        }
        if self
            .updated_at
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > 128)
        {
            return Err(Failure::Invalid(
                "Invalid reader preference timestamp".into(),
            ));
        }
        Ok(())
    }
}
