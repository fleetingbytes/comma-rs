//! Rendering for the comma TUI.

use crate::app::{App, Tab};
use crate::keys::{status_bar_hints, displays_for, Action};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(0),    // content
            Constraint::Length(1), // status
        ])
        .split(area);

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(Span::styled(t.title(), Style::default())))
        .collect();
    let selected = Tab::ALL.iter().position(|&t| t == app.tab).unwrap_or(0);
    let tabs = Tabs::new(titles)
        .select(selected)
        .block(Block::default().borders(Borders::ALL).title("comma"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = app.tab.title();
    let block = Block::default().borders(Borders::ALL).title(title);
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
    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });
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
    lines.push("Bottom bar shows a ranked subset of these bindings.".into());
    lines.push("Press q to quit.".into());
    lines.join("\n")
}

fn draw_status(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let hints: Vec<String> = status_bar_hints()
        .into_iter()
        .take(6) // simple width-independent limit for now
        .map(|b| format!("{}: {}", b.display, b.action.description()))
        .collect();
    let text = format!(
        " {} | dongle: {} | {}",
        hints.join("  "),
        app.active_dongle.as_deref().unwrap_or("-"),
        app.tab.title()
    );
    let para = Paragraph::new(text).style(Style::default().bg(Color::DarkGray));
    f.render_widget(para, area);
}
