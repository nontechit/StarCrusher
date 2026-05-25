use crate::platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupRoute {
    Title,
    Adventure,
    MissionSelect,
}

impl StartupRoute {
    pub fn from_platform() -> Self {
        match platform::initial_mode_code() {
            1 => Self::Adventure,
            2 => Self::MissionSelect,
            _ => Self::Title,
        }
    }
}
