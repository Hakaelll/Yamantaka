use super::*;
use yamantaka_domain::reader_preferences::ReaderTheme;
#[derive(Clone, Copy)]
pub(super) enum Command {
    Shortcut(Shortcut),
    First,
    Last,
    Layout(PageLayout),
    Panel(outline::ReaderPanelTab),
    Position(navigation::PositionAction),
    Theme(ReaderTheme),
    Info,
    Settings,
    Library,
    History,
    Notes,
    Recent(usize),
}
pub(super) struct Entry {
    pub id: &'static str,
    pub label: String,
    pub hint: String,
    pub command: Command,
    pub enabled: bool,
}
pub(super) fn fold(value: &str) -> String {
    value
        .chars()
        .take(512)
        .flat_map(char::to_lowercase)
        .filter_map(|ch| match ch {
            'á' | 'à' | 'ä' => Some('a'),
            'é' | 'è' | 'ë' => Some('e'),
            'í' | 'ì' | 'ï' => Some('i'),
            'ó' | 'ò' | 'ö' => Some('o'),
            'ú' | 'ù' | 'ü' => Some('u'),
            'ñ' => Some('n'),
            '\u{0300}'..='\u{036f}' => None,
            ch => Some(ch),
        })
        .collect()
}
impl YamantakaApp {
    pub(super) fn palette_entries(&self) -> Vec<Entry> {
        let english = self.english;
        let query = fold(self.command_palette.query.trim());
        let reader = self.palette_document().is_some();
        let mut entries = Vec::new();
        let mut add = |id, es: &str, en: &str, command, enabled, hint: String| {
            if query.is_empty() || fold(&format!("{es} {en} {id}")).contains(&query) {
                entries.push(Entry {
                    id,
                    label: if english { en } else { es }.into(),
                    command,
                    enabled,
                    hint,
                });
            }
        };
        for item in shortcuts::bindings::DEFINITIONS {
            if item.command != Shortcut::CommandPalette && (item.global || reader) {
                add(
                    item.id,
                    item.es,
                    item.en,
                    Command::Shortcut(item.command),
                    true,
                    shortcuts::bindings::caption(self.shortcut_bindings.token(item.command)),
                );
            }
        }
        add(
            "settings",
            "Ajustes",
            "Settings",
            Command::Settings,
            true,
            String::new(),
        );
        add(
            "library",
            "Biblioteca",
            "Library",
            Command::Library,
            true,
            String::new(),
        );
        add(
            "readingHistory",
            "Historial de lectura",
            "Reading history",
            Command::History,
            true,
            String::new(),
        );
        add(
            "globalNotes",
            "Notas de la biblioteca",
            "Library notes",
            Command::Notes,
            true,
            String::new(),
        );
        if reader {
            for (id, es, en, command) in [
                ("firstPage", "Primera página", "First page", Command::First),
                ("lastPage", "Última página", "Last page", Command::Last),
                (
                    "singlePageLayout",
                    "Disposición: una página",
                    "Layout: single page",
                    Command::Layout(PageLayout::Single),
                ),
                (
                    "facingPageLayout",
                    "Disposición: dos páginas",
                    "Layout: facing pages",
                    Command::Layout(PageLayout::Spread),
                ),
                (
                    "continuousPageLayout",
                    "Disposición: continua",
                    "Layout: continuous",
                    Command::Layout(PageLayout::Continuous),
                ),
                (
                    "showOutline",
                    "Mostrar índice",
                    "Show contents",
                    Command::Panel(outline::ReaderPanelTab::Outline),
                ),
                (
                    "showAnnotations",
                    "Mostrar anotaciones",
                    "Show annotations",
                    Command::Panel(outline::ReaderPanelTab::Annotations),
                ),
                (
                    "showBookmarks",
                    "Mostrar marcadores",
                    "Show bookmarks",
                    Command::Panel(outline::ReaderPanelTab::Bookmarks),
                ),
                (
                    "savePositionA",
                    "Guardar posición A",
                    "Save position A",
                    Command::Position(navigation::PositionAction::Save(MarkSlot::A)),
                ),
                (
                    "savePositionB",
                    "Guardar posición B",
                    "Save position B",
                    Command::Position(navigation::PositionAction::Save(MarkSlot::B)),
                ),
                (
                    "goToPositionA",
                    "Ir a posición A",
                    "Go to position A",
                    Command::Position(navigation::PositionAction::Jump(MarkSlot::A)),
                ),
                (
                    "goToPositionB",
                    "Ir a posición B",
                    "Go to position B",
                    Command::Position(navigation::PositionAction::Jump(MarkSlot::B)),
                ),
                (
                    "swapPositions",
                    "Intercambiar posiciones A y B",
                    "Swap positions A and B",
                    Command::Position(navigation::PositionAction::Swap),
                ),
                (
                    "themeLight",
                    "Tema de lectura: claro",
                    "Reading theme: light",
                    Command::Theme(ReaderTheme::Light),
                ),
                (
                    "themeIvory",
                    "Tema de lectura: marfil",
                    "Reading theme: ivory",
                    Command::Theme(ReaderTheme::Ivory),
                ),
                (
                    "themeSepia",
                    "Tema de lectura: sepia",
                    "Reading theme: sepia",
                    Command::Theme(ReaderTheme::Sepia),
                ),
                (
                    "themeGray",
                    "Tema de lectura: gris",
                    "Reading theme: gray",
                    Command::Theme(ReaderTheme::Gray),
                ),
                (
                    "themeDark",
                    "Tema de lectura: oscuro",
                    "Reading theme: dark",
                    Command::Theme(ReaderTheme::Dark),
                ),
                (
                    "themeBlack",
                    "Tema de lectura: negro",
                    "Reading theme: black",
                    Command::Theme(ReaderTheme::Black),
                ),
                (
                    "documentInfo",
                    "Información del documento",
                    "Document information",
                    Command::Info,
                ),
            ] {
                let pane = if self
                    .command_palette
                    .context
                    .as_ref()
                    .is_some_and(|(target, _)| *target == ReadingTarget::Reference)
                {
                    &self.reference.as_ref().unwrap().pane
                } else {
                    &self.reader
                };
                let enabled = match command {
                    Command::Position(navigation::PositionAction::Jump(slot)) => pane
                        .navigation
                        .as_ref()
                        .is_some_and(|nav| nav.mark(slot).is_some()),
                    Command::Position(navigation::PositionAction::Swap) => {
                        pane.navigation.as_ref().is_some_and(|nav| {
                            nav.mark(MarkSlot::A).is_some() || nav.mark(MarkSlot::B).is_some()
                        })
                    }
                    _ => true,
                };
                add(id, es, en, command, enabled, String::new());
            }
        }
        for (index, document) in self.command_palette.recents.iter().enumerate() {
            let title: String = document.display_title().chars().take(120).collect();
            let searchable = format!(
                "{} {} {}",
                document
                    .display_title()
                    .chars()
                    .take(512)
                    .collect::<String>(),
                document
                    .author
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(128)
                    .collect::<String>(),
                document.file_name.chars().take(128).collect::<String>()
            );
            if query.is_empty() || fold(&searchable).contains(&query) {
                entries.push(Entry {
                    id: "recentDocument",
                    label: title,
                    hint: self.tr("Documento reciente", "Recent document").into(),
                    command: Command::Recent(index),
                    enabled: !document.missing,
                });
            }
        }
        entries
    }
}
