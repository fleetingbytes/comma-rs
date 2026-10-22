//! Source-defined keybindings.
//!
//! All interactive bindings live here. Each binding carries a display string
//! used by the Help tab and the bottom status bar. Changing a key or its
//! label requires no other edits.
//!
//! Multiple keys may map to the same action. Modifiers are supported.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions the TUI understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    Help,
    NextTab,
    PrevTab,
    MoveDown,
    MoveUp,
    Select,
    PageDown,
    PageUp,
    /// Refresh upload status of the currently selected route (Routes tab).
    RefreshUploadStatus,
    /// Request upload of full logs for the selected route.
    UploadLog,
    /// Request upload of fcamera (road) for the selected route.
    UploadFCamera,
    /// Request upload of ecamera (wide road) for the selected route.
    UploadECamera,
    /// Request upload of dcamera (driver) for the selected route.
    UploadDCamera,
    /// Request upload of all file kinds for the selected route.
    UploadRoute,
    /// Request upload of all file kinds for all currently listed routes.
    UploadAllListed,
    /// Cancel the selected upload queue item.
    CancelSelected,
    /// Cancel all pending (not currently uploading) queue items.
    CancelPending,
}

impl Action {
    pub fn description(self) -> &'static str {
        match self {
            Action::Quit => "quit",
            Action::Help => "show help",
            Action::NextTab => "next tab",
            Action::PrevTab => "previous tab",
            Action::MoveDown => "move down",
            Action::MoveUp => "move up",
            Action::Select => "select / activate",
            Action::PageDown => "page down",
            Action::PageUp => "page up",
            Action::RefreshUploadStatus => "refresh upload status",
            Action::UploadLog => "upload log",
            Action::UploadFCamera => "upload fcamera",
            Action::UploadECamera => "upload ecamera",
            Action::UploadDCamera => "upload dcamera",
            Action::UploadRoute => "upload route",
            Action::UploadAllListed => "upload all listed",
            Action::CancelSelected => "cancel selected",
            Action::CancelPending => "cancel pending",
        }
    }
}

/// A single key binding.
#[derive(Debug, Clone, Copy)]
pub struct Binding {
    pub action: Action,
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub display: &'static str,
    /// Higher = more important for the bottom bar.
    pub importance: u8,
}

/// All bindings, ordered roughly by importance for the status bar.
pub static BINDINGS: &[Binding] = &[
    // New high-priority route/queue actions (shown first in the hint bar)
    Binding { action: Action::UploadLog, code: KeyCode::Char('L'), modifiers: KeyModifiers::SHIFT, display: "L", importance: 99 },
    Binding { action: Action::UploadFCamera, code: KeyCode::Char('F'), modifiers: KeyModifiers::SHIFT, display: "F", importance: 99 },
    Binding { action: Action::UploadECamera, code: KeyCode::Char('W'), modifiers: KeyModifiers::SHIFT, display: "W", importance: 99 },
    Binding { action: Action::UploadDCamera, code: KeyCode::Char('D'), modifiers: KeyModifiers::SHIFT, display: "D", importance: 99 },
    Binding { action: Action::UploadRoute, code: KeyCode::Char('u'), modifiers: KeyModifiers::NONE, display: "u", importance: 98 },
    Binding { action: Action::UploadAllListed, code: KeyCode::Char('U'), modifiers: KeyModifiers::SHIFT, display: "U", importance: 98 },
    Binding { action: Action::CancelSelected, code: KeyCode::Char('c'), modifiers: KeyModifiers::NONE, display: "c", importance: 97 },
    Binding { action: Action::CancelPending, code: KeyCode::Char('C'), modifiers: KeyModifiers::SHIFT, display: "C", importance: 97 },
    Binding { action: Action::Quit, code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, display: "q", importance: 100 },
    Binding { action: Action::Help, code: KeyCode::Char('?'), modifiers: KeyModifiers::NONE, display: "?", importance: 90 },
    Binding { action: Action::MoveDown, code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, display: "j", importance: 80 },
    Binding { action: Action::MoveUp, code: KeyCode::Char('k'), modifiers: KeyModifiers::NONE, display: "k", importance: 80 },
    Binding { action: Action::MoveDown, code: KeyCode::Down, modifiers: KeyModifiers::NONE, display: "down", importance: 70 },
    Binding { action: Action::MoveUp, code: KeyCode::Up, modifiers: KeyModifiers::NONE, display: "up", importance: 70 },
    Binding { action: Action::Select, code: KeyCode::Char(' '), modifiers: KeyModifiers::NONE, display: "space", importance: 60 },
    Binding { action: Action::Select, code: KeyCode::Enter, modifiers: KeyModifiers::NONE, display: "enter", importance: 50 },
    Binding { action: Action::NextTab, code: KeyCode::Char('l'), modifiers: KeyModifiers::NONE, display: "l", importance: 40 },
    Binding { action: Action::PrevTab, code: KeyCode::Char('h'), modifiers: KeyModifiers::NONE, display: "h", importance: 40 },
    Binding { action: Action::NextTab, code: KeyCode::Right, modifiers: KeyModifiers::NONE, display: "right", importance: 30 },
    Binding { action: Action::PrevTab, code: KeyCode::Left, modifiers: KeyModifiers::NONE, display: "left", importance: 30 },
    Binding { action: Action::PageDown, code: KeyCode::Char('f'), modifiers: KeyModifiers::CONTROL, display: "ctrl-f", importance: 20 },
    Binding { action: Action::PageUp, code: KeyCode::Char('b'), modifiers: KeyModifiers::CONTROL, display: "ctrl-b", importance: 20 },
    Binding { action: Action::PageDown, code: KeyCode::PageDown, modifiers: KeyModifiers::NONE, display: "pgdn", importance: 15 },
    Binding { action: Action::PageUp, code: KeyCode::PageUp, modifiers: KeyModifiers::NONE, display: "pgup", importance: 15 },
    Binding { action: Action::RefreshUploadStatus, code: KeyCode::Char('r'), modifiers: KeyModifiers::CONTROL, display: "ctrl-r", importance: 10 },
];

