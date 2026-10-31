use super::Shortcut;
use eframe::egui::{Key, Modifiers};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
pub(in crate::app) struct Definition {
    pub command: Shortcut,
    pub id: &'static str,
    pub default: &'static str,
    pub es: &'static str,
    pub en: &'static str,
    pub global: bool,
    pub canvas: bool,
    pub repeats: bool,
}
pub(in crate::app) const DEFINITIONS: &[Definition] = &[
    Definition {
        command: Shortcut::UndoAnnotation,
        id: "undoAnnotation",
        default: "ctrl-z",
        es: "Deshacer cambio de anotación",
        en: "Undo annotation change",
        global: false,
        canvas: true,
        repeats: false,
    },
    Definition {
        command: Shortcut::RedoAnnotation,
        id: "redoAnnotation",
        default: "ctrl-shift-z",
        es: "Rehacer cambio de anotación",
        en: "Redo annotation change",
        global: false,
        canvas: true,
        repeats: false,
    },
    Definition {
        command: Shortcut::CommandPalette,
        id: "commandPalette",
        default: "ctrl-k",
        es: "Paleta de comandos",
        en: "Command palette",
        global: true,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::GoToPage,
        id: "goToPage",
        default: "ctrl-g",
        es: "Ir a una página",
        en: "Go to page",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::Navigator,
        id: "documentNavigator",
        default: "ctrl-j",
        es: "Navegador del documento",
        en: "Document navigator",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::TogglePanel,
        id: "togglePanel",
        default: "ctrl-shift-j",
        es: "Mostrar u ocultar panel",
        en: "Toggle panel",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::ToggleBookmark,
        id: "toggleBookmark",
        default: "ctrl-b",
        es: "Añadir o quitar marcador",
        en: "Toggle bookmark",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::ToggleChrome,
        id: "toggleChrome",
        default: "ctrl-h",
        es: "Mostrar u ocultar controles",
        en: "Toggle reader controls",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::ToggleImmersive,
        id: "toggleImmersive",
        default: "ctrl-shift-i",
        es: "Lectura inmersiva",
        en: "Immersive reading",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::OpenDocument,
        id: "openDocument",
        default: "ctrl-o",
        es: "Abrir PDF",
        en: "Open PDF",
        global: true,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::GlobalSearch,
        id: "rust.globalSearch",
        default: "ctrl-shift-f",
        es: "Buscar en la biblioteca",
        en: "Search library",
        global: true,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::DocumentFind,
        id: "search",
        default: "ctrl-f",
        es: "Buscar en el PDF",
        en: "Find in PDF",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::PreviousMatch,
        id: "rust.previousMatch",
        default: "shift-f3",
        es: "Coincidencia anterior",
        en: "Previous match",
        global: false,
        canvas: false,
        repeats: true,
    },
    Definition {
        command: Shortcut::NextMatch,
        id: "rust.nextMatch",
        default: "f3",
        es: "Coincidencia siguiente",
        en: "Next match",
        global: false,
        canvas: false,
        repeats: true,
    },
    Definition {
        command: Shortcut::PreviousPage,
        id: "previousPage",
        default: "arrow-left",
        es: "Página anterior",
        en: "Previous page",
        global: false,
        canvas: true,
        repeats: true,
    },
    Definition {
        command: Shortcut::NextPage,
        id: "nextPage",
        default: "arrow-right",
        es: "Página siguiente",
        en: "Next page",
        global: false,
        canvas: true,
        repeats: true,
    },
    Definition {
        command: Shortcut::SwitchPane,
        id: "rust.switchPane",
        default: "f6",
        es: "Cambiar de lector",
        en: "Switch reader pane",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::HistoryBack,
        id: "rust.historyBack",
        default: "alt-left",
        es: "Volver en el historial",
        en: "History back",
        global: false,
        canvas: true,
        repeats: true,
    },
    Definition {
        command: Shortcut::HistoryForward,
        id: "rust.historyForward",
        default: "alt-right",
        es: "Avanzar en el historial",
        en: "History forward",
        global: false,
        canvas: true,
        repeats: true,
    },
    Definition {
        command: Shortcut::ZoomIn,
        id: "zoomIn",
        default: "ctrl-plus",
        es: "Aumentar zoom",
        en: "Zoom in",
        global: false,
        canvas: false,
        repeats: true,
    },
    Definition {
        command: Shortcut::ZoomOut,
        id: "zoomOut",
        default: "ctrl-minus",
        es: "Reducir zoom",
        en: "Zoom out",
        global: false,
        canvas: false,
        repeats: true,
    },
    Definition {
        command: Shortcut::ActualSize,
        id: "actualSize",
        default: "ctrl-1",
        es: "Tamaño real",
        en: "Actual size",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::FitWidth,
        id: "fitWidth",
        default: "ctrl-2",
        es: "Ajustar al ancho",
        en: "Fit width",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::FitPage,
        id: "rust.fitPage",
        default: "ctrl-0",
        es: "Ajustar página",
        en: "Fit page",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::Fullscreen,
        id: "fullScreen",
        default: "f11",
        es: "Pantalla completa",
        en: "Fullscreen",
        global: true,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::Focus,
        id: "rust.focus",
        default: "f8",
        es: "Lectura sin controles",
        en: "Hide reader controls",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::NextTab,
        id: "rust.nextTab",
        default: "ctrl-tab",
        es: "Siguiente lectura",
        en: "Next reading",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::PreviousTab,
        id: "rust.previousTab",
        default: "ctrl-shift-tab",
        es: "Lectura anterior",
        en: "Previous reading",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::CloseTab,
        id: "rust.closeTab",
        default: "ctrl-w",
        es: "Cerrar lectura",
        en: "Close reading",
        global: false,
        canvas: false,
        repeats: false,
    },
    Definition {
        command: Shortcut::ReopenTab,
        id: "rust.reopenTab",
        default: "ctrl-shift-t",
        es: "Reabrir lectura",
        en: "Reopen reading",
        global: true,
        canvas: false,
        repeats: false,
    },
];
pub(super) const CHOICES: &[&str] = &[
    "ctrl-z",
    "ctrl-shift-z",
    "ctrl-y",
    "ctrl-o",
    "ctrl-shift-o",
    "ctrl-f",
    "ctrl-shift-f",
    "ctrl-k",
    "ctrl-shift-k",
    "ctrl-l",
    "ctrl-g",
    "ctrl-0",
    "ctrl-1",
    "ctrl-2",
    "ctrl-shift-i",
    "ctrl-j",
    "ctrl-shift-j",
    "ctrl-b",
    "ctrl-d",
    "arrow-right",
    "ctrl-right",
    "arrow-left",
    "ctrl-left",
    "ctrl-plus",
    "ctrl-shift-plus",
    "ctrl-minus",
    "ctrl-shift-minus",
    "f11",
    "ctrl-enter",
    "ctrl-h",
    "ctrl-shift-h",
    "f3",
    "shift-f3",
    "f5",
    "f6",
    "f7",
    "f8",
    "f9",
    "f10",
    "f12",
    "alt-left",
    "alt-right",
    "ctrl-tab",
    "ctrl-shift-tab",
    "ctrl-w",
    "ctrl-shift-t",
];
pub(super) fn chord(token: &str) -> Option<(Key, bool, bool, bool)> {
    if !CHOICES.contains(&token) {
        return None;
    }
    let key = match token.rsplit('-').next()? {
        "o" => Key::O,
        "z" => Key::Z,
        "y" => Key::Y,
        "f" => Key::F,
        "k" => Key::K,
        "l" => Key::L,
        "g" => Key::G,
        "0" => Key::Num0,
        "1" => Key::Num1,
        "2" => Key::Num2,
        "i" => Key::I,
        "j" => Key::J,
        "b" => Key::B,
        "d" => Key::D,
        "right" => Key::ArrowRight,
        "left" => Key::ArrowLeft,
        "plus" => Key::Plus,
        "minus" => Key::Minus,
        "enter" => Key::Enter,
        "h" => Key::H,
        "f3" => Key::F3,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "tab" => Key::Tab,
        "w" => Key::W,
        "t" => Key::T,
        _ => return None,
    };
    Some((
        key,
        token.starts_with("ctrl-"),
        token.contains("shift-") && key != Key::Plus,
        token.starts_with("alt-"),
    ))
}
pub(super) fn matches(token: &str, key: Key, modifiers: Modifiers) -> bool {
    let Some((wanted, ctrl, shift, alt)) = chord(token) else {
        return false;
    };
    let key = if key == Key::Equals && modifiers.ctrl {
        Key::Plus
    } else {
        key
    };
    wanted == key
        && ctrl == modifiers.ctrl
        && alt == modifiers.alt
        && !modifiers.mac_cmd
        && (wanted == Key::Plus || shift == modifiers.shift)
}
pub(in crate::app) fn caption(token: &str) -> String {
    let token = token.strip_prefix("arrow-").unwrap_or(token);
    token
        .split('-')
        .map(|part| match part {
            "ctrl" => "Ctrl".into(),
            "shift" => "Shift".into(),
            "alt" => "Alt".into(),
            "plus" => "+".into(),
            "minus" => "−".into(),
            "left" => "←".into(),
            "right" => "→".into(),
            other => other.to_uppercase(),
        })
        .collect::<Vec<_>>()
        .join("+")
}
pub(in crate::app) struct Bindings {
    raw: Map<String, Value>,
    keys: BTreeMap<Shortcut, String>,
    pub warning: bool,
}
impl Bindings {
    pub fn load(value: Option<&Value>) -> Self {
        let mut warning = false;
        let decoded = value.and_then(|value| match value {
            Value::String(value) => serde_json::from_str::<Value>(value).ok(),
            Value::Object(_) => Some(value.clone()),
            _ => None,
        });
        let raw = match decoded {
            Some(Value::Object(raw)) => raw,
            None if value.is_none() => Map::new(),
            _ => {
                warning = true;
                Map::new()
            }
        };
        let mut keys: BTreeMap<_, _> = DEFINITIONS
            .iter()
            .map(|item| (item.command, item.default.to_owned()))
            .collect();
        for item in DEFINITIONS {
            if let Some(value) = raw.get(item.id) {
                if let Some(token) = value.as_str().filter(|token| chord(token).is_some()) {
                    keys.insert(item.command, token.to_owned());
                } else {
                    warning = true;
                }
            }
        }
        for (index, item) in DEFINITIONS.iter().enumerate() {
            if DEFINITIONS[..index]
                .iter()
                .any(|other| chord(&keys[&item.command]) == chord(&keys[&other.command]))
            {
                warning = true;
            }
        }
        if warning {
            keys = DEFINITIONS
                .iter()
                .map(|item| (item.command, item.default.to_owned()))
                .collect();
        }
        Self { raw, keys, warning }
    }
    pub fn token(&self, command: Shortcut) -> &str {
        &self.keys[&command]
    }
    pub fn changed(&self, command: Shortcut, token: &str) -> Result<Self, Shortcut> {
        if self.warning || chord(token).is_none() {
            return Err(command);
        }
        if let Some(conflict) = DEFINITIONS.iter().find(|other| {
            other.command != command && chord(self.token(other.command)) == chord(token)
        }) {
            return Err(conflict.command);
        }
        let mut raw = self.raw.clone();
        let item = DEFINITIONS
            .iter()
            .find(|item| item.command == command)
            .unwrap();
        raw.insert(item.id.into(), token.into());
        Ok(Self::load(Some(&Value::Object(raw))))
    }
    pub fn reset(&self) -> Self {
        let mut raw = self.raw.clone();
        for item in DEFINITIONS {
            raw.remove(item.id);
        }
        Self::load(Some(&Value::Object(raw)))
    }
    pub fn summary(&self) -> Value {
        serde_json::json!({"warning":self.warning,"known":DEFINITIONS.len(),"unavailable":self.inherited_unavailable(),"nextPage":self.token(Shortcut::NextPage),"switchPane":self.token(Shortcut::SwitchPane),"zoomIn":self.token(Shortcut::ZoomIn)})
    }
    pub fn encoded(&self) -> Value {
        Value::String(Value::Object(self.raw.clone()).to_string())
    }
    pub fn inherited_unavailable(&self) -> usize {
        self.raw
            .keys()
            .filter(|id| !DEFINITIONS.iter().any(|item| item.id == id.as_str()))
            .count()
    }
}
