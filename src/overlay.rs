use serde::Serialize;

use crate::platform;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct OverlayControl {
    label: &'static str,
    action: &'static str,
    slot: &'static str,
}

impl OverlayControl {
    pub const fn new(label: &'static str, action: &'static str, slot: &'static str) -> Self {
        Self {
            label,
            action,
            slot,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct OverlayHome {
    label: &'static str,
    action: &'static str,
}

impl OverlayHome {
    pub const fn new(label: &'static str, action: &'static str) -> Self {
        Self { label, action }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OverlayState {
    #[serde(skip_serializing_if = "Option::is_none")]
    home: Option<OverlayHome>,
    buttons: Vec<OverlayControl>,
}

impl OverlayState {
    pub fn empty() -> Self {
        Self {
            home: None,
            buttons: Vec::new(),
        }
    }

    pub fn with_home(mut self, label: &'static str, action: &'static str) -> Self {
        self.home = Some(OverlayHome::new(label, action));
        self
    }

    pub fn button(mut self, label: &'static str, action: &'static str, slot: &'static str) -> Self {
        self.buttons.push(OverlayControl::new(label, action, slot));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.home.is_none() && self.buttons.is_empty()
    }
}

pub fn clear() {
    platform::set_html_overlay("");
}

pub fn publish(state: &OverlayState) {
    if state.is_empty() {
        clear();
        return;
    }

    if let Ok(json) = serde_json::to_string(state) {
        platform::set_html_overlay(&json);
    }
}

#[cfg(test)]
mod tests {
    use super::OverlayState;

    #[test]
    fn serializes_home_and_ordered_buttons_for_html_shell() {
        let state = OverlayState::empty()
            .with_home("HOME", "escape")
            .button("Reading Planet", "key:r", "menu-0")
            .button("Math Orbit", "key:p", "menu-1");

        let json = serde_json::to_string(&state).expect("overlay state should serialize");

        assert_eq!(
            json,
            r#"{"home":{"label":"HOME","action":"escape"},"buttons":[{"label":"Reading Planet","action":"key:r","slot":"menu-0"},{"label":"Math Orbit","action":"key:p","slot":"menu-1"}]}"#
        );
    }

    #[test]
    fn empty_overlay_has_no_controls() {
        assert!(OverlayState::empty().is_empty());
    }
}
