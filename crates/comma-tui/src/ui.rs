//! Rendering for the comma TUI.
//!
//! Layout follows rustnet-style space saving:
//! - Top bar: left-aligned tabs, right-aligned device status (● alias/id)
//! - No frame around the main content
//! - Bottom bar: left-aligned ranked hints that fit, right-aligned "? help  q quit"

use crate::app::{App, Tab};
use crate::keys::{displays_for, status_bar_hints, Action};
use crate::status::{upload_status_symbols, status_spans};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(Span::styled(t.title(), Style::default())))
        .collect();
    let selected = Tab::ALL.iter().position(|&t| t == app.tab).unwrap_or(0);

    let label = app
        .active_alias()
        .or(app.active_dongle.as_deref())
        .unwrap_or("(none)");
    let indicator = "●";
    // Green only when we have confirmed the device is online.
    // Grey when offline, unknown, or no dongle selected.
    let status_style = if app.online == Some(true) {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let status_text = format!(" {indicator} {label}");

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

    let sep = "─".repeat(lines[1].width as usize);
    f.render_widget(Paragraph::new(sep), lines[1]);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Routes => draw_routes(f, app, area),
        Tab::Queue => draw_queue(f, app, area),
        Tab::Device => draw_device(f, app, area),
        Tab::Devices => draw_devices(f, app, area),
        Tab::Help => {
            let para = Paragraph::new(help_text()).wrap(Wrap { trim: false });
            f.render_widget(para, area);
        }
    }
}

fn draw_routes(f: &mut Frame, app: &App, area: Rect) {
    if !app.routes_status.is_empty() {
        let para = Paragraph::new(app.routes_status.as_str());
        f.render_widget(para, area);
        return;
    }
    let items: Vec<ListItem> = app
        .routes
        .iter()
        .map(|r| {
            let ct = r
                .create_time
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".into());
            let status = if app.fully_uploaded.contains(&r.fullname) {
                // Known fully uploaded: show all letters in normal color without a query.
                let full = [
                    ('l', Color::DarkGray),
                    ('c', Color::Yellow),
                    ('L', Color::Green),
                    ('F', Color::Cyan),
                    ('W', Color::Magenta),
                    ('D', Color::Red),
                ];
                full.iter().map(|(ch, color)| {
                    Span::styled(ch.to_string(), Style::default().fg(*color))
                }).collect()
            } else if let Some(files) = app.route_files.get(&r.fullname) {
                let segs = r.maxqlog.map(|m| m + 1).unwrap_or(0);
                let symbols = upload_status_symbols(files, segs);
                status_spans(&symbols)
            } else {
                vec![Span::raw("      ")] // placeholder while loading
            };
            let mut line = vec![Span::raw(format!("{}  ", r.fullname))];
            line.extend(status);
            line.push(Span::raw(format!("  {}", ct)));
            ListItem::new(Line::from(line))
        })
        .collect();
    render_list(f, items, app.selected, area);
}

fn draw_queue(f: &mut Frame, app: &App, area: Rect) {
    if !app.queue_status.is_empty() {
        let para = Paragraph::new(app.queue_status.as_str());
        f.render_widget(para, area);
        return;
    }
    let items: Vec<ListItem> = app
        .queue
        .iter()
        .map(|q| {
            let path = q.path.as_deref().unwrap_or("?");
            let prog = q
                .progress
                .map(|p| format!("{:.0}%", p * 100.0))
                .unwrap_or_else(|| "-".into());
            let cur = if q.current.unwrap_or(false) { " *" } else { "" };
            ListItem::new(format!("{path}  {prog}{cur}"))
        })
        .collect();
    render_list(f, items, app.selected, area);
}

fn draw_devices(f: &mut Frame, app: &App, area: Rect) {
    if !app.devices_status.is_empty() {
        let para = Paragraph::new(app.devices_status.as_str());
        f.render_widget(para, area);
        return;
    }
    let items: Vec<ListItem> = app
        .devices
        .iter()
        .map(|d| {
            let alias = d.alias.as_deref().unwrap_or("");
            let owner = if d.is_owner { "owned" } else { "shared" };
            ListItem::new(format!("{}  {}  ({})", d.dongle_id, alias, owner))
        })
        .collect();
    render_list(f, items, app.selected, area);
}

fn draw_device(f: &mut Frame, app: &App, area: Rect) {
    if !app.device_status.is_empty() {
        let para = Paragraph::new(app.device_status.as_str());
        f.render_widget(para, area);
        return;
    }
    let text = if let Some(d) = &app.device_detail {
        format!(
            "dongle_id: {}\nalias: {}\nserial: {}\ndevice_type: {}\nis_owner: {}\nis_paired: {}\nprime: {}\nopenpilot_version: {}\nlast_gps: {}, {}\nlast_gps_time: {}",
            d.dongle_id,
            d.alias.as_deref().unwrap_or("-"),
            d.serial.as_deref().unwrap_or("-"),
            d.device_type.as_deref().unwrap_or("-"),
            d.is_owner,
            d.is_paired,
            d.prime,
            d.openpilot_version.as_deref().unwrap_or("-"),
            d.last_gps_lat.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            d.last_gps_lng.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            d.last_gps_time.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
        )
    } else {
        "no device selected".into()
    };
    let para = Paragraph::new(text).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_list(f: &mut Frame, items: Vec<ListItem>, selected: usize, area: Rect) {
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    let mut state = ListState::default().with_selected(Some(selected));
    f.render_stateful_widget(list, area, &mut state);
}

fn help_text() -> String {
    let mut lines = vec![
        "Keybindings (source-defined, not configurable):".to_string(),
        String::new(),
    ];
    let actions = [
        Action::Quit,
        Action::Help,
        Action::NextTab,
        Action::PrevTab,
        Action::MoveDown,
        Action::MoveUp,
        Action::Select,
        Action::PageDown,
        Action::PageUp,
        Action::RefreshUploadStatus,
    ];
    for a in actions {
        let ds = displays_for(a);
        if !ds.is_empty() {
            lines.push(format!("{:12} {}", ds.join("/"), a.description()));
        }
    }
    lines.push(String::new());
    lines.push("Routes shows the most recent 20 routes (fullname + create_time).".into());
    lines.push("Press q to quit.".into());
    lines.join("\n")
}

fn draw_status(f: &mut Frame, _app: &App, area: Rect) {
    let right = "? help  q quit";
    let right_len = right.len() as u16;
    let available = area.width.saturating_sub(right_len + 2);

    let mut left_parts: Vec<String> = Vec::new();
    let mut used = 0u16;
    for b in status_bar_hints() {
        if matches!(b.action, Action::Help | Action::Quit) {
            continue;
        }
        let part = format!("{} {}", b.display, b.action.description());
        let part_len = part.len() as u16 + 2;
        if used + part_len > available {
            break;
        }
        left_parts.push(part);
        used += part_len;
    }
    let left = left_parts.join("  ");

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
