use serde::Serialize;

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupMode {
    #[default]
    Normal,
    Requested,
    RepeatedUnclean {
        failures: u32,
    },
    StateUnavailable,
}
impl StartupMode {
    pub fn is_safe(self) -> bool {
        self != Self::Normal
    }
}
