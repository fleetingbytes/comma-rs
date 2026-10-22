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
    let parsed: Vec<_> = app.queue.iter().map(parse_queue_item).collect();
    let max_route = parsed.iter().map(|p| p.route.len()).max().unwrap_or(8).max(8);
    let max_seg = parsed.iter().map(|p| p.segment.len()).max().unwrap_or(3).max(3);
    let max_kind = parsed.iter().map(|p| p.kind.len()).max().unwrap_or(8).max(8);

    let items: Vec<ListItem> = parsed
        .into_iter()
        .map(|p| {
            let bar = if p.current {
                progress_bar(p.progress)
            } else {
                "pending".to_string()
            };
            let line = format!(
                "{:<10}  {:<width_r$}  {:>width_s$}  {:<width_k$}",
                bar,
                p.route,
                p.segment,
                p.kind,
                width_r = max_route,
                width_s = max_seg,
                width_k = max_kind,
            );
            ListItem::new(line)
        })
        .collect();
    render_list(f, items, app.selected, area);
}

struct ParsedQueue {
    route: String,
    segment: String,
    kind: String,
    progress: f64,
    current: bool,
}

fn parse_queue_item(q: &comma_connect_api::UploadQueueItem) -> ParsedQueue {
    let path = q.path.as_deref().unwrap_or("");
    let (route, segment, kind) = parse_upload_path(path);
    ParsedQueue {
        route,
        segment,
        kind,
        progress: q.progress.unwrap_or(0.0),
        current: q.current.unwrap_or(false),
    }
}

/// Parse a queue path/filename into (short route name, segment, log type).
/// Handles both `logid--seg/filename` and `dongle_logid--seg--filename` forms.
fn parse_upload_path(path: &str) -> (String, String, String) {
    // Forms:
    //   dongle_logid--hash--seg--filename          (issue example)
    //   logid--hash--seg/filename                  (device path style)
    //   logid--hash--seg--filename
    let filename = path.rsplit('/').next().unwrap_or(path);
    let parent = path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");

    // Prefer parsing the parent directory when it looks like logid--seg
    if !parent.is_empty() && parent.contains("--") {
        let parent_tail = parent.rsplit(['/', '_']).next().unwrap_or(parent);
        let parts: Vec<&str> = parent_tail.split("--").collect();
        if parts.len() >= 2 {
            if let Some(i) = parts.iter().rposition(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
                if i >= 1 {
                    let route = parts[..i].join("--");
                    let segment = parts[i].to_string();
                    return (route, segment, log_type_from_filename(filename));
                }
            }
        }
    }

    // Otherwise parse the filename / whole tail itself
    let candidate = if filename.contains('_') && filename.contains("--") {
        filename.rsplit_once('_').map(|(_, rest)| rest).unwrap_or(filename)
    } else {
        filename
    };
    let parts: Vec<&str> = candidate.split("--").collect();
    if parts.len() >= 2 {
        if let Some(i) = parts.iter().position(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
            if i >= 1 {
                let route = parts[..i].join("--");
                let segment = parts[i].to_string();
                let kind = if i + 1 < parts.len() {
                    log_type_from_filename(parts[i + 1])
                } else {
                    log_type_from_filename(filename)
                };
                return (route, segment, kind);
            }
        }
    }
    ("?".into(), "?".into(), log_type_from_filename(filename))
}

fn log_type_from_filename(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("fcamera") { "fcamera".into() }
    else if lower.contains("ecamera") { "ecamera".into() }
    else if lower.contains("dcamera") { "dcamera".into() }
    else if lower.contains("qcamera") { "qcamera".into() }
    else if lower.contains("qlog") { "qlog".into() }
    else if lower.contains("rlog") || lower.contains("log") { "log".into() }
    else { name.split('.').next().unwrap_or(name).to_string() }
}

fn progress_bar(p: f64) -> String {
    let p = p.clamp(0.0, 1.0);
    let width = 8;
    let filled = (p * width as f64).round() as usize;
    let empty = width - filled;
    format!("[{}{}] {:3.0}%", "=".repeat(filled), " ".repeat(empty), p * 100.0)
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
        Action::UploadLog,
        Action::UploadFCamera,
        Action::UploadECamera,
        Action::UploadDCamera,
        Action::UploadRoute,
        Action::UploadAllListed,
        Action::CancelSelected,
        Action::CancelPending,
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

#[cfg(test)]
mod queue_parse_tests {
    use super::parse_upload_path;

    #[test]
    fn parse_upload_path_examples() {
        let (r, s, k) = parse_upload_path("44379c05b64fd7e3_0000014c--d6e9be51b8--0--fcamera.hevc");
        assert_eq!(r, "0000014c--d6e9be51b8");
        assert_eq!(s, "0");
        assert_eq!(k, "fcamera");

        let (r, s, k) = parse_upload_path("0000014c--d6e9be51b8--6/rlog.zst");
        assert_eq!(r, "0000014c--d6e9be51b8");
        assert_eq!(s, "6");
        assert_eq!(k, "log");
    }
}
