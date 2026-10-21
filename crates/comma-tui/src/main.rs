//! comma — terminal UI for comma devices.
//!
//! Early skeleton: tabbed layout, fixed keybindings, shared config + env + CLI
//! overrides, last-used dongle in state, placeholder data views, Help tab.

mod app;
mod config;
mod keys;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs::{self, OpenOptions};
use std::io;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::App;
use crate::config::Config;
use crate::keys::action_for;
use comma_common::state_dir;

#[derive(Parser)]
#[command(name = "comma", version, about = "Terminal UI for comma devices")]
struct Cli {
    /// Increase logging verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Override the active dongle ID
    #[arg(short, long)]
    dongle: Option<String>,

    /// Override the JWT token
    #[arg(long)]
    token: Option<String>,
}

fn init_tracing(verbosity: u8) -> Result<std::path::PathBuf> {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let dir = state_dir();
    fs::create_dir_all(&dir)?;
    let log_path = dir.join("comma-tui.log");

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .with_filter(EnvFilter::new(level));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(true)
        .with_ansi(false)
        .with_writer(file)
        .with_filter(EnvFilter::new("trace"));

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    debug!(verbosity, level, log_file = %log_path.display(), "tracing initialized");
    Ok(log_path)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let _log_path = init_tracing(cli.verbose)?;

    let config = Config::load()?;
    // Fail early if no token is available. Later we will also validate it by listing devices.
    let _token = config.resolve_token(cli.token.as_deref()).map_err(|e| {
        // Ensure the helpful message reaches stderr before the TUI starts.
        eprintln!("{e}");
        e
    })?;

    info!("starting comma TUI");
    let mut app = App::new(config, cli.dongle.as_deref());

    // Placeholder data (real API calls will replace these).
    app.routes = vec![
        "2026-10-11 15:24  route-001".into(),
        "2026-10-10 09:12  route-002".into(),
        "2026-10-09 18:45  route-003".into(),
    ];
    app.queue = vec!["(empty)".into()];
    app.devices = vec![
        "0123456789abcdef  (owned)".into(),
        "fedcba9876543210  (shared)".into(),
    ];
    app.device_info = "alias: my-device\nversion: 0.9.7\nonline: yes".into();

    run_tui(&mut app)
}

fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = loop_tui(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn loop_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = action_for(&key) {
                        app.on_action(action);
                        app.clamp_selection();
                    }
                }
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
