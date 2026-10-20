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
    // Future: Upload, Cancel, etc.
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
];

/// Map a key event to an action, if bound.
pub fn action_for(key: &KeyEvent) -> Option<Action> {
    BINDINGS.iter().find(|b| b.code == key.code && b.modifiers == key.modifiers).map(|b| b.action)
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
