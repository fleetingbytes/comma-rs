//! Application state for the comma TUI.

use crate::config::{resolve_active_dongle, store_last_dongle, Config};
use crate::data::DataMsg;
use crate::keys::Action;
use comma_connect_api::{Device, Route, UploadQueueItem};

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

pub struct App {
    pub tab: Tab,
    pub selected: usize,
    pub config: Config,
    pub active_dongle: Option<String>,
    /// None = unknown, Some(true) = online, Some(false) = offline.
    pub online: Option<bool>,
    pub routes: Vec<Route>,
    pub queue: Vec<UploadQueueItem>,
    pub devices: Vec<Device>,
    pub device_detail: Option<Device>,
    pub routes_status: String,
    pub queue_status: String,
    pub devices_status: String,
    pub device_status: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config, cli_dongle: Option<&str>) -> Self {
        let active_dongle = resolve_active_dongle(&config, cli_dongle);
        let tab = if active_dongle.is_none() { Tab::Devices } else { Tab::Routes };
        Self {
            tab,
            selected: 0,
            config,
            active_dongle,
            online: None,
            routes: Vec::new(),
            queue: Vec::new(),
            devices: Vec::new(),
            device_detail: None,
            routes_status: "loading…".into(),
            queue_status: "loading…".into(),
            devices_status: "loading…".into(),
            device_status: "loading…".into(),
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
                    if let Some(d) = self.devices.get(self.selected) {
                        let id = d.dongle_id.clone();
                        self.active_dongle = Some(id.clone());
                        let _ = store_last_dongle(&id);
                        // Caller (main loop) will see the change and trigger a reload.
                    }
                }
            }
            Action::PageDown => self.selected = self.selected.saturating_add(10),
            Action::PageUp => self.selected = self.selected.saturating_sub(10),
        }
    }

    pub fn apply_msg(&mut self, msg: DataMsg) {
        match msg {
            DataMsg::Devices(Ok(devs)) => {
                self.devices = devs;
                self.devices_status = if self.devices.is_empty() {
                    "no devices".into()
                } else {
                    String::new()
                };
            }
            DataMsg::Devices(Err(e)) => {
                self.devices_status = format!("error: {e}");
            }
            DataMsg::Routes(Ok(routes)) => {
                self.routes = routes;
                self.routes_status = if self.routes.is_empty() {
                    "no routes".into()
                } else {
                    String::new()
                };
                self.clamp_selection();
            }
            DataMsg::Routes(Err(e)) => {
                self.routes_status = format!("error: {e}");
            }
            DataMsg::Queue(Ok(q)) => {
                self.queue = q;
                self.online = Some(true);
                self.queue_status = if self.queue.is_empty() {
                    "queue empty".into()
                } else {
                    String::new()
                };
                self.clamp_selection();
            }
            DataMsg::Queue(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("device offline") || msg.contains("not registered") {
                    self.online = Some(false);
                    self.queue.clear();
                    self.queue_status = "device offline".into();
                } else {
                    self.queue_status = format!("error: {e}");
                }
            }
            DataMsg::DeviceDetail(Ok(d)) => {
                self.device_detail = Some(d);
                self.device_status = String::new();
            }
            DataMsg::DeviceDetail(Err(e)) => {
                self.device_status = format!("error: {e}");
            }
        }
    }

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

    /// Alias for the active dongle, if known from the devices list.
    pub fn active_alias(&self) -> Option<&str> {
        let id = self.active_dongle.as_deref()?;
        self.devices
            .iter()
            .find(|d| d.dongle_id == id)
            .and_then(|d| d.alias.as_deref())
            .filter(|s| !s.is_empty())
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
        app.devices = vec![
            Device {
                dongle_id: "aaa".into(),
                alias: None,
                serial: None,
                device_type: None,
                is_owner: true,
                is_paired: true,
                prime: false,
                prime_type: None,
                openpilot_version: None,
                last_gps_lat: None,
                last_gps_lng: None,
                last_gps_time: None,
                extra: serde_json::Value::Null,
            },
            Device {
                dongle_id: "bbb".into(),
                alias: Some("my car".into()),
                serial: None,
                device_type: None,
                is_owner: true,
                is_paired: true,
                prime: false,
                prime_type: None,
                openpilot_version: None,
                last_gps_lat: None,
                last_gps_lng: None,
                last_gps_time: None,
                extra: serde_json::Value::Null,
            },
        ];
        app.selected = 1;
        app.on_action(Action::Select);
        assert_eq!(app.active_dongle.as_deref(), Some("bbb"));
        assert_eq!(app.active_alias(), Some("my car"));
    }
}
