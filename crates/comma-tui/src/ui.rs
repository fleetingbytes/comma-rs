//! Rendering for the comma TUI.
//!
//! Layout follows rustnet-style space saving:
//! - Top bar: left-aligned tabs, right-aligned device status (● alias/id)
//! - No frame around the main content
//! - Bottom bar: left-aligned ranked hints that fit, right-aligned "? help  q quit"

use crate::app::{App, Tab};
use crate::keys::{status_bar_hints, displays_for, Action};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Tabs, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // tab bar + separator
            Constraint::Min(0),    // content (no frame)
            Constraint::Length(1), // status bar
        ])
        .split(area);

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    // Split the top area into a line for tabs+status and a separator line.
    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(Span::styled(t.title(), Style::default())))
        .collect();
    let selected = Tab::ALL.iter().position(|&t| t == app.tab).unwrap_or(0);

    // Device status on the right: green ● if we have an active dongle (placeholder online),
    // grey ● otherwise. Shows alias if available, else the ID.
    let (indicator, label) = if let Some(id) = &app.active_dongle {
        // Placeholder: treat presence as online. Real online status comes later.
        ("●", id.as_str())
    } else {
        ("●", "(none)")
    };
    let status_style = if app.active_dongle.is_some() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let status_text = format!(" {indicator} {label}");

    // Use a horizontal layout so tabs take the left and status the right.
    let tab_area = Rect {
        x: lines[0].x,
        y: lines[0].y,
        width: lines[0].width.saturating_sub(status_text.len() as u16),
        height: 1,
    };
    let status_area = Rect {
        x: lines[0].x + tab_area.width,
        y: lines[0].y,
        width: (status_text.len() as u16).min(lines[0].width),
        height: 1,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider("|");
    f.render_widget(tabs, tab_area);

    let status_line = Line::from(vec![
        Span::styled(indicator, status_style),
        Span::raw(format!(" {label}")),
    ]);
    f.render_widget(Paragraph::new(status_line), status_area);

    // Separator line under the tabs.
    let sep = "─".repeat(lines[1].width as usize);
    f.render_widget(Paragraph::new(sep), lines[1]);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.tab {
        Tab::Routes => app.routes.join("\n"),
        Tab::Queue => app.queue.join("\n"),
        Tab::Device => format!(
            "Active dongle: {}\n\n{}",
            app.active_dongle.as_deref().unwrap_or("(none)"),
            app.device_info
        ),
        Tab::Devices => app.devices.join("\n"),
        Tab::Help => help_text(),
    };
    // No block/frame — content starts immediately.
    let para = Paragraph::new(text).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn help_text() -> String {
    let mut lines = vec!["Keybindings (source-defined, not configurable):".to_string(), String::new()];
    let actions = [
        Action::Quit, Action::Help, Action::NextTab, Action::PrevTab,
        Action::MoveDown, Action::MoveUp, Action::Select, Action::PageDown, Action::PageUp,
    ];
    for a in actions {
        let ds = displays_for(a);
        if !ds.is_empty() {
            lines.push(format!("{:12} {}", ds.join("/"), a.description()));
        }
    }
    lines.push(String::new());
    lines.push("Bottom bar shows ranked hints that fit on the left, and always-present help/quit on the right.".into());
    lines.push("Press q to quit.".into());
    lines.join("\n")
}

fn draw_status(f: &mut Frame, _app: &App, area: Rect) {
    // Right-aligned always-present commands.
    let right = "? help  q quit";
    let right_len = right.len() as u16;

    // Available width for left-aligned ranked hints.
    let available = area.width.saturating_sub(right_len + 2); // small gap

    let mut left_parts: Vec<String> = Vec::new();
    let mut used = 0u16;
    for b in status_bar_hints() {
        // Skip the always-present ones so they only appear on the right.
        if matches!(b.action, Action::Help | Action::Quit) {
            continue;
        }
        let part = format!("{} {}", b.display, b.action.description());
        let part_len = part.len() as u16 + 2; // separator
        if used + part_len > available {
            break;
        }
        left_parts.push(part);
        used += part_len;
    }
    let left = left_parts.join("  ");

    // Render left and right.
    let left_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width.saturating_sub(right_len),
        height: 1,
    };
    let right_area = Rect {
        x: area.x + left_area.width,
        y: area.y,
        width: right_len.min(area.width),
        height: 1,
    };

    let style = Style::default().bg(Color::DarkGray).fg(Color::White);
    f.render_widget(Paragraph::new(left).style(style), left_area);
    f.render_widget(Paragraph::new(right).style(style), right_area);
}
