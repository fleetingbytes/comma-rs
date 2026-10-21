//! Application state for the comma TUI.

use crate::config::{resolve_active_dongle, store_last_dongle, Config};
use crate::keys::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Routes,
    Queue,
    Device,
    Devices,
    Help,
}

impl Tab {
    pub const ALL: &[Tab] = &[Tab::Routes, Tab::Queue, Tab::Device, Tab::Devices, Tab::Help];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Routes => "Routes",
            Tab::Queue => "Queue",
            Tab::Device => "Device",
            Tab::Devices => "Devices",
            Tab::Help => "Help",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[allow(dead_code)]
pub struct App {
    pub tab: Tab,
    pub selected: usize,
    pub config: Config,
    pub active_dongle: Option<String>,
    /// Placeholder content per tab (real data will replace this).
    pub routes: Vec<String>,
    pub queue: Vec<String>,
    pub devices: Vec<String>,
    pub device_info: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config, cli_dongle: Option<&str>) -> Self {
        let active_dongle = resolve_active_dongle(&config, cli_dongle);
        // If no dongle resolved, start on the Devices tab so the user can pick one.
        let tab = if active_dongle.is_none() { Tab::Devices } else { Tab::Routes };
        Self {
            tab,
            selected: 0,
            config,
            active_dongle,
            routes: vec!["(loading routes...)".into()],
            queue: vec!["(loading queue...)".into()],
            devices: vec!["(loading devices...)".into()],
            device_info: "(loading device info...)".into(),
            should_quit: false,
        }
    }

    pub fn on_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Help => self.tab = Tab::Help,
            Action::NextTab => {
                self.tab = self.tab.next();
                self.selected = 0;
            }
            Action::PrevTab => {
                self.tab = self.tab.prev();
                self.selected = 0;
            }
            Action::MoveDown => self.selected = self.selected.saturating_add(1),
            Action::MoveUp => self.selected = self.selected.saturating_sub(1),
            Action::Select => {
                if self.tab == Tab::Devices {
                    if let Some(d) = self.devices.get(self.selected).cloned() {
                        // Extract the bare dongle ID (first token) if the list shows extra info.
                        let id = d.split_whitespace().next().unwrap_or(&d).to_string();
                        self.active_dongle = Some(id.clone());
                        let _ = store_last_dongle(&id); // best-effort persist to state file
                    }
                }
            }
            Action::PageDown => self.selected = self.selected.saturating_add(10),
            Action::PageUp => self.selected = self.selected.saturating_sub(10),
        }
    }

    /// Current list length for the active tab (for clamping selection).
    pub fn current_len(&self) -> usize {
        match self.tab {
            Tab::Routes => self.routes.len(),
            Tab::Queue => self.queue.len(),
            Tab::Devices => self.devices.len(),
            Tab::Device | Tab::Help => 1,
        }
    }

    pub fn clamp_selection(&mut self) {
        let len = self.current_len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_cycle() {
        assert_eq!(Tab::Routes.next(), Tab::Queue);
        assert_eq!(Tab::Help.next(), Tab::Routes);
        assert_eq!(Tab::Routes.prev(), Tab::Help);
    }

    #[test]
    fn select_on_devices_updates_active() {
        let mut app = App::new(Config::default(), None);
        app.tab = Tab::Devices;
        app.devices = vec!["aaa".into(), "bbb".into()];
        app.selected = 1;
        app.on_action(Action::Select);
        assert_eq!(app.active_dongle.as_deref(), Some("bbb"));
        // Persistence is best-effort to the state file; we only check the in-memory update here.
    }
}
