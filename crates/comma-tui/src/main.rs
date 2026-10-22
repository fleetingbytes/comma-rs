//! comma — terminal UI for comma devices.

mod app;
mod config;
mod data;
mod keys;
mod status;
mod ui;

use anyhow::Result;
use clap::Parser;
use comma_common::state_dir;
use comma_connect_api::Client;
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
use crate::data::{Loader, channel};
use crate::keys::action_for;

#[derive(Parser)]
#[command(name = "comma", version, about = "Terminal UI for comma devices")]
struct Cli {
    /// Reserved for future use. Live log output is disabled while the TUI runs;
    /// all logs go to the log file only.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    #[allow(dead_code)]
    verbose: u8,

    /// Override the active dongle ID
    #[arg(short, long)]
    dongle: Option<String>,

    /// Override the JWT token
    #[arg(long)]
    token: Option<String>,
}

fn init_tracing() -> Result<std::path::PathBuf> {
    // All TUI logs go to the file only. No live stderr output while the TUI runs
    // (it would corrupt the alternate screen). The -v flag is currently unused.
    let dir = state_dir();
    fs::create_dir_all(&dir)?;
    let log_path = dir.join("comma-tui.log");

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(true)
        .with_ansi(false)
        .with_writer(file)
        .with_filter(EnvFilter::new("trace"));

    tracing_subscriber::registry()
        .with(file_layer)
        .init();

    debug!(log_file = %log_path.display(), "tracing initialized (file only)");
    Ok(log_path)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let _log_path = init_tracing()?;

    let config = Config::load()?;
    let token = config.resolve_token(cli.token.as_deref()).map_err(|e| {
        eprintln!("{e}");
        e
    })?;

    info!("starting comma TUI");
    let mut app = App::new(config, cli.dongle.as_deref());

    let rt = tokio::runtime::Runtime::new()?;
    let (tx, rx) = channel();
    let client = Client::new(token);
    let loader = Loader::new(client, tx, rt.handle().clone());

    // Initial loads
    loader.load_devices();
    if let Some(id) = app.active_dongle.clone() {
        loader.load_for_dongle(&id, app.fully_uploaded.clone());
    }

    let mut last_dongle = app.active_dongle.clone();
    run_tui(&mut app, &loader, &rx, &mut last_dongle)
}

fn run_tui(
    app: &mut App,
    loader: &Loader,
    rx: &std::sync::mpsc::Receiver<crate::data::DataMsg>,
    last_dongle: &mut Option<String>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = loop_tui(&mut terminal, app, loader, rx, last_dongle);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn loop_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    loader: &Loader,
    rx: &std::sync::mpsc::Receiver<crate::data::DataMsg>,
    last_dongle: &mut Option<String>,
) -> Result<()> {
    loop {
        // Drain any completed fetches
        while let Ok(msg) = rx.try_recv() {
            app.apply_msg(msg);
        }

        // If the user switched dongle, reload its data
        if app.active_dongle != *last_dongle {
            *last_dongle = app.active_dongle.clone();
            if let Some(id) = last_dongle.clone() {
                app.routes_status = "loading…".into();
                app.queue_status = "loading…".into();
                app.device_status = "loading…".into();
                app.online = None;
                app.routes.clear();
                app.route_files.clear();
                app.queue.clear();
                app.device_detail = None;
                loader.load_for_dongle(&id, app.fully_uploaded.clone());
            }
        }

        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = action_for(&key) {
                        app.on_action(action);
                        app.clamp_selection();
                        if let Some(name) = app.refresh_requested.take() {
                            loader.load_route_files(name);
                        }
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