/// Map a key event to an action, if bound.
///
/// Matching is intentionally lenient for CONTROL combinations:
/// - Exact code + modifiers match is preferred.
/// - If the binding requires CONTROL, we also accept the event when CONTROL
///   is set (even if other modifiers are present) and the character matches.
/// - We also accept the raw control character (e.g. Ctrl-r arrives as `\u{12}`
///   on some terminals) mapped back to the letter.
pub fn action_for(key: &KeyEvent) -> Option<Action> {
    // 1. Exact match
    if let Some(b) = BINDINGS.iter().find(|b| b.code == key.code && b.modifiers == key.modifiers) {
        return Some(b.action);
    }

    // 2. CONTROL or SHIFT + char, ignoring extra modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) || key.modifiers.contains(KeyModifiers::SHIFT) {
        if let KeyCode::Char(c) = key.code {
            let letter = c.to_ascii_lowercase();
            if let Some(b) = BINDINGS.iter().find(|b| {
                (b.modifiers.contains(KeyModifiers::CONTROL) || b.modifiers.contains(KeyModifiers::SHIFT) || b.modifiers.is_empty())
                    && matches!(b.code, KeyCode::Char(bc) if bc.to_ascii_lowercase() == letter)
            }) {
                return Some(b.action);
            }
        }
    }

    // 3. Raw control character (no CONTROL modifier reported)
    if key.modifiers.is_empty() {
        if let KeyCode::Char(c) = key.code {
            // Map common control chars back to their letter
            let letter = match c as u8 {
                0x01 => Some('a'), // Ctrl-a
                0x02 => Some('b'), // Ctrl-b
                0x06 => Some('f'), // Ctrl-f
                0x12 => Some('r'), // Ctrl-r
                _ => None,
            };
            if let Some(letter) = letter {
                if let Some(b) = BINDINGS.iter().find(|b| {
                    b.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(b.code, KeyCode::Char(bc) if bc.to_ascii_lowercase() == letter)
                }) {
                    return Some(b.action);
                }
            }
        }
    }

    None
}

/// Display strings for an action (may be multiple).
pub fn displays_for(action: Action) -> Vec<&'static str> {
    BINDINGS.iter().filter(|b| b.action == action).map(|b| b.display).collect()
}

/// Ranked bindings for the bottom bar (highest importance first).
pub fn status_bar_hints() -> Vec<&'static Binding> {
    let mut v: Vec<_> = BINDINGS.iter().collect();
    v.sort_by(|a, b| b.importance.cmp(&a.importance));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_is_q() {
        let ev = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(action_for(&ev), Some(Action::Quit));
    }

    #[test]
    fn ctrl_r_refresh() {
        // Normal CONTROL + 'r'
        let ev = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(action_for(&ev), Some(Action::RefreshUploadStatus));
        // Extra modifiers should still match
        let ev = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(action_for(&ev), Some(Action::RefreshUploadStatus));
        // Raw control character (some terminals)
        let ev = KeyEvent::new(KeyCode::Char('\u{12}'), KeyModifiers::NONE);
        assert_eq!(action_for(&ev), Some(Action::RefreshUploadStatus));
    }

    fn help_is_question() {
        let ev = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(action_for(&ev), Some(Action::Help));
    }

    #[test]
    fn multiple_keys_for_movement() {
        assert!(displays_for(Action::MoveDown).contains(&"j"));
        assert!(displays_for(Action::MoveDown).contains(&"down"));
    }

    #[test]
    fn ctrl_f_pages_down() {
        let ev = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        assert_eq!(action_for(&ev), Some(Action::PageDown));
    }

    #[test]
    fn displays_are_non_empty() {
        for b in BINDINGS {
            assert!(!b.display.is_empty());
        }
    }
}
